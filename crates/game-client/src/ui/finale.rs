//! Fins de campagne : chaque fin a sa propre animation, puis un mini-jeu de 30 secondes
//! avant l'épilogue.
//!
//! * La Licorne Mondiale : la licorne vole et attrape les sacs de la Série C en évitant
//!   les nuages de « down round ».
//! * L'Introduction en Bourse : tenir le cours de VRDX en chassant les rumeurs et en
//!   saisissant les bonnes nouvelles.
//! * Le Rachat par NVIDIA : attraper la pluie de GPU dans la baie Veridix, en évitant
//!   les clauses d'earn-out.
//! * La Boîte Rentable : couper les dépenses inutiles avant qu'elles ne soient payées.
//! * Les Communs : fusionner les bonnes pull requests, ignorer le spam.
//!
//! Tout est dessiné avec le peintre d'egui, en coordonnées normalisées (0..1) du canevas.

use super::Ctx;
use super::style::*;
use bevy_egui::egui::{self, Align2, Color32, Pos2, Rect, Stroke, Vec2, pos2, vec2};

const INTRO_SECS: f32 = 7.0;
const GAME_SECS: f32 = 30.0;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Stage {
    #[default]
    Intro,
    Play,
    Score,
    Done,
}

#[derive(Clone, Debug)]
struct Item {
    pos: Pos2,
    vel: Vec2,
    kind: u8,
    life: f32,
    label: usize,
    value: f64,
}

#[derive(Clone, Debug)]
struct Spark {
    pos: Pos2,
    vel: Vec2,
    life: f32,
    color: Color32,
    text: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Finale {
    key: String,
    pub stage: Stage,
    t: f32,
    rng: u64,
    spawn: f32,
    score: f64,
    best: Option<f64>,
    player: f32,
    items: Vec<Item>,
    sparks: Vec<Spark>,
    history: Vec<f32>,
    hits: u32,
    last_time: f64,
}

impl Finale {
    fn rand(&mut self) -> f32 {
        // xorshift64* : déterministe, sans dépendance.
        let mut x = self.rng.max(1);
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        ((x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32) / (1u64 << 24) as f32
    }

    fn reset_game(&mut self, ending: &str) {
        self.t = 0.0;
        self.spawn = 0.0;
        self.items.clear();
        self.sparks.clear();
        self.history.clear();
        self.hits = 0;
        self.player = 0.5;
        self.score = match ending {
            "unicorn" => 1.0,
            "ipo" => 100.0,
            _ => 0.0,
        };
    }

    fn burst(&mut self, at: Pos2, color: Color32, n: usize, text: Option<String>) {
        for i in 0..n {
            let a = self.rand() * std::f32::consts::TAU;
            let s = 0.15 + self.rand() * 0.35;
            self.sparks.push(Spark {
                pos: at,
                vel: vec2(a.cos() * s, a.sin() * s - 0.1),
                life: 0.9,
                color,
                text: None,
            });
            if i == 0 && text.is_some() {
                self.sparks.push(Spark {
                    pos: at,
                    vel: vec2(0.0, -0.12),
                    life: 1.2,
                    color,
                    text: text.clone(),
                });
            }
        }
    }
}

/// Nom et règle du mini-jeu de chaque fin.
fn game_info(ending: &str) -> (&'static str, &'static str) {
    match ending {
        "unicorn" => (
            "La course à la Série C",
            "Souris ou ↑ ↓ : attrapez les sacs de la Série C, évitez les nuages de « down round ».",
        ),
        "ipo" => (
            "Tenez le cours",
            "Cliquez sur les rumeurs (rouges) pour les démentir, et sur les bonnes nouvelles (vertes) pour faire monter VRDX.",
        ),
        "acquisition" => (
            "La pluie de GPU",
            "Souris ou ← → : attrapez les GPU dans la baie Veridix, évitez les clauses d'earn-out.",
        ),
        "profitable" => (
            "La chasse aux dépenses",
            "Cliquez sur les dépenses inutiles avant qu'elles ne soient payées. Ne coupez pas les clients !",
        ),
        _ => (
            "Merge ou close",
            "Cliquez sur les bonnes pull requests (vertes) pour les fusionner. Ne fusionnez pas le spam (rouge).",
        ),
    }
}

fn score_text(ending: &str, score: f64) -> String {
    match ending {
        "unicorn" => format!("Valorisation : {:.1} Md€", score),
        "ipo" => format!(
            "Cours de clôture : {:.0} € ({:+.0} %)",
            score,
            score - 100.0
        ),
        "acquisition" => format!("{} GPU dans le prix de rachat", score.round() as i64),
        "profitable" => format!("EBITDA : {:+.0} k€", score),
        _ => format!("★ {} étoiles sur le dépôt", score.round() as i64),
    }
}

fn verdict(ending: &str, score: f64) -> &'static str {
    let good = match ending {
        "unicorn" => score >= 3.0,
        "ipo" => score >= 130.0,
        "acquisition" => score >= 40.0,
        "profitable" => score >= 400.0,
        _ => score >= 6000.0,
    };
    let ok = match ending {
        "unicorn" => score >= 1.8,
        "ipo" => score >= 100.0,
        "acquisition" => score >= 20.0,
        "profitable" => score >= 100.0,
        _ => score >= 2500.0,
    };
    if good {
        "Légendaire. Les Échos préparent déjà un portrait en une."
    } else if ok {
        "Honorable. Le board est content, votre mère aussi."
    } else {
        "Hmm. On dira que c'était « une année de transition »."
    }
}

/// Affiche la fin (animation, mini-jeu, score). Renvoie vrai quand il faut montrer l'épilogue.
pub fn show(ctx: &egui::Context, c: &mut Ctx, ending: &str, title: &str) -> bool {
    let key = format!("{ending}:{}:{}", c.game.0.seed, c.game.0.day);
    let id = egui::Id::new("finale_state");
    let mut f: Finale = ctx.data(|d| d.get_temp::<Finale>(id)).unwrap_or_default();
    if f.key != key {
        f = Finale {
            key,
            rng: c.game.0.seed ^ 0x9E37_79B9_7F4A_7C15,
            ..Default::default()
        };
        f.reset_game(ending);
    }
    if f.stage == Stage::Done {
        ctx.data_mut(|d| d.insert_temp(id, f));
        return true;
    }
    // Temps réel écoulé depuis l'image précédente (et non le pas « stable » d'egui, qui
    // suppose 60 images par seconde) : l'animation et le chrono restent justes même
    // quand le navigateur affiche peu d'images.
    let now = ctx.input(|i| i.time);
    let dt = if f.last_time > 0.0 { (now - f.last_time).clamp(0.0, 0.1) as f32 } else { 0.0 };
    f.last_time = now;
    ctx.request_repaint();

    super::widgets::dim(ctx, 220);
    let cr = ctx.content_rect();
    let w = (cr.width() - 40.0).clamp(320.0, 880.0);
    let h = (w * 9.0 / 16.0).min(cr.height() - 170.0).max(200.0);
    let w = w.min(h * 16.0 / 9.0);
    egui::Area::new(egui::Id::new("finale"))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            modal_frame().show(ui, |ui| {
                ui.set_width(w);
                let (name, rule) = game_info(ending);
                let header = match f.stage {
                    Stage::Intro => title.to_uppercase(),
                    Stage::Play => format!("MINI-JEU · {}", name.to_uppercase()),
                    _ => "RÉSULTAT".to_string(),
                };
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(header).font(pixel(13.0)).color(GREEN));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if f.stage == Stage::Play {
                            let left = (GAME_SECS - f.t).max(0.0);
                            ui.label(
                                egui::RichText::new(format!("{:.0} s", left.ceil()))
                                    .font(pixel(13.0))
                                    .color(ORANGE),
                            );
                        }
                    });
                });
                let (rect, resp) = ui.allocate_exact_size(vec2(w, h), egui::Sense::click());
                let painter = ui.painter_at(rect);
                let pointer = ctx
                    .input(|i| i.pointer.hover_pos())
                    .filter(|p| rect.contains(*p))
                    .map(|p| norm(rect, p));
                let click = if resp.clicked() {
                    resp.interact_pointer_pos().map(|p| norm(rect, p))
                } else {
                    None
                };
                let keys = ctx.input(|i| {
                    (
                        i.key_down(egui::Key::ArrowUp) || i.key_down(egui::Key::ArrowLeft),
                        i.key_down(egui::Key::ArrowDown) || i.key_down(egui::Key::ArrowRight),
                        i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space),
                    )
                });
                match f.stage {
                    Stage::Intro => {
                        f.t += dt;
                        intro(&painter, rect, ending, f.t);
                        let k = (f.t / 0.6).min(1.0);
                        painter.text(
                            rect.right_top() + vec2(-10.0, 10.0),
                            Align2::RIGHT_TOP,
                            "Cliquer ou Entrée pour passer",
                            pixel(11.0),
                            Color32::from_white_alpha((150.0 * k) as u8),
                        );
                        let secs = if ending == "commons" { INTRO_SECS + 5.0 } else { INTRO_SECS };
                        if f.t >= secs || click.is_some() || keys.2 {
                            f.stage = Stage::Play;
                            f.reset_game(ending);
                        }
                    }
                    Stage::Play => {
                        f.t += dt;
                        step(&mut f, ending, dt, pointer, click, keys.0, keys.1);
                        draw_game(&painter, rect, &f, ending);
                        if f.t >= GAME_SECS {
                            f.stage = Stage::Score;
                            f.best = Some(f.best.map_or(f.score, |b| b.max(f.score)));
                        }
                    }
                    Stage::Score | Stage::Done => {
                        draw_game(&painter, rect, &f, ending);
                        painter.rect_filled(rect, 0.0, Color32::from_black_alpha(215));
                        let c0 = rect.center();
                        painter.text(
                            c0 - vec2(0.0, 50.0),
                            Align2::CENTER_CENTER,
                            score_text(ending, f.score),
                            pixel(26.0),
                            GREEN,
                        );
                        painter.text(
                            c0 - vec2(0.0, 12.0),
                            Align2::CENTER_CENTER,
                            verdict(ending, f.score),
                            egui::FontId::proportional(16.0),
                            OFFWHITE,
                        );
                        if let Some(b) = f.best {
                            painter.text(
                                c0 + vec2(0.0, 18.0),
                                Align2::CENTER_CENTER,
                                format!("Meilleur score : {}", score_text(ending, b)),
                                egui::FontId::proportional(13.0),
                                MUTED,
                            );
                        }
                    }
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| match f.stage {
                    Stage::Intro | Stage::Play => {
                        ui.add(
                            egui::Label::new(egui::RichText::new(rule).size(13.0).color(LIGHTGRAY))
                                .wrap(),
                        );
                    }
                    _ => {
                        if super::widgets::big_button(ui, "Rejouer", true, 160.0).clicked() {
                            f.stage = Stage::Play;
                            f.reset_game(ending);
                        }
                        if super::widgets::big_button(ui, "Voir l'épilogue", true, 240.0).clicked()
                            || keys.2
                        {
                            f.stage = Stage::Done;
                        }
                    }
                });
                if f.stage == Stage::Play && ui.small_button("Passer le mini-jeu").clicked() {
                    f.stage = Stage::Score;
                }
            });
        });
    let done = f.stage == Stage::Done;
    ctx.data_mut(|d| d.insert_temp(id, f));
    done
}

