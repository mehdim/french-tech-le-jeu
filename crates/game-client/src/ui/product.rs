//! Product & R&D tab: detection metrics, task queue, product/research tree.

use super::style::*;
use super::widgets::*;
use super::Ctx;
use bevy_egui::egui::{self, RichText};
use game_content::dialogue;
use game_core::Command;
use game_core::content::TaskCategory;
use game_core::employees::Assignment;
use game_core::research::{Approach, TaskBlock, task_availability};

const CATEGORIES: [(TaskCategory, &str); 6] = [
    (TaskCategory::Mvp, "MVP"),
    (TaskCategory::Engineering, "Ingénierie"),
    (TaskCategory::AiResearch, "Recherche IA"),
    (TaskCategory::Enterprise, "Entreprise"),
    (TaskCategory::AiNative, "Pivot IA native"),
    (TaskCategory::Maintenance, "Maintenance"),
];

pub fn show(ui: &mut egui::Ui, c: &mut Ctx) {
    heading(ui, "Produit & R&D");
    metrics(ui, c);
    queue(ui, c);
    tree(ui, c);
}

fn metrics(ui: &mut egui::Ui, c: &mut Ctx) {
    let g = &c.game.0;
    let p = &g.product;
    let m = &g.metrics;
    card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            icon(ui, c.tex, c.atlas, "quality", 16.0);
            ui.label(RichText::new("Qualité produit").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("{:.0}/100", m.quality)).font(bold(16.0)).color(good_color(m.quality)));
            });
        })
        .response
        .on_hover_text("Qualité = 35 % fonctionnalités + 45 % détection + 20 % fiabilité.\nDétection = (précision − 50) × 3 − faux positifs × 1,2.\nFiabilité = 100 − dette × 0,6 − incidents récents.");
        bar(ui, (p.feature_score / 100.0) as f32, DEEP_GREEN, format!("Fonctionnalités {:.0}", p.feature_score), ui.available_width());
        bar(ui, (p.detection_score() / 100.0) as f32, DEEP_GREEN, format!("Détection {:.0}", p.detection_score()), ui.available_width());
        bar(ui, (p.reliability() / 100.0) as f32, DEEP_GREEN, format!("Fiabilité {:.0}", p.reliability()), ui.available_width());
    });
    ui.columns(2, |cols| {
        let ui = &mut cols[0];
        kv(ui, "Précision", dialogue::pct(m.accuracy), good_color((m.accuracy - 50.0) * 2.5),
            "Précision mesurée sur le benchmark actuel = précision du modèle − dérive.\nLes gains diminuent près du plafond théorique (97 %) : la détection universelle n'existe pas.");
        kv(ui, "Faux positifs", dialogue::pct(m.fpr), bad_color(m.fpr * 8.0),
            "Textes humains accusés à tort d'être générés. Le pire ennemi de la confiance client.");
        kv(ui, "Dérive", format!("-{:.1} pt", p.drift), if p.drift > 5.0 { ORANGE } else { LIGHTGRAY },
            "Chaque nouveau modèle de fondation dégrade le benchmark. Le réentraînement en efface 70 %.");
        kv(ui, "Robustesse", format!("{:.0} %", p.robustness * 100.0), LIGHTGRAY, "Réduit la vitesse de la dérive.");
        let ui = &mut cols[1];
        kv(ui, "Dette technique", format!("{:.0}/100", m.tech_debt), bad_color(m.tech_debt * 1.5),
            "Ralentit l'ingénierie (-0,6 % par point), augmente les incidents, use le moral au-delà de 50.");
        kv(ui, "Coût / 1000 analyses", money(m.inference_cost_per_k), LIGHTGRAY,
            "Coût d'inférence (GPU, API). Multiplié par l'usage des clients, il fait les COGS.");
        kv(ui, "Usage client", format!("×{:.2}", p.usage_mult), LIGHTGRAY, "Les clients utilisent Veridix un peu plus chaque mois (+2 %).");
        kv(ui, "Langues", format!("{}", p.languages), LIGHTGRAY, "Niveau de couverture linguistique (1 = français + anglais au départ).");
    });
    if !g.is_unlocked("product_dev") {
        muted(ui, "🔒 Développement produit verrouillé.");
    }
}

