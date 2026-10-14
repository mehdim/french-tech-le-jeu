//! Onglet Équipe : effectif, affectations, fiche détaillée, recrutement.

use super::style::*;
use super::widgets::*;
use super::Ctx;
use bevy_egui::egui::{self, RichText};
use game_content::characters::{role_name, trait_effect, trait_name};
use game_content::dialogue;
use game_core::Command;
use game_core::employees::{Assignment, allowed_assignments};
use game_core::types::{Role, Seniority};

fn assignment_label(c: &Ctx, a: Assignment) -> String {
    match a {
        Assignment::Task(inst) => {
            let name = c
                .game
                .0
                .tasks
                .iter()
                .find(|t| t.instance == inst)
                .and_then(|t| c.content.0.task(&t.task_id))
                .map(|d| d.name.clone())
                .unwrap_or_else(|| "Chantier".into());
            format!("🛠 {name}")
        }
        other => other.label().to_string(),
    }
}

fn assignment_help(a: Assignment) -> &'static str {
    match a {
        Assignment::Unassigned => "Ne fait rien de productif. Récupère de la fatigue.",
        Assignment::Task(_) => "Produit des points d'ingénierie (compétence Ingé) et de recherche (compétence Recherche) pour ce chantier.",
        Assignment::Sales => "Travaille les leads : chaque point de vente convertit des prospects (les grands comptes demandent beaucoup d'effort).",
        Assignment::Marketing => "Génère des leads organiques (compétence Marketing).",
        Assignment::Ops => "Réduit les incidents et la dette, absorbe le support client. Indispensable quand les clients se comptent en milliers.",
        Assignment::Admin => "Remplit les dossiers (France Travail, Bpifrance, CIR...). Un CFO va deux fois plus vite.",
        Assignment::Fundraising => "Prépare le deck et rencontre les investisseurs pendant une levée.",
        Assignment::Management => "PM : +7 % aux chantiers (1 PM utile pour 5 ingénieurs). RH : moral +4 et recrutement plus rapide. CFO : paperasse et crédibilité des dossiers.",
    }
}

fn assignment_combo(ui: &mut egui::Ui, c: &mut Ctx, id: u32, role: Role, current: Assignment) {
    let mut options: Vec<Assignment> = allowed_assignments(role);
    for t in &c.game.0.tasks {
        options.insert(1, Assignment::Task(t.instance));
    }
    let mut chosen = None;
    egui::ComboBox::from_id_salt(("assign", id))
        .width(190.0)
        .selected_text(RichText::new(assignment_label(c, current)).size(12.5))
        .show_ui(ui, |ui| {
            for a in options {
                let r = ui.selectable_label(a == current, assignment_label(c, a)).on_hover_text(assignment_help(a));
                if r.clicked() {
                    chosen = Some(a);
                }
            }
        });
    if let Some(a) = chosen {
        c.run(Command::Assign { employee: id, assignment: a });
    }
}

pub fn show(ui: &mut egui::Ui, c: &mut Ctx) {
    heading(ui, "Équipe");
    let g = &c.game.0;
    let payroll: f64 = g.metrics.run_rate.payroll;
    ui.horizontal_wrapped(|ui| {
        muted(ui, format!("{} personnes · {} places · masse salariale {}/mois (charges incluses)", g.employees.len(), g.metrics.seats, money(payroll)));
    });
    let mut crunch = g.crunch;
    if ui
        .checkbox(&mut crunch, "Mode crunch (travailler le week-end)")
        .on_hover_text("+40 % de jours travaillés, mais fatigue qui grimpe, moral -8 et risque de burn-out.")
        .changed()
    {
        c.run(Command::SetCrunch(crunch));
    }
    ui.add_space(4.0);

    let ids: Vec<u32> = c.game.0.employees.iter().map(|e| e.id).collect();
    for id in ids {
        let Some(e) = c.game.0.employee(id).cloned() else { continue };
        let selected = c.ui.inspect_employee == Some(id);
        let frame = if selected { card().stroke(egui::Stroke::new(1.5, GREEN)) } else { card() };
        let resp = frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    avatar(ui, c.tex, c.atlas, e.role, e.look, 34.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&e.name).strong());
                            ui.label(RichText::new(role_name(e.role)).size(12.0).color(MUTED));
                            if !e.is_available(c.game.0.day) {
                                ui.label(RichText::new("en arrêt").size(12.0).color(RED));
                            }
                        });
                        assignment_combo(ui, c, id, e.role, e.assignment);
                        ui.horizontal(|ui| {
                            bar(ui, (e.morale / 100.0) as f32, good_color(e.morale), format!("moral {:.0}", e.morale), 110.0);
                            bar(ui, (e.fatigue / 100.0) as f32, bad_color(e.fatigue), format!("fatigue {:.0}", e.fatigue), 110.0);
                        });
                    });
                });
            })
            .response;
        if resp.interact(egui::Sense::click()).clicked() {
            c.ui.inspect_employee = Some(id);
            c.selection.employee = Some(id);
        }
        if selected {
            inspector(ui, c, id);
        }
    }
    if c.game.0.is_unlocked("recruitment") {
        recruitment(ui, c);
    } else {
        ui.add_space(8.0);
        muted(ui, "🔒 Le recrutement se débloque au niveau 2 (quand il y aura un bureau pour accueillir quelqu'un).");
    }
}