/// Relance le mini-jeu depuis l'épilogue.
pub fn replay(ctx: &egui::Context) {
    let id = egui::Id::new("finale_state");
    ctx.data_mut(|d| {
        if let Some(mut f) = d.get_temp::<Finale>(id) {
            f.stage = Stage::Intro;
            f.t = 0.0;
            d.insert_temp(id, f);
        }
    });
}

fn norm(rect: Rect, p: Pos2) -> Pos2 {
    pos2(
        (p.x - rect.min.x) / rect.width(),
        (p.y - rect.min.y) / rect.height(),
    )
}

fn at(rect: Rect, p: Pos2) -> Pos2 {
    pos2(
        rect.min.x + p.x * rect.width(),
        rect.min.y + p.y * rect.height(),
    )
}

fn alpha(c: Color32, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a.clamp(0.0, 1.0) * 255.0) as u8)
}

const RAINBOW: [Color32; 6] = [
    Color32::from_rgb(228, 91, 91),
    Color32::from_rgb(246, 162, 60),
    Color32::from_rgb(250, 214, 92),
    Color32::from_rgb(124, 255, 82),
    Color32::from_rgb(110, 160, 230),
    Color32::from_rgb(170, 120, 230),
];
const GOLD: Color32 = Color32::from_rgb(250, 206, 72);
const NV: Color32 = Color32::from_rgb(118, 185, 0);

fn sky(p: &egui::Painter, rect: Rect, top: Color32, bottom: Color32) {
    let n = 24;
    for i in 0..n {
        let k = i as f32 / n as f32;
        let c = lerp_color(top, bottom, k);
        let y0 = rect.min.y + rect.height() * k;
        p.rect_filled(
            Rect::from_min_max(
                pos2(rect.min.x, y0),
                pos2(rect.max.x, y0 + rect.height() / n as f32 + 1.0),
            ),
            0.0,
            c,
        );
    }
}

