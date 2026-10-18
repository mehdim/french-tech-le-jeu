//! Interface de gestion (egui). egui sert aux panneaux denses en données
//! (tableaux, curseurs, barres de progression, infobulles, fils défilants), où une
//! interface en mode immédiat est bien plus productive et lisible que Bevy UI ;
//! le monde du jeu lui-même (bureau, personnages, particules) est en sprites Bevy.

mod dialogs;
mod feed;
mod fight;
pub mod tutorial;
mod finale;
mod finance;
mod funding;
mod hud;
mod mentors;
mod menu;
mod office;
mod product;
mod quests;
mod sales;
pub mod style;
mod team;
pub mod widgets;

use crate::assets::Atlas;
use crate::camera::{CameraState, MainCamera, WorldViewport};
use crate::fx::FxQueue;
use crate::rendering::{Placement, Selection};
use crate::saves::{SaveStore, Settings, SlotList};
use crate::sim::{DemoMode, Game, GameContent, SimClock};
use crate::states::AppState;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle, egui};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Tab {
    #[default]
    Team,
    Product,
    Sales,
    Funding,
    Finance,
    Office,
    Mentors,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MenuPage {
    #[default]
    Main,
    Load,
    Save,
    Settings,
    Credits,
    ConfirmReset,
    /// Création du héros ou de l'héroïne avant une nouvelle partie.
    NewHero,
    /// Choix du mentor principal, juste après le héros.
    NewMentor,
}

#[derive(Resource, Default)]
pub struct UiState {
    pub tab: Tab,
    pub right_open: bool,
    pub left_open: bool,
    pub inspect_employee: Option<u32>,
    pub inspect_furniture: Option<u32>,
    pub menu_page: MenuPage,
    pub intro_line: usize,
    pub recruit_role: Option<game_core::types::Role>,
    pub recruit_channel: Option<String>,
    pub task_approach: Option<game_core::research::Approach>,
    pub confirm_fire: Option<u32>,
    pub product_filter: Option<game_core::content::TaskCategory>,
    pub feed_filter: u8,
    pub initialised: bool,
    /// egui applique les nouvelles polices à l'image suivante : on ne dessine qu'après.
    pub fonts_ready: bool,
    pub show_help: bool,
    /// Écran de création du héros.
    pub hero_name: String,
    pub hero_female: bool,
    pub hero_tutorial: bool,
    pub hero_mentor: String,
}

/// Notifications éphémères dans le coin.
#[derive(Resource, Default)]
pub struct Toasts(pub Vec<(String, bool, f32)>);

impl Toasts {
    /// Les messages longs (issues d'événements) restent affichés plus longtemps.
    fn ttl(msg: &str, base: f32) -> f32 {
        (base + msg.chars().count() as f32 / 28.0).clamp(base, 10.0)
    }
    pub fn push_bad(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        let t = Self::ttl(&m, 4.0);
        self.0.push((m, false, t));
    }
    pub fn push_good(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        let t = Self::ttl(&m, 3.5);
        self.0.push((m, true, t));
    }
}

/// Identifiants de textures egui pour nos atlas (enregistrés une fois chargés).
#[derive(Resource, Default, Clone)]
pub struct UiTextures {
    pub portraits: Option<egui::TextureId>,
    pub icons: Option<egui::TextureId>,
    pub title: Option<egui::TextureId>,
    pub endings: std::collections::HashMap<String, egui::TextureId>,
    pub characters: Option<egui::TextureId>,
    pub furniture: Option<egui::TextureId>,
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .init_resource::<Toasts>()
            .init_resource::<UiTextures>()
            .add_systems(EguiPrimaryContextPass, (setup_style, register_textures, ui_root).chain());
    }
}

fn setup_style(mut contexts: EguiContexts, mut ui: ResMut<UiState>, settings: Res<Settings>) -> Result {
    let ctx = contexts.ctx_mut()?;
    if !ui.initialised {
        style::install(ctx);
        ui.initialised = true;
        ui.right_open = true;
        ui.left_open = true;
    } else if !ui.fonts_ready {
        ui.fonts_ready = true;
    }
    if settings.is_changed() {
        ctx.set_zoom_factor(settings.ui_scale.clamp(0.75, 1.5));
    }
    Ok(())
}

