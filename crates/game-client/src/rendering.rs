//! Isometric office rendering. The office is rebuilt from the simulation
//! whenever its layout changes (move, placement, decor unlocked).

use crate::assets::{Atlas, depth_z, iso};
use crate::sim::{Game, GameContent};
use bevy::prelude::*;
use game_core::office::footprint;

#[derive(Component)]
pub struct OfficeEntity;

/// A furniture sprite, linked to its simulation instance.
#[derive(Component)]
pub struct FurnitureSprite {
    pub uid: u32,
    pub item: String,
}

/// Fingerprint of everything that affects the office's appearance.
#[derive(Resource, Default, PartialEq)]
pub struct OfficeSignature(pub u64);

/// Furniture placement mode (driven by the Bureau panel and input).
#[derive(Resource, Default)]
pub struct Placement {
    /// Item being placed (new purchase) or moved (existing uid).
    pub item: Option<String>,
    pub moving_uid: Option<u32>,
    pub stored_index: Option<usize>,
    pub rotated: bool,
    /// Grid cell under the cursor.
    pub hover: Option<IVec2>,
    pub valid: bool,
    pub reason: String,
}

/// Currently selected things (shown with a highlight).
#[derive(Resource, Default)]
pub struct Selection {
    pub employee: Option<u32>,
    pub furniture: Option<u32>,
}

#[derive(Component)]
pub struct PlacementPreview;

/// Office (re)building runs before employee animation.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderingSet;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OfficeSignature>()
            .init_resource::<Placement>()
            .init_resource::<Selection>()
            .add_systems(Update, (sync_office, draw_placement_preview, highlight_furniture).in_set(RenderingSet));
    }
}

fn signature(game: &Game) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    game.0.office.tier.hash(&mut h);
    for it in &game.0.office.items {
        (it.uid, &it.item, it.x, it.y, it.rotated).hash(&mut h);
    }
    game.0.flags.contains("bpi_certificate").hash(&mut h);
    h.finish()
}

/// Sprite placed so that its manifest anchor lands on `anchor_world`.
pub fn anchored_sprite(image: Handle<Image>, meta: &crate::assets::SpriteMeta, anchor_world: Vec2, z: f32, flip: bool) -> (Sprite, Transform) {
    let sprite = Sprite { image, rect: Some(meta.bevy_rect()), flip_x: flip, ..default() };
    let pos = anchor_world + meta.center_offset(flip);
    (sprite, Transform::from_xyz(pos.x, pos.y, z))
}

pub fn furniture_z(x: i32, y: i32, w: i32, h: i32) -> f32 {
    depth_z((x + w) as f32 - 0.5, (y + h) as f32 - 0.5)
}

