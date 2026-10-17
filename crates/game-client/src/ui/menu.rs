//! Écran titre, menu pause, sauvegarde/chargement, paramètres et crédits.

use super::style::*;
use super::widgets::*;
use super::{Ctx, MenuPage};
use crate::saves::{SLOTS, load_from, refresh_slots, save_to, slot_label};
use crate::states::AppState;
use bevy_egui::egui::{self, RichText};
use game_content::dialogue;

pub fn to_main_menu(c: &mut Ctx) {
    if !c.demo.0 && c.game.0.campaign.phase == game_core::missions::Phase::Playing {
        let _ = save_to(c.store, "autosave", &c.game.0, c.content);
        *c.slots = refresh_slots(c.store);
    }
    c.game.0 = crate::sim::demo_state(&c.content.0);
    c.demo.0 = true;
    *c.placement = Default::default();
    c.selection.employee = None;
    c.selection.furniture = None;
    c.next_state.set(AppState::MainMenu);
    if c.ui.menu_page != MenuPage::Credits {
        c.ui.menu_page = MenuPage::Main;
    }
}

fn start_new(c: &mut Ctx) {
    let seed = platform::clock::now_millis();
    c.game.0 = game_core::GameState::new(&c.content.0, seed);
    c.demo.0 = false;
    c.ui.intro_line = 0;
    c.ui.inspect_employee = None;
    c.ui.right_open = true;
    c.ui.left_open = true;
    c.clock.set_speed(1);
    c.next_state.set(AppState::Playing);
}

fn load_slot(c: &mut Ctx, slot: &str) {
    match load_from(c.store, slot, c.content) {
        Ok(s) => {
            c.game.0 = s;
            c.demo.0 = false;
            c.ui.intro_line = 0;
            c.clock.set_speed(0);
            c.toasts.push_good("Partie chargée. Le temps est en pause : Espace pour reprendre.");
            c.next_state.set(AppState::Playing);
        }
        Err(e) => c.toasts.push_bad(e),
    }
}

pub fn main_menu(ctx: &egui::Context, c: &mut Ctx) {
    // Un vignettage sombre sur le bureau animé, pour la lisibilité.
    let rect = ctx.content_rect();
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Background, egui::Id::new("menu_bg")));
    painter.rect_filled(egui::Rect::from_min_max(rect.min, egui::pos2(rect.min.x + 470.0, rect.max.y)), 0.0, egui::Color32::from_black_alpha(205));
    painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(60));

    egui::Area::new("menu".into())
        .fixed_pos(rect.min + egui::vec2(32.0, 24.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            ui.set_width(410.0);
            if let Some(id) = c.tex.title {
                ui.add(egui::Image::from_texture(egui::load::SizedTexture::new(id, egui::vec2(406.0, 228.0))).texture_options(egui::TextureOptions::NEAREST));
            }
            ui.add_space(6.0);
            ui.label(RichText::new("A LA").font(pixel(44.0)).color(OFFWHITE));
            ui.label(RichText::new("FRENCH").font(pixel(44.0)).color(GREEN));
            ui.label(RichText::new("De France Travail à la licorne").font(pixel(11.0)).color(LIGHTGRAY));
            ui.add_space(4.0);
            ui.add(egui::Label::new(RichText::new("« La France est une startup nation. À vous de payer l'URSSAF. »").italics().size(14.0).color(LIGHTGRAY)).wrap());
            ui.add_space(14.0);
            match c.ui.menu_page {
                MenuPage::Main => main_buttons(ui, c),
                MenuPage::Load => slots_page(ui, c, false),
                MenuPage::Save => slots_page(ui, c, true),
                MenuPage::Settings => settings_page(ui, c),
                MenuPage::Credits => credits_page(ui, c),
                MenuPage::ConfirmReset => confirm_reset(ui, c),
            }
        });
    egui::Area::new("menu_footer".into())
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-16.0, -12.0))
        .show(ctx, |ui| {
            ui.label(RichText::new(format!("v{} · Rust + Bevy · données de jeu satiriques", env!("CARGO_PKG_VERSION"))).size(11.5).color(LIGHTGRAY));
        });
}

