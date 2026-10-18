//! Barre du haut : trésorerie, MRR, runway, équipe, clients, churn, date, vitesse.

use super::style::*;
use super::widgets::*;
use super::{Ctx, Tab};
use crate::states::AppState;
use bevy_egui::egui::{self, RichText};
use game_content::dialogue;

const MONTHS: [&str; 12] = ["janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.", "nov.", "déc."];

/// Calendrier du jeu : jour 0 = lundi 3 novembre 2025 (jour de la démission).
pub fn date_label(day: u32) -> String {
    // Les mois comptables de 28 jours sont une abstraction de jeu ; le calendrier
    // montré au joueur est le vrai.
    let mut d = 3 + day as i64;
    let mut m = 10usize; // novembre (indice à partir de 0)
    let mut y = 2025;
    let dim = |m: usize, y: i64| -> i64 {
        match m {
            1 => if y % 4 == 0 { 29 } else { 28 },
            3 | 5 | 8 | 10 => 30,
            _ => 31,
        }
    };
    while d > dim(m, y) {
        d -= dim(m, y);
        m += 1;
        if m == 12 {
            m = 0;
            y += 1;
        }
    }
    format!("{d} {} {y}", MONTHS[m])
}

fn stat(ui: &mut egui::Ui, c: &Ctx, icon_key: &str, value: String, color: egui::Color32, label: &str, help: &str) -> egui::Response {
    let r = ui
        .horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            icon(ui, c.tex, c.atlas, icon_key, 18.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(RichText::new(label).size(10.5).color(MUTED));
                ui.label(RichText::new(value).font(bold(15.0)).color(color));
            });
        })
        .response;
    r.on_hover_text(help)
}

pub fn top_bar(root: &mut egui::Ui, c: &mut Ctx) {
    egui::Panel::top("hud")
        .exact_size(52.0)
        .frame(egui::Frame::new().fill(NEAR_BLACK).inner_margin(egui::Margin::symmetric(10, 6)))
        .show(root, |ui| {
            ui.horizontal_centered(|ui| {
                // Sur les petits écrans, on retire les indicateurs secondaires plutôt que de
                // les laisser passer sous les boutons de vitesse.
                let width = ui.available_width();
                ui.spacing_mut().item_spacing.x = if width < 1200.0 { 10.0 } else { 16.0 };
                ui.label(RichText::new("VERIDIX").font(pixel(22.0)).color(GREEN))
                    .on_hover_text("Détection de contenus générés par IA. « Fiable » à 53 % au départ.");
                let g = &c.game.0;
                let m = &g.metrics;
                let cash = g.finance.cash;
                let cash_col = if cash < 0.0 { RED } else if m.runway < 3.0 { ORANGE } else { OFFWHITE };
                stat(ui, c, "cash", money(cash), cash_col, "Trésorerie",
                    "Argent disponible. Si elle reste négative 14 jours (ou passe sous -50 000 €) : redressement judiciaire.");
                stat(ui, c, "mrr", money(m.mrr), GREEN, "MRR",
                    "Revenu mensuel récurrent = Σ clients payants × prix mensuel.\nARR (run-rate) = MRR × 12.");
                let runway = if m.runway.is_infinite() { "∞ rentable".to_string() } else { dialogue::months(m.runway) };
                let rr = stat(ui, c, "runway", runway, if m.runway < 3.0 { RED } else if m.runway < 6.0 { ORANGE } else { OFFWHITE }, "Runway",
                    "Mois avant d'être à court d'argent au rythme actuel.\nRunway = trésorerie / burn net, burn net = dépenses − revenus.");
                super::tutorial::mark(ui, "runway", rr.rect);
                stat(ui, c, "people", format!("{} / {} places", g.employees.len(), m.seats), if g.employees.len() as u32 > m.seats { ORANGE } else { OFFWHITE }, "Équipe",
                    "Personnes / places assises. Sans bureau, chacun travaille moins bien et le moral baisse.");
                if width >= 1000.0 {
                let cust = if g.product.launched { format!("{}", m.paying_customers) } else { format!("{} (bêta)", m.customers) };
                stat(ui, c, "customers", cust, OFFWHITE, "Clients", "Comptes clients actifs. Avant le lancement, ce sont des bêta-testeurs gratuits.");
                // Sans clients, le churn n'est qu'une projection pour le produit
                // actuel : on l'affiche, sans déclencher l'alarme.
                let (churn_txt, churn_col) = if m.customers == 0 {
                    (format!("{:.1} % (proj.)", m.churn), LIGHTGRAY)
                } else {
                    (format!("{:.1} %/mois", m.churn), if m.churn > 10.0 { RED } else if m.churn > 6.0 { ORANGE } else { GREEN })
                };
                if width >= 1180.0 {
                stat(ui, c, "churn", churn_txt, churn_col, "Churn",
                    "Part des clients qui résilient chaque mois (projection).\nDépend de la qualité, des faux positifs, du prix, de la concurrence, du support et des incidents.");
                }
                }
                stat(ui, c, "day", date_label(g.day), LIGHTGRAY, &format!("Jour {} · niveau {}", g.day, g.campaign.level), "Un jour simulé par tick ; un mois comptable = 28 jours.");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("☰").on_hover_text("Menu (Échap)").clicked() {
                        c.next_state.set(AppState::Paused);
                        *c.click += 1;
                    }
                    if ui.button("?").on_hover_text("Aide et raccourcis").clicked() {
                        c.ui.show_help = !c.ui.show_help;
                    }
                    ui.add_space(6.0);
                    let mut speed_rect = egui::Rect::NOTHING;
                    for (s, label, tip) in [(4u8, "4×", "Très rapide (3)"), (2, "2×", "Rapide (2)"), (1, "1×", "Normal (1)"), (0, "⏸", "Pause (Espace)")] {
                        let sel = c.clock.speed == s;
                        let b = egui::Button::selectable(sel, RichText::new(label).size(15.0)).min_size(egui::vec2(34.0, 30.0));
                        let r = ui.add(b);
                        speed_rect = speed_rect.union(r.rect);
                        if r.on_hover_text(tip).clicked() {
                            c.clock.set_speed(s);
                            *c.click += 1;
                        }
                    }
                    super::tutorial::mark(ui, "speed", speed_rect);
                    if !c.ui.right_open && {
                        let r = ui.button("Panneaux ▸");
                        super::tutorial::mark(ui, "panels", r.rect);
                        r.clicked()
                    } {
                        c.ui.right_open = true;
                        c.ui.tab = Tab::Team;
                    }
                    if !c.ui.left_open && ui.button("◂ Objectifs").clicked() {
                        c.ui.left_open = true;
                    }
                });
            });
        });
}
