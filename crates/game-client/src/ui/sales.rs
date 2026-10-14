//! Onglet Ventes & marketing : segments, prix, budget marketing, marchés.

use super::style::*;
use super::widgets::*;
use super::Ctx;
use bevy_egui::egui::{self, RichText};
use game_content::characters::segment_name;
use game_content::dialogue;
use game_core::Command;
use game_core::customers::{churn_breakdown, conversion_prob};
use game_core::types::Segment;

pub fn show(ui: &mut egui::Ui, c: &mut Ctx) {
    heading(ui, "Ventes & marketing");
    let g = &c.game.0;
    let m = &g.metrics;
    ui.columns(2, |cols| {
        kv(&mut cols[0], "MRR", money(m.mrr), GREEN, "Revenu mensuel récurrent.");
        kv(&mut cols[0], "ARPA", money(m.arpa), OFFWHITE, "Revenu moyen par compte = MRR / clients payants.");
        kv(&mut cols[0], "Leads en stock", format!("{:.0}", m.leads), OFFWHITE, "Prospects à travailler. Ils refroidissent (-4 %/jour).");
        kv(&mut cols[1], "CAC", money(m.cac), OFFWHITE, "Coût d'acquisition = (budget marketing + salaires ventes/marketing) / nouveaux clients sur 28 jours.");
        kv(&mut cols[1], "LTV", money(m.ltv), OFFWHITE, "Valeur vie client = ARPA × marge brute / churn mensuel.");
        kv(&mut cols[1], "LTV / CAC", dialogue::decimal(m.ltv_cac, 1), if m.ltv_cac >= 3.0 { GREEN } else { ORANGE }, "Au-dessus de 3 : la croissance paie. En dessous de 1 : chaque client vous appauvrit.");
    });
    if !g.product.launched {
        egui::Frame::new().fill(egui::Color32::from_rgb(40, 40, 26)).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
            ui.add(egui::Label::new(RichText::new("Bêta gratuite : les clients ne paient pas encore. Le lancement public (Produit) active les plans payants.").color(ORANGE)).wrap());
        });
    }
    let w = g.last_work;
    muted(ui, format!("Capacité commerciale : {:.1} pts/jour · marketing : {:.1} pts/jour · ops/support : {:.1}", w.sales, w.marketing, w.ops));

    subheading(ui, "Budget marketing mensuel");
    let mut budget = c.game.0.market.marketing_budget as f32;
    let max = (c.game.0.metrics.mrr as f32 * 1.5).clamp(20_000.0, 400_000.0);
    let r = ui.add(egui::Slider::new(&mut budget, 0.0..=max).step_by(100.0).custom_formatter(|v, _| dialogue::money(v)));
    r.clone().on_hover_text("Publicité, salons, sponsoring. Rendement décroissant : chaque marché a un nombre limité de prospects, et les enchères montent avec la concurrence.");
    if r.drag_stopped() || (r.changed() && !r.dragged()) {
        c.run(Command::SetMarketingBudget(budget as f64));
    }

    let inp = c.game.0.funnel_inputs(&c.content.0);
    for seg in Segment::ALL {
        let def = c.content.0.segment(seg).clone();
        let unlocked = c.game.0.segment_unlocked(&c.content.0, seg);
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(segment_name(seg)).strong().color(if unlocked { GREEN } else { MUTED }));
                if !unlocked {
                    ui.label(RichText::new("🔒 plus tard dans la campagne").size(12.0).color(MUTED));
                }
            });
            if !unlocked {
                return;
            }
            let s = c.game.0.market.seg(seg).clone();
            ui.columns(3, |cols| {
                cols[0].label(RichText::new(format!("{} clients", s.customers)).strong());
                cols[0].label(RichText::new(format!("{:.1} leads/j", s.leads_per_day)).size(12.0).color(LIGHTGRAY));
                let conv = conversion_prob(&c.content.0, seg, &s, &inp);
                cols[1].label(RichText::new(format!("conv. {:.0} %", conv * 100.0)).size(12.5))
                    .on_hover_text("Probabilité qu'un lead travaillé signe : qualité, prix, réputation, concurrence, fonctionnalités requises.");
                let churn_col = if s.churn_rate > 10.0 { RED } else if s.churn_rate > 6.0 { ORANGE } else { GREEN };
                let mut tip = String::from("Churn mensuel projeté :\n");
                for (label, v) in churn_breakdown(&c.content.0, seg, &s, &inp) {
                    if v > 0.05 {
                        tip.push_str(&format!("  {label} : +{:.1} pt\n", v));
                    }
                }
                cols[1].label(RichText::new(format!("churn {:.1} %", s.churn_rate)).size(12.5).color(churn_col)).on_hover_text(tip);
                cols[2].label(RichText::new(format!("{}/mois", money(s.customers as f64 * s.price))).size(12.5).color(GREEN));
            });
            // Fonctionnalités requises.
            let missing: Vec<String> = def
                .required_features
                .iter()
                .filter(|f| !c.game.0.product.has(f))
                .filter_map(|f| c.content.0.task(f).map(|t| t.name.clone()))
                .collect();
            if !missing.is_empty() {
                ui.add(egui::Label::new(RichText::new(format!("Fonctionnalités attendues : {}", missing.join(", "))).size(12.0).color(ORANGE)).wrap());
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new("Prix/mois").size(12.5));
                let mut price = s.price as f32;
                let r = ui.add(
                    egui::DragValue::new(&mut price)
                        .range(def.min_price as f32..=def.max_price as f32)
                        .speed(def.reference_price as f32 / 100.0)
                        .custom_formatter(|v, _| dialogue::money(v)),
                );
                r.clone().on_hover_text(format!(
                    "Référence marché : {}. Plus cher = moins de conversions et plus de churn ; une hausse de plus de 10 % fait partir des clients existants pendant un mois.",
                    dialogue::money(def.reference_price)
                ));
                if r.drag_stopped() || r.lost_focus() {
                    c.run(Command::SetPrice { segment: seg, price: price as f64 });
                }
            });
        });
    }

    if c.game.0.is_unlocked("international") {
        subheading(ui, "Marchés européens");
        for mk in c.content.0.markets.clone() {
            let open = c.game.0.market.markets.contains(&mk.id);
            card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{} {}", mk.flag, mk.name)).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if open {
                            ui.label(RichText::new("✔ ouvert").color(GREEN));
                        } else {
                            let ok = c.game.0.product.languages >= mk.languages_required;
                            if ui.add_enabled(ok, egui::Button::new(format!("Ouvrir ({})", money(mk.cost)))).clicked() {
                                c.run(Command::OpenMarket { market: mk.id.clone() });
                            }
                        }
                    });
                });
                muted(ui, format!("{} · leads ×{} · {} langue(s) requise(s)", mk.description, dialogue::decimal(mk.lead_mult, 2), mk.languages_required));
            });
        }
    }
}
