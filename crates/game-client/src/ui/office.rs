//! Office tab: current office, moving, furniture shop and placement.

use super::style::*;
use super::widgets::*;
use super::Ctx;
use crate::rendering::Placement;
use bevy_egui::egui::{self, RichText};
use game_content::dialogue;
use game_core::Command;
use game_core::content::FurnitureEffects;

fn effects_text(e: &FurnitureEffects) -> Vec<String> {
    let mut v = Vec::new();
    if e.seats > 0 {
        v.push(format!("{} place", e.seats));
    }
    if e.morale != 0.0 {
        v.push(format!("moral {:+}", e.morale));
    }
    if e.fatigue_recovery != 0.0 {
        v.push(format!("fatigue -{}/j", dialogue::decimal(e.fatigue_recovery, 1)));
    }
    if e.eng_bonus != 0.0 {
        v.push(format!("ingé +{:.0} %", e.eng_bonus * 100.0));
    }
    if e.research_bonus != 0.0 {
        v.push(format!("recherche +{:.0} %", e.research_bonus * 100.0));
    }
    if e.sales_bonus != 0.0 {
        v.push(format!("ventes +{:.0} %", e.sales_bonus * 100.0));
    }
    if e.inference_cost != 0.0 {
        v.push(format!("inférence -{:.0} %", e.inference_cost * 100.0));
    }
    if e.incident_risk != 0.0 {
        v.push(format!("incidents {:+.0} %", e.incident_risk * 100.0));
    }
    if e.productivity != 0.0 {
        v.push(format!("productivité {:+.0} %", e.productivity * 100.0));
    }
    v
}