fn sync_office(
    mut commands: Commands,
    game: Res<Game>,
    content: Res<GameContent>,
    atlas: Option<Res<Atlas>>,
    mut sig: ResMut<OfficeSignature>,
    existing: Query<Entity, With<OfficeEntity>>,
) {
    let Some(atlas) = atlas else { return };
    let s = signature(&game);
    if sig.0 == s {
        return;
    }
    sig.0 = s;
    for e in &existing {
        commands.entity(e).despawn();
    }
    let c = &content.0;
    let Some(tier) = c.office(game.0.office.tier) else { return };
    let (w, h) = (tier.width, tier.height);

    // Floor.
    let floor = format!("{:?}", tier.floor);
    for y in 0..h {
        for x in 0..w {
            let v = ((x * 7 + y * 3) % 2) as usize;
            if let Some(meta) = atlas.office(&format!("floor_{floor}_{v}")) {
                let (spr, tf) = anchored_sprite(atlas.office.clone(), meta, iso(x as f32 + 0.5, y as f32 + 0.5) + Vec2::new(0.0, 8.0), 0.0, false);
                commands.spawn((spr, tf, OfficeEntity));
            }
        }
    }
    // Doormat at the entrance.
    let door = game_core::office::door(tier);
    if let Some(meta) = atlas.furniture("doormat") {
        let (spr, tf) = anchored_sprite(atlas.furniture.clone(), meta, iso(door.0 as f32, door.1 as f32), 0.5, false);
        commands.spawn((spr, tf, OfficeEntity));
    }
    // Back walls: left wall along x = 0, right wall along y = 0.
    let wall_tier = tier.id.min(5);
    let is_window = |i: i32| i % 3 == 1;
    for y in 0..h {
        let kind = if is_window(y) { "window" } else { "plain" };
        if let Some(meta) = atlas.office(&format!("wall_{wall_tier}_L_{kind}")) {
            let (spr, tf) = anchored_sprite(atlas.office.clone(), meta, iso(0.0, y as f32), 1.0 + y as f32 * 0.001, false);
            commands.spawn((spr, tf, OfficeEntity));
        }
    }
    for x in 0..w {
        let kind = if is_window(x) { "window" } else { "plain" };
        if let Some(meta) = atlas.office(&format!("wall_{wall_tier}_R_{kind}")) {
            let (spr, tf) = anchored_sprite(atlas.office.clone(), meta, iso(x as f32, 0.0), 1.0 + x as f32 * 0.001, false);
            commands.spawn((spr, tf, OfficeEntity));
        }
    }
    // Wall decorations on segments without windows, alternating walls.
    let mut decor: Vec<String> = tier.wall_decor.clone();
    if game.0.flags.contains("bpi_certificate") {
        decor.push("bpi_certificate".into());
    }
    let mut left_slots: Vec<i32> = (0..h).filter(|i| !is_window(*i)).collect();
    let mut right_slots: Vec<i32> = (1..w).filter(|i| !is_window(*i)).collect();
    left_slots.reverse();
    right_slots.reverse();
    for (i, name) in decor.iter().enumerate() {
        if name == "window_roofs" || name.starts_with("window") {
            continue;
        }
        let on_left = i % 2 == 0;
        let slot = if on_left { left_slots.pop() } else { right_slots.pop() };
        let Some(slot) = slot else { continue };
        let key = format!("{name}_{}", if on_left { "L" } else { "R" });
        let Some(meta) = atlas.decor(&key) else { continue };
        // Anchor: top of the decoration, hung ~44 px above the floor line.
        let base = if on_left {
            iso(0.0, slot as f32 + 0.1)
        } else {
            iso(slot as f32 + 0.1, 0.0)
        };
        let z = 2.0 + i as f32 * 0.001;
        let (spr, tf) = anchored_sprite(atlas.decor.clone(), meta, base + Vec2::new(0.0, 46.0), z, false);
        commands.spawn((spr, tf, OfficeEntity));
    }
    // Furniture.
    for it in &game.0.office.items {
        let Some(def) = c.furniture(&it.item) else { continue };
        let Some(meta) = atlas.furniture(&def.sprite) else { continue };
        let (fw, fh) = footprint(def, it.rotated);
        let z = furniture_z(it.x, it.y, fw, fh);
        let (spr, tf) = anchored_sprite(atlas.furniture.clone(), meta, iso(it.x as f32, it.y as f32), z, it.rotated);
        commands.spawn((spr, tf, OfficeEntity, FurnitureSprite { uid: it.uid, item: it.item.clone() }));
    }
}

fn draw_placement_preview(
    mut commands: Commands,
    placement: Res<Placement>,
    content: Res<GameContent>,
    atlas: Option<Res<Atlas>>,
    previews: Query<Entity, With<PlacementPreview>>,
) {
    for e in &previews {
        commands.entity(e).despawn();
    }
    let (Some(atlas), Some(item), Some(cell)) = (atlas, placement.item.as_ref(), placement.hover) else { return };
    let Some(def) = content.0.furniture(item) else { return };
    let (w, h) = footprint(def, placement.rotated);
    let tile_key = if placement.valid { "fx_place_ok" } else { "fx_place_bad" };
    if let Some(meta) = atlas.furniture(tile_key) {
        for dx in 0..w {
            for dy in 0..h {
                let (spr, tf) = anchored_sprite(atlas.furniture.clone(), meta, iso((cell.x + dx) as f32, (cell.y + dy) as f32), 9.0, false);
                commands.spawn((spr, tf, PlacementPreview));
            }
        }
    }
    if let Some(meta) = atlas.furniture(&def.sprite) {
        let (mut spr, tf) = anchored_sprite(atlas.furniture.clone(), meta, iso(cell.x as f32, cell.y as f32), 60.0, placement.rotated);
        spr.color = if placement.valid { Color::srgba(1.0, 1.0, 1.0, 0.75) } else { Color::srgba(1.0, 0.5, 0.5, 0.6) };
        commands.spawn((spr, tf, PlacementPreview));
    }
}

fn highlight_furniture(selection: Res<Selection>, mut q: Query<(&FurnitureSprite, &mut Sprite)>) {
    if !selection.is_changed() {
        return;
    }
    for (f, mut s) in &mut q {
        s.color = if Some(f.uid) == selection.furniture {
            Color::srgb(0.75, 1.0, 0.65)
        } else {
            Color::WHITE
        };
    }
}