fn inspector(ui: &mut egui::Ui, c: &mut Ctx, id: u32) {
    let Some(e) = c.game.0.employee(id).cloned() else { return };
    let contrib = c.game.0.contrib_rate(&c.content.0, &e);
    egui::Frame::new().fill(GRAPHITE).inner_margin(egui::Margin::same(8)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        let arch = c.content.0.archetype(&e.archetype);
        ui.label(RichText::new(arch.map(|a| a.name.as_str()).unwrap_or("")).strong().color(GREEN));
        if let Some(a) = arch {
            ui.add(egui::Label::new(RichText::new(&a.description).size(12.5).color(LIGHTGRAY)).wrap());
        }
        ui.label(RichText::new(format!("Trait : {}", trait_name(e.trait_))).size(12.5))
            .on_hover_text(trait_effect(e.trait_));
        ui.label(RichText::new(trait_effect(e.trait_)).size(12.0).color(MUTED));
        let sen = match e.seniority {
            Seniority::Junior => "Junior",
            Seniority::Mid => "Confirmé·e",
            Seniority::Senior => "Senior",
            Seniority::Staff => "Staff",
        };
        kv(ui, "Séniorité", format!("{sen} (×{:.1})", e.seniority.output_mult()), OFFWHITE, "Multiplie toute la production.");
        let cost = e.monthly_cost(contrib, c.game.0.day);
        let salary_txt = if let Some(rate) = e.day_rate {
            format!("TJM {} (≈ {}/mois)", money(rate), money(cost))
        } else if e.salary == 0.0 {
            "0 € (vous vivez de l'allocation)".into()
        } else {
            format!("{} brut → {} chargé", money(e.salary), money(cost))
        };
        kv(ui, "Salaire", salary_txt, OFFWHITE, "Coût employeur = brut × (1 + charges patronales). Approximations de jeu, pas un conseil de paie.");
        kv(ui, "Productivité", format!("×{:.2}", e.productivity * e.morale_factor() * e.fatigue_factor()), OFFWHITE,
            "productivité perso × moral (0,5 + moral/100) × fatigue (pénalité au-delà de 50).");
        kv(ui, "Loyauté", format!("{:.0} %", e.loyalty * 100.0), OFFWHITE, "Plus elle est haute, moins on démissionne quand le moral est bas.");
        kv(ui, "Arrivée", super::hud::date_label(e.hired_day), LIGHTGRAY, "");
        if let Some(end) = e.contract_end {
            kv(ui, "Fin de contrat", super::hud::date_label(end), ORANGE, "Stage ou mission ESN : départ automatique.");
        }
        ui.add_space(4.0);
        skill_row(ui, "Ingénierie", e.skills.eng);
        skill_row(ui, "Recherche", e.skills.research);
        skill_row(ui, "Ventes", e.skills.sales);
        skill_row(ui, "Marketing", e.skills.marketing);
        skill_row(ui, "Ops / admin", e.skills.ops);
        if e.role != Role::Founder {
            ui.add_space(4.0);
            if c.ui.confirm_fire == Some(id) {
                let years = (c.game.0.day - e.hired_day) as f64 / 336.0;
                let sev = if e.day_rate.is_some() { 0.0 } else { e.salary * (1.0 + years * c.content.0.balancing.severance_months_per_year * 4.0) };
                ui.label(RichText::new(format!("Indemnités estimées : {}. Moral de l'équipe -5, réputation -1.", money(sev))).color(ORANGE));
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Confirmer le départ").color(RED)).clicked() {
                        c.run(Command::Fire { employee: id });
                        c.ui.confirm_fire = None;
                        c.ui.inspect_employee = None;
                    }
                    if ui.button("Annuler").clicked() {
                        c.ui.confirm_fire = None;
                    }
                });
            } else if ui.button("Se séparer de cette personne…").clicked() {
                c.ui.confirm_fire = Some(id);
            }
        }
    });
}

