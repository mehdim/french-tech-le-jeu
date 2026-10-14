//! Funding tab: France Travail, Bpifrance & co, love money, angels and VCs.

use super::style::*;
use super::widgets::*;
use super::Ctx;
use bevy_egui::egui::{self, RichText};
use game_content::characters::portrait_for;
use game_content::dialogue;
use game_core::Command;
use game_core::content::{ProgramKind, RoundKind};
use game_core::funding::{AppStatus, RaiseStatus};

pub fn show(ui: &mut egui::Ui, c: &mut Ctx) {
    heading(ui, "Financement");
    muted(ui, "Les dispositifs sont des versions satiriques et simplifiées : ceci est un jeu, pas un conseil juridique ou financier.");
    applications(ui, c);
    programs(ui, c);
    equity(ui, c);
    money_flows(ui, c);
}

fn applications(ui: &mut egui::Ui, c: &mut Ctx) {
    let apps: Vec<_> = c.game.0.funding.applications.iter().filter(|a| matches!(a.status, AppStatus::Drafting | AppStatus::Submitted)).cloned().collect();
    if apps.is_empty() {
        return;
    }
    subheading(ui, "Dossiers en cours");
    let admin_pts = c.game.0.last_work.admin;
    for a in apps {
        let def = c.content.0.program(&a.program).cloned();
        let name = def.as_ref().map(|d| d.name.clone()).unwrap_or_default();
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(&name).strong());
            match a.status {
                AppStatus::Drafting => {
                    bar(ui, a.progress() as f32, ORANGE, format!("Paperasse {:.0}/{:.0} pts", a.admin_done, a.admin_required), ui.available_width());
                    if a.revisions > 0 {
                        muted(ui, format!("Renvoyé {} fois pour pièces complémentaires.", a.revisions));
                    }
                    if admin_pts <= 0.0 {
                        ui.label(RichText::new("Personne n'est affecté à « Administratif ».").size(12.5).color(ORANGE));
                    } else {
                        muted(ui, format!("≈ {:.0} jours de paperasse restants", ((a.admin_required - a.admin_done) / admin_pts).ceil()));
                    }
                }
                AppStatus::Submitted => {
                    let d = a.decision_day.unwrap_or(0);
                    muted(ui, format!("Déposé. Décision du comité vers le {}.", super::hud::date_label(d)));
                }
                _ => {}
            }
        });
    }
}

fn programs(ui: &mut egui::Ui, c: &mut Ctx) {
    subheading(ui, "Aides & dispositifs");
    let programs = c.content.0.programs.clone();
    let mut shown = 0;
    for p in programs {
        if p.requires_unlock.as_ref().is_some_and(|u| !c.game.0.is_unlocked(u)) {
            continue;
        }
        shown += 1;
        let approved = c.game.0.funding.approved.contains(&p.id);
        let active = c.game.0.funding.active_applications().any(|a| a.program == p.id);
        card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new(&p.name).strong());
                    ui.label(RichText::new(&p.institution).size(11.5).color(MUTED));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if active {
                        ui.label(RichText::new("en cours").color(BLUE));
                    } else if approved && p.cooldown_days == 0 {
                        ui.label(RichText::new("✔ obtenu").color(GREEN));
                    } else {
                        let ok = p.requires.iter().all(|r| c.game.0.check(r));
                        let cooldown_left = c.game.0.funding.last_applied.get(&p.id).map(|d| (d + p.cooldown_days).saturating_sub(c.game.0.day)).unwrap_or(0);
                        let label = if cooldown_left > 0 { format!("dans {cooldown_left} j") } else { "Constituer le dossier".into() };
                        if ui.add_enabled(ok && cooldown_left == 0, egui::Button::new(RichText::new(label).color(GREEN))).clicked() {
                            c.run(Command::StartApplication { program: p.id.clone() });
                        }
                    }
                });
            });
            ui.add(egui::Label::new(RichText::new(&p.description).size(12.0).color(LIGHTGRAY)).wrap());
            let amount = match p.kind {
                ProgramKind::TaxCredit => format!("30 % des dépenses R&D déclarables (≈ {})", money((c.game.0.stats.rd_spend * 0.3 / 1000.0).round() * 1000.0)),
                ProgramKind::Status => "statut (charges réduites)".into(),
                ProgramKind::FounderSupport => "accompagnement".into(),
                _ => money(p.amount),
            };
            muted(ui, format!("Montant : {amount} · paperasse : {:.0} pts · instruction ≈ {} j · succès de base {:.0} %", p.admin_points, p.wait_days, p.base_success * 100.0));
            if !p.requires.is_empty() {
                requirements(ui, c, &p.requires);
            }
        });
    }
    if shown == 0 {
        muted(ui, "Aucun dispositif accessible pour l'instant.");
    }
}

