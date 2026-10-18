//! Fin « Les Communs » : deux salles isométriques en pixel art (tools/art/gen_rooms.py).
//! D'abord la remise de l'ordre national du Mérite à l'Élysée, puis, le lendemain
//! matin, l'agence France Travail. Le décor est une image ; le héros (apparence
//! choisie à la création de la partie), la médaille, les flashs, le numéro de ticket
//! et les bulles de dialogue sont animés par-dessus.

use super::super::Ctx;
use super::super::style::*;
use bevy_egui::egui::{self, Align2, Color32, FontId, Pos2, Rect, Stroke, pos2, vec2};
use game_core::types::Role;

/// Taille des images de décor, en pixels.
const IMG_W: f32 = 320.0;
const IMG_H: f32 = 180.0;
/// Début de la scène France Travail, et durée totale des deux scènes (secondes).
pub const FT_START: f32 = 7.0;
pub const DURATION: f32 = 15.0;

// Repères calculés par gen_rooms.positions() (pixels du décor).
const ELYSEE_START: (f32, f32) = (222.4, 147.2);
const ELYSEE_END: (f32, f32) = (148.0, 110.0);
const PRESIDENT_HAND: (f32, f32) = (136.4, 92.2);
const CAMERAS: [(f32, f32); 3] = [(182.8, 80.6), (198.8, 88.6), (214.8, 96.6)];
const FT_START_POS: (f32, f32) = (206.4, 158.4);
const FT_MID: (f32, f32) = (184.8, 127.6);
const FT_END: (f32, f32) = (150.4, 100.8);
const COUNSELOR_HEAD: (f32, f32) = (180.0, 61.2);
const FT_SCREEN: (f32, f32) = (248.8, 80.0);

const GOLD: Color32 = Color32::from_rgb(250, 206, 72);
const RIBBON: Color32 = Color32::from_rgb(40, 90, 180);

/// Passage des coordonnées du décor à l'écran.
struct View {
    origin: Pos2,
    s: f32,
}

impl View {
    fn new(rect: Rect) -> Self {
        let s = (rect.width() / IMG_W).min(rect.height() / IMG_H);
        let origin = rect.center() - vec2(IMG_W, IMG_H) * s / 2.0;
        Self { origin, s }
    }
    fn at(&self, p: (f32, f32)) -> Pos2 {
        self.origin + vec2(p.0, p.1) * self.s
    }
    fn rect(&self) -> Rect {
        Rect::from_min_size(self.origin, vec2(IMG_W, IMG_H) * self.s)
    }
}

fn lerp(a: (f32, f32), b: (f32, f32), k: f32) -> (f32, f32) {
    let k = k.clamp(0.0, 1.0);
    (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k)
}

/// Dessine le héros : `frame` dans la ligne du sprite (0-1 attente, 2-5 marche, 6-7 assis).
fn hero(p: &egui::Painter, v: &View, c: &Ctx, feet: (f32, f32), frame: u32, flip: bool) {
    let (Some(tex), Some(atlas)) = (c.tex.characters, c.atlas) else { return };
    let m = &atlas.manifest.characters;
    let look = c.game.0.employees.iter().find(|e| e.role == Role::Founder).map(|e| e.look as u32).unwrap_or(0);
    let row = look % m.looks; // le fondateur est le premier rôle de l'atlas
    let (fw, fh) = (m.frame[0], m.frame[1]);
    let size = vec2(fw * 16.0, fh * m.looks as f32 * m.roles.len() as f32);
    let u0 = frame as f32 * fw / size.x;
    let u1 = (frame as f32 + 1.0) * fw / size.x;
    let (u0, u1) = if flip { (u1, u0) } else { (u0, u1) };
    let uv = Rect::from_min_max(pos2(u0, row as f32 * fh / size.y), pos2(u1, (row as f32 + 1.0) * fh / size.y));
    let base = v.at(feet);
    let r = Rect::from_min_size(base - vec2(m.anchor[0], m.anchor[1]) * v.s, vec2(fw, fh) * v.s);
    p.image(tex, r, uv, Color32::WHITE);
}