fn lerp_color(a: Color32, b: Color32, k: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

fn stars(p: &egui::Painter, rect: Rect, t: f32, n: u32) {
    for i in 0..n {
        let x = ((i * 7919) % 1000) as f32 / 1000.0;
        let y = ((i * 104_729) % 600) as f32 / 1000.0;
        let tw = 0.5 + 0.5 * (t * 3.0 + i as f32).sin();
        p.circle_filled(
            at(rect, pos2(x, y)),
            1.2 + tw,
            alpha(OFFWHITE, 0.3 + 0.6 * tw),
        );
    }
}

/// La licorne, dessinée en formes simples ; `s` = taille en pixels.
fn draw_unicorn(p: &egui::Painter, c: Pos2, s: f32, t: f32) {
    let body = Rect::from_center_size(c, vec2(s * 1.6, s * 0.8));
    p.rect_filled(body, s * 0.35, OFFWHITE);
    let neck = Rect::from_min_size(c + vec2(s * 0.45, -s * 0.75), vec2(s * 0.35, s * 0.6));
    p.rect_filled(neck, 2.0, OFFWHITE);
    let head = Rect::from_min_size(c + vec2(s * 0.45, -s * 0.95), vec2(s * 0.7, s * 0.38));
    p.rect_filled(head, s * 0.12, OFFWHITE);
    p.circle_filled(c + vec2(s * 0.95, -s * 0.82), s * 0.05, NEAR_BLACK);
    p.line_segment(
        [c + vec2(s * 0.75, -s * 0.95), c + vec2(s * 0.95, -s * 1.4)],
        Stroke::new(s * 0.09, GOLD),
    );
    for (i, col) in RAINBOW.iter().enumerate() {
        let k = i as f32;
        p.line_segment(
            [
                c + vec2(s * 0.42 - k * 2.0, -s * 0.9 + k * s * 0.1),
                c + vec2(s * 0.2 - k * 3.0, -s * 0.6 + k * s * 0.12),
            ],
            Stroke::new(3.0, *col),
        );
    }
    let gallop = (t * 12.0).sin() * s * 0.15;
    for (dx, sign) in [(-0.55, 1.0), (-0.3, -1.0), (0.3, 1.0), (0.55, -1.0)] {
        let top = c + vec2(s * dx, s * 0.3);
        p.line_segment(
            [top, top + vec2(gallop * sign, s * 0.5)],
            Stroke::new(s * 0.1, LIGHTGRAY),
        );
    }
}

fn draw_gpu(p: &egui::Painter, c: Pos2, s: f32) {
    let r = Rect::from_center_size(c, vec2(s * 1.6, s));
    p.rect_filled(r, 2.0, Color32::from_rgb(30, 34, 30));
    p.rect_stroke(r, 2.0, Stroke::new(1.5, NV), egui::StrokeKind::Inside);
    p.rect_filled(Rect::from_center_size(c, vec2(s * 0.6, s * 0.5)), 1.0, NV);
    for i in 0..5 {
        let x = r.min.x + s * 0.2 + i as f32 * s * 0.28;
        p.line_segment(
            [pos2(x, r.max.y), pos2(x, r.max.y + s * 0.18)],
            Stroke::new(1.5, GOLD),
        );
    }
}

// ------------------------------------------------------------------ animations

fn intro(p: &egui::Painter, rect: Rect, ending: &str, t: f32) {
    let px = |v: f32| v * rect.height();
    match ending {
        "unicorn" => {
            sky(
                p,
                rect,
                Color32::from_rgb(20, 10, 40),
                Color32::from_rgb(80, 40, 100),
            );
            stars(p, rect, t, 60);
            let k = (t / 5.5).min(1.0);
            // Traînée arc-en-ciel puis licorne qui traverse l'écran en ondulant.
            for j in 0..40 {
                let tt = (k - j as f32 * 0.012).max(0.0);
                let x = -0.1 + tt * 1.1;
                let y = 0.55 - tt * 0.25 + (tt * 14.0).sin() * 0.04;
                for (i, col) in RAINBOW.iter().enumerate() {
                    p.circle_filled(
                        at(rect, pos2(x - 0.05, y + i as f32 * 0.012)),
                        px(0.012),
                        alpha(*col, 0.9 - j as f32 / 40.0),
                    );
                }
            }
            let x = -0.1 + k * 1.1;
            let y = 0.55 - k * 0.25 + (k * 14.0).sin() * 0.04;
            draw_unicorn(p, at(rect, pos2(x, y - 0.02)), px(0.09), t);
            let val = (k * k * 1000.0).round();
            p.text(
                at(rect, pos2(0.5, 0.12)),
                Align2::CENTER_CENTER,
                format!("VALORISATION : {val:.0} M€"),
                pixel(px(0.06)),
                GOLD,
            );
            if t > 5.5 {
                confetti(p, rect, t - 5.5);
                p.text(
                    at(rect, pos2(0.5, 0.85)),
                    Align2::CENTER_CENTER,
                    "SÉRIE C BOUCLÉE",
                    pixel(px(0.08)),
                    GREEN,
                );
            }
        }
        "ipo" => {
            sky(
                p,
                rect,
                Color32::from_rgb(230, 236, 244),
                Color32::from_rgb(250, 214, 160),
            );
            // Façade de la Bourse.
            let base = 0.78;
            p.rect_filled(
                Rect::from_min_max(at(rect, pos2(0.2, 0.30)), at(rect, pos2(0.8, 0.36))),
                0.0,
                Color32::from_rgb(210, 198, 172),
            );
            let tri = vec![
                at(rect, pos2(0.18, 0.30)),
                at(rect, pos2(0.5, 0.16)),
                at(rect, pos2(0.82, 0.30)),
            ];
            p.add(egui::Shape::convex_polygon(
                tri,
                Color32::from_rgb(226, 214, 190),
                Stroke::NONE,
            ));
            for i in 0..7 {
                let x = 0.24 + i as f32 * 0.087;
                p.rect_filled(
                    Rect::from_min_max(at(rect, pos2(x, 0.36)), at(rect, pos2(x + 0.035, base))),
                    0.0,
                    Color32::from_rgb(236, 228, 208),
                );
            }
            p.rect_filled(
                Rect::from_min_max(
                    at(rect, pos2(0.14, base)),
                    at(rect, pos2(0.86, base + 0.04)),
                ),
                0.0,
                Color32::from_rgb(186, 174, 150),
            );
            // La cloche se balance, « DING » à chaque extrémité.
            let swing = (t * 5.0).sin();
            let bell = at(rect, pos2(0.5 + swing * 0.03, 0.47));
            p.circle_filled(bell, px(0.06), GOLD);
            p.rect_filled(
                Rect::from_center_size(bell + vec2(0.0, px(0.05)), vec2(px(0.16), px(0.03))),
                2.0,
                GOLD,
            );
            p.circle_filled(bell + vec2(swing * px(0.03), px(0.08)), px(0.015), ORANGE);
            if swing.abs() > 0.9 {
                let side = if swing > 0.0 { 0.66 } else { 0.34 };
                p.text(
                    at(rect, pos2(side, 0.44)),
                    Align2::CENTER_CENTER,
                    "DING !",
                    pixel(px(0.07)),
                    RED,
                );
            }
            // Bandeau de cotation défilant.
            p.rect_filled(
                Rect::from_min_max(at(rect, pos2(0.0, 0.88)), at(rect, pos2(1.0, 1.0))),
                0.0,
                NEAR_BLACK,
            );
            let tape = "VRDX +38 %   ·   CAC 40 +0,2 %   ·   VRDX +41 %   ·   LES BANQUIERS SOURIENT   ·   VRDX +44 %   ·   ";
            let off = (t * 0.12) % 1.0;
            for k in 0..2 {
                p.text(
                    at(rect, pos2(1.0 - off + k as f32 - 1.0, 0.94)),
                    Align2::LEFT_CENTER,
                    tape,
                    pixel(px(0.05)),
                    GREEN,
                );
            }
            // Flèche verte qui monte.
            let k = (t / 6.0).min(1.0);
            let pts: Vec<Pos2> = (0..=40)
                .map(|i| {
                    let x = i as f32 / 40.0 * k;
                    at(
                        rect,
                        pos2(0.05 + x * 0.9, 0.7 - x * 0.55 - (x * 20.0).sin() * 0.03),
                    )
                })
                .collect();
            p.add(egui::Shape::line(
                pts,
                Stroke::new(4.0, Color32::from_rgb(40, 170, 70)),
            ));
            p.text(
                at(rect, pos2(0.5, 0.08)),
                Align2::CENTER_CENTER,
                "INTRODUCTION EN BOURSE",
                pixel(px(0.06)),
                Color32::from_rgb(40, 50, 90),
            );
        }
        "acquisition" => {
            p.rect_filled(rect, 0.0, Color32::from_rgb(8, 16, 8));
            // Circuit imprimé qui défile.
            for i in 0..16 {
                let y = ((i as f32 * 0.07 + t * 0.05) % 1.1) - 0.05;
                p.line_segment(
                    [at(rect, pos2(0.0, y)), at(rect, pos2(1.0, y))],
                    Stroke::new(1.0, alpha(NV, 0.15)),
                );
                let x = (i as f32 * 0.067 + 0.02) % 1.0;
                p.line_segment(
                    [at(rect, pos2(x, 0.0)), at(rect, pos2(x, 1.0))],
                    Stroke::new(1.0, alpha(NV, 0.1)),
                );
            }
            // La baie Veridix se remplit de GPU.
            let bx = Rect::from_min_max(at(rect, pos2(0.32, 0.62)), at(rect, pos2(0.68, 0.95)));
            p.rect_filled(bx, 3.0, Color32::from_rgb(24, 30, 26));
            p.rect_stroke(bx, 3.0, Stroke::new(2.0, GREEN), egui::StrokeKind::Inside);
            p.text(
                bx.center_bottom() - vec2(0.0, 8.0),
                Align2::CENTER_BOTTOM,
                "VERIDIX",
                pixel(px(0.05)),
                GREEN,
            );
            for i in 0..18 {
                let start = i as f32 * 0.3;
                let tt = t - start;
                if tt < 0.0 {
                    continue;
                }
                let lane = 0.37 + ((i * 37) % 26) as f32 / 100.0;
                let land = 0.88 - (i / 6) as f32 * 0.07;
                let y = (-0.1 + tt * 0.9).min(land);
                draw_gpu(p, at(rect, pos2(lane, y)), px(0.045));
            }
            if t > 5.0 {
                let s = 1.0 + (1.0 - ((t - 5.0) / 0.4).min(1.0)) * 1.5;
                let c0 = at(rect, pos2(0.5, 0.3));
                let r = Rect::from_center_size(c0, vec2(px(0.6) * s, px(0.16) * s));
                p.rect_stroke(r, 4.0, Stroke::new(4.0, RED), egui::StrokeKind::Inside);
                p.text(
                    c0,
                    Align2::CENTER_CENTER,
                    "ACQUIRED",
                    pixel(px(0.1) * s),
                    RED,
                );
            } else {
                p.text(
                    at(rect, pos2(0.5, 0.12)),
                    Align2::CENTER_CENTER,
                    "NVIDIA × VERIDIX",
                    pixel(px(0.07)),
                    NV,
                );
            }
        }
        "profitable" => {
            sky(
                p,
                rect,
                Color32::from_rgb(240, 170, 110),
                Color32::from_rgb(250, 214, 160),
            );
            // Immeuble aux fenêtres éclairées.
            let bld = Rect::from_min_max(at(rect, pos2(0.06, 0.3)), at(rect, pos2(0.36, 1.0)));
            p.rect_filled(bld, 0.0, Color32::from_rgb(226, 214, 190));
            for r in 0..6 {
                for col in 0..4 {
                    let w = Rect::from_min_size(
                        at(rect, pos2(0.08 + col as f32 * 0.07, 0.35 + r as f32 * 0.1)),
                        vec2(px(0.06), px(0.07)),
                    );
                    p.rect_filled(w, 0.0, Color32::from_rgb(250, 214, 120));
                }
            }
            p.text(
                at(rect, pos2(0.21, 0.26)),
                Align2::CENTER_CENTER,
                "VERIDIX",
                pixel(px(0.05)),
                DEEP_GREEN,
            );
            // Les pièces tombent et s'empilent en colonnes.
            let cols = 6;
            for i in 0..60 {
                let start = i as f32 * 0.09;
                let tt = t - start;
                if tt < 0.0 {
                    continue;
                }
                let col = i % cols;
                let level = (i / cols) as f32;
                let x = 0.45 + col as f32 * 0.08;
                let land = 0.92 - level * 0.035;
                let y = (-0.05 + tt * 1.6).min(land);
                let c0 = at(rect, pos2(x, y));
                p.add(egui::Shape::ellipse_filled(
                    c0,
                    vec2(px(0.03), px(0.014)),
                    GOLD,
                ));
                p.add(egui::Shape::ellipse_stroke(
                    c0,
                    vec2(px(0.03), px(0.014)),
                    Stroke::new(1.0, ORANGE),
                ));
            }
            // Une plante qui pousse.
            let g = (t / 6.0).min(1.0);
            let stem_top = at(rect, pos2(0.94, 0.95 - g * 0.3));
            p.line_segment(
                [at(rect, pos2(0.94, 0.95)), stem_top],
                Stroke::new(3.0, DEEP_GREEN),
            );
            for i in 0..(g * 5.0) as i32 {
                let y = 0.92 - i as f32 * 0.06;
                let s = if i % 2 == 0 { 1.0 } else { -1.0 };
                p.add(egui::Shape::ellipse_filled(
                    at(rect, pos2(0.94 + s * 0.02, y)),
                    vec2(px(0.025), px(0.012)),
                    Color32::from_rgb(60, 160, 80),
                ));
            }
            let ebitda = (t / 6.0).min(1.0) * 1_240.0;
            p.text(
                at(rect, pos2(0.66, 0.15)),
                Align2::CENTER_CENTER,
                format!("EBITDA : +{ebitda:.0} k€"),
                pixel(px(0.065)),
                DEEP_GREEN,
            );
        }
        _ if (4.0..8.0).contains(&t) => elysee(p, rect, t - 4.0),
        _ if t >= 8.0 => france_travail(p, rect, t - 8.0),
        _ => {
            p.rect_filled(rect, 0.0, Color32::from_rgb(12, 18, 16));
            // Un arbre de commits qui pousse branche par branche.
            let depth = (t / 0.8).min(7.0);
            branch(
                p,
                rect,
                pos2(0.5, 0.95),
                std::f32::consts::FRAC_PI_2,
                0.2,
                depth,
                0,
            );
            let stars_n = ((t / 6.0).min(1.0) * 10_000.0).round();
            p.text(
                at(rect, pos2(0.5, 0.1)),
                Align2::CENTER_CENTER,
                format!("★ {stars_n:.0}"),
                pixel(px(0.08)),
                GOLD,
            );
            // Des contributeurs arrivent du monde entier.
            for i in 0..14 {
                let tt = t - i as f32 * 0.35;
                if tt < 0.0 {
                    continue;
                }
                let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                let target = 0.08 + i as f32 * 0.065;
                let x = if side < 0.0 {
                    (-0.05 + tt * 0.6).min(target)
                } else {
                    (1.05 - tt * 0.6).max(target)
                };
                let c0 = at(rect, pos2(x, 0.95));
                let col = RAINBOW[i % RAINBOW.len()];
                p.circle_filled(
                    c0 - vec2(0.0, px(0.05)),
                    px(0.018),
                    Color32::from_rgb(230, 190, 150),
                );
                p.rect_filled(
                    Rect::from_center_size(c0 - vec2(0.0, px(0.015)), vec2(px(0.035), px(0.04))),
                    2.0,
                    col,
                );
            }
            p.text(
                at(rect, pos2(0.5, 0.2)),
                Align2::CENTER_CENTER,
                "git push --tags · licence libre",
                pixel(px(0.04)),
                GREEN,
            );
        }
    }
}

/// Un personnage simple vu de face (tête, corps), pour les scènes de fin.
fn person(p: &egui::Painter, rect: Rect, c: Pos2, h: f32, skin: Color32, suit: Color32, hair: Color32, long_hair: bool) {
    let px = |v: f32| v * rect.height();
    let base = at(rect, c);
    let body = Rect::from_center_size(base - vec2(0.0, px(h * 0.32)), vec2(px(h * 0.36), px(h * 0.5)));
    p.rect_filled(body, px(h * 0.06), suit);
    let head = base - vec2(0.0, px(h * 0.72));
    if long_hair {
        p.rect_filled(Rect::from_center_size(head + vec2(0.0, px(h * 0.06)), vec2(px(h * 0.26), px(h * 0.3))), px(h * 0.05), hair);
    }
    p.circle_filled(head, px(h * 0.11), skin);
    p.rect_filled(Rect::from_center_size(head - vec2(0.0, px(h * 0.08)), vec2(px(h * 0.22), px(h * 0.06))), px(h * 0.02), hair);
    p.circle_filled(head + vec2(-px(h * 0.04), 0.0), px(h * 0.012), Color32::from_rgb(30, 24, 20));
    p.circle_filled(head + vec2(px(h * 0.04), 0.0), px(h * 0.012), Color32::from_rgb(30, 24, 20));
}

/// La remise de médaille à l'Élysée : façade, drapeaux, tapis rouge, garde républicaine.
fn elysee(p: &egui::Painter, rect: Rect, t: f32) {
    let px = |v: f32| v * rect.height();
    sky(p, rect, Color32::from_rgb(120, 170, 220), Color32::from_rgb(210, 225, 240));
    // Façade en pierre claire, colonnes et fenêtres.
    let facade = Rect::from_min_max(at(rect, pos2(0.08, 0.22)), at(rect, pos2(0.92, 0.78)));
    p.rect_filled(facade, 0.0, Color32::from_rgb(232, 224, 204));
    p.rect_filled(Rect::from_min_max(at(rect, pos2(0.06, 0.18)), at(rect, pos2(0.94, 0.24))), 0.0, Color32::from_rgb(200, 192, 172));
    for i in 0..9 {
        let x = 0.12 + i as f32 * 0.095;
        p.rect_filled(Rect::from_min_max(at(rect, pos2(x, 0.3)), at(rect, pos2(x + 0.045, 0.44))), 0.0, Color32::from_rgb(90, 110, 130));
        p.rect_filled(Rect::from_min_max(at(rect, pos2(x - 0.01, 0.26)), at(rect, pos2(x + 0.055, 0.28))), 0.0, Color32::from_rgb(200, 192, 172));
    }
    // Le drapeau sur le toit, qui ondule.
    let pole = at(rect, pos2(0.86, 0.04));
    p.line_segment([pole, at(rect, pos2(0.86, 0.18))], Stroke::new(2.0, Color32::from_rgb(80, 80, 80)));
    for (k, col) in [Color32::from_rgb(40, 70, 160), OFFWHITE, Color32::from_rgb(210, 50, 50)].into_iter().enumerate() {
        let wave = (t * 5.0 + k as f32).sin() * px(0.006);
        let r = Rect::from_min_size(pole + vec2(px(0.04) * k as f32, wave), vec2(px(0.04), px(0.07)));
        p.rect_filled(r, 0.0, col);
    }
    // Tapis rouge et sol.
    p.rect_filled(Rect::from_min_max(at(rect, pos2(0.0, 0.78)), rect.max), 0.0, Color32::from_rgb(150, 140, 120));
    p.add(egui::Shape::convex_polygon(
        vec![at(rect, pos2(0.42, 0.78)), at(rect, pos2(0.58, 0.78)), at(rect, pos2(0.66, 1.0)), at(rect, pos2(0.34, 1.0))],
        Color32::from_rgb(170, 30, 40),
        Stroke::NONE,
    ));
    // Deux gardes républicains, casque à crinière.
    for x in [0.2, 0.8] {
        person(p, rect, pos2(x, 0.86), 0.32, Color32::from_rgb(236, 200, 170), Color32::from_rgb(30, 40, 80), Color32::from_rgb(200, 170, 60), false);
        p.line_segment([at(rect, pos2(x, 0.6)), at(rect, pos2(x - 0.015, 0.64))], Stroke::new(px(0.012), Color32::from_rgb(170, 30, 40)));
    }
    // Le Président (écharpe tricolore) remet la médaille au héros.
    let pres = pos2(0.42, 0.9);
    person(p, rect, pres, 0.42, Color32::from_rgb(232, 196, 166), Color32::from_rgb(34, 36, 48), Color32::from_rgb(120, 110, 100), false);
    for (k, col) in [Color32::from_rgb(40, 70, 160), OFFWHITE, Color32::from_rgb(210, 50, 50)].into_iter().enumerate() {
        let a = at(rect, pos2(0.40 + k as f32 * 0.008, 0.63));
        p.line_segment([a, a + vec2(px(0.07), px(0.13))], Stroke::new(px(0.008), col));
    }
    let hero = pos2(0.58, 0.9);
    person(p, rect, hero, 0.4, Color32::from_rgb(240, 200, 170), Color32::from_rgb(38, 125, 75), Color32::from_rgb(90, 60, 40), false);
    // La médaille voyage de la main du Président jusqu'au sweat du héros.
    let k = ((t - 0.6) / 1.4).clamp(0.0, 1.0);
    let from = at(rect, pos2(0.47, 0.72));
    let to = at(rect, pos2(0.585, 0.68));
    let m = from + (to - from) * k - vec2(0.0, (k * std::f32::consts::PI).sin() * px(0.06));
    p.line_segment([m - vec2(px(0.008), px(0.03)), m], Stroke::new(px(0.01), Color32::from_rgb(40, 90, 180)));
    p.circle_filled(m, px(0.016), GOLD);
    if k >= 1.0 {
        // Flashs des photographes.
        for i in 0..5 {
            let ph = (t * 3.0 + i as f32 * 1.7).fract();
            if ph < 0.15 {
                p.circle_filled(at(rect, pos2(0.1 + i as f32 * 0.2, 0.74)), px(0.04), Color32::from_white_alpha(200));
            }
        }
    }
    p.text(at(rect, pos2(0.5, 0.12)), Align2::CENTER_CENTER, "PALAIS DE L'ÉLYSÉE", pixel(px(0.05)), Color32::from_rgb(40, 50, 90));
    let fade = ((t - 1.6) / 0.6).clamp(0.0, 1.0);
    p.text(
        at(rect, pos2(0.5, 0.52)),
        Align2::CENTER_CENTER,
        "Ordre national du Mérite · pour services rendus aux communs",
        pixel(px(0.035)),
        alpha(Color32::from_rgb(30, 30, 40), fade),
    );
}

/// Le lendemain matin : retour à l'agence France Travail, la médaille sur le sweat.
fn france_travail(p: &egui::Painter, rect: Rect, t: f32) {
    let px = |v: f32| v * rect.height();
    p.rect_filled(rect, 0.0, Color32::from_rgb(214, 220, 226));
    p.rect_filled(Rect::from_min_max(at(rect, pos2(0.0, 0.8)), rect.max), 0.0, Color32::from_rgb(170, 176, 186));
    // Enseigne et écran d'appel des tickets.
    let sign = Rect::from_min_max(at(rect, pos2(0.06, 0.06)), at(rect, pos2(0.5, 0.18)));
    p.rect_filled(sign, 4.0, Color32::from_rgb(30, 60, 140));
    p.text(sign.center(), Align2::CENTER_CENTER, "FRANCE TRAVAIL", pixel(px(0.05)), OFFWHITE);
    let screen = Rect::from_min_max(at(rect, pos2(0.66, 0.06)), at(rect, pos2(0.94, 0.2)));
    p.rect_filled(screen, 2.0, Color32::from_rgb(20, 24, 22));
    let ticket = 38 + (t * 1.5) as i32;
    p.text(screen.center(), Align2::CENTER_CENTER, format!("TICKET {ticket:03}"), pixel(px(0.045)), Color32::from_rgb(250, 120, 60));
    // Les chaises en plastique et la file d'attente.
    for i in 0..4 {
        let x = 0.08 + i as f32 * 0.09;
        p.rect_filled(Rect::from_min_max(at(rect, pos2(x, 0.72)), at(rect, pos2(x + 0.06, 0.75))), 2.0, Color32::from_rgb(230, 120, 40));
        if i != 2 {
            person(p, rect, pos2(x + 0.03, 0.74), 0.26, Color32::from_rgb(220, 180, 150), RAINBOW[i % RAINBOW.len()], Color32::from_rgb(60, 50, 40), i % 2 == 1);
        }
    }
    // Le bureau de la conseillère.
    let desk = Rect::from_min_max(at(rect, pos2(0.6, 0.6)), at(rect, pos2(0.95, 0.8)));
    person(p, rect, pos2(0.82, 0.66), 0.3, Color32::from_rgb(200, 160, 130), Color32::from_rgb(120, 90, 140), Color32::from_rgb(70, 50, 40), true);
    p.rect_filled(desk, 2.0, Color32::from_rgb(140, 110, 80));
    p.rect_filled(Rect::from_min_max(at(rect, pos2(0.63, 0.5)), at(rect, pos2(0.73, 0.6))), 2.0, Color32::from_rgb(40, 44, 50));
    // Le héros entre, médaille au cou, et s'assoit en face.
    let x = (0.1 + t * 0.25).min(0.52);
    person(p, rect, pos2(x, 0.86), 0.34, Color32::from_rgb(240, 200, 170), Color32::from_rgb(38, 125, 75), Color32::from_rgb(90, 60, 40), false);
    let medal = at(rect, pos2(x + 0.005, 0.66));
    p.circle_filled(medal, px(0.014), GOLD);
    if x >= 0.52 {
        let b = Rect::from_min_size(at(rect, pos2(0.42, 0.28)), vec2(px(0.8), px(0.16)));
        p.rect_filled(b, 8.0, OFFWHITE);
        p.rect_stroke(b, 8.0, Stroke::new(2.0, Color32::from_rgb(30, 60, 140)), egui::StrokeKind::Inside);
        let line1 = "« Une médaille ? Je ne peux pas la mettre";
        let line2 = "dans « expérience professionnelle ». »";
        p.text(b.center() - vec2(0.0, px(0.03)), Align2::CENTER_CENTER, line1, egui::FontId::proportional(px(0.035)), Color32::from_rgb(30, 30, 40));
        p.text(b.center() + vec2(0.0, px(0.03)), Align2::CENTER_CENTER, line2, egui::FontId::proportional(px(0.035)), Color32::from_rgb(30, 30, 40));
    }
    p.text(at(rect, pos2(0.28, 0.24)), Align2::CENTER_CENTER, "Le lendemain, 9h00", pixel(px(0.04)), Color32::from_rgb(60, 70, 90));
}

fn branch(p: &egui::Painter, rect: Rect, from: Pos2, a: f32, len: f32, depth: f32, level: i32) {
    if depth <= 0.0 {
        return;
    }
    let k = depth.min(1.0);
    let to = pos2(
        from.x + a.cos() * len * k * 0.56,
        from.y - a.sin() * len * k,
    );
    let col = if level < 3 { DEEP_GREEN } else { GREEN };
    p.line_segment(
        [at(rect, from), at(rect, to)],
        Stroke::new((5.0 - level as f32).max(1.0), col),
    );
    p.circle_filled(at(rect, to), 3.0, OFFWHITE);
    if depth > 1.0 {
        branch(p, rect, to, a + 0.45, len * 0.72, depth - 1.0, level + 1);
        branch(p, rect, to, a - 0.45, len * 0.72, depth - 1.0, level + 1);
    }
}

fn confetti(p: &egui::Painter, rect: Rect, t: f32) {
    for i in 0..60 {
        let x = ((i * 37) % 100) as f32 / 100.0 + (t * 2.0 + i as f32).sin() * 0.02;
        let y = (((i * 53) % 100) as f32 / 300.0 + t * (0.3 + (i % 5) as f32 * 0.08)) % 1.0;
        let c = RAINBOW[i % RAINBOW.len()];
        p.rect_filled(
            Rect::from_center_size(at(rect, pos2(x, y)), vec2(6.0, 3.0)),
            0.0,
            c,
        );
    }
}

// ------------------------------------------------------------------ mini-jeux

const IPO_RUMORS: [&str; 6] = [
    "Rumeur : départ du CTO",
    "Short seller !",
    "« Bulle de l'IA »",
    "Panne chez un client",
    "Analyste sceptique",
    "Tweet ambigu du CEO",
];
const IPO_NEWS: [&str; 5] = [
    "Contrat signé !",
    "Résultats +40 %",
    "Note : achat fort",
    "Nouveau pays",
    "Brevet accordé",
];
const EXPENSES: [&str; 10] = [
    "Kombucha bio",
    "Offsite à Ibiza",
    "Licence SaaS oubliée",
    "Consultant stratégie",
    "Goodies fluo",
    "4 CRM différents",
    "Séminaire « vision »",
    "Drone de démo",
    "Baby-foot connecté",
    "Agence de « branding »",
];
const GOOD_PRS: [&str; 8] = [
    "fix: faute dans le README",
    "feat: détection du klingon",
    "perf: -40 % d'énergie",
    "docs: tuto en breton",
    "test: +300 tests",
    "fix: fuite mémoire",
    "feat: export PDF",
    "a11y: contraste",
];
const SPAM_PRS: [&str; 6] = [
    "add: token crypto",
    "Réécrit tout en COBOL",
    "chore: supprime les tests",
    "feat: mineur de bitcoin",
    "Ajoute des pubs",
    "Renomme tout en majuscules",
];

fn step(
    f: &mut Finale,
    ending: &str,
    dt: f32,
    pointer: Option<Pos2>,
    click: Option<Pos2>,
    up: bool,
    down: bool,
) {
    let difficulty = 1.0 + f.t / GAME_SECS;
    f.spawn -= dt;
    for s in &mut f.sparks {
        s.pos += s.vel * dt;
        s.vel.y += 0.3 * dt;
        s.life -= dt;
    }
    f.sparks.retain(|s| s.life > 0.0);
    match ending {
        "unicorn" | "acquisition" => {
            let vertical = ending == "unicorn";
            // Le joueur suit la souris, ou les flèches.
            if let Some(p) = pointer {
                let target = if vertical { p.y } else { p.x };
                f.player += (target - f.player) * (dt * 12.0).min(1.0);
            }
            if up {
                f.player -= dt * 0.9;
            }
            if down {
                f.player += dt * 0.9;
            }
            f.player = f.player.clamp(0.08, 0.92);
            if f.spawn <= 0.0 {
                f.spawn = (0.55 - f.t * 0.008).max(0.25);
                let bad = f.rand() < 0.3 + f.t * 0.006;
                let lane = 0.1 + f.rand() * 0.8;
                let speed = (0.32 + f.rand() * 0.2) * difficulty;
                let item = if vertical {
                    Item {
                        pos: pos2(1.05, lane),
                        vel: vec2(-speed, 0.0),
                        kind: bad as u8,
                        life: 9.0,
                        label: 0,
                        value: 0.0,
                    }
                } else {
                    Item {
                        pos: pos2(lane, -0.05),
                        vel: vec2(0.0, speed),
                        kind: bad as u8,
                        life: 9.0,
                        label: (f.rand() * 3.0) as usize,
                        value: 0.0,
                    }
                };
                f.items.push(item);
            }
            let player = if vertical {
                pos2(0.15, f.player)
            } else {
                pos2(f.player, 0.88)
            };
            let mut caught = Vec::new();
            for (i, it) in f.items.iter_mut().enumerate() {
                it.pos += it.vel * dt;
                it.life -= dt;
                let d = vec2((it.pos.x - player.x) * 1.78, it.pos.y - player.y).length();
                if d < if vertical { 0.1 } else { 0.11 } {
                    caught.push(i);
                }
            }
            for i in caught.into_iter().rev() {
                let it = f.items.remove(i);
                if it.kind == 0 {
                    let gain = if vertical {
                        0.08 + f.rand() as f64 * 0.1
                    } else {
                        1.0 + (f.rand() * 2.0).floor() as f64
                    };
                    f.score += gain;
                    f.hits += 1;
                    let txt = if vertical {
                        format!("+{:.0} M€", gain * 1000.0)
                    } else {
                        format!("+{gain:.0} GPU")
                    };
                    f.burst(it.pos, if vertical { GOLD } else { NV }, 8, Some(txt));
                } else {
                    let loss = if vertical { 0.35 } else { 4.0 };
                    f.score = (f.score - loss).max(0.0);
                    let txt = if vertical {
                        "DOWN ROUND -350 M€".to_string()
                    } else {
                        "EARN-OUT -4 GPU".to_string()
                    };
                    f.burst(it.pos, RED, 10, Some(txt));
                }
            }
            f.items
                .retain(|it| it.life > 0.0 && it.pos.x > -0.1 && it.pos.y < 1.1);
        }
        "ipo" => {
            // Le cours dérive ; chaque rumeur visible le tire vers le bas.
            let rumors = f.items.iter().filter(|i| i.kind == 1).count() as f64;
            let noise = (f.rand() as f64 - 0.5) * 6.0;
            f.score = (f.score + (noise - 1.5 - rumors * 4.0) * dt as f64).max(5.0);
            if f.history.len() < 600 && (f.t * 20.0) as usize >= f.history.len() {
                f.history.push(f.score as f32);
            }
            if f.spawn <= 0.0 {
                f.spawn = (0.9 - f.t * 0.012).max(0.45);
                let bad = f.rand() < 0.62;
                let pos = pos2(0.12 + f.rand() * 0.76, 0.12 + f.rand() * 0.5);
                let label = (f.rand()
                    * if bad {
                        IPO_RUMORS.len()
                    } else {
                        IPO_NEWS.len()
                    } as f32) as usize;
                f.items.push(Item {
                    pos,
                    vel: Vec2::ZERO,
                    kind: bad as u8,
                    life: if bad { 4.0 } else { 2.4 },
                    label,
                    value: 0.0,
                });
            }
            if let Some(cp) = click
                && let Some(i) = f
                    .items
                    .iter()
                    .position(|it| (it.pos.x - cp.x).abs() < half_width(item_text(ending, it).chars().count()) && (it.pos.y - cp.y).abs() < 0.06)
            {
                let it = f.items.remove(i);
                if it.kind == 1 {
                    f.score += 2.0;
                    f.burst(it.pos, GREEN, 6, Some("Démenti !".into()));
                } else {
                    f.score += 9.0;
                    f.burst(it.pos, GREEN, 10, Some("+9 €".into()));
                }
                f.hits += 1;
            }
            let mut expired = Vec::new();
            for (i, it) in f.items.iter_mut().enumerate() {
                it.life -= dt;
                if it.life <= 0.0 {
                    expired.push(i);
                }
            }
            for i in expired.into_iter().rev() {
                let it = f.items.remove(i);
                if it.kind == 1 {
                    f.score = (f.score - 7.0).max(5.0);
                    f.burst(it.pos, RED, 8, Some("-7 €".into()));
                }
            }
        }
        "profitable" => {
            if f.spawn <= 0.0 {
                f.spawn = (0.8 - f.t * 0.012).max(0.35);
                let client = f.rand() < 0.2;
                let pos = pos2(0.1 + f.rand() * 0.8, 0.12 + f.rand() * 0.72);
                let label = (f.rand() * EXPENSES.len() as f32) as usize;
                let value = if client {
                    120.0
                } else {
                    20.0 + (f.rand() * 80.0).round() as f64
                };
                f.items.push(Item {
                    pos,
                    vel: Vec2::ZERO,
                    kind: client as u8,
                    life: 3.5,
                    label,
                    value,
                });
            }
            for it in &mut f.items {
                it.life -= dt;
                if it.kind == 0 {
                    it.value += 18.0 * dt as f64;
                }
            }
            if let Some(cp) = click
                && let Some(i) = f
                    .items
                    .iter()
                    .position(|it| (it.pos.x - cp.x).abs() < half_width(item_text(ending, it).chars().count()) && (it.pos.y - cp.y).abs() < 0.06)
            {
                let it = f.items.remove(i);
                if it.kind == 0 {
                    f.score += it.value;
                    f.hits += 1;
                    f.burst(
                        it.pos,
                        GREEN,
                        8,
                        Some(format!("Coupé : +{:.0} k€", it.value)),
                    );
                } else {
                    f.score -= it.value;
                    f.burst(
                        it.pos,
                        RED,
                        10,
                        Some(format!("Client perdu : -{:.0} k€", it.value)),
                    );
                }
            }
            let mut expired = Vec::new();
            for (i, it) in f.items.iter().enumerate() {
                if it.life <= 0.0 {
                    expired.push(i);
                }
            }
            for i in expired.into_iter().rev() {
                let it = f.items.remove(i);
                if it.kind == 0 {
                    f.score -= it.value * 0.5;
                    f.burst(
                        it.pos,
                        RED,
                        6,
                        Some(format!("Payé : -{:.0} k€", it.value * 0.5)),
                    );
                } else {
                    f.score += it.value;
                    f.burst(
                        it.pos,
                        GREEN,
                        6,
                        Some(format!("Client signé : +{:.0} k€", it.value)),
                    );
                }
            }
        }
        _ => {
            if f.spawn <= 0.0 {
                f.spawn = (0.75 - f.t * 0.01).max(0.32);
                let spam = f.rand() < 0.35;
                let lane = (f.rand() * 4.0).floor() as usize;
                let label =
                    (f.rand() * if spam { SPAM_PRS.len() } else { GOOD_PRS.len() } as f32) as usize;
                let speed = (0.12 + f.rand() * 0.06) * difficulty;
                f.items.push(Item {
                    pos: pos2(0.14 + lane as f32 * 0.24, -0.05),
                    vel: vec2(0.0, speed),
                    kind: spam as u8,
                    life: 20.0,
                    label,
                    value: 0.0,
                });
            }
            for it in &mut f.items {
                it.pos += it.vel * dt;
            }
            if let Some(cp) = click
                && let Some(i) = f
                    .items
                    .iter()
                    .position(|it| (it.pos.x - cp.x).abs() < half_width(item_text(ending, it).chars().count()) && (it.pos.y - cp.y).abs() < 0.05)
            {
                let it = f.items.remove(i);
                if it.kind == 0 {
                    f.score += 250.0;
                    f.hits += 1;
                    f.burst(it.pos, GOLD, 8, Some("Merged ★ +250".into()));
                } else {
                    f.score = (f.score - 500.0).max(0.0);
                    f.burst(it.pos, RED, 10, Some("Spam fusionné ! -500".into()));
                }
            }
            let mut fell = Vec::new();
            for (i, it) in f.items.iter().enumerate() {
                if it.pos.y > 1.02 {
                    fell.push(i);
                }
            }
            for i in fell.into_iter().rev() {
                let it = f.items.remove(i);
                if it.kind == 0 {
                    f.score = (f.score - 50.0).max(0.0);
                    f.burst(
                        pos2(it.pos.x, 0.95),
                        MUTED,
                        4,
                        Some("PR oubliée -50".into()),
                    );
                }
            }
        }
    }
}

fn draw_game(p: &egui::Painter, rect: Rect, f: &Finale, ending: &str) {
    let px = |v: f32| v * rect.height();
    let small = egui::FontId::proportional(px(0.035).clamp(10.0, 15.0));
    match ending {
        "unicorn" => {
            sky(
                p,
                rect,
                Color32::from_rgb(20, 10, 40),
                Color32::from_rgb(70, 40, 100),
            );
            stars(p, rect, f.t, 40);
            for it in &f.items {
                let c = at(rect, it.pos);
                if it.kind == 0 {
                    p.circle_filled(c, px(0.045), Color32::from_rgb(70, 140, 70));
                    p.text(c, Align2::CENTER_CENTER, "€", pixel(px(0.05)), GOLD);
                } else {
                    for (dx, dy, r) in [(-0.04, 0.0, 0.04), (0.0, -0.02, 0.05), (0.04, 0.0, 0.04)] {
                        p.circle_filled(
                            c + vec2(px(dx), px(dy)),
                            px(r),
                            Color32::from_rgb(70, 70, 80),
                        );
                    }
                    p.text(
                        c + vec2(0.0, px(0.06)),
                        Align2::CENTER_TOP,
                        "DOWN ROUND",
                        pixel(px(0.03)),
                        RED,
                    );
                }
            }
            draw_unicorn(p, at(rect, pos2(0.15, f.player)), px(0.07), f.t);
        }
        "acquisition" => {
            p.rect_filled(rect, 0.0, Color32::from_rgb(8, 16, 8));
            for i in 0..12 {
                let y = (i as f32 * 0.09 + f.t * 0.05) % 1.0;
                p.line_segment(
                    [at(rect, pos2(0.0, y)), at(rect, pos2(1.0, y))],
                    Stroke::new(1.0, alpha(NV, 0.12)),
                );
            }
            for it in &f.items {
                let c = at(rect, it.pos);
                if it.kind == 0 {
                    draw_gpu(p, c, px(0.04));
                } else {
                    let r = Rect::from_center_size(c, vec2(px(0.1), px(0.07)));
                    p.rect_filled(r, 1.0, OFFWHITE);
                    p.rect_stroke(r, 1.0, Stroke::new(1.5, RED), egui::StrokeKind::Inside);
                    let label = ["EARN-OUT", "NON-CONC.", "CLAWBACK"][it.label % 3];
                    p.text(c, Align2::CENTER_CENTER, label, pixel(px(0.022)), RED);
                }
            }
            let bay =
                Rect::from_center_size(at(rect, pos2(f.player, 0.9)), vec2(px(0.3), px(0.08)));
            p.rect_filled(bay, 3.0, Color32::from_rgb(24, 30, 26));
            p.rect_stroke(bay, 3.0, Stroke::new(2.0, GREEN), egui::StrokeKind::Inside);
            p.text(
                bay.center(),
                Align2::CENTER_CENTER,
                "VERIDIX",
                pixel(px(0.035)),
                GREEN,
            );
        }
        "ipo" => {
            sky(
                p,
                rect,
                Color32::from_rgb(232, 238, 246),
                Color32::from_rgb(246, 226, 196),
            );
            // Graphique du cours en bas.
            let chart = Rect::from_min_max(at(rect, pos2(0.03, 0.68)), at(rect, pos2(0.97, 0.97)));
            p.rect_filled(chart, 3.0, NEAR_BLACK);
            let ipo_y = chart.max.y - (100.0 / 200.0) * chart.height();
            p.line_segment(
                [pos2(chart.min.x, ipo_y), pos2(chart.max.x, ipo_y)],
                Stroke::new(1.0, MUTED),
            );
            p.text(
                pos2(chart.min.x + 4.0, ipo_y - 2.0),
                Align2::LEFT_BOTTOM,
                "prix d'introduction : 100 €",
                small.clone(),
                MUTED,
            );
            if f.history.len() > 1 {
                let n = 600.0;
                let pts: Vec<Pos2> = f
                    .history
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        pos2(
                            chart.min.x + i as f32 / n * chart.width(),
                            chart.max.y - (v / 200.0).clamp(0.0, 1.0) * chart.height(),
                        )
                    })
                    .collect();
                let col = if f.score >= 100.0 { GREEN } else { RED };
                p.add(egui::Shape::line(pts, Stroke::new(2.5, col)));
            }
            p.text(
                at(rect, pos2(0.5, 0.06)),
                Align2::CENTER_CENTER,
                format!("VRDX : {:.1} €", f.score),
                pixel(px(0.06)),
                if f.score >= 100.0 { DEEP_GREEN } else { RED },
            );
            for it in &f.items {
                let c = at(rect, it.pos);
                let (text, col) = if it.kind == 1 {
                    (IPO_RUMORS[it.label % IPO_RUMORS.len()], RED)
                } else {
                    (IPO_NEWS[it.label % IPO_NEWS.len()], DEEP_GREEN)
                };
                bubble(p, c, text, col, px(0.24), px(0.1), small.clone(), it.life);
            }
        }
        "profitable" => {
            sky(
                p,
                rect,
                Color32::from_rgb(244, 232, 210),
                Color32::from_rgb(250, 214, 160),
            );
            let col = if f.score >= 0.0 { DEEP_GREEN } else { RED };
            p.text(
                at(rect, pos2(0.5, 0.06)),
                Align2::CENTER_CENTER,
                format!("EBITDA : {:+.0} k€", f.score),
                pixel(px(0.06)),
                col,
            );
            for it in &f.items {
                let c = at(rect, it.pos);
                if it.kind == 0 {
                    let text = format!(
                        "{} · {:.0} k€",
                        EXPENSES[it.label % EXPENSES.len()],
                        it.value
                    );
                    bubble(p, c, &text, RED, px(0.3), px(0.1), small.clone(), it.life);
                } else {
                    bubble(
                        p,
                        c,
                        "Client qui signe",
                        DEEP_GREEN,
                        px(0.26),
                        px(0.1),
                        small.clone(),
                        it.life,
                    );
                }
            }
        }
        _ => {
            p.rect_filled(rect, 0.0, Color32::from_rgb(13, 17, 23));
            for lane in 0..4 {
                let x = 0.14 + lane as f32 * 0.24;
                p.line_segment(
                    [at(rect, pos2(x, 0.0)), at(rect, pos2(x, 1.0))],
                    Stroke::new(1.0, Color32::from_rgb(40, 48, 58)),
                );
            }
            p.text(
                at(rect, pos2(0.5, 0.05)),
                Align2::CENTER_CENTER,
                format!("★ {:.0}", f.score),
                pixel(px(0.06)),
                GOLD,
            );
            for it in &f.items {
                let c = at(rect, it.pos);
                let (text, col) = if it.kind == 0 {
                    (
                        GOOD_PRS[it.label % GOOD_PRS.len()],
                        Color32::from_rgb(46, 160, 67),
                    )
                } else {
                    (
                        SPAM_PRS[it.label % SPAM_PRS.len()],
                        Color32::from_rgb(200, 60, 60),
                    )
                };
                let galley = p.layout_no_wrap(text.to_string(), small.clone(), OFFWHITE);
                let r = Rect::from_center_size(c, vec2(galley.size().x + 16.0, px(0.075).max(galley.size().y + 10.0)));
                p.rect_filled(r, 4.0, Color32::from_rgb(22, 27, 34));
                p.rect_stroke(r, 4.0, Stroke::new(2.0, col), egui::StrokeKind::Inside);
                p.galley(c - galley.size() / 2.0, galley, OFFWHITE);
            }
        }
    }
    for s in &f.sparks {
        let c = at(rect, s.pos);
        match &s.text {
            Some(t) => {
                p.text(
                    c + vec2(1.0, 1.0),
                    Align2::CENTER_CENTER,
                    t,
                    pixel(px(0.04)),
                    Color32::from_black_alpha((s.life.min(1.0) * 200.0) as u8),
                );
                p.text(
                    c,
                    Align2::CENTER_CENTER,
                    t,
                    pixel(px(0.04)),
                    alpha(s.color, s.life),
                );
            }
            None => {
                p.circle_filled(c, 3.0, alpha(s.color, s.life));
            }
        }
    }
    // Score permanent en haut à gauche pour les jeux d'adresse.
    if matches!(ending, "unicorn" | "acquisition") {
        p.text(
            rect.min + vec2(10.0, 10.0),
            Align2::LEFT_TOP,
            score_text(ending, f.score),
            pixel(px(0.045)),
            GOLD,
        );
    }
}

