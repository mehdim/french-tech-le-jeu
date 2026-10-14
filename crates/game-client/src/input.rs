//! Entrées pointeur et clavier sur le monde du jeu : sélection des salariés et du
//! mobilier, placement des meubles sur la grille, et raccourcis globaux.

use crate::animation::EmployeeSprite;
use crate::assets::world_to_grid;
use crate::camera::{CameraState, MainCamera, WorldViewport};
use crate::rendering::{FurnitureSprite, Placement, Selection};
use crate::sim::{Game, GameContent, SimClock};
use crate::states::AppState;
use crate::ui::{Tab, Toasts, UiState};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use game_core::Command;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorld>().add_systems(
            Update,
            (update_cursor, (placement_input, pick_input, shortcuts).run_if(in_state(AppState::Playing))).chain(),
        );
    }
}

/// Position du curseur dans le monde (None hors de la zone d'affichage du monde).
#[derive(Resource, Default)]
pub struct CursorWorld(pub Option<Vec2>);

fn update_cursor(
    window: Single<&Window, With<PrimaryWindow>>,
    cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    vp: Res<WorldViewport>,
    mut cw: ResMut<CursorWorld>,
) {
    cw.0 = None;
    let Some(cursor) = window.cursor_position() else { return };
    if vp.pointer_over_ui || cursor.x < vp.min.x || cursor.y < vp.min.y || cursor.x > vp.max.x || cursor.y > vp.max.y {
        return;
    }
    let Ok((camera, gtf)) = cam.single() else { return };
    if let Ok(p) = camera.viewport_to_world_2d(gtf, cursor) {
        cw.0 = Some(p);
    }
}

