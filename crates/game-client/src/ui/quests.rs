//! Left panel: level, quests with progress bars, and alerts.

use super::style::*;
use super::widgets::*;
use super::{Ctx, Tab};
use bevy_egui::egui::{self, RichText};
use game_content::dialogue;
use game_core::content::QuestKind;
use game_core::employees::Assignment;
use game_core::funding::{AppStatus, RaiseStatus};

pub fn left(root: &mut egui::Ui, c: &mut Ctx, narrow: bool) {
    egui::Panel::left("quests")
        .resizable(true)
        .default_size(if narrow { 240.0 } else { 290.0 })
        .min_size(220.0)
        .max_size(420.0)
        .frame(panel_frame())
        .show(root, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| body(ui, c));
        });
}

fn body(ui: &mut egui::Ui, c: &mut Ctx) {
    let level = c.game.0.campaign.level;
    let Some(def) = c.content.0.level(level) else { return };
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("NIVEAU {level}/12")).font(pixel(11.0)).color(MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("◂").on_hover_text("Masquer").clicked() {
                c.ui.left_open = false;
            }
        });
    });
    ui.label(RichText::new(&def.title).font(pixel(22.0)).color(GREEN));
    ui.label(RichText::new(&def.objective).size(13.0).color(LIGHTGRAY));
    ui.add_space(6.0);
    let def = def.clone();
    for (i, q) in def.quests.iter().enumerate() {
        let done = c.game.0.campaign.quests_done.get(i).copied().unwrap_or(false);
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (mark, col) = if done { ("✔", GREEN) } else { ("○", LIGHTGRAY) };
                ui.label(RichText::new(mark).color(col).strong());
                let txt = RichText::new(&q.label).color(if done { GREEN } else { OFFWHITE });
                let r = ui.add(egui::Label::new(txt).wrap());
                let mut tip = String::new();
                if q.kind == QuestKind::Maintain {
                    tip.push_str("À maintenir : doit être vrai au moment où tous les autres objectifs sont atteints.\n");
                }
                if !q.hint.is_empty() {
                    tip.push_str(&q.hint);
                }
                if !tip.is_empty() {
                    r.on_hover_text(tip);
                }
            });
            if let Some(p) = c.game.0.progress(&q.condition)
                && !(done && q.kind == QuestKind::Reach) {
                    let frac = p.fraction() as f32;
                    let cur = fmt_progress(p.current, p.target);
                    bar(ui, frac, if frac >= 1.0 { DEEP_GREEN } else { GRAPHITE_3 }, cur, ui.available_width());
                }
        });
        ui.add_space(2.0);
    }
    ui.add_space(8.0);
    alerts(ui, c);
    if c.settings.hints {
        ui.add_space(8.0);
        hints(ui, c);
    }
}

fn fmt_progress(cur: f64, target: f64) -> String {
    if target >= 1000.0 {
        format!("{} / {}", dialogue::money(cur), dialogue::money(target))
    } else if target < 20.0 && target.fract() != 0.0 || cur.fract() != 0.0 && target < 100.0 {
        format!("{} / {}", dialogue::decimal(cur, 1), dialogue::decimal(target, 1))
    } else {
        format!("{:.0} / {:.0}", cur, target)
    }
}

fn alert(ui: &mut egui::Ui, text: &str, color: egui::Color32) -> egui::Response {
    egui::Frame::new()
        .fill(GRAPHITE_2)
        .stroke(egui::Stroke::new(1.0, color))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .corner_radius(2)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(egui::Label::new(RichText::new(text).size(12.5).color(color)).wrap());
        })
        .response
        .interact(egui::Sense::click())
}

