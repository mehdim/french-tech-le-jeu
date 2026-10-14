//! Grid-based office: furniture placement and its gameplay effects.
//!
//! Coordinates: x grows along the right-hand back wall, y along the
//! left-hand back wall (isometric). The entrance is the front cell
//! `door()`; every seat or appliance must stay reachable from it.

use crate::content::{Content, FurnitureDef, OfficeTierDef};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlacedItem {
    pub uid: u32,
    pub item: String,
    pub x: i32,
    pub y: i32,
    pub rotated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeState {
    pub tier: u32,
    pub items: Vec<PlacedItem>,
    /// Owned furniture that did not fit after a move.
    pub storage: Vec<String>,
    pub next_uid: u32,
    /// Rent increases negotiated (or not) with the landlord.
    #[serde(default)]
    pub rent_extra: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaceError {
    OutOfBounds,
    Overlap,
    BlocksDoor,
    Unreachable,
    UnknownItem,
}

impl PlaceError {
    pub fn label(&self) -> &'static str {
        match self {
            PlaceError::OutOfBounds => "Hors des murs",
            PlaceError::Overlap => "Emplacement occupé",
            PlaceError::BlocksDoor => "Bloque l'entrée",
            PlaceError::Unreachable => "Rendrait un poste inaccessible",
            PlaceError::UnknownItem => "Objet inconnu",
        }
    }
}

/// Aggregated effects of all placed furniture.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OfficeEffects {
    pub seats: u32,
    pub morale: f64,
    pub fatigue_recovery: f64,
    pub eng_bonus: f64,
    pub research_bonus: f64,
    pub sales_bonus: f64,
    pub incident_risk: f64,
    pub inference_cost: f64,
    pub productivity: f64,
    pub monthly: f64,
    pub workstations: u32,
}

pub fn footprint(def: &FurnitureDef, rotated: bool) -> (i32, i32) {
    if rotated { (def.size.1, def.size.0) } else { def.size }
}

pub fn door(tier: &OfficeTierDef) -> (i32, i32) {
    (tier.width / 2, tier.height - 1)
}

impl OfficeState {
    pub fn new(tier: u32) -> Self {
        Self { tier, items: Vec::new(), storage: Vec::new(), next_uid: 1, rent_extra: 0.0 }
    }

    fn occupancy(
        &self,
        content: &Content,
        tier: &OfficeTierDef,
        ignore: Option<u32>,
    ) -> Vec<Option<u32>> {
        let mut grid = vec![None; (tier.width * tier.height) as usize];
        for it in &self.items {
            if Some(it.uid) == ignore {
                continue;
            }
            let Some(def) = content.furniture(&it.item) else { continue };
            let (w, h) = footprint(def, it.rotated);
            for dx in 0..w {
                for dy in 0..h {
                    let (x, y) = (it.x + dx, it.y + dy);
                    if x >= 0 && y >= 0 && x < tier.width && y < tier.height {
                        grid[(y * tier.width + x) as usize] = Some(it.uid);
                    }
                }
            }
        }
        grid
    }

    /// Check whether `item` can go at (x, y). `ignore` lets an existing item
    /// be moved without colliding with itself.
    pub fn can_place(
        &self,
        content: &Content,
        item: &str,
        x: i32,
        y: i32,
        rotated: bool,
        ignore: Option<u32>,
    ) -> Result<(), PlaceError> {
        let tier = content.office(self.tier).ok_or(PlaceError::UnknownItem)?;
        let def = content.furniture(item).ok_or(PlaceError::UnknownItem)?;
        let (w, h) = footprint(def, rotated);
        if x < 0 || y < 0 || x + w > tier.width || y + h > tier.height {
            return Err(PlaceError::OutOfBounds);
        }
        let mut grid = self.occupancy(content, tier, ignore);
        let (dx, dy) = door(tier);
        for cx in x..x + w {
            for cy in y..y + h {
                if grid[(cy * tier.width + cx) as usize].is_some() {
                    return Err(PlaceError::Overlap);
                }
                if (cx, cy) == (dx, dy) {
                    return Err(PlaceError::BlocksDoor);
                }
            }
        }
        for cx in x..x + w {
            for cy in y..y + h {
                grid[(cy * tier.width + cx) as usize] = Some(u32::MAX);
            }
        }
        // Every item must have a free neighbour reachable from the door.
        let reach = reachable(tier, &grid);
        let mut items: Vec<(i32, i32, i32, i32)> = self
            .items
            .iter()
            .filter(|it| Some(it.uid) != ignore)
            .filter_map(|it| {
                let d = content.furniture(&it.item)?;
                let (w, h) = footprint(d, it.rotated);
                Some((it.x, it.y, w, h))
            })
            .collect();
        items.push((x, y, w, h));
        for (ix, iy, iw, ih) in items {
            if !touches_reachable(tier, &reach, ix, iy, iw, ih) {
                return Err(PlaceError::Unreachable);
            }
        }
        Ok(())
    }