fn placement_input(
    mut placement: ResMut<Placement>,
    cw: Res<CursorWorld>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    cam_state: Res<CameraState>,
    mut game: ResMut<Game>,
    content: Res<GameContent>,
    mut toasts: ResMut<Toasts>,
) {
    let Some(item) = placement.item.clone() else { return };
    if keys.just_pressed(KeyCode::KeyR) {
        placement.rotated = !placement.rotated;
    }
    if keys.just_pressed(KeyCode::Escape) || mouse.just_pressed(MouseButton::Right) {
        *placement = Placement::default();
        return;
    }
    let Some(p) = cw.0 else {
        placement.hover = None;
        return;
    };
    let g = world_to_grid(p).floor().as_ivec2();
    placement.hover = Some(g);
    let check = game.0.office.can_place(&content.0, &item, g.x, g.y, placement.rotated, placement.moving_uid);
    placement.valid = check.is_ok();
    placement.reason = check.err().map(|e| e.label().to_string()).unwrap_or_default();
    if mouse.just_released(MouseButton::Left) && cam_state.drag_distance < 6.0 {
        let cmd = if let Some(uid) = placement.moving_uid {
            Command::MoveFurniture { uid, x: g.x, y: g.y, rotated: placement.rotated }
        } else if let Some(index) = placement.stored_index {
            Command::PlaceStored { index, x: g.x, y: g.y, rotated: placement.rotated }
        } else {
            Command::PlaceFurniture { item: item.clone(), x: g.x, y: g.y, rotated: placement.rotated }
        };
        match game.0.apply(&content.0, cmd) {
            Ok(()) => {
                // Continue de poser des copies des nouveaux achats tant que Maj est enfoncée.
                let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
                if !(shift && placement.moving_uid.is_none() && placement.stored_index.is_none()) {
                    *placement = Placement::default();
                }
            }
            Err(e) => toasts.push_bad(e.label()),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn pick_input(
    placement: Res<Placement>,
    cw: Res<CursorWorld>,
    mouse: Res<ButtonInput<MouseButton>>,
    cam_state: Res<CameraState>,
    employees: Query<(&EmployeeSprite, &Transform, &Visibility)>,
    furniture: Query<(&FurnitureSprite, &Transform, &Sprite)>,
    mut selection: ResMut<Selection>,
    mut ui: ResMut<UiState>,
) {
    if placement.item.is_some() || !mouse.just_released(MouseButton::Left) || cam_state.drag_distance >= 6.0 {
        return;
    }
    let Some(p) = cw.0 else { return };
    // Les salariés d'abord (ils sont devant le mobilier).
    let mut best: Option<(f32, u32)> = None;
    for (s, tf, vis) in &employees {
        if *vis == Visibility::Hidden {
            continue;
        }
        let c = tf.translation.truncate();
        let half = Vec2::new(7.0, 13.0);
        if (p - c).abs().cmple(half).all() && best.is_none_or(|b| tf.translation.z > b.0) {
            best = Some((tf.translation.z, s.id));
        }
    }
    if let Some((_, id)) = best {
        selection.employee = Some(id);
        selection.furniture = None;
        ui.inspect_employee = Some(id);
        ui.tab = Tab::Team;
        ui.right_open = true;
        return;
    }
    let mut bestf: Option<(f32, u32, String)> = None;
    for (f, tf, sprite) in &furniture {
        let size = sprite.rect.map(|r| r.size()).unwrap_or(Vec2::splat(16.0));
        let c = tf.translation.truncate();
        if (p - c).abs().cmple(size / 2.0).all() && bestf.as_ref().is_none_or(|b| tf.translation.z > b.0) {
            bestf = Some((tf.translation.z, f.uid, f.item.clone()));
        }
    }
    if let Some((_, uid, item)) = bestf {
        selection.furniture = Some(uid);
        selection.employee = None;
        ui.inspect_furniture = Some(uid);
        // Cliquer sur le tableau blanc ouvre la R&D, une table de réunion ouvre les ventes, etc.
        ui.tab = match item.as_str() {
            "whiteboard" | "gpu_server" | "bookshelf_item" => Tab::Product,
            "meeting_table" => Tab::Sales,
            "server_rack" | "k8s_cluster" => Tab::Finance,
            _ => Tab::Office,
        };
        ui.right_open = true;
        return;
    }
    selection.employee = None;
    selection.furniture = None;
}

fn shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    vp: Res<WorldViewport>,
    mut clock: ResMut<SimClock>,
    mut next: ResMut<NextState<AppState>>,
    placement: Res<Placement>,
    mut ui: ResMut<UiState>,
    game: Res<Game>,
    mut dialog_last_frame: Local<bool>,
) {
    if vp.keyboard_captured {
        return;
    }
    // Quand un dialogue ou un événement est ouvert, Entrée et 1-4 lui appartiennent.
    // (Aussi pendant une image après sa fermeture, pour que la touche qui y a répondu
    // ne change pas aussi la vitesse.)
    let open_now = game.0.campaign.intro_pending || game.0.events.pending.is_some();
    let dialog_open = open_now || *dialog_last_frame;
    *dialog_last_frame = open_now;
    if keys.just_pressed(KeyCode::Space) {
        clock.toggle_pause();
    }
    if !dialog_open {
        if keys.just_pressed(KeyCode::Digit1) {
            clock.set_speed(1);
        }
        if keys.just_pressed(KeyCode::Digit2) {
            clock.set_speed(2);
        }
        if keys.just_pressed(KeyCode::Digit3) || keys.just_pressed(KeyCode::Digit4) {
            clock.set_speed(4);
        }
    }
    if keys.just_pressed(KeyCode::Escape) && placement.item.is_none() {
        next.set(AppState::Paused);
    }
    let tabs = [
        (KeyCode::KeyE, Tab::Team),
        (KeyCode::KeyP, Tab::Product),
        (KeyCode::KeyV, Tab::Sales),
        (KeyCode::KeyF, Tab::Funding),
        (KeyCode::KeyC, Tab::Finance),
        (KeyCode::KeyB, Tab::Office),
    ];
    for (k, t) in tabs {
        if keys.just_pressed(k) {
            if ui.tab == t && ui.right_open {
                ui.right_open = false;
            } else {
                ui.tab = t;
                ui.right_open = true;
            }
        }
    }
}