fn recruitment(ui: &mut egui::Ui, c: &mut Ctx) {
    heading(ui, "Recrutement");
    let role = c.ui.recruit_role.unwrap_or(Role::Backend);
    ui.horizontal(|ui| {
        ui.label("Profil :");
        egui::ComboBox::from_id_salt("recruit_role").selected_text(role_name(role)).width(170.0).show_ui(ui, |ui| {
            for r in Role::HIRABLE {
                if ui.selectable_label(r == role, role_name(r)).clicked() {
                    c.ui.recruit_role = Some(r);
                    c.ui.recruit_channel = None;
                }
            }
        });
    });
    let channels: Vec<_> = c
        .content
        .0
        .channels
        .iter()
        .filter(|ch| ch.archetypes.iter().filter_map(|a| c.content.0.archetype(a)).any(|a| a.role == role))
        .cloned()
        .collect();
    if channels.is_empty() {
        muted(ui, "Aucun canal ne propose ce profil.");
    }
    for ch in channels {
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(&ch.name).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let pending = c.game.0.searches.iter().any(|s| s.channel == ch.id && s.role == role);
                    let ok = ch.requires.iter().all(|r| c.game.0.check(r));
                    let label = if pending { "En cours…" } else { "Lancer" };
                    if ui.add_enabled(!pending && ok, egui::Button::new(label)).clicked()
                        && c.run(Command::StartRecruitment { channel: ch.id.clone(), role }) {
                            c.toasts.push_good(format!("Recherche lancée via {}", ch.name));
                        }
                });
            });
            ui.add(egui::Label::new(RichText::new(&ch.description).size(12.0).color(LIGHTGRAY)).wrap());
            let mut cost = if ch.cost > 0.0 { money(ch.cost) } else { "gratuit".into() };
            if ch.fee_pct > 0.0 {
                cost = format!("{} % du salaire annuel à l'embauche", (ch.fee_pct * 100.0).round());
            }
            if ch.hire_bonus > 0.0 {
                cost = format!("prime de {} à l'embauche", money(ch.hire_bonus));
            }
            muted(ui, format!("Coût : {cost} · délai {}–{} j · {}–{} profils", ch.delay.0, ch.delay.1, ch.candidates.0, ch.candidates.1));
            if !ch.requires.is_empty() {
                requirements(ui, c, &ch.requires);
            }
        });
    }
    let searches: Vec<_> = c.game.0.searches.clone();
    if !searches.is_empty() {
        subheading(ui, "Recherches en cours");
        for s in searches {
            let name = c.content.0.channel(&s.channel).map(|ch| ch.name.clone()).unwrap_or_default();
            muted(ui, format!("{} via {} — résultats {}", role_name(s.role), name, super::hud::date_label(s.ready_day)));
        }
    }
    let cands: Vec<_> = c.game.0.candidates.clone();
    if !cands.is_empty() {
        subheading(ui, "Candidat·es");
    }
    for cd in cands {
        let e = &cd.employee;
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                avatar(ui, c.tex, c.atlas, e.role, e.look, 34.0);
                ui.vertical(|ui| {
                    let arch = c.content.0.archetype(&e.archetype).map(|a| a.name.clone()).unwrap_or_default();
                    ui.label(RichText::new(format!("{} — {}", e.name, arch)).strong());
                    let pay = if let Some(rate) = e.day_rate { format!("TJM {}", money(rate)) } else { format!("{} brut/mois", money(e.salary)) };
                    muted(ui, format!("{} · {} · expire {}", pay, trait_name(e.trait_), super::hud::date_label(cd.expires_day)));
                    ui.label(RichText::new(format!(
                        "Ingé {:.0} · Rech. {:.0} · Ventes {:.0} · Mkt {:.0} · Ops {:.0}",
                        e.skills.eng, e.skills.research, e.skills.sales, e.skills.marketing, e.skills.ops
                    )).size(12.0));
                    if !cd.pitch.is_empty() {
                        ui.label(RichText::new(&cd.pitch).italics().size(12.0).color(LIGHTGRAY));
                    }
                });
            });
            ui.horizontal(|ui| {
                let label = if cd.hiring_cost > 0.0 { format!("Embaucher ({})", money(cd.hiring_cost)) } else { "Embaucher".into() };
                if ui.button(RichText::new(label).color(GREEN)).on_hover_text(trait_effect(e.trait_)).clicked() {
                    c.run(Command::Hire { candidate: cd.id });
                }
                if ui.button("Décliner").clicked() {
                    c.run(Command::RejectCandidate { candidate: cd.id });
                }
            });
        });
    }
    let _ = dialogue::decimal(0.0, 0);
}