    pub fn place(
        &mut self,
        content: &Content,
        item: &str,
        x: i32,
        y: i32,
        rotated: bool,
    ) -> Result<u32, PlaceError> {
        self.can_place(content, item, x, y, rotated, None)?;
        let uid = self.next_uid;
        self.next_uid += 1;
        self.items.push(PlacedItem { uid, item: item.to_string(), x, y, rotated });
        Ok(uid)
    }

    pub fn relocate(
        &mut self,
        content: &Content,
        uid: u32,
        x: i32,
        y: i32,
        rotated: bool,
    ) -> Result<(), PlaceError> {
        let item = self
            .items
            .iter()
            .find(|i| i.uid == uid)
            .map(|i| i.item.clone())
            .ok_or(PlaceError::UnknownItem)?;
        self.can_place(content, &item, x, y, rotated, Some(uid))?;
        if let Some(it) = self.items.iter_mut().find(|i| i.uid == uid) {
            it.x = x;
            it.y = y;
            it.rotated = rotated;
        }
        Ok(())
    }

    /// First free position for an item (row-major scan), if any.
    pub fn find_spot(&self, content: &Content, item: &str) -> Option<(i32, i32, bool)> {
        let tier = content.office(self.tier)?;
        for y in 0..tier.height {
            for x in 0..tier.width {
                for rot in [false, true] {
                    if self.can_place(content, item, x, y, rot, None).is_ok() {
                        return Some((x, y, rot));
                    }
                }
            }
        }
        None
    }

    /// Move into a bigger office, carrying the furniture along.
    pub fn move_to(&mut self, content: &Content, tier: u32) {
        let mut owned: Vec<String> = self.items.drain(..).map(|i| i.item).collect();
        owned.append(&mut self.storage);
        self.tier = tier;
        self.rent_extra = 0.0;
        if let Some(def) = content.office(tier) {
            for (x, y, item) in def.starter.clone() {
                let _ = self.place(content, &item, x, y, false);
            }
        }
        // Place big items first so they find room.
        owned.sort_by_key(|i| {
            content
                .furniture(i)
                .map(|d| -(d.size.0 * d.size.1))
                .unwrap_or(0)
        });
        for item in owned {
            match self.find_spot(content, &item) {
                Some((x, y, r)) => {
                    let _ = self.place(content, &item, x, y, r);
                }
                None => self.storage.push(item),
            }
        }
    }

    pub fn count(&self, item: &str) -> usize {
        self.items.iter().filter(|i| i.item == item).count()
    }

    pub fn effects(&self, content: &Content) -> OfficeEffects {
        let mut counts: BTreeMap<&str, u32> = BTreeMap::new();
        let mut fx = OfficeEffects::default();
        for it in &self.items {
            let Some(def) = content.furniture(&it.item) else { continue };
            let n = counts.entry(def.id.as_str()).or_insert(0);
            *n += 1;
            fx.monthly += def.monthly;
            fx.seats += def.effects.seats;
            if def.effects.seats > 0 && def.effects.eng_bonus > 0.0 {
                fx.workstations += 1;
            }
            // Beyond max_effective, an item only counts for seats.
            let w = if *n <= def.max_effective { 1.0 } else { 0.0 };
            let e = &def.effects;
            fx.morale += e.morale * w;
            fx.fatigue_recovery += e.fatigue_recovery * w;
            fx.eng_bonus += e.eng_bonus * w;
            fx.research_bonus += e.research_bonus * w;
            fx.sales_bonus += e.sales_bonus * w;
            fx.incident_risk += e.incident_risk * w;
            fx.inference_cost += e.inference_cost * w;
            fx.productivity += e.productivity * w;
        }
        fx
    }
}

fn reachable(tier: &OfficeTierDef, grid: &[Option<u32>]) -> Vec<bool> {
    let (w, h) = (tier.width, tier.height);
    let mut seen = vec![false; (w * h) as usize];
    let (dx, dy) = door(tier);
    let mut q = VecDeque::new();
    if grid[(dy * w + dx) as usize].is_none() {
        seen[(dy * w + dx) as usize] = true;
        q.push_back((dx, dy));
    }
    while let Some((x, y)) = q.pop_front() {
        for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            let i = (ny * w + nx) as usize;
            if !seen[i] && grid[i].is_none() {
                seen[i] = true;
                q.push_back((nx, ny));
            }
        }
    }
    seen
}

fn touches_reachable(tier: &OfficeTierDef, reach: &[bool], x: i32, y: i32, w: i32, h: i32) -> bool {
    for cx in x - 1..=x + w {
        for cy in y - 1..=y + h {
            let inside = cx >= x && cx < x + w && cy >= y && cy < y + h;
            let diagonal = (cx == x - 1 || cx == x + w) && (cy == y - 1 || cy == y + h);
            if inside || diagonal || cx < 0 || cy < 0 || cx >= tier.width || cy >= tier.height {
                continue;
            }
            if reach[(cy * tier.width + cx) as usize] {
                return true;
            }
        }
    }
    false
}

/// Free walkable cells reachable from the door (for client navigation).
pub fn walkable(content: &Content, office: &OfficeState) -> Vec<bool> {
    let Some(tier) = content.office(office.tier) else { return Vec::new() };
    let grid = office.occupancy(content, tier, None);
    reachable(tier, &grid)
}