/// Petite médaille (ruban bleu, disque d'or), en pixels du décor.
fn medal(p: &egui::Painter, v: &View, at: (f32, f32)) {
    let px = |x: f32, y: f32, w: f32, h: f32, col: Color32| {
        p.rect_filled(Rect::from_min_size(v.at((at.0 + x, at.1 + y)), vec2(w, h) * v.s), 0.0, col);
    };
    px(-1.0, -3.0, 1.0, 2.0, RIBBON);
    px(0.0, -3.0, 1.0, 2.0, Color32::WHITE);
    px(1.0, -3.0, 1.0, 2.0, Color32::from_rgb(214, 48, 52));
    px(-1.0, -1.0, 3.0, 3.0, GOLD);
    px(0.0, 0.0, 1.0, 1.0, Color32::from_rgb(255, 244, 190));
}

/// Bulle de dialogue au-dessus d'un point du décor, en style rétro (cadre net, ombre).
fn bubble(p: &egui::Painter, v: &View, clip: Rect, tail: (f32, f32), text: &str) {
    let size = (v.s * 4.6).clamp(12.0, 16.0);
    let max_w = (clip.width() * 0.42).max(160.0);
    let galley = p.layout(text.to_string(), FontId::proportional(size), Color32::from_rgb(24, 26, 30), max_w);
    let pad = vec2(10.0, 7.0);
    let tip = v.at(tail);
    let mut r = Rect::from_min_size(tip - vec2(galley.size().x / 2.0 + pad.x, galley.size().y + pad.y * 2.0 + 12.0), galley.size() + pad * 2.0);
    // rester dans le cadre
    let dx = (clip.min.x + 6.0 - r.min.x).max(0.0) - (r.max.x - clip.max.x + 6.0).max(0.0);
    let dy = (clip.min.y + 6.0 - r.min.y).max(0.0);
    r = r.translate(vec2(dx, dy));
    p.rect_filled(r.translate(vec2(3.0, 3.0)), 0.0, Color32::from_black_alpha(140));
    p.rect_filled(r, 0.0, OFFWHITE);
    p.rect_stroke(r, 0.0, Stroke::new(2.0, Color32::from_rgb(24, 26, 30)), egui::StrokeKind::Inside);
    let base_x = tip.x.clamp(r.min.x + 10.0, r.max.x - 10.0);
    p.add(egui::Shape::convex_polygon(
        vec![pos2(base_x - 6.0, r.max.y - 1.0), pos2(base_x + 6.0, r.max.y - 1.0), pos2(tip.x, tip.y - 2.0)],
        OFFWHITE,
        Stroke::NONE,
    ));
    p.line_segment([pos2(base_x - 6.0, r.max.y), pos2(tip.x, tip.y - 2.0)], Stroke::new(2.0, Color32::from_rgb(24, 26, 30)));
    p.line_segment([pos2(base_x + 6.0, r.max.y), pos2(tip.x, tip.y - 2.0)], Stroke::new(2.0, Color32::from_rgb(24, 26, 30)));
    p.galley(r.min + pad, galley, Color32::from_rgb(24, 26, 30));
}

/// Bandeau de titre en haut à gauche de la scène.
fn caption(p: &egui::Painter, clip: Rect, text: &str, sub: &str) {
    let g = p.layout_no_wrap(text.to_string(), pixel(16.0), GREEN);
    let g2 = p.layout_no_wrap(sub.to_string(), pixel(11.0), LIGHTGRAY);
    let w = g.size().x.max(g2.size().x) + 20.0;
    let r = Rect::from_min_size(clip.min + vec2(10.0, 10.0), vec2(w, g.size().y + g2.size().y + 16.0));
    p.rect_filled(r, 0.0, Color32::from_rgba_unmultiplied(16, 20, 18, 225));
    p.rect_stroke(r, 0.0, Stroke::new(2.0, DEEP_GREEN), egui::StrokeKind::Inside);
    p.galley(r.min + vec2(10.0, 6.0), g, GREEN);
    p.galley(r.min + vec2(10.0, 10.0 + 16.0), g2, LIGHTGRAY);
}

fn walk_frame(t: f32) -> u32 {
    2 + ((t * 8.0) as u32 % 4)
}