fn main_buttons(ui: &mut egui::Ui, c: &mut Ctx) {
    let w = 300.0;
    if big_button(ui, "Nouvelle partie", true, w).clicked() {
        *c.click += 1;
        start_new(c);
    }
    let latest = c.slots.0.iter().filter(|(_, m)| m.is_ok()).max_by_key(|(_, m)| m.as_ref().map(|m| m.day).unwrap_or(0)).map(|(s, _)| s.clone());
    if big_button(ui, "Continuer", latest.is_some(), w).clicked()
        && let Some(slot) = latest {
            load_slot(c, &slot);
        }
    if big_button(ui, "Charger", !c.slots.0.is_empty(), w).clicked() {
        c.ui.menu_page = MenuPage::Load;
    }
    if big_button(ui, "Paramètres", true, w).clicked() {
        c.ui.menu_page = MenuPage::Settings;
    }
    if big_button(ui, "Crédits", true, w).clicked() {
        c.ui.menu_page = MenuPage::Credits;
    }
    ui.add_space(8.0);
    muted(ui, "En arrière-plan : une Veridix pilotée par l'autopilote. Elle ne paie pas l'URSSAF non plus.");
}

fn back(ui: &mut egui::Ui, c: &mut Ctx) {
    ui.add_space(8.0);
    if ui.button("◂ Retour").clicked() {
        c.ui.menu_page = MenuPage::Main;
    }
}

fn slots_page(ui: &mut egui::Ui, c: &mut Ctx, saving: bool) {
    ui.label(RichText::new(if saving { "Sauvegarder" } else { "Charger" }).font(pixel(22.0)).color(GREEN));
    for slot in SLOTS {
        if saving && slot == "autosave" {
            continue;
        }
        let meta = c.slots.0.iter().find(|(s, _)| s == slot).map(|(_, m)| m.clone());
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(slot_label(slot)).strong());
                    match &meta {
                        Some(Ok(m)) => muted(ui, format!("Niveau {} « {} » · jour {} · {} · MRR {}", m.level, m.label, m.day, dialogue::money(m.cash), dialogue::money(m.mrr))),
                        Some(Err(e)) => {
                            ui.label(RichText::new(format!("⚠ {e}")).size(12.0).color(ORANGE));
                        }
                        None => muted(ui, "vide"),
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if saving {
                        if ui.button(RichText::new("Sauvegarder ici").color(GREEN)).clicked() {
                            match save_to(c.store, slot, &c.game.0, c.content) {
                                Ok(()) => c.toasts.push_good(format!("Sauvegardé : {}", slot_label(slot))),
                                Err(e) => c.toasts.push_bad(e),
                            }
                            *c.slots = refresh_slots(c.store);
                        }
                    } else if matches!(meta, Some(Ok(_))) && ui.button(RichText::new("Charger").color(GREEN)).clicked() {
                        load_slot(c, slot);
                    }
                    if meta.is_some() && ui.small_button("🗑").on_hover_text("Supprimer").clicked() {
                        let _ = c.store.0.remove(slot);
                        *c.slots = refresh_slots(c.store);
                    }
                });
            });
        });
    }
    back(ui, c);
}

fn settings_page(ui: &mut egui::Ui, c: &mut Ctx) {
    ui.label(RichText::new("Paramètres").font(pixel(22.0)).color(GREEN));
    settings_body(ui, c);
    back(ui, c);
}

pub fn settings_body(ui: &mut egui::Ui, c: &mut Ctx) {
    let s = &mut *c.settings;
    let mut music = s.music_volume;
    let mut sfx = s.sfx_volume;
    let mut scale = s.ui_scale;
    let mut low = s.low_power;
    let mut hints = s.hints;
    ui.add(egui::Slider::new(&mut music, 0.0..=1.0).text("Musique"));
    ui.add(egui::Slider::new(&mut sfx, 0.0..=1.0).text("Effets sonores"));
    ui.add(egui::Slider::new(&mut scale, 0.75..=1.5).step_by(0.05).text("Taille de l'interface"));
    ui.checkbox(&mut low, "Mode économie (30 images/s)").on_hover_text("Pour les ordinateurs modestes et les batteries fatiguées.");
    ui.checkbox(&mut hints, "Conseils de démarrage");
    if music != s.music_volume || sfx != s.sfx_volume || scale != s.ui_scale || low != s.low_power || hints != s.hints {
        s.music_volume = music;
        s.sfx_volume = sfx;
        s.ui_scale = scale;
        s.low_power = low;
        s.hints = hints;
    }
    #[cfg(not(feature = "audio"))]
    muted(ui, "Cette version a été compilée sans audio.");
    ui.add_space(6.0);
    if ui.button(RichText::new("Réinitialiser la progression…").color(RED)).clicked() {
        c.ui.menu_page = MenuPage::ConfirmReset;
    }
}