fn register_textures(mut contexts: EguiContexts, atlas: Option<Res<Atlas>>, mut tex: ResMut<UiTextures>) {
    let Some(atlas) = atlas else { return };
    if tex.portraits.is_some() {
        return;
    }
    tex.portraits = Some(contexts.add_image(EguiTextureHandle::Strong(atlas.portraits.clone())));
    tex.icons = Some(contexts.add_image(EguiTextureHandle::Strong(atlas.icons.clone())));
    tex.title = Some(contexts.add_image(EguiTextureHandle::Strong(atlas.title.clone())));
    tex.characters = Some(contexts.add_image(EguiTextureHandle::Strong(atlas.characters.clone())));
    tex.furniture = Some(contexts.add_image(EguiTextureHandle::Strong(atlas.furniture.clone())));
    for (k, h) in &atlas.endings {
        let id = contexts.add_image(EguiTextureHandle::Strong(h.clone()));
        tex.endings.insert(k.clone(), id);
    }
}

/// Tout ce dont les panneaux ont besoin, regroupé pour garder des signatures de systèmes raisonnables.
pub struct Ctx<'a> {
    pub game: &'a mut Game,
    pub content: &'a GameContent,
    pub ui: &'a mut UiState,
    pub toasts: &'a mut Toasts,
    pub clock: &'a mut SimClock,
    pub placement: &'a mut Placement,
    pub selection: &'a mut Selection,
    pub tex: &'a UiTextures,
    pub atlas: Option<&'a Atlas>,
    pub settings: &'a mut Settings,
    pub store: &'a mut SaveStore,
    pub slots: &'a mut SlotList,
    pub demo: &'a mut DemoMode,
    pub next_state: &'a mut NextState<AppState>,
    pub click: &'a mut u32,
}

impl Ctx<'_> {
    /// Exécute une commande et signale les erreurs par des notifications.
    pub fn run(&mut self, cmd: game_core::Command) -> bool {
        *self.click += 1;
        match self.game.0.apply(&self.content.0, cmd) {
            Ok(()) => true,
            Err(e) => {
                self.toasts.push_bad(e.label());
                false
            }
        }
    }
}

/// Ressources utilisées par l'interface, regroupées (un système Bevy prend au plus 16 paramètres).
#[derive(bevy::ecs::system::SystemParam)]
pub struct UiRes<'w, 's> {
    game: ResMut<'w, Game>,
    content: Res<'w, GameContent>,
    ui: ResMut<'w, UiState>,
    toasts: ResMut<'w, Toasts>,
    clock: ResMut<'w, SimClock>,
    placement: ResMut<'w, Placement>,
    selection: ResMut<'w, Selection>,
    tex: Res<'w, UiTextures>,
    atlas: Option<Res<'w, Atlas>>,
    settings: ResMut<'w, Settings>,
    store: ResMut<'w, SaveStore>,
    slots: ResMut<'w, SlotList>,
    demo: ResMut<'w, DemoMode>,
    next_state: ResMut<'w, NextState<AppState>>,
    state: Res<'w, State<AppState>>,
    cam: Query<'w, 's, (&'static Camera, &'static GlobalTransform), With<MainCamera>>,
}