pub fn show(p: &egui::Painter, rect: Rect, t: f32, c: &Ctx) {
    p.rect_filled(rect, 0.0, Color32::from_rgb(16, 20, 18));
    let v = View::new(rect);
    let (key, local) = if t < FT_START { ("scene_elysee", t) } else { ("scene_france_travail", t - FT_START) };
    if let Some(id) = c.tex.endings.get(key) {
        p.image(*id, v.rect(), Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
    let name = c.game.0.founder.name.clone();
    if t < FT_START {
        elysee(p, &v, rect, local, &name, c);
    } else {
        france_travail(p, &v, rect, local, c);
    }
    // fondu au noir entre les deux salles
    let fade = if t < FT_START { ((t - (FT_START - 0.5)) / 0.5).clamp(0.0, 1.0) } else { 1.0 - (local / 0.5).clamp(0.0, 1.0) };
    if fade > 0.0 {
        p.rect_filled(rect, 0.0, Color32::from_black_alpha((fade * 255.0) as u8));
    }
}

fn elysee(p: &egui::Painter, v: &View, clip: Rect, t: f32, name: &str, c: &Ctx) {
    const WALK: f32 = 2.8;
    let chest = (ELYSEE_END.0 - 0.5, ELYSEE_END.1 - 12.0);
    if t < WALK {
        hero(p, v, c, lerp(ELYSEE_START, ELYSEE_END, t / WALK), walk_frame(t), true);
    } else {
        hero(p, v, c, ELYSEE_END, ((t * 2.0) as u32) % 2, true);
    }
    // la médaille passe de la main du Président au sweat du héros
    let k = ((t - 3.1) / 0.9).clamp(0.0, 1.0);
    if t > 3.1 {
        let pos = lerp(PRESIDENT_HAND, chest, k);
        let hop = (k * std::f32::consts::PI).sin() * 8.0;
        medal(p, v, (pos.0, pos.1 - hop));
    }
    // flashs des photographes une fois la médaille remise
    if k >= 1.0 {
        for (i, cam) in CAMERAS.iter().enumerate() {
            let ph = (t * 2.3 + i as f32 * 0.37).fract();
            if ph < 0.12 {
                let c0 = v.at(*cam);
                let r = v.s * (4.0 + ph * 30.0);
                p.circle_filled(c0, r, Color32::from_white_alpha(170));
                p.rect_filled(Rect::from_center_size(c0, vec2(r * 3.0, v.s)), 0.0, Color32::from_white_alpha(200));
                p.rect_filled(Rect::from_center_size(c0, vec2(v.s, r * 3.0)), 0.0, Color32::from_white_alpha(200));
            }
        }
    }
    caption(p, clip, "PALAIS DE L'ÉLYSÉE", "Salle des fêtes, 18h30");
    if (1.0..3.1).contains(&t) {
        bubble(p, v, clip, (PRESIDENT_HAND.0 - 4.0, PRESIDENT_HAND.1 - 18.0), &format!("Approchez, {name}. La République a deux mots à vous dire."));
    } else if t >= 4.1 {
        bubble(
            p,
            v,
            clip,
            (PRESIDENT_HAND.0 - 4.0, PRESIDENT_HAND.1 - 18.0),
            "Chevalier de l'ordre national du Mérite, pour services rendus aux communs numériques !",
        );
    }
}

fn france_travail(p: &egui::Painter, v: &View, clip: Rect, t: f32, c: &Ctx) {
    const A: f32 = 1.4;
    const B: f32 = 3.0;
    let (pos, frame, flip) = if t < A {
        (lerp(FT_START_POS, FT_MID, t / A), walk_frame(t), true)
    } else if t < B {
        (lerp(FT_MID, FT_END, (t - A) / (B - A)), walk_frame(t), true)
    } else {
        (FT_END, 6 + ((t * 1.5) as u32) % 2, false)
    };
    hero(p, v, c, pos, frame, flip);
    let seated = t >= B;
    let chest = if seated { (pos.0 + 0.5, pos.1 - 11.0) } else { (pos.0 - 0.5, pos.1 - 12.0) };
    medal(p, v, chest);
    // écran d'appel : les numéros défilent jusqu'au vôtre
    let n = (38 + (t * 1.4) as u32).min(42);
    let at = v.at(FT_SCREEN);
    let blink = n == 42 && (t * 3.0).fract() < 0.5;
    let col = if blink { Color32::from_rgb(255, 230, 120) } else { Color32::from_rgb(250, 140, 60) };
    p.text(at, Align2::CENTER_CENTER, format!("{n:03}"), pixel((v.s * 7.0).max(11.0)), col);
    caption(p, clip, "FRANCE TRAVAIL", "Le lendemain, 9h00 · agence de Belleville");
    if (B + 0.4..B + 3.2).contains(&t) {
        bubble(p, v, clip, COUNSELOR_HEAD, "Re-bonjour ! Une médaille ? Je ne peux pas la mettre dans « expérience professionnelle »…");
    } else if t >= B + 3.2 {
        bubble(p, v, clip, COUNSELOR_HEAD, "Vous avez un projet ? Une startup, peut-être ? Atelier « Créer sa boîte », mardi 9h.");
    }
}