fn alerts(ui: &mut egui::Ui, c: &mut Ctx) {
    let g = &c.game.0;
    let m = &g.metrics;
    let mut list: Vec<(String, egui::Color32, Option<Tab>)> = Vec::new();
    if g.finance.cash < 0.0 {
        list.push(("🚨 Trésorerie négative : levez, vendez ou réduisez les coûts immédiatement !".into(), RED, Some(Tab::Finance)));
    } else if m.runway < 3.0 {
        list.push((format!("⚠ Runway de {} seulement.", dialogue::months(m.runway)), ORANGE, Some(Tab::Funding)));
    }
    let idle = g.employees.iter().filter(|e| e.assignment == Assignment::Unassigned && e.is_available(g.day)).count();
    if idle > 0 {
        list.push((format!("{idle} personne(s) sans affectation."), ORANGE, Some(Tab::Team)));
    }
    if g.employees.len() as u32 > m.seats {
        list.push(("Pas assez de bureaux : productivité et moral en baisse.".into(), ORANGE, Some(Tab::Office)));
    }
    if g.tasks.iter().any(|t| !g.employees.iter().any(|e| e.assignment == Assignment::Task(t.instance))) {
        list.push(("Un chantier n'a personne dessus.".into(), ORANGE, Some(Tab::Product)));
    }
    if g.funding.applications.iter().any(|a| a.status == AppStatus::Drafting)
        && !g.employees.iter().any(|e| e.assignment == Assignment::Admin || (e.role == game_core::types::Role::Cfo))
    {
        list.push(("Dossier en attente : affectez quelqu'un à « Administratif ».".into(), ORANGE, Some(Tab::Funding)));
    }
    if let Some(fr) = &g.funding.fundraising {
        if fr.status == RaiseStatus::Pitching && !g.employees.iter().any(|e| e.assignment == Assignment::Fundraising) {
            list.push(("Levée en cours : quelqu'un doit pitcher (« Levée de fonds »).".into(), ORANGE, Some(Tab::Funding)));
        }
        if fr.status == RaiseStatus::Offers {
            list.push(("💶 Des offres d'investissement vous attendent !".into(), GREEN, Some(Tab::Funding)));
        }
    }
    if !g.candidates.is_empty() {
        list.push((format!("{} candidat(s) à examiner.", g.candidates.len()), LIGHTGRAY, Some(Tab::Team)));
    }
    if g.product.drift > 6.0 {
        list.push(("La précision dérive : pensez au réentraînement.".into(), ORANGE, Some(Tab::Product)));
    }
    if m.tech_debt > 40.0 {
        list.push(("Dette technique élevée : incidents et lenteur en vue.".into(), ORANGE, Some(Tab::Product)));
    }
    if m.fatigue > 60.0 {
        list.push(("L'équipe est épuisée : risque de burn-out.".into(), RED, Some(Tab::Team)));
    }
    if g.crunch {
        list.push(("Mode crunch actif (week-ends travaillés).".into(), ORANGE, Some(Tab::Team)));
    }
    if list.is_empty() {
        return;
    }
    subheading(ui, "Alertes");
    for (t, col, tab) in list {
        if alert(ui, &t, col).clicked()
            && let Some(tab) = tab {
                c.ui.tab = tab;
                c.ui.right_open = true;
            }
    }
}

/// Contextual first-steps guidance (the first ten minutes).
fn hints(ui: &mut egui::Ui, c: &mut Ctx) {
    let g = &c.game.0;
    let tip = if g.campaign.level == 1 && g.tasks.is_empty() && !g.product.has("mvp_text") {
        Some("Ouvrez « Produit & R&D » (P) et lancez « Détection de texte (MVP) ». Puis cliquez sur vous dans le bureau pour vous affecter au chantier.")
    } else if g.campaign.level == 1 && g.funding.applications.is_empty() {
        Some("Dans « Financement » (F), ouvrez le dossier France Travail, puis passez une journée en « Administratif ».")
    } else if g.campaign.level == 1 && g.product.has("mvp_text") && g.market.total_customers() < 3 {
        Some("Le MVP existe ! Affectez-vous aux Ventes ou au Marketing pour trouver des bêta-testeurs.")
    } else if g.campaign.level == 2 && g.employees.len() == 1 {
        Some("Équipe → Recruter. France Travail est gratuit mais lent ; LinkedIn coûte 400 € mais va plus vite.")
    } else if g.campaign.level == 2 && !g.product.launched {
        Some("Le lancement public fait passer les bêta-testeurs en clients payants. Il faut d'abord le dashboard.")
    } else {
        None
    };
    if let Some(t) = tip {
        egui::Frame::new().fill(egui::Color32::from_rgb(28, 46, 34)).inner_margin(egui::Margin::same(8)).corner_radius(3).show(ui, |ui| {
            ui.label(RichText::new("💡 Conseil").strong().color(GREEN));
            ui.add(egui::Label::new(RichText::new(t).size(12.5)).wrap());
        });
    }
}