#[allow(clippy::too_many_arguments)]
fn ui_root(
    mut contexts: EguiContexts,
    mut r: UiRes,
    mut vp: ResMut<WorldViewport>,
    fx: Res<FxQueue>,
    cam_state: Res<CameraState>,
    time: Res<Time>,
    #[cfg(feature = "audio")] mut ui_sound: ResMut<crate::audio::UiSound>,
) -> Result {
    if !r.ui.fonts_ready {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?.clone();
    let mut clicks = 0u32;
    let current = *r.state.get();
    let UiRes { game, content, ui, toasts, clock, placement, selection, tex, atlas, settings, store, slots, demo, next_state, cam, .. } = &mut r;
    let mut c = Ctx {
        game,
        content,
        ui,
        toasts,
        clock,
        placement,
        selection,
        tex,
        atlas: atlas.as_deref(),
        settings,
        store,
        slots,
        demo,
        next_state,
        click: &mut clicks,
    };

    let mut root = egui::Ui::new(
        ctx.clone(),
        "root".into(),
        egui::UiBuilder::new().layer_id(egui::LayerId::background()).max_rect(ctx.content_rect()),
    );

    let central = match current {
        AppState::MainMenu => {
            menu::main_menu(&ctx, &mut c);
            ctx.content_rect()
        }
        _ => {
            let narrow = ctx.content_rect().width() < 1100.0;
            hud::top_bar(&mut root, &mut c);
            feed::bottom(&mut root, &mut c);
            if c.ui.left_open {
                quests::left(&mut root, &mut c, narrow);
            }
            if c.ui.right_open {
                egui::Panel::right("right_panel")
                    .resizable(true)
                    .default_size(if narrow { 340.0 } else { 420.0 })
                    .min_size(300.0)
                    .max_size(640.0)
                    .frame(style::panel_frame())
                    .show(&mut root, |ui| right_panel(ui, &mut c));
            }
            let r = root.available_rect_before_wrap();
            match current {
                AppState::Playing => {
                    dialogs::in_game(&ctx, &mut c);
                    tutorial::show(&ctx, &mut c);
                }
                AppState::Paused => menu::pause_menu(&ctx, &mut c),
                AppState::MissionComplete => dialogs::mission_complete(&ctx, &mut c),
                AppState::GameOver => dialogs::game_over(&ctx, &mut c),
                AppState::Victory => dialogs::victory(&ctx, &mut c),
                AppState::MainMenu => {}
            }
            r
        }
    };

    // Textes flottants au-dessus du monde.
    if let Ok((camera, gtf)) = cam.single() {
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("fx_text")));
        let painter = painter.with_clip_rect(central);
        for t in &fx.texts {
            if let Ok(p) = camera.world_to_viewport(gtf, t.world.extend(0.0)) {
                let alpha = (1.0 - t.age / t.ttl).clamp(0.0, 1.0);
                let col = egui::Color32::from_rgba_unmultiplied(t.color[0], t.color[1], t.color[2], (alpha * 255.0) as u8);
                let size = if t.big { 22.0 } else { 15.0 } * (cam_state.zoom / 3.0).clamp(0.7, 1.4);
                let pos = egui::pos2(p.x, p.y);
                let font = egui::FontId::new(size, egui::FontFamily::Name("pixel".into()));
                painter.text(pos + egui::vec2(1.0, 1.0), egui::Align2::CENTER_BOTTOM, &t.text, font.clone(), egui::Color32::from_black_alpha((alpha * 200.0) as u8));
                painter.text(pos, egui::Align2::CENTER_BOTTOM, &t.text, font, col);
            }
        }
    }
    widgets::toasts(&ctx, c.toasts, time.delta_secs());
    if let Some(item) = c.placement.item.clone() {
        widgets::placement_hint(&ctx, &c, &item, central);
    }

    // Indique aux systèmes de caméra et d'entrée ce qui reste pour le monde.
    let pointer = ctx.pointer_hover_pos();
    vp.min = Vec2::new(central.min.x, central.min.y);
    vp.max = Vec2::new(central.max.x, central.max.y);
    vp.pointer_over_ui = ctx.egui_wants_pointer_input() || pointer.is_some_and(|p| !central.contains(p));
    vp.keyboard_captured = ctx.egui_wants_keyboard_input();
    #[cfg(feature = "audio")]
    {
        ui_sound.0 += clicks;
    }
    Ok(())
}

fn right_panel(ui: &mut egui::Ui, c: &mut Ctx) {
    let unlocked = |k: &str, c: &Ctx| c.game.0.is_unlocked(k);
    ui.horizontal_wrapped(|ui| {
        let tabs = [
            (Tab::Team, "👥 Équipe", "E", true),
            (Tab::Product, "🧪 Produit & R&D", "P", true),
            (Tab::Sales, "📈 Ventes", "V", unlocked("launch", c) || unlocked("product_dev", c)),
            (Tab::Funding, "🏦 Financement", "F", true),
            (Tab::Finance, "📊 Finances", "C", true),
            (Tab::Office, "🏢 Bureau", "B", true),
            (Tab::Mentors, "🎙 Mentors", "M", true),
        ];
        for (tab, label, key, enabled) in tabs {
            let selected = c.ui.tab == tab;
            let b = ui.add_enabled(enabled, egui::Button::selectable(selected, egui::RichText::new(label).size(14.0)));
            tutorial::mark(ui, &format!("tab_{tab:?}"), b.rect);
            if b.on_hover_text(format!("Raccourci : {key}")).clicked() {
                c.ui.tab = tab;
                *c.click += 1;
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("✕").on_hover_text("Fermer le panneau").clicked() {
                c.ui.right_open = false;
            }
        });
    });
    ui.separator();
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match c.ui.tab {
        Tab::Team => team::show(ui, c),
        Tab::Product => product::show(ui, c),
        Tab::Sales => sales::show(ui, c),
        Tab::Funding => funding::show(ui, c),
        Tab::Finance => finance::show(ui, c),
        Tab::Office => office::show(ui, c),
        Tab::Mentors => mentors::show(ui, c),
    });
}