fn queue(ui: &mut egui::Ui, c: &mut Ctx) {
    subheading(ui, "Chantiers en cours");
    let tasks = c.game.0.tasks.clone();
    if tasks.is_empty() {
        muted(ui, "Aucun chantier. Choisissez dans l'arbre ci-dessous.");
    }
    for t in tasks {
        let Some(def) = c.content.0.task(&t.task_id).cloned() else { continue };
        let workers: Vec<(u32, String)> = c
            .game
            .0
            .employees
            .iter()
            .filter(|e| e.assignment == Assignment::Task(t.instance))
            .map(|e| (e.id, e.name.clone()))
            .collect();
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(&def.name).strong());
                ui.label(RichText::new(t.approach.label()).size(11.5).color(MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").on_hover_text("Annuler le chantier (remboursé à 50 % s'il n'a pas commencé)").clicked() {
                        c.run(Command::CancelTask { instance: t.instance });
                    }
                });
            });
            let w = ui.available_width();
            if t.eng_total > 0.0 {
                bar(ui, (t.eng_done / t.eng_total) as f32, BLUE, format!("Ingénierie {:.0}/{:.0}", t.eng_done, t.eng_total), w);
            }
            if t.research_total > 0.0 {
                bar(ui, (t.research_done / t.research_total) as f32, PURPLE_UI, format!("Recherche {:.0}/{:.0}", t.research_done, t.research_total), w);
            }
            ui.horizontal_wrapped(|ui| {
                if workers.is_empty() {
                    ui.label(RichText::new("Personne dessus !").color(ORANGE).size(12.5));
                } else {
                    ui.label(RichText::new(format!("👷 {}", workers.iter().map(|w| w.1.as_str()).collect::<Vec<_>>().join(", "))).size(12.5));
                }
            });
            ui.horizontal(|ui| {
                let idle: Vec<u32> = c
                    .game
                    .0
                    .employees
                    .iter()
                    .filter(|e| e.assignment == Assignment::Unassigned && e.role.is_engineering())
                    .map(|e| e.id)
                    .collect();
                if ui
                    .add_enabled(!idle.is_empty(), egui::Button::new(RichText::new("+ Affecter les ingénieurs libres").size(12.5)))
                    .clicked()
                {
                    for id in idle {
                        c.run(Command::Assign { employee: id, assignment: Assignment::Task(t.instance) });
                    }
                }
                if !c.game.0.employees.iter().any(|e| e.id == 1 && e.assignment == Assignment::Task(t.instance))
                    && ui.button(RichText::new("+ M'y mettre").size(12.5)).on_hover_text("Affecter le fondateur (vous)").clicked()
                {
                    c.run(Command::Assign { employee: 1, assignment: Assignment::Task(t.instance) });
                }
            });
        });
    }
}

const PURPLE_UI: egui::Color32 = egui::Color32::from_rgb(150, 110, 210);

