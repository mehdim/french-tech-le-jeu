//! Éléments d'interface réutilisables.

use super::style::*;
use super::{Ctx, Toasts, UiTextures};
use crate::assets::Atlas;
use bevy_egui::egui::{self, Color32, RichText, Vec2};
use game_content::dialogue;
use game_core::types::{Condition, Effect};

pub fn money(v: f64) -> String {
    dialogue::money(v)
}

fn sub_image(tex: Option<egui::TextureId>, atlas_size: Vec2, rect: [f32; 4], size: f32) -> Option<egui::Image<'static>> {
    let id = tex?;
    let uv = egui::Rect::from_min_max(
        egui::pos2(rect[0] / atlas_size.x, rect[1] / atlas_size.y),
        egui::pos2((rect[0] + rect[2]) / atlas_size.x, (rect[1] + rect[3]) / atlas_size.y),
    );
    let aspect = rect[2] / rect[3];
    Some(
        egui::Image::from_texture(egui::load::SizedTexture::new(id, Vec2::new(size * aspect, size)))
            .uv(uv)
            .texture_options(egui::TextureOptions::NEAREST),
    )
}

pub fn icon(ui: &mut egui::Ui, tex: &UiTextures, atlas: Option<&Atlas>, key: &str, size: f32) {
    let Some(atlas) = atlas else { return };
    let n = atlas.manifest.icons.len() as f32;
    if let Some(meta) = atlas.manifest.icons.get(key)
        && let Some(img) = sub_image(tex.icons, Vec2::new(12.0 * n, 12.0), meta.rect, size) {
            ui.add(img);
        }
}

pub fn portrait(ui: &mut egui::Ui, tex: &UiTextures, atlas: Option<&Atlas>, key: &str, size: f32) {
    let Some(atlas) = atlas else { return };
    let n = atlas.manifest.portraits.len() as f32;
    let meta = atlas.manifest.portraits.get(key).or_else(|| atlas.manifest.portraits.get("generic"));
    if let Some(meta) = meta
        && let Some(img) = sub_image(tex.portraits, Vec2::new(40.0 * n, 40.0), meta.rect, size) {
            ui.add(img);
        }
}

/// Le sprite d'attente d'un salarié, en mini-avatar.
pub fn avatar(ui: &mut egui::Ui, tex: &UiTextures, atlas: Option<&Atlas>, role: game_core::types::Role, look: u8, size: f32) {
    let Some(atlas) = atlas else { return };
    let m = &atlas.manifest.characters;
    let name = format!("{role:?}");
    let idx = m.roles.iter().position(|r| *r == name).unwrap_or(0) as f32;
    let row = idx * m.looks as f32 + (look as u32 % m.looks) as f32;
    let atlas_size = Vec2::new(m.frame[0] * 16.0, m.frame[1] * m.looks as f32 * m.roles.len() as f32);
    if let Some(img) = sub_image(tex.characters, atlas_size, [0.0, row * m.frame[1], m.frame[0], m.frame[1]], size) {
        ui.add(img);
    }
}

pub fn furniture_icon(ui: &mut egui::Ui, tex: &UiTextures, atlas: Option<&Atlas>, sprite: &str, size: f32) {
    let Some(atlas) = atlas else { return };
    if let Some(meta) = atlas.furniture(sprite)
        && let Some(img) = sub_image(tex.furniture, Vec2::new(512.0, furniture_atlas_height(atlas)), meta.rect, size) {
            ui.add(img);
        }
}

fn furniture_atlas_height(atlas: &Atlas) -> f32 {
    let max = atlas.manifest.furniture.values().map(|m| m.rect[1] + m.rect[3]).fold(0.0, f32::max);
    let mut h = 1.0;
    while h < max + 1.0 {
        h *= 2.0;
    }
    h
}

/// Barre de progression horizontale avec un libellé facultatif.
pub fn bar(ui: &mut egui::Ui, frac: f32, color: Color32, text: impl Into<String>, width: f32) -> egui::Response {
    let frac = frac.clamp(0.0, 1.0);
    // Le libellé commence au bord gauche de la barre : dès qu'un remplissage clair le
    // recouvre, on passe en texte sombre pour qu'il reste lisible.
    let luminance = 0.2126 * color.r() as f32 + 0.7152 * color.g() as f32 + 0.0722 * color.b() as f32;
    let text_color = if frac >= 0.35 && luminance > 140.0 { NEAR_BLACK } else { OFFWHITE };
    ui.add(
        egui::ProgressBar::new(frac)
            .desired_width(width)
            .desired_height(16.0)
            .fill(color)
            .corner_radius(2)
            .text(RichText::new(text.into()).size(11.5).color(text_color)),
    )
}

