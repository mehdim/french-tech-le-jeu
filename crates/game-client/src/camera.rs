//! Caméra orthographique : zoom par paliers entiers (pixels nets), déplacement à la souris,
//! au clavier ou au toucher, cadrage automatique du bureau, et zone d'affichage limitée
//! à la partie non couverte par les panneaux de gestion.

use crate::assets::iso;
use crate::sim::{Game, GameContent};
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{CameraOutputMode, ClearColorConfig, Viewport};
use bevy::render::render_resource::BlendState;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub struct MainCamera;

/// Zone centrale de l'écran laissée libre par l'interface (pixels logiques), écrite par l'interface.
#[derive(Resource, Clone, Copy)]
pub struct WorldViewport {
    pub min: Vec2,
    pub max: Vec2,
    /// Vrai quand egui veut le pointeur à cette image (survol d'un panneau).
    pub pointer_over_ui: bool,
    pub keyboard_captured: bool,
}

impl Default for WorldViewport {
    fn default() -> Self {
        Self { min: Vec2::ZERO, max: Vec2::new(1280.0, 720.0), pointer_over_ui: false, keyboard_captured: false }
    }
}

#[derive(Resource)]
pub struct CameraState {
    pub zoom: f32,
    pub target: Vec2,
    pub framed_tier: Option<u32>,
    pub dragging: bool,
    pub drag_distance: f32,
    pub pinch_start: Option<(f32, f32)>,
}

impl Default for CameraState {
    fn default() -> Self {
        Self { zoom: 3.0, target: Vec2::ZERO, framed_tier: None, dragging: false, drag_distance: 0.0, pinch_start: None }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldViewport>()
            .init_resource::<CameraState>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, (frame_office, pan_zoom, apply_camera).chain());
    }
}

fn spawn_camera(mut commands: Commands, mut egui_settings: ResMut<bevy_egui::EguiGlobalSettings>) {
    // Caméra du monde : dessine le bureau dans la zone laissée libre par les panneaux.
    commands.spawn((Camera2d, MainCamera, Transform::from_xyz(0.0, 0.0, 100.0)));
    // Caméra d'interface : toute la fenêtre, ne dessine qu'egui, par-dessus.
    egui_settings.auto_create_primary_context = false;
    commands.spawn((
        bevy_egui::PrimaryEguiContext,
        Camera2d,
        RenderLayers::none(),
        Camera {
            order: 1,
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::ALPHA_BLENDING),
                clear_color: ClearColorConfig::None,
            },
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
    ));
}

/// Limites du bureau dans le monde.
pub fn office_bounds(content: &GameContent, game: &Game) -> Rect {
    let tier = content.0.office(game.0.office.tier);
    let (w, h) = tier.map(|t| (t.width as f32, t.height as f32)).unwrap_or((7.0, 6.0));
    let pts = [iso(0.0, 0.0), iso(w, 0.0), iso(w, h), iso(0.0, h)];
    let mut r = Rect::from_center_size(pts[0], Vec2::ZERO);
    for p in pts {
        r = r.union_point(p);
    }
    // Les murs s'élèvent au-dessus du coin du fond.
    r.max.y += 70.0;
    r
}

fn frame_office(mut state: ResMut<CameraState>, content: Res<GameContent>, game: Res<Game>, vp: Res<WorldViewport>) {
    let tier = game.0.office.tier;
    if state.framed_tier == Some(tier) {
        return;
    }
    let b = office_bounds(&content, &game);
    let avail = (vp.max - vp.min).max(Vec2::splat(200.0));
    let fit = (avail.x / (b.width() + 40.0)).min(avail.y / (b.height() + 40.0));
    state.zoom = fit.floor().clamp(1.0, 5.0);
    state.target = b.center();
    state.framed_tier = Some(tier);
}

