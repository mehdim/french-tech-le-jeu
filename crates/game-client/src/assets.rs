//! Atlas de textures et leur manifeste (`assets/sprites/atlas.json`).
//!
//! Les images sont chargées par l'AssetServer ; le manifeste (rectangles et
//! ancres isométriques, produit par `tools/art/gen_all.py`) est intégré au binaire.

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

const MANIFEST: &str = include_str!("../../../assets/sprites/atlas.json");

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct SpriteMeta {
    pub rect: [f32; 4],
    #[serde(default)]
    pub anchor: [f32; 2],
}

impl SpriteMeta {
    pub fn bevy_rect(&self) -> Rect {
        Rect::new(self.rect[0], self.rect[1], self.rect[0] + self.rect[2], self.rect[1] + self.rect[3])
    }
    /// Décalage entre le point d'ancrage (dans le monde, y vers le haut) et le centre
    /// du sprite. `flip` inverse l'ancre horizontalement.
    pub fn center_offset(&self, flip: bool) -> Vec2 {
        let (w, h) = (self.rect[2], self.rect[3]);
        let ax = if flip { w - self.anchor[0] } else { self.anchor[0] };
        Vec2::new(w / 2.0 - ax, -(h / 2.0 - self.anchor[1]))
    }
}

#[derive(Deserialize, Debug)]
pub struct CharacterMeta {
    pub frame: [f32; 2],
    pub looks: u32,
    pub roles: Vec<String>,
    pub anims: HashMap<String, [u32; 2]>,
    pub anchor: [f32; 2],
}

#[derive(Deserialize, Debug)]
pub struct Manifest {
    pub office: HashMap<String, SpriteMeta>,
    pub furniture: HashMap<String, SpriteMeta>,
    pub decor: HashMap<String, SpriteMeta>,
    pub characters: CharacterMeta,
    pub portraits: HashMap<String, SpriteMeta>,
    pub icons: HashMap<String, SpriteMeta>,
}

#[derive(Resource)]
pub struct Atlas {
    pub manifest: Manifest,
    pub office: Handle<Image>,
    pub furniture: Handle<Image>,
    pub decor: Handle<Image>,
    pub characters: Handle<Image>,
    pub portraits: Handle<Image>,
    pub icons: Handle<Image>,
    pub title: Handle<Image>,
    pub endings: HashMap<String, Handle<Image>>,
}

impl Atlas {
    pub fn furniture(&self, key: &str) -> Option<&SpriteMeta> {
        self.manifest.furniture.get(key)
    }
    pub fn office(&self, key: &str) -> Option<&SpriteMeta> {
        self.manifest.office.get(key)
    }
    pub fn decor(&self, key: &str) -> Option<&SpriteMeta> {
        self.manifest.decor.get(key)
    }
}

pub fn parse_manifest() -> Manifest {
    serde_json::from_str(MANIFEST).expect("atlas.json est généré par tools/art/gen_all.py")
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_atlas);
    }
}

fn load_atlas(mut commands: Commands, server: Res<AssetServer>) {
    let endings = ["ending_unicorn", "ending_profitable", "ending_acquisition", "ending_commons"]
        .iter()
        .map(|k| (k.to_string(), server.load(format!("backgrounds/{k}.png"))))
        .collect();
    commands.insert_resource(Atlas {
        manifest: parse_manifest(),
        office: server.load("tilesets/office.png"),
        furniture: server.load("sprites/furniture.png"),
        decor: server.load("sprites/decor.png"),
        characters: server.load("characters/characters.png"),
        portraits: server.load("ui/portraits.png"),
        icons: server.load("ui/icons.png"),
        title: server.load("backgrounds/title.png"),
        endings,
    });
}

/// Projection isométrique d'un point de la grille vers le monde (y vers le haut).
pub fn iso(gx: f32, gy: f32) -> Vec2 {
    Vec2::new((gx - gy) * 16.0, -(gx + gy) * 8.0)
}

/// Projection inverse : point du monde vers coordonnées continues de la grille.
pub fn world_to_grid(p: Vec2) -> Vec2 {
    let a = p.x / 16.0;
    let b = -p.y / 8.0;
    Vec2::new((a + b) / 2.0, (b - a) / 2.0)
}

/// Ordre de dessin pour un objet posé au point de grille (gx, gy).
pub fn depth_z(gx: f32, gy: f32) -> f32 {
    10.0 + (gx + gy) * 0.1
}
