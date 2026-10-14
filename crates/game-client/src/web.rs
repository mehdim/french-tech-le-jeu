//! Intégration navigateur (wasm32 uniquement).
//!
//! Le chargeur HTML (`web/loader.js`) affiche un écran de chargement pendant que le module
//! wasm se télécharge. Une fois que Bevy a chargé les atlas de textures et dessiné quelques
//! images, on envoie un événement DOM `ftt-ready` pour que le chargeur fasse disparaître
//! l'écran de chargement sans révéler un canvas vide.

use crate::assets::Atlas;
use bevy::asset::UntypedAssetId;
use bevy::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function ftt_ready() {
  window.dispatchEvent(new CustomEvent('ftt-ready'));
}
export function ftt_progress(loaded, total) {
  window.dispatchEvent(new CustomEvent('ftt-progress', { detail: { loaded, total } }));
}
")]
extern "C" {
    fn ftt_ready();
    fn ftt_progress(loaded: u32, total: u32);
}

pub struct WebPlugin;

impl Plugin for WebPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, signal_ready);
    }
}

#[derive(Default)]
struct ReadyState {
    done: bool,
    frames_after_load: u32,
    last_loaded: u32,
}

fn signal_ready(atlas: Option<Res<Atlas>>, server: Res<AssetServer>, mut st: Local<ReadyState>) {
    if st.done {
        return;
    }
    let Some(atlas) = atlas else { return };
    let mut handles: Vec<UntypedAssetId> = vec![
        atlas.office.id().untyped(),
        atlas.furniture.id().untyped(),
        atlas.decor.id().untyped(),
        atlas.characters.id().untyped(),
        atlas.portraits.id().untyped(),
        atlas.icons.id().untyped(),
        atlas.title.id().untyped(),
    ];
    handles.extend(atlas.endings.values().map(|h| h.id().untyped()));
    let total = handles.len() as u32;
    let loaded = handles.iter().filter(|id| server.is_loaded_with_dependencies(**id)).count() as u32;
    if loaded != st.last_loaded {
        st.last_loaded = loaded;
        ftt_progress(loaded, total);
    }
    if loaded == total {
        // Laisse les premières images entièrement texturées arriver jusqu'au canvas.
        st.frames_after_load += 1;
        if st.frames_after_load >= 3 {
            st.done = true;
            ftt_ready();
        }
    }
}