#[allow(clippy::too_many_arguments)]
fn pan_zoom(
    mut state: ResMut<CameraState>,
    vp: Res<WorldViewport>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<MouseWheel>,
    mut motion: MessageReader<MouseMotion>,
    touches: Res<Touches>,
    time: Res<Time>,
    content: Res<GameContent>,
    game: Res<Game>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let cursor_in_world = window
        .cursor_position()
        .is_some_and(|c| c.x >= vp.min.x && c.x <= vp.max.x && c.y >= vp.min.y && c.y <= vp.max.y);
    // Zoom (des paliers entiers gardent les pixels nets).
    for ev in wheel.read() {
        if vp.pointer_over_ui || !cursor_in_world {
            continue;
        }
        let dir = ev.y.signum();
        state.zoom = (state.zoom + dir).clamp(1.0, 6.0);
    }
    if !vp.keyboard_captured {
        if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
            state.zoom = (state.zoom + 1.0).min(6.0);
        }
        if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
            state.zoom = (state.zoom - 1.0).max(1.0);
        }
        let mut dir = Vec2::ZERO;
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            dir.x += 1.0;
        }
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            dir.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            dir.y -= 1.0;
        }
        if dir != Vec2::ZERO {
            let z = state.zoom;
            state.target += dir.normalize() * 300.0 / z * time.delta_secs();
        }
    }
    // Glisser avec le bouton droit/milieu n'importe où, ou le bouton gauche sur un sol vide.
    let drag_button = mouse.pressed(MouseButton::Right) || mouse.pressed(MouseButton::Middle) || mouse.pressed(MouseButton::Left);
    if (mouse.just_pressed(MouseButton::Right) || mouse.just_pressed(MouseButton::Middle) || mouse.just_pressed(MouseButton::Left))
        && cursor_in_world
        && !vp.pointer_over_ui
    {
        state.dragging = true;
        state.drag_distance = 0.0;
    }
    if !drag_button {
        state.dragging = false;
    }
    let mut delta = Vec2::ZERO;
    for m in motion.read() {
        delta += m.delta;
    }
    if state.dragging && delta != Vec2::ZERO {
        state.drag_distance += delta.length();
        // Le glisser gauche ne déplace la vue que si c'est clairement un glisser, pas un clic.
        if state.drag_distance > 6.0 || !mouse.pressed(MouseButton::Left) {
            let z = state.zoom;
            state.target += Vec2::new(-delta.x, delta.y) / z;
        }
    }
    // Tactile : un doigt déplace, deux doigts zooment.
    let active: Vec<_> = touches.iter().collect();
    if active.len() == 1 {
        let t = active[0];
        let d = t.delta();
        if !vp.pointer_over_ui {
            let z = state.zoom;
            state.target += Vec2::new(-d.x, d.y) / z;
        }
        state.pinch_start = None;
    } else if active.len() >= 2 {
        let dist = active[0].position().distance(active[1].position());
        match state.pinch_start {
            None => state.pinch_start = Some((dist, state.zoom)),
            Some((d0, z0)) => {
                state.zoom = (z0 * dist / d0.max(1.0)).round().clamp(1.0, 6.0);
            }
        }
    } else {
        state.pinch_start = None;
    }
    // Garde le bureau dans le champ.
    let b = office_bounds(&content, &game);
    state.target = state.target.clamp(b.min, b.max);
}

fn apply_camera(
    state: Res<CameraState>,
    vp: Res<WorldViewport>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut cam: Query<(&mut Transform, &mut Projection, &mut Camera), With<MainCamera>>,
) {
    let Ok((mut tf, mut proj, mut camera)) = cam.single_mut() else { return };
    let scale = window.scale_factor();
    if let Projection::Orthographic(o) = proj.as_mut() {
        o.scale = 1.0 / (state.zoom * scale).max(0.25) * scale;
    }
    // Aligne sur des pixels entiers de l'écran pour éviter le scintillement.
    let snap = 1.0 / state.zoom;
    tf.translation.x = (state.target.x / snap).round() * snap;
    tf.translation.y = (state.target.y / snap).round() * snap;
    // Limite le rendu à la zone non couverte par les panneaux.
    let w = window.physical_width();
    let h = window.physical_height();
    let min = (vp.min * scale).as_uvec2().min(UVec2::new(w.saturating_sub(1), h.saturating_sub(1)));
    let max = (vp.max * scale).as_uvec2().min(UVec2::new(w, h));
    let size = max.saturating_sub(min).max(UVec2::ONE);
    camera.viewport = Some(Viewport { physical_position: min, physical_size: size, ..default() });
}