/// Couleur pour une valeur 0..100 où haut = bien.
pub fn good_color(v: f64) -> Color32 {
    if v >= 60.0 {
        GREEN
    } else if v >= 35.0 {
        ORANGE
    } else {
        RED
    }
}

/// Couleur pour une valeur 0..100 où haut = mal.
pub fn bad_color(v: f64) -> Color32 {
    good_color(100.0 - v)
}

pub fn skill_row(ui: &mut egui::Ui, label: &str, v: f64) {
    ui.horizontal(|ui| {
        ui.add_sized([86.0, 16.0], egui::Label::new(RichText::new(label).size(12.5).color(LIGHTGRAY)));
        bar(ui, (v / 100.0) as f32, if v >= 65.0 { DEEP_GREEN } else { GRAPHITE_3 }, format!("{:.0}", v), 150.0);
    });
}

pub fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).font(pixel(22.0)).color(GREEN));
    ui.add_space(2.0);
}

pub fn subheading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(text).font(bold(14.0)).color(OFFWHITE));
}

pub fn muted(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(RichText::new(text.into()).size(12.5).color(MUTED));
}

/// Ligne clé/valeur avec une infobulle qui explique l'indicateur.
pub fn kv(ui: &mut egui::Ui, k: &str, v: impl Into<String>, color: Color32, help: &str) {
    ui.horizontal(|ui| {
        let r = ui.label(RichText::new(k).color(LIGHTGRAY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(v.into()).color(color).strong());
        });
        if !help.is_empty() {
            r.on_hover_text(help);
        }
    });
}

/// Liste les conséquences d'un ensemble d'effets (vert = bon / rouge = mauvais).
pub fn effects_list(ui: &mut egui::Ui, effects: &[Effect]) {
    for e in effects {
        if let Some((text, good)) = dialogue::effect_text(e) {
            ui.label(RichText::new(format!("{} {}", if good { "▲" } else { "▼" }, text)).size(12.5).color(if good { GREEN } else { RED }));
        }
    }
}

pub fn effects_inline(effects: &[Effect]) -> Vec<(String, bool)> {
    effects.iter().filter_map(dialogue::effect_text).collect()
}

/// Texte d'une condition requise, avec les noms des chantiers à la place de leurs ids.
pub fn cond_text(c: &Ctx, cond: &Condition) -> String {
    match cond {
        Condition::Shipped(t) => {
            let name = c.content.0.task(t).map(|d| d.name.as_str()).unwrap_or(t);
            format!("« {name} » livré")
        }
        Condition::All(v) => v.iter().map(|x| cond_text(c, x)).collect::<Vec<_>>().join(" et "),
        other => dialogue::condition_text(other),
    }
}

/// Liste de conditions avec des ✓ / ✗. Renvoie true quand toutes sont remplies.
pub fn requirements(ui: &mut egui::Ui, c: &Ctx, reqs: &[Condition]) -> bool {
    let mut all = true;
    for r in reqs {
        let ok = c.game.0.check(r);
        all &= ok;
        ui.label(RichText::new(format!("{} {}", if ok { "✓" } else { "✗" }, cond_text(c, r))).size(12.5).color(if ok { GREEN } else { ORANGE }));
    }
    all
}

pub fn toasts(ctx: &egui::Context, toasts: &mut Toasts, dt: f32) {
    for t in &mut toasts.0 {
        t.2 -= dt;
    }
    toasts.0.retain(|t| t.2 > 0.0);
    if toasts.0.len() > 5 {
        let n = toasts.0.len() - 5;
        toasts.0.drain(0..n);
    }
    if toasts.0.is_empty() {
        return;
    }
    // Une Area se met en page dans la taille de l'image précédente : sans largeur
    // explicite, un long message peut s'écraser en colonne étroite, plus haute que
    // large. On fixe donc la largeur de chaque bulle d'après son texte (une ligne si
    // possible, sinon des lignes de 560 px au plus).
    let font = egui::FontId::proportional(14.5);
    let max_w = (ctx.content_rect().width() - 80.0).clamp(220.0, 560.0);
    egui::Area::new("toasts".into())
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 64.0))
        .order(egui::Order::Tooltip)
        .interactable(false)
        .show(ctx, |ui| {
            for (msg, good, ttl) in &toasts.0 {
                let a = (ttl.min(1.0) * 255.0) as u8;
                let natural = ctx.fonts_mut(|f| f.layout_no_wrap(msg.clone(), font.clone(), OFFWHITE).size().x);
                egui::Frame::new()
                    .fill(Color32::from_rgba_unmultiplied(20, 26, 22, a.min(235)))
                    .stroke(egui::Stroke::new(1.5, if *good { GREEN } else { RED }))
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .corner_radius(3)
                    .show(ui, |ui| {
                        ui.set_width(natural.clamp(120.0, max_w));
                        ui.add(egui::Label::new(RichText::new(msg).font(font.clone()).color(OFFWHITE)).wrap());
                    });
                ui.add_space(4.0);
            }
        });
}

