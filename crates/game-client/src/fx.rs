//! Effets : textes flottants, particules et bulles déclenchés par les
//! signaux de la simulation (clients signés, fonctionnalités livrées, incidents, argent).

use crate::animation::{EmployeeSprite, Nav, Poi, show_bubble};
use crate::assets::{Atlas, iso};
use crate::rendering::anchored_sprite;
use crate::sim::{Cues, Game, GameContent};
use bevy::prelude::*;
use game_core::employees::Assignment;
use game_core::messages::Cue;

pub struct FloatText {
    pub world: Vec2,
    pub text: String,
    pub color: [u8; 3],
    pub ttl: f32,
    pub age: f32,
    pub big: bool,
}

#[derive(Resource, Default)]
pub struct FxQueue {
    pub texts: Vec<FloatText>,
    pending_particles: Vec<(Vec2, &'static str, u32)>,
}

impl FxQueue {
    pub fn text(&mut self, grid: Vec2, text: impl Into<String>, color: [u8; 3], big: bool) {
        self.texts.push(FloatText { world: iso(grid.x, grid.y) + Vec2::new(0.0, 30.0), text: text.into(), color, ttl: 2.2, age: 0.0, big });
    }
    pub fn smoke(&mut self, grid: Vec2) {
        self.pending_particles.push((grid, "fx_smoke", 5));
    }
    pub fn burst(&mut self, grid: Vec2, key: &'static str, n: u32) {
        self.pending_particles.push((grid, key, n));
    }
}

#[derive(Component)]
pub struct Particle {
    vel: Vec2,
    ttl: f32,
}

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FxQueue>()
            .insert_resource(SteamTimer(Timer::from_seconds(0.7, TimerMode::Repeating)))
            .add_systems(Update, (react_to_cues, coffee_steam, spawn_particles, update_particles, age_texts).chain());
    }
}

#[derive(Resource)]
struct SteamTimer(Timer);

fn pos_of(q: &Query<&mut EmployeeSprite>, id: u32) -> Option<Vec2> {
    q.iter().find(|s| s.id == id).map(|s| s.pos)
}

fn react_to_cues(
    cues: Res<Cues>,
    game: Res<Game>,
    content: Res<GameContent>,
    nav: Res<Nav>,
    mut fx: ResMut<FxQueue>,
    mut sprites: Query<&mut EmployeeSprite>,
) {
    let centre = Vec2::new(nav.w as f32 / 2.0, nav.h as f32 / 2.0);
    for cue in &cues.0 {
        match cue {
            Cue::NewCustomers(n) => {
                // Attribue la vente à un commercial (ou au fondateur).
                let seller = game
                    .0
                    .employees
                    .iter()
                    .find(|e| e.assignment == Assignment::Sales)
                    .or_else(|| game.0.employees.first())
                    .map(|e| e.id);
                let at = seller.and_then(|id| pos_of(&sprites, id)).unwrap_or(nav.door.as_vec2() + 0.5);
                let label = if *n == 1 { "+1 client".to_string() } else { format!("+{n} clients") };
                fx.text(at, label, [124, 255, 82], false);
                fx.burst(at, "fx_euro", (*n).min(4));
                if let Some(id) = seller {
                    show_bubble(&mut sprites, id, "bubble_money");
                }
            }
            Cue::Churned(n) => {
                let at = nav.door.as_vec2() + 0.5;
                fx.text(at, format!("-{n} client{}", if *n > 1 { "s" } else { "" }), [228, 91, 91], false);
            }
            Cue::Shipped(task) => {
                let worker = game.0.employees.iter().find(|e| e.role.is_engineering()).map(|e| e.id);
                let at = worker.and_then(|id| pos_of(&sprites, id)).unwrap_or(centre);
                let name = content.0.task(task).map(|t| t.name.clone()).unwrap_or_default();
                let short: String = name.chars().take(28).collect();
                fx.text(at, format!("Livré : {short}"), [124, 255, 82], false);
                fx.burst(at, "fx_spark", 6);
                if let Some(id) = worker {
                    show_bubble(&mut sprites, id, "bubble_idea");
                }
            }
            Cue::Incident => {
                for s in sprites.iter().take(3) {
                    fx.smoke(s.pos);
                }
                fx.text(centre, "INCIDENT DE PROD", [228, 91, 91], true);
            }
            Cue::Funding(amount) => {
                fx.text(centre, format!("+{}", game_content::dialogue::money(*amount)), [124, 255, 82], true);
                fx.burst(centre, "fx_spark", 12);
                fx.burst(centre, "fx_euro", 8);
            }
            Cue::Hired(id) => {
                show_bubble(&mut sprites, *id, "bubble_heart");
            }
            Cue::LevelComplete | Cue::Victory => {
                fx.burst(centre, "fx_spark", 20);
            }
            _ => {}
        }
    }
}

fn coffee_steam(time: Res<Time>, mut timer: ResMut<SteamTimer>, nav: Res<Nav>, mut fx: ResMut<FxQueue>) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    let mut seen = Vec::new();
    for (p, _, centre) in &nav.pois {
        if *p == Poi::Coffee && !seen.contains(centre) {
            seen.push(*centre);
            fx.pending_particles.push((*centre, "fx_steam", 1));
        }
    }
}

fn spawn_particles(mut commands: Commands, mut fx: ResMut<FxQueue>, atlas: Option<Res<Atlas>>, time: Res<Time>) {
    let Some(atlas) = atlas else { return };
    let t = time.elapsed_secs();
    for (i, (grid, key, n)) in fx.pending_particles.drain(..).enumerate() {
        let Some(meta) = atlas.furniture(key) else { continue };
        for k in 0..n {
            let a = (t * 13.7 + i as f32 * 3.1 + k as f32 * 1.9).sin();
            let b = (t * 7.3 + k as f32 * 2.3).cos();
            let (vel, ttl, lift) = match key {
                "fx_steam" => (Vec2::new(a * 3.0, 14.0), 1.4, 22.0),
                "fx_smoke" => (Vec2::new(a * 10.0, 18.0 + b * 5.0), 1.6, 12.0),
                _ => (Vec2::new(a * 30.0, 30.0 + b * 15.0), 1.1, 18.0),
            };
            let (spr, tf) = anchored_sprite(atlas.furniture.clone(), meta, iso(grid.x, grid.y) + Vec2::new(0.0, lift), 50.0, false);
            commands.spawn((spr, tf, Particle { vel, ttl }));
        }
    }
}

fn update_particles(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>) {
    let dt = time.delta_secs();
    for (e, mut p, mut tf, mut s) in &mut q {
        p.ttl -= dt;
        if p.ttl <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        tf.translation.x += p.vel.x * dt;
        tf.translation.y += p.vel.y * dt;
        p.vel.y -= 20.0 * dt;
        s.color = s.color.with_alpha((p.ttl / 1.2).clamp(0.0, 1.0));
    }
}

fn age_texts(time: Res<Time>, mut fx: ResMut<FxQueue>) {
    let dt = time.delta_secs();
    for t in &mut fx.texts {
        t.age += dt;
        t.world.y += 12.0 * dt;
    }
    fx.texts.retain(|t| t.age < t.ttl);
    if fx.texts.len() > 30 {
        let excess = fx.texts.len() - 30;
        fx.texts.drain(0..excess);
    }
}
