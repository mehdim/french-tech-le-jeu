//! Salariés animés. La simulation décide *ce que* fait chaque personne
//! (travail, réunion, épuisement, inactivité, panique) ; ce module décide
//! *à quoi ça ressemble* : trajet en grille jusqu'au bureau, pauses café,
//! réunions autour de la table, flânerie, et l'animation des sprites elle-même.

use crate::assets::{Atlas, depth_z, iso};
use crate::rendering::{OfficeSignature, Selection, anchored_sprite, furniture_z};
use crate::sim::{Game, GameContent, SimClock};
use bevy::prelude::*;
use game_core::employees::Activity;
use game_core::office::{door, footprint, walkable};
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Anim {
    Idle,
    Walk,
    Work,
    Coffee,
    Talk,
    Tired,
    Panic,
}

impl Anim {
    fn key(self) -> &'static str {
        match self {
            Anim::Idle => "idle",
            Anim::Walk => "walk",
            Anim::Work => "work",
            Anim::Coffee => "coffee",
            Anim::Talk => "talk",
            Anim::Tired => "tired",
            Anim::Panic => "panic",
        }
    }
    fn frame_time(self) -> f32 {
        match self {
            Anim::Walk => 0.13,
            Anim::Panic => 0.1,
            Anim::Work => 0.18,
            Anim::Idle | Anim::Tired => 0.55,
            Anim::Coffee | Anim::Talk => 0.4,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Poi {
    Coffee,
    Sofa,
    Whiteboard,
    Meeting,
    Fun,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Mode {
    /// En chemin vers quelque part ; `then` indique quoi faire à l'arrivée.
    Walking(Arrive),
    Seated,
    AtPoi(Poi),
    Wandering,
    Panic,
    Away,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Arrive {
    Seat,
    Poi(Poi),
    Wander,
}

#[derive(Component)]
pub struct EmployeeSprite {
    pub id: u32,
    pub row: u32,
    /// Position continue des pieds sur la grille.
    pub pos: Vec2,
    path: VecDeque<Vec2>,
    mode: Mode,
    pub anim: Anim,
    frame: u32,
    frame_t: f32,
    flip: bool,
    /// Secondes restantes avant la fin de la pause ou de la flânerie en cours.
    mode_t: f32,
    next_break: f32,
    pub bubble: Option<(&'static str, f32)>,
    seed: u32,
}

#[derive(Component)]
pub struct Shadow(pub u32);

#[derive(Component)]
pub struct Bubble(pub u32);

#[derive(Component)]
pub struct SelectMarker;

#[derive(Clone, Copy, Debug)]
pub struct Seat {
    pub point: Vec2,
    pub access: IVec2,
    pub z: f32,
}

/// Données de navigation reconstruites avec le bureau.
#[derive(Resource, Default)]
pub struct Nav {
    pub sig: u64,
    pub w: i32,
    pub h: i32,
    pub walk: Vec<bool>,
    pub seats: Vec<Seat>,
    pub pois: Vec<(Poi, IVec2, Vec2)>,
    pub door: IVec2,
}

impl Nav {
    fn ok(&self, c: IVec2) -> bool {
        c.x >= 0 && c.y >= 0 && c.x < self.w && c.y < self.h && self.walk[(c.y * self.w + c.x) as usize]
    }
    fn nearest_walkable(&self, p: Vec2) -> IVec2 {
        let c = p.floor().as_ivec2();
        if self.ok(c) {
            return c;
        }
        let mut best = (f32::MAX, self.door);
        for y in 0..self.h {
            for x in 0..self.w {
                let q = IVec2::new(x, y);
                if self.ok(q) {
                    let d = (q.as_vec2() + 0.5).distance_squared(p);
                    if d < best.0 {
                        best = (d, q);
                    }
                }
            }
        }
        best.1
    }
    /// Chemin en largeur d'abord entre cases praticables (les grilles sont minuscules : ≤ 18×12).
    fn path(&self, from: IVec2, to: IVec2) -> Option<Vec<IVec2>> {
        if !self.ok(from) || !self.ok(to) {
            return None;
        }
        let idx = |c: IVec2| (c.y * self.w + c.x) as usize;
        let mut prev = vec![None; (self.w * self.h) as usize];
        let mut seen = vec![false; (self.w * self.h) as usize];
        let mut q = VecDeque::new();
        seen[idx(from)] = true;
        q.push_back(from);
        while let Some(c) = q.pop_front() {
            if c == to {
                let mut out = vec![c];
                let mut cur = c;
                while let Some(p) = prev[idx(cur)] {
                    out.push(p);
                    cur = p;
                }
                out.reverse();
                return Some(out);
            }
            for d in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
                let n = c + d;
                if self.ok(n) && !seen[idx(n)] {
                    seen[idx(n)] = true;
                    prev[idx(n)] = Some(c);
                    q.push_back(n);
                }
            }
        }
        None
    }
    fn random_cell(&self, seed: u32) -> IVec2 {
        let cells: Vec<IVec2> = (0..self.h)
            .flat_map(|y| (0..self.w).map(move |x| IVec2::new(x, y)))
            .filter(|c| self.ok(*c))
            .collect();
        if cells.is_empty() {
            return self.door;
        }
        cells[(seed as usize) % cells.len()]
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Nav>().add_systems(
            Update,
            (rebuild_nav, sync_employees, behave, animate_sprites, place_sprites, update_marker)
                .chain()
                .after(crate::rendering::RenderingSet),
        );
    }
}

fn rebuild_nav(game: Res<Game>, content: Res<GameContent>, sig: Res<OfficeSignature>, mut nav: ResMut<Nav>) {
    if nav.sig == sig.0 && nav.w > 0 {
        return;
    }
    let c = &content.0;
    let Some(tier) = c.office(game.0.office.tier) else { return };
    nav.sig = sig.0;
    nav.w = tier.width;
    nav.h = tier.height;
    nav.walk = walkable(c, &game.0.office);
    let d = door(tier);
    nav.door = IVec2::new(d.0, d.1);
    // Places assises (bureaux et postes de travail) dans l'ordre des uid.
    let mut items: Vec<_> = game.0.office.items.iter().collect();
    items.sort_by_key(|i| i.uid);
    nav.seats.clear();
    nav.pois.clear();
    for it in items {
        let Some(def) = c.furniture(&it.item) else { continue };
        let (fw, fh) = footprint(def, it.rotated);
        let cells: Vec<IVec2> = (0..fw).flat_map(|dx| (0..fh).map(move |dy| IVec2::new(it.x + dx, it.y + dy))).collect();
        let neighbours: Vec<IVec2> = cells
            .iter()
            .flat_map(|c| [*c + IVec2::X, *c - IVec2::X, *c + IVec2::Y, *c - IVec2::Y])
            .filter(|n| !cells.contains(n) && nav.ok(*n))
            .collect();
        if def.effects.seats > 0 {
            let (point, preferred) = if it.rotated {
                (Vec2::new(it.x as f32 + 0.3, it.y as f32 + 1.0), IVec2::new(it.x - 1, it.y))
            } else {
                (Vec2::new(it.x as f32 + 1.0, it.y as f32 + 0.3), IVec2::new(it.x, it.y - 1))
            };
            let access = if nav.ok(preferred) { preferred } else { neighbours.first().copied().unwrap_or(nav.door) };
            let z = furniture_z(it.x, it.y, fw, fh) - 0.03;
            nav.seats.push(Seat { point, access, z });
        }
        let poi = match it.item.as_str() {
            "coffee_machine" | "coffee_moka" => Some(Poi::Coffee),
            "sofa" | "beanbag" => Some(Poi::Sofa),
            "whiteboard" => Some(Poi::Whiteboard),
            "meeting_table" => Some(Poi::Meeting),
            "pingpong" | "arcade" => Some(Poi::Fun),
            _ => None,
        };
        if let Some(p) = poi {
            let centre = Vec2::new(it.x as f32 + fw as f32 / 2.0, it.y as f32 + fh as f32 / 2.0);
            for n in neighbours.iter().take(4) {
                nav.pois.push((p, *n, centre));
            }
        }
    }
}

fn row_for(atlas: &Atlas, role: game_core::types::Role, look: u8) -> u32 {
    crate::assets::character_row(&atlas.manifest.characters, role, look)
}

fn sync_employees(
    mut commands: Commands,
    game: Res<Game>,
    atlas: Option<Res<Atlas>>,
    nav: Res<Nav>,
    q: Query<(Entity, &EmployeeSprite)>,
    shadows: Query<(Entity, &Shadow)>,
    bubbles: Query<(Entity, &Bubble)>,
    mut fx: ResMut<crate::fx::FxQueue>,
) {
    let Some(atlas) = atlas else { return };
    // Identifiant → ligne du sprite : une apparence qui change (nouvelle partie,
    // héroïne choisie) fait recréer le sprite.
    let ids: HashMap<u32, u32> = game.0.employees.iter().map(|e| (e.id, row_for(&atlas, e.role, e.look))).collect();
    let mut present: HashMap<u32, ()> = HashMap::new();
    for (ent, s) in &q {
        if ids.get(&s.id) != Some(&s.row) {
            if !ids.contains_key(&s.id) {
                fx.smoke(s.pos);
            }
            commands.entity(ent).despawn();
            for (se, sh) in &shadows {
                if sh.0 == s.id {
                    commands.entity(se).despawn();
                }
            }
            for (be, b) in &bubbles {
                if b.0 == s.id {
                    commands.entity(be).despawn();
                }
            }
        } else {
            present.insert(s.id, ());
        }
    }
    for (i, e) in game.0.employees.iter().enumerate() {
        if present.contains_key(&e.id) {
            continue;
        }
        // Le fondateur commence à son bureau ; les nouveaux arrivent par la porte.
        let start = if game.0.day < 2 || i == 0 {
            nav.seats.get(i).map(|s| s.point).unwrap_or(nav.door.as_vec2() + 0.5)
        } else {
            nav.door.as_vec2() + 0.5
        };
        let row = row_for(&atlas, e.role, e.look);
        let image = atlas.characters.clone();
        commands.spawn((
            Sprite { image, rect: Some(Rect::new(0.0, 0.0, 16.0, 26.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 10.0),
            EmployeeSprite {
                id: e.id,
                row,
                pos: start,
                path: VecDeque::new(),
                mode: if i == 0 && game.0.day < 2 { Mode::Seated } else { Mode::Wandering },
                anim: Anim::Idle,
                frame: 0,
                frame_t: 0.0,
                flip: false,
                mode_t: 0.0,
                next_break: 20.0 + (e.id * 7 % 30) as f32,
                bubble: None,
                seed: e.id.wrapping_mul(2654435761),
            },
        ));
        if let Some(meta) = atlas.furniture("fx_shadow") {
            let (spr, tf) = anchored_sprite(atlas.furniture.clone(), meta, Vec2::ZERO, 9.9, false);
            commands.spawn((spr, tf, Shadow(e.id)));
        }
    }
}

fn rng(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed % 10_000) as f32 / 10_000.0
}

fn employee_index(game: &Game, id: u32) -> Option<usize> {
    game.0.employees.iter().position(|e| e.id == id)
}

#[allow(clippy::too_many_arguments)]
fn behave(time: Res<Time>, clock: Res<SimClock>, game: Res<Game>, nav: Res<Nav>, mut q: Query<(&mut EmployeeSprite, &mut Visibility)>) {
    let dt = time.delta_secs();
    let tempo = match clock.speed {
        0 => 0.0,
        1 => 1.0,
        2 => 1.35,
        _ => 1.7,
    };
    for (mut s, mut vis) in &mut q {
        let Some(idx) = employee_index(&game, s.id) else { continue };
        let emp = &game.0.employees[idx];
        let activity = emp.activity;
        let seat = nav.seats.get(idx).copied();
        // Absent (en congé) : caché.
        if activity == Activity::OnLeave {
            *vis = Visibility::Hidden;
            s.mode = Mode::Away;
            continue;
        }
        if s.mode == Mode::Away {
            *vis = Visibility::Visible;
            s.pos = nav.door.as_vec2() + 0.5;
            s.mode = Mode::Wandering;
            s.path.clear();
        }
        // La panique passe avant tout.
        if activity == Activity::Panic && s.mode != Mode::Panic {
            s.mode = Mode::Panic;
            s.path.clear();
            s.bubble = Some(("bubble_alert", 2.5));
        } else if activity != Activity::Panic && s.mode == Mode::Panic {
            s.mode = Mode::Wandering;
            s.path.clear();
        }
        if tempo == 0.0 {
            // Jeu en pause : figé sur place (les poses d'attente restent animées).
            continue;
        }
        let speed = if s.mode == Mode::Panic { 2.8 } else { 1.7 } * tempo;
        // Suit le chemin en cours.
        if let Some(next) = s.path.front().copied() {
            let d = next - s.pos;
            let step = speed * dt;
            if d.length() <= step {
                s.pos = next;
                s.path.pop_front();
            } else {
                s.pos += d.normalize() * step;
            }
            let screen_dx = (d.x - d.y) * 16.0;
            if screen_dx.abs() > 0.01 {
                s.flip = screen_dx < 0.0;
            }
            s.anim = if s.mode == Mode::Panic { Anim::Panic } else { Anim::Walk };
            continue;
        }
        // Arrivé quelque part / décide de la suite.
        s.mode_t -= dt * tempo;
        s.next_break -= dt * tempo;
        let mut seed = s.seed;
        match s.mode {
            Mode::Walking(Arrive::Seat) => {
                if let Some(seat) = seat {
                    s.pos = seat.point;
                }
                s.mode = Mode::Seated;
                s.flip = false;
            }
            Mode::Walking(Arrive::Poi(p)) => {
                s.mode = Mode::AtPoi(p);
                s.mode_t = 4.0 + rng(&mut seed) * 4.0;
                if let Some((_, _, centre)) = nav.pois.iter().find(|x| x.0 == p) {
                    let d = *centre - s.pos;
                    s.flip = (d.x - d.y) < 0.0;
                }
            }
            Mode::Walking(Arrive::Wander) => {
                s.mode = Mode::Wandering;
                s.mode_t = 2.0 + rng(&mut seed) * 4.0;
            }
            _ => {}
        }
        let wants_seat = matches!(activity, Activity::Working | Activity::Exhausted)
            || (activity == Activity::Meeting && !nav.pois.iter().any(|p| p.0 == Poi::Meeting));
        let wants_meeting = activity == Activity::Meeting && nav.pois.iter().any(|p| p.0 == Poi::Meeting);
        let decide_new = match s.mode {
            Mode::Seated => !wants_seat || (s.next_break <= 0.0 && activity == Activity::Working),
            Mode::AtPoi(p) => s.mode_t <= 0.0 || (wants_meeting && p != Poi::Meeting) || (!wants_meeting && p == Poi::Meeting),
            Mode::Wandering => s.mode_t <= 0.0 || wants_seat || wants_meeting,
            Mode::Panic => true,
            _ => false,
        };
        if decide_new {
            let target: Option<(IVec2, Arrive)> = if s.mode == Mode::Panic {
                Some((nav.random_cell((rng(&mut seed) * 1000.0) as u32), Arrive::Wander))
            } else if s.mode == Mode::Seated && s.next_break <= 0.0 {
                s.next_break = 25.0 + rng(&mut seed) * 35.0;
                let pick = if rng(&mut seed) < 0.7 { Poi::Coffee } else { Poi::Whiteboard };
                poi_target(&nav, pick, &mut seed).map(|c| (c, Arrive::Poi(pick)))
            } else if wants_meeting {
                poi_target(&nav, Poi::Meeting, &mut seed).map(|c| (c, Arrive::Poi(Poi::Meeting)))
            } else if wants_seat && seat.is_some() {
                seat.map(|st| (st.access, Arrive::Seat))
            } else {
                // Inactif : flâne entre les points d'intérêt.
                let r = rng(&mut seed);
                let p = if r < 0.35 { Some(Poi::Coffee) } else if r < 0.55 { Some(Poi::Sofa) } else if r < 0.7 { Some(Poi::Fun) } else { None };
                match p.and_then(|p| poi_target(&nav, p, &mut seed).map(|c| (c, Arrive::Poi(p)))) {
                    Some(t) => Some(t),
                    None => Some((nav.random_cell((rng(&mut seed) * 1000.0) as u32), Arrive::Wander)),
                }
            };
            if let Some((cell, arrive)) = target {
                let from = if s.mode == Mode::Seated { seat.map(|st| st.access).unwrap_or(nav.door) } else { nav.nearest_walkable(s.pos) };
                if s.mode == Mode::Seated {
                    s.pos = from.as_vec2() + 0.5;
                }
                match nav.path(from, cell) {
                    Some(p) => {
                        s.path = p.into_iter().map(|c| c.as_vec2() + 0.5).collect();
                        if arrive == Arrive::Seat
                            && let Some(st) = seat {
                                s.path.push_back(st.point);
                            }
                        s.mode = if s.mode == Mode::Panic { Mode::Panic } else { Mode::Walking(arrive) };
                    }
                    None => {
                        // Inaccessible : téléportation, pour que le bureau reste vivant.
                        if arrive == Arrive::Seat {
                            if let Some(st) = seat {
                                s.pos = st.point;
                                s.mode = Mode::Seated;
                            }
                        } else {
                            s.mode = Mode::Wandering;
                            s.mode_t = 3.0;
                        }
                    }
                }
            } else {
                s.mode = Mode::Wandering;
                s.mode_t = 3.0;
            }
        }
        s.seed = seed;
        s.anim = match s.mode {
            Mode::Seated => match activity {
                Activity::Exhausted => Anim::Tired,
                Activity::Meeting => Anim::Talk,
                Activity::Idle => Anim::Idle,
                _ => Anim::Work,
            },
            Mode::AtPoi(Poi::Coffee) => Anim::Coffee,
            Mode::AtPoi(Poi::Meeting) | Mode::AtPoi(Poi::Whiteboard) | Mode::AtPoi(Poi::Fun) => Anim::Talk,
            Mode::AtPoi(Poi::Sofa) => if activity == Activity::Exhausted { Anim::Tired } else { Anim::Idle },
            Mode::Panic => Anim::Panic,
            _ => if activity == Activity::Exhausted { Anim::Tired } else { Anim::Idle },
        };
        if let Some((_, t)) = &mut s.bubble {
            *t -= dt;
            if *t <= 0.0 {
                s.bubble = None;
            }
        }
        if activity == Activity::Exhausted && s.bubble.is_none() && rng(&mut seed) < 0.002 {
            s.bubble = Some(("bubble_alert", 2.0));
        }
        s.seed = seed;
    }
}

fn poi_target(nav: &Nav, p: Poi, seed: &mut u32) -> Option<IVec2> {
    let cands: Vec<IVec2> = nav.pois.iter().filter(|x| x.0 == p).map(|x| x.1).collect();
    if cands.is_empty() {
        return None;
    }
    Some(cands[(rng(seed) * cands.len() as f32) as usize % cands.len()])
}

fn animate_sprites(time: Res<Time>, atlas: Option<Res<Atlas>>, mut q: Query<(&mut EmployeeSprite, &mut Sprite)>) {
    let Some(atlas) = atlas else { return };
    let meta = &atlas.manifest.characters;
    for (mut s, mut sprite) in &mut q {
        let [start, count] = meta.anims.get(s.anim.key()).copied().unwrap_or([0, 2]);
        s.frame_t += time.delta_secs();
        if s.frame_t >= s.anim.frame_time() {
            s.frame_t = 0.0;
            s.frame = (s.frame + 1) % count.max(1);
        }
        let f = start + (s.frame % count.max(1));
        let (fw, fh) = (meta.frame[0], meta.frame[1]);
        let x = f as f32 * fw;
        let y = s.row as f32 * fh;
        sprite.rect = Some(Rect::new(x, y, x + fw, y + fh));
        sprite.flip_x = s.flip;
    }
}

fn place_sprites(
    atlas: Option<Res<Atlas>>,
    nav: Res<Nav>,
    game: Res<Game>,
    mut commands: Commands,
    mut q: Query<(&EmployeeSprite, &mut Transform, &Visibility), (Without<Shadow>, Without<Bubble>)>,
    mut shadows: Query<(&Shadow, &mut Transform, &mut Visibility), (Without<EmployeeSprite>, Without<Bubble>)>,
    bubbles: Query<(Entity, &Bubble)>,
) {
    let Some(atlas) = atlas else { return };
    let meta = &atlas.manifest.characters;
    let anchor = Vec2::new(meta.anchor[0], meta.anchor[1]);
    let size = Vec2::new(meta.frame[0], meta.frame[1]);
    let mut feet: HashMap<u32, (Vec2, f32, bool, Option<&'static str>)> = HashMap::new();
    for (s, mut tf, vis) in &mut q {
        let idx = employee_index(&game, s.id).unwrap_or(0);
        let seated = s.path.is_empty() && nav.seats.get(idx).is_some_and(|st| st.point.distance(s.pos) < 0.05);
        let z = if seated { nav.seats[idx].z } else { depth_z(s.pos.x, s.pos.y) + 0.05 };
        let base = iso(s.pos.x, s.pos.y);
        let ax = if s.flip { size.x - anchor.x } else { anchor.x };
        let centre = base + Vec2::new(size.x / 2.0 - ax, size.y / 2.0 - (size.y - anchor.y));
        tf.translation = Vec3::new(centre.x.round(), centre.y.round(), z);
        feet.insert(s.id, (base, z, *vis != Visibility::Hidden && !seated, s.bubble.map(|b| b.0)));
    }
    for (sh, mut tf, mut vis) in &mut shadows {
        if let Some((base, z, show, _)) = feet.get(&sh.0) {
            tf.translation = Vec3::new(base.x.round(), base.y.round(), z - 0.01);
            *vis = if *show { Visibility::Visible } else { Visibility::Hidden };
        }
    }
    // Les bulles ne coûtent rien : on reconstruit à chaque image les quelques bulles présentes.
    for (e, _) in &bubbles {
        commands.entity(e).despawn();
    }
    for (id, (base, z, _, bubble)) in &feet {
        if let Some(key) = bubble
            && let Some(m) = atlas.furniture(key) {
                let (spr, tf) = anchored_sprite(atlas.furniture.clone(), m, *base + Vec2::new(4.0, 28.0), z + 0.5, false);
                commands.spawn((spr, tf, Bubble(*id)));
            }
    }
}

fn update_marker(
    mut commands: Commands,
    selection: Res<Selection>,
    atlas: Option<Res<Atlas>>,
    q: Query<&EmployeeSprite>,
    markers: Query<Entity, With<SelectMarker>>,
) {
    for m in &markers {
        commands.entity(m).despawn();
    }
    let (Some(atlas), Some(id)) = (atlas, selection.employee) else { return };
    let Some(s) = q.iter().find(|s| s.id == id) else { return };
    if let Some(meta) = atlas.furniture("fx_select") {
        let c = s.pos.floor();
        let (spr, tf) = anchored_sprite(atlas.furniture.clone(), meta, iso(c.x, c.y), 0.8, false);
        commands.spawn((spr, tf, SelectMarker));
    }
}

/// Fait afficher une bulle à un salarié (utilisé par les signaux : ventes, livraisons...).
pub fn show_bubble(q: &mut Query<&mut EmployeeSprite>, id: u32, key: &'static str) {
    for mut s in q.iter_mut() {
        if s.id == id {
            s.bubble = Some((key, 2.5));
        }
    }
}
