//! FRENCH TECH TYCOON — Bevy presentation layer.
//!
//! The simulation lives in `game-core` (pure Rust, deterministic). This crate
//! renders it: an isometric pixel-art office, animated employees, an egui
//! management interface, audio, saves and menus.

// Bevy systems routinely take many parameters and nested query types.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod animation;
mod assets;
#[cfg(feature = "audio")]
mod audio;
mod camera;
mod dev;
mod fx;
mod input;
mod rendering;
mod saves;
mod sim;
mod states;
mod ui;
#[cfg(target_arch = "wasm32")]
mod web;

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_egui::EguiPlugin;

fn asset_root() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        "assets".to_string()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(p) = std::env::var("FTT_ASSETS") {
            return p;
        }
        let cwd = std::env::current_dir().unwrap_or_default();
        if cwd.join("assets/sprites/atlas.json").exists() {
            return cwd.join("assets").to_string_lossy().into_owned();
        }
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
                && dir.join("assets/sprites/atlas.json").exists() {
                    return dir.join("assets").to_string_lossy().into_owned();
                }
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets").to_string()
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "French Tech Tycoon — From France Travail to Unicorn".into(),
                    resolution: WindowResolution::from((1280u32, 720u32)),
                    canvas: Some("#game-canvas".into()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                file_path: asset_root(),
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(EguiPlugin::default())
    .insert_resource(ClearColor(Color::srgb_u8(16, 20, 18)))
    .add_plugins((
        states::StatesPlugin,
        sim::SimPlugin,
        assets::AssetsPlugin,
        camera::CameraPlugin,
        rendering::RenderingPlugin,
        animation::AnimationPlugin,
        input::InputPlugin,
        fx::FxPlugin,
        saves::SavesPlugin,
        ui::UiPlugin,
        dev::DevPlugin,
    ));
    #[cfg(feature = "audio")]
    app.add_plugins(audio::GameAudioPlugin);
    #[cfg(target_arch = "wasm32")]
    app.add_plugins(web::WebPlugin);
    app.run();
}