pub fn show(ui: &mut egui::Ui, c: &mut Ctx) {
    heading(ui, "Bureau");
    let tier = c.game.0.office.tier;
    let Some(def) = c.content.0.office(tier).cloned() else { return };
    let fx = c.game.0.office.effects(&c.content.0);
    card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(&def.name).strong().color(GREEN));
        ui.add(egui::Label::new(RichText::new(&def.description).size(12.5).color(LIGHTGRAY)).wrap());
        let rent = def.rent + c.game.0.office.rent_extra;
        kv(ui, "Loyer", format!("{}/mois", money(rent)), OFFWHITE, "");
        kv(ui, "Places assises", format!("{} pour {} personnes", fx.seats, c.game.0.employees.len()), if fx.seats as usize >= c.game.0.employees.len() { GREEN } else { ORANGE }, "Bureaux et postes de travail.");
        kv(ui, "Grille", format!("{} × {}", def.width, def.height), LIGHTGRAY, "");
        let mut parts = vec![format!("moral {:+.1} (+{:.0} lieu)", fx.morale.min(20.0), def.morale_bonus)];
        if fx.fatigue_recovery > 0.0 {
            parts.push(format!("récup. fatigue {}", dialogue::decimal(fx.fatigue_recovery.min(1.2), 1)));
        }
        if fx.eng_bonus > 0.0 {
            parts.push(format!("ingé +{:.0} %", fx.eng_bonus * 100.0));
        }
        if fx.research_bonus > 0.0 {
            parts.push(format!("recherche +{:.0} %", fx.research_bonus * 100.0));
        }
        if fx.inference_cost > 0.0 {
            parts.push(format!("inférence -{:.0} %", fx.inference_cost.min(0.5) * 100.0));
        }
        muted(ui, format!("Effets du mobilier : {}", parts.join(" · ")));
    });

    // Selected furniture.
    if let Some(uid) = c.ui.inspect_furniture
        && let Some(it) = c.game.0.office.items.iter().find(|i| i.uid == uid).cloned()
            && let Some(fdef) = c.content.0.furniture(&it.item).cloned() {
                card().stroke(egui::Stroke::new(1.5, GREEN)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        furniture_icon(ui, c.tex, c.atlas, &fdef.sprite, 40.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&fdef.name).strong());
                            ui.add(egui::Label::new(RichText::new(&fdef.description).size(12.0).color(LIGHTGRAY)).wrap());
                        });
                    });
                    ui.horizontal(|ui| {
                        if c.game.0.is_unlocked("office_custom") && ui.button("Déplacer").clicked() {
                            *c.placement = Placement { item: Some(it.item.clone()), moving_uid: Some(uid), rotated: it.rotated, ..Default::default() };
                        }
                        if ui.button(format!("Revendre ({})", money(fdef.cost * 0.3))).clicked() {
                            c.run(Command::SellFurniture { uid });
                            c.ui.inspect_furniture = None;
                            c.selection.furniture = None;
                        }
                        if ui.button("Fermer").clicked() {
                            c.ui.inspect_furniture = None;
                            c.selection.furniture = None;
                        }
                    });
                });
            }

    // Moving.
    let next: Vec<_> = c.content.0.offices.iter().filter(|o| o.id > tier).cloned().collect();
    if !next.is_empty() {
        subheading(ui, "Déménager");
    }
    for o in next.into_iter().take(2) {
        let unlocked = o.requires_unlock.as_ref().is_none_or(|u| c.game.0.is_unlocked(u));
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(&o.name).strong().color(if unlocked { OFFWHITE } else { MUTED }));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let cost = o.rent * o.deposit_months;
                    if unlocked {
                        if ui.button(RichText::new(format!("Signer ({})", money(cost))).color(GREEN)).on_hover_text("Dépôt de garantie. Le mobilier déménage avec vous.").clicked() {
                            c.run(Command::MoveOffice { tier: o.id });
                        }
                    } else {
                        ui.label(RichText::new("🔒").color(MUTED));
                    }
                });
            });
            muted(ui, format!("{} · {}×{} · loyer {}/mois · moral +{:.0} · réputation +{:.0}", o.description, o.width, o.height, money(o.rent), o.morale_bonus, o.reputation_bonus));
        });
    }

    // Storage.
    let storage = c.game.0.office.storage.clone();
    if !storage.is_empty() {
        subheading(ui, "Mobilier en carton");
        for (i, item) in storage.iter().enumerate() {
            let name = c.content.0.furniture(item).map(|f| f.name.clone()).unwrap_or_default();
            ui.horizontal(|ui| {
                ui.label(&name);
                if ui.small_button("Placer").clicked() {
                    *c.placement = Placement { item: Some(item.clone()), stored_index: Some(i), ..Default::default() };
                }
            });
        }
    }

    // Shop.
    subheading(ui, "Mobilier");
    if !c.game.0.is_unlocked("office_custom") {
        muted(ui, "🔒 L'aménagement se débloque au niveau 2. Pour l'instant, c'est votre salon.");
        return;
    }
    muted(ui, "Choisissez un objet puis cliquez dans le bureau. R pour pivoter, Maj pour en poser plusieurs.");
    for f in c.content.0.furniture.clone() {
        if f.requires_unlock.as_ref().is_some_and(|u| !c.game.0.is_unlocked(u)) {
            continue;
        }
        let owned = c.game.0.office.count(&f.id);
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                furniture_icon(ui, c.tex, c.atlas, &f.sprite, 36.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&f.name).strong());
                        if owned > 0 {
                            ui.label(RichText::new(format!("×{owned}")).size(12.0).color(MUTED));
                        }
                    });
                    let mut line = effects_text(&f.effects).join(" · ");
                    if f.monthly > 0.0 {
                        line.push_str(&format!(" · {}/mois", money(f.monthly)));
                    }
                    muted(ui, line);
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let afford = c.game.0.finance.cash >= f.cost;
                    if ui.add_enabled(afford, egui::Button::new(money(f.cost))).on_hover_text(&f.description).clicked() {
                        *c.placement = Placement { item: Some(f.id.clone()), ..Default::default() };
                        *c.click += 1;
                    }
                });
            });
        });
    }
}