pub fn placement_hint(ctx: &egui::Context, c: &Ctx, item: &str, rect: egui::Rect) {
    let name = c.content.0.furniture(item).map(|f| f.name.clone()).unwrap_or_default();
    egui::Area::new("placement_hint".into())
        .fixed_pos(egui::pos2(rect.center().x - 180.0, rect.max.y - 54.0))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::new().fill(Color32::from_black_alpha(200)).inner_margin(egui::Margin::symmetric(12, 6)).corner_radius(3).show(ui, |ui| {
                ui.set_width(360.0);
                let reason = if c.placement.valid || c.placement.hover.is_none() { String::new() } else { format!(" — {}", c.placement.reason) };
                ui.label(RichText::new(format!("Placer : {name}{reason}")).color(if c.placement.valid { GREEN } else { ORANGE }));
                ui.label(RichText::new("Clic gauche : poser · R : pivoter · Maj : en poser plusieurs · Échap / clic droit : annuler").size(11.5).color(LIGHTGRAY));
            });
        });
}

/// Curseur de montant qui ne modifie la partie qu'une fois le geste terminé.
///
/// La valeur affichée est relue dans l'état du jeu à chaque image : pendant un
/// glisser ou une saisie, on la garde donc en mémoire egui, sinon chaque image
/// repartirait de l'ancienne valeur. Renvoie `Some(valeur)` quand le joueur
/// relâche le curseur, clique sur la piste, ou valide une saisie.
pub fn money_slider(
    ui: &mut egui::Ui,
    id_salt: impl std::hash::Hash + std::fmt::Debug,
    current: f64,
    range: std::ops::RangeInclusive<f64>,
    step: f64,
    hover: &str,
) -> Option<f64> {
    let id = ui.id().with(("curseur_montant", id_salt));
    let mut value = ui.data(|d| d.get_temp::<f64>(id)).unwrap_or(current);
    // La piste prend la place disponible (moins la case du montant), et reste
    // visible sur les cartes, dont le fond a la même couleur que la piste par défaut.
    let r = ui
        .scope(|ui| {
            ui.spacing_mut().slider_width = (ui.available_width() - 90.0).clamp(80.0, 260.0);
            ui.visuals_mut().widgets.inactive.bg_fill = NEAR_BLACK;
            ui.add(
                egui::Slider::new(&mut value, range)
                    .step_by(step)
                    .trailing_fill(true)
                    .custom_formatter(|v, _| dialogue::money(v))
                    .custom_parser(dialogue::parse_money),
            )
        })
        .inner;
    let r = r.on_hover_text(hover);
    // Seule une action du joueur valide : au premier affichage, egui arrondit parfois la
    // valeur au pas du curseur (56,12 € → 56 €) et signale un « changement » fantôme.
    // Un clic sur la piste place la valeur à l'appui et la valide au relâchement.
    let editing = r.dragged() || r.has_focus() || r.is_pointer_button_down_on();
    let commit = r.drag_stopped() || r.lost_focus() || r.clicked();
    if editing && !commit {
        ui.data_mut(|d| d.insert_temp(id, value));
    } else {
        ui.data_mut(|d| d.remove::<f64>(id));
    }
    (commit && (value - current).abs() > f64::EPSILON).then_some(value)
}

/// Un grand bouton cliquable en forme de carte, pour les menus et les choix d'événements.
pub fn big_button(ui: &mut egui::Ui, text: &str, enabled: bool, width: f32) -> egui::Response {
    let b = egui::Button::new(RichText::new(text).font(pixel(22.0)).color(if enabled { OFFWHITE } else { MUTED }))
        .min_size(egui::vec2(width, 40.0))
        .fill(if enabled { GRAPHITE_2 } else { GRAPHITE });
    ui.add_enabled(enabled, b)
}

/// Assombrit tout l'écran derrière une fenêtre modale.
pub fn dim(ctx: &egui::Context, alpha: u8) {
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Middle, egui::Id::new("dim")));
    painter.rect_filled(ctx.content_rect(), 0.0, Color32::from_black_alpha(alpha));
}