fn tree(ui: &mut egui::Ui, c: &mut Ctx) {
    subheading(ui, "Feuille de route");
    let approach = c.ui.task_approach.unwrap_or(Approach::Standard);
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Approche :").size(12.5));
        for a in Approach::ALL {
            let tip = match a {
                Approach::Quick => "Effort ×0,7 · dette ×2,2 · risque ×1,6",
                Approach::Standard => "Effort ×1 · dette ×1 · risque ×1",
                Approach::Clean => "Effort ×1,3 · dette ×0,3 · risque ×0,5",
            };
            if ui.selectable_label(a == approach, RichText::new(a.label()).size(12.5)).on_hover_text(tip).clicked() {
                c.ui.task_approach = Some(a);
            }
        }
    });
    ui.horizontal_wrapped(|ui| {
        if ui.selectable_label(c.ui.product_filter.is_none(), "Tout").clicked() {
            c.ui.product_filter = None;
        }
        for (cat, label) in CATEGORIES {
            if ui.selectable_label(c.ui.product_filter == Some(cat), label).clicked() {
                c.ui.product_filter = Some(cat);
            }
        }
    });
    for (cat, label) in CATEGORIES {
        if c.ui.product_filter.is_some_and(|f| f != cat) {
            continue;
        }
        let tasks: Vec<_> = c.content.0.tasks.iter().filter(|t| t.category == cat).cloned().collect();
        let open = c.ui.product_filter == Some(cat) || matches!(cat, TaskCategory::Mvp | TaskCategory::AiResearch | TaskCategory::Engineering);
        egui::CollapsingHeader::new(RichText::new(label).font(pixel(11.0)).color(GREEN)).id_salt(("cat", label)).default_open(open).show(ui, |ui| {
            for def in tasks {
                let status = task_availability(&def, &c.game.0.product, &c.game.0.campaign.unlocks, &c.game.0.tasks, c.game.0.finance.cash);
                let shipped = c.game.0.product.has(&def.id);
                if matches!(status, Err(TaskBlock::Locked(_))) && !shipped {
                    continue; // hidden until its level unlocks it
                }
                let frame = if shipped && !def.repeatable { card().fill(GRAPHITE) } else { card() };
                frame.show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let col = if shipped && !def.repeatable { MUTED } else { OFFWHITE };
                        ui.label(RichText::new(&def.name).strong().color(col));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| match &status {
                            Ok(()) => {
                                let label = if def.cost > 0.0 { format!("Lancer ({})", money(def.cost)) } else { "Lancer".into() };
                                if ui.button(RichText::new(label).color(GREEN)).clicked() {
                                    c.run(Command::StartTask { task: def.id.clone(), approach });
                                }
                            }
                            Err(TaskBlock::AlreadyShipped) => {
                                ui.label(RichText::new("✔ livré").color(GREEN));
                            }
                            Err(TaskBlock::AlreadyActive) => {
                                ui.label(RichText::new("en cours").color(BLUE));
                            }
                            Err(TaskBlock::NotEnoughCash) => {
                                ui.label(RichText::new(format!("{} requis", money(def.cost))).color(ORANGE));
                            }
                            Err(TaskBlock::MissingDep(d)) => {
                                let n = c.content.0.task(d).map(|t| t.name.clone()).unwrap_or_default();
                                ui.label(RichText::new("prérequis").color(MUTED)).on_hover_text(format!("Nécessite : {n}"));
                            }
                            Err(TaskBlock::Locked(_)) => {}
                        });
                    });
                    if !(shipped && !def.repeatable) {
                        ui.add(egui::Label::new(RichText::new(&def.description).size(12.0).color(LIGHTGRAY)).wrap());
                        let mut line = Vec::new();
                        if def.eng > 0.0 {
                            line.push(format!("{:.0} ingé", def.eng * approach.effort_mult()));
                        }
                        if def.research > 0.0 {
                            line.push(format!("{:.0} recherche", def.research * approach.effort_mult()));
                        }
                        if def.debt > 0.0 {
                            line.push(format!("dette +{:.0}", def.debt * approach.debt_mult()));
                        }
                        if def.risk > 0.0 {
                            line.push(format!("risque {:.0} %", (def.risk * approach.risk_mult() * 100.0).min(100.0)));
                        }
                        if !def.deps.is_empty() {
                            let names: Vec<String> = def.deps.iter().filter_map(|d| c.content.0.task(d)).map(|t| {
                                if c.game.0.product.has(&t.id) { format!("✓ {}", t.name) } else { format!("✗ {}", t.name) }
                            }).collect();
                            line.push(format!("après : {}", names.join(", ")));
                        }
                        muted(ui, line.join(" · "));
                        if !def.benefit.is_empty() {
                            ui.label(RichText::new(format!("→ {}", def.benefit)).size(12.0).color(GREEN));
                        }
                    }
                });
            }
        });
    }
}