fn confirm_reset(ui: &mut egui::Ui, c: &mut Ctx) {
    ui.label(RichText::new("Tout effacer ?").font(pixel(22.0)).color(RED));
    ui.label("Toutes les sauvegardes (y compris la sauvegarde auto) seront supprimées. C'est définitif, comme une décision de l'URSSAF.");
    ui.horizontal(|ui| {
        if ui.button(RichText::new("Oui, tout effacer").color(RED)).clicked() {
            for slot in SLOTS {
                let _ = c.store.0.remove(slot);
            }
            *c.slots = refresh_slots(c.store);
            c.toasts.push_good("Progression réinitialisée.");
            c.ui.menu_page = MenuPage::Main;
        }
        if ui.button("Annuler").clicked() {
            c.ui.menu_page = MenuPage::Settings;
        }
    });
}

fn credits_page(ui: &mut egui::Ui, c: &mut Ctx) {
    ui.label(RichText::new("Crédits").font(pixel(22.0)).color(GREEN));
    for (k, v) in [
        ("Jeu", "A la French — un jeu de gestion satirique et affectueux sur l'écosystème tech français, inspiré du podcast du même nom."),
        ("Mentors", "Mehdi, Steeve et Jean-Baptiste, qui refont le monde chaque semaine dans A la French… avec l'accent."),
        ("Moteur", "Rust, Bevy 0.19 (ECS, wgpu/WebGPU), egui pour les panneaux de gestion."),
        ("Graphismes", "Pixel art procédural original (tools/art), police « Veridix Pixel » originale, DejaVu Sans pour le texte."),
        ("Musique & sons", "Thème d'À la French en 16 bits et airs traditionnels en 8-bit, synthétisés de zéro (tools/audio)."),
        ("Inspirations", "Game Dev Tycoon, E-Startup 2, Two Point Hospital, Software Inc., Papers, Please — et l'esprit du podcast A la French."),
        ("Avertissement", "Les dispositifs (France Travail, Bpifrance, CIR, JEI…) sont des versions satiriques et simplifiées. Ceci n'est ni un conseil juridique, ni un conseil financier, ni un business plan."),
        ("Merci", "Aux ingénieur·es, chercheur·ses et entrepreneur·es qui construisent vraiment des choses. Et à la machine à café."),
    ] {
        ui.label(RichText::new(k).strong().color(OFFWHITE));
        ui.add(egui::Label::new(RichText::new(v).size(13.0).color(LIGHTGRAY)).wrap());
        ui.add_space(4.0);
    }
    back(ui, c);
}

pub fn pause_menu(ctx: &egui::Context, c: &mut Ctx) {
    dim(ctx, 170);
    egui::Area::new("pause".into())
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            modal_frame().show(ui, |ui| {
                ui.set_width(380.0);
                match c.ui.menu_page {
                    MenuPage::Save => {
                        slots_page(ui, c, true);
                    }
                    MenuPage::Load => {
                        slots_page(ui, c, false);
                    }
                    MenuPage::Settings => {
                        ui.label(RichText::new("Paramètres").font(pixel(22.0)).color(GREEN));
                        settings_body(ui, c);
                        back(ui, c);
                    }
                    MenuPage::ConfirmReset => confirm_reset(ui, c),
                    _ => {
                        ui.label(RichText::new("PAUSE").font(pixel(33.0)).color(GREEN));
                        ui.add_space(8.0);
                        let w = 340.0;
                        if big_button(ui, "Reprendre", true, w).clicked() {
                            c.next_state.set(AppState::Playing);
                        }
                        if big_button(ui, "Sauvegarder", true, w).clicked() {
                            c.ui.menu_page = MenuPage::Save;
                        }
                        if big_button(ui, "Charger", !c.slots.0.is_empty(), w).clicked() {
                            c.ui.menu_page = MenuPage::Load;
                        }
                        if big_button(ui, "Paramètres", true, w).clicked() {
                            c.ui.menu_page = MenuPage::Settings;
                        }
                        if big_button(ui, "Menu principal", true, w).clicked() {
                            to_main_menu(c);
                        }
                        muted(ui, "La partie est sauvegardée automatiquement chaque mois et à chaque niveau.");
                    }
                }
            });
        });
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        if c.ui.menu_page == MenuPage::Main {
            c.next_state.set(AppState::Playing);
        } else {
            c.ui.menu_page = MenuPage::Main;
        }
    }
}