fn equity(ui: &mut egui::Ui, c: &mut Ctx) {
    subheading(ui, "Capital");
    let cap = c.game.0.funding.cap.clone();
    let founder = cap.founder_pct();
    kv(ui, "Votre part", dialogue::pct(founder), if founder > 60.0 { GREEN } else { ORANGE }, "Part du capital détenue par le fondateur.");
    kv(ui, "Valorisation estimée", money(c.game.0.metrics.valuation), OFFWHITE,
        "max(ARR × multiple, plancher équipe/tech) × conditions de marché.\nmultiple = base × croissance × marge × rétention × crédibilité.");
    if c.game.0.is_unlocked("cap_table") || !cap.holders.is_empty() {
        let total = cap.total();
        for (name, shares) in &cap.holders {
            kv(ui, name, dialogue::pct(100.0 * shares / total), LIGHTGRAY, "");
        }
    }
    if !c.game.0.funding.friends_family_done {
        ui.horizontal(|ui| {
            if ui
                .button("Love money : 15 000 € (≈ 7,5 %)")
                .on_hover_text("Tante Martine et les cousins investissent. Ils poseront des questions à chaque repas de famille.")
                .clicked()
            {
                c.run(Command::TakeFriendsFamily);
            }
        });
    }
    // Fundraising.
    let fr = c.game.0.funding.fundraising.clone();
    match fr {
        None => {
            let rounds: Vec<_> = c
                .content
                .0
                .rounds
                .iter()
                .filter(|r| r.kind != RoundKind::FriendsFamily && r.requires_unlock.as_ref().is_none_or(|u| c.game.0.is_unlocked(u)))
                .cloned()
                .collect();
            if rounds.is_empty() {
                muted(ui, "🔒 Les levées de fonds se débloquent au niveau 4.");
            }
            for r in rounds {
                card().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&r.name).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(RichText::new("Lancer la levée").color(GREEN)).clicked() {
                                c.run(Command::StartFundraising { round: r.kind });
                            }
                        });
                    });
                    muted(ui, format!("{} Pitch : {:.0} pts. Dilution visée {:.0}–{:.0} %.", r.description, r.pitch_points, r.dilution.0, r.dilution.1));
                });
            }
        }
        Some(fr) => {
            let rname = c.content.0.round(fr.round).map(|r| r.name.clone()).unwrap_or_default();
            card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(format!("Levée en cours : {rname}")).strong().color(GREEN));
                match fr.status {
                    RaiseStatus::Pitching => {
                        bar(ui, (fr.pitch_done / fr.pitch_required) as f32, BLUE, format!("Deck & rendez-vous {:.0}/{:.0}", fr.pitch_done, fr.pitch_required), ui.available_width());
                        if c.game.0.last_work.pitch <= 0.0 {
                            ui.label(RichText::new("Affectez le fondateur (ou le CFO) à « Levée de fonds ».").size(12.5).color(ORANGE));
                        }
                    }
                    RaiseStatus::Offers => {
                        ui.label("Offres reçues :");
                    }
                }
                if ui.small_button("Abandonner la levée").clicked() {
                    c.run(Command::DeclineOffers);
                }
            });
            if fr.status == RaiseStatus::Offers {
                for (i, o) in fr.offers.iter().enumerate() {
                    let Some(inv) = c.content.0.investor(&o.investor).cloned() else { continue };
                    card().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            portrait(ui, c.tex, c.atlas, portrait_for(&inv.name), 44.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(&inv.name).strong());
                                ui.label(RichText::new(&inv.firm).size(11.5).color(MUTED));
                                ui.label(RichText::new(format!("{} pour {} (pré-money {})", money(o.amount), dialogue::pct(o.dilution), money(o.pre_money))).color(GREEN));
                            });
                        });
                        ui.label(RichText::new(&inv.pitch_line).italics().size(12.5).color(LIGHTGRAY));
                        muted(ui, format!("Métrique préférée : {}", inv.favourite_metric));
                        if !inv.demand.is_empty() {
                            ui.label(RichText::new(format!("Exigence : {}", inv.demand)).size(12.0).color(ORANGE));
                        }
                        effects_list(ui, &inv.on_accept);
                        ui.horizontal(|ui| {
                            if ui.button(RichText::new("Signer la term sheet").color(GREEN)).clicked() {
                                c.run(Command::AcceptOffer { index: i });
                            }
                            muted(ui, format!("expire le {}", super::hud::date_label(o.expires_day)));
                        });
                    });
                }
            }
        }
    }
}

fn money_flows(ui: &mut egui::Ui, c: &mut Ctx) {
    let f = &c.game.0.funding;
    if f.disbursements.is_empty() && f.loans.is_empty() && c.game.0.finance.incomes.is_empty() {
        return;
    }
    subheading(ui, "Versements attendus & prêts");
    for d in f.disbursements.clone() {
        muted(ui, format!("{} — {} le {}", d.label, money(d.amount), super::hud::date_label(d.due_day)));
    }
    for l in f.loans.clone() {
        let grace = if l.grace_left > 0 { format!(" (différé encore {} mois)", l.grace_left) } else { String::new() };
        muted(ui, format!("Prêt {} : {}/mois, {} mensualités restantes{grace}", l.label, money(l.monthly), l.months_left));
    }
    for i in c.game.0.finance.incomes.clone() {
        muted(ui, format!("{} : {}/mois, encore {} mois", i.label, money(i.monthly), i.months_left));
    }
}