/// Texte affiché sur un élément cliquable (sert aussi à dimensionner sa zone de clic).
fn item_text(ending: &str, it: &Item) -> String {
    match (ending, it.kind) {
        ("ipo", 1) => IPO_RUMORS[it.label % IPO_RUMORS.len()].to_string(),
        ("ipo", _) => IPO_NEWS[it.label % IPO_NEWS.len()].to_string(),
        ("profitable", 0) => format!("{} · {:.0} k€", EXPENSES[it.label % EXPENSES.len()], it.value),
        ("profitable", _) => "Client qui signe".to_string(),
        (_, 0) => GOOD_PRS[it.label % GOOD_PRS.len()].to_string(),
        _ => SPAM_PRS[it.label % SPAM_PRS.len()].to_string(),
    }
}

/// Demi-largeur (en fraction du canevas) d'une bulle de `chars` caractères.
fn half_width(chars: usize) -> f32 {
    (chars as f32 * 0.0048 + 0.025).max(0.07)
}

/// Bulle de texte cliquable qui s'estompe avec le temps restant.
#[allow(clippy::too_many_arguments)]
fn bubble(
    p: &egui::Painter,
    c: Pos2,
    text: &str,
    col: Color32,
    w: f32,
    h: f32,
    font: egui::FontId,
    life: f32,
) {
    // La bulle prend la largeur de son texte : jamais de texte qui déborde.
    let galley = p.layout_no_wrap(text.to_string(), font, col);
    let w = (galley.size().x + 20.0).max(w);
    let r = Rect::from_center_size(c, vec2(w, h.max(galley.size().y + 12.0)));
    p.rect_filled(r, 6.0, OFFWHITE);
    p.rect_stroke(r, 6.0, Stroke::new(2.5, col), egui::StrokeKind::Inside);
    p.galley(c - galley.size() / 2.0 - vec2(0.0, 2.0), galley, col);
    let k = (life / 4.0).clamp(0.0, 1.0);
    p.rect_filled(
        Rect::from_min_size(pos2(r.min.x + 4.0, r.max.y - 5.0), vec2((w - 8.0) * k, 3.0)),
        1.0,
        col,
    );
}
