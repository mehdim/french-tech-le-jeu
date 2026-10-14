//! Scènes d'histoire et décisions : intros de niveau, événements, mission accomplie,
//! faillite et fins.

use super::style::*;
use super::widgets::*;
use super::Ctx;
use crate::states::AppState;
use bevy_egui::egui::{self, RichText};
use game_content::characters::portrait_for;
use game_content::dialogue;
use game_core::Command;
use game_core::content::Line;

fn modal(ctx: &egui::Context, id: &str, width: f32, add: impl FnOnce(&mut egui::Ui)) {
    dim(ctx, 150);
    egui::Area::new(egui::Id::new(id))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            modal_frame().show(ui, |ui| {
                let cr = ctx.content_rect();
                ui.set_width(width.min(cr.width() - 60.0).max(240.0));
                // Une Area se met en page dans la taille de l'image précédente, et une ScrollArea
                // ne dépasse jamais l'espace qu'on lui donne : on relève le plafond
                // explicitement, sinon le dialogue reste bloqué à sa première hauteur.
                let limit = (cr.height() - 100.0).max(200.0);
                ui.set_max_height(limit);
                egui::ScrollArea::vertical().max_height(limit).show(ui, add);
            });
        });
}

fn speaker_line(ui: &mut egui::Ui, c: &Ctx, line: &Line) {
    ui.horizontal_top(|ui| {
        portrait(ui, c.tex, c.atlas, portrait_for(&line.speaker), 72.0);
        ui.vertical(|ui| {
            ui.label(RichText::new(&line.speaker).font(pixel(11.0)).color(GREEN));
            ui.add(egui::Label::new(RichText::new(&line.text).size(15.5).color(OFFWHITE)).wrap());
        });
    });
}

pub fn in_game(ctx: &egui::Context, c: &mut Ctx) {
    if c.ui.show_help {
        help(ctx, c);
    }
    if c.game.0.campaign.intro_pending {
        intro(ctx, c);
    } else if c.game.0.events.pending.is_some() {
        event(ctx, c);
    }
}

fn intro(ctx: &egui::Context, c: &mut Ctx) {
    let level = c.game.0.campaign.level;
    let Some(def) = c.content.0.level(level).cloned() else { return };
    let n = def.intro.len();
    let idx = c.ui.intro_line.min(n);
    // Entrée fait avancer l'histoire (réplique suivante, puis « C'est parti ! »).
    let enter = ctx.input(|i| i.key_pressed(egui::Key::Enter));
    modal(ctx, "intro", 680.0, |ui| {
        ui.label(RichText::new(format!("{} — NIVEAU {}/12", def.subtitle.to_uppercase(), level)).font(pixel(11.0)).color(MUTED));
        ui.label(RichText::new(&def.title).font(pixel(33.0)).color(GREEN));
        ui.add_space(8.0);
        if idx < n {
            for line in &def.intro[..=idx] {
                speaker_line(ui, c, line);
                ui.add_space(6.0);
            }
            ui.horizontal(|ui| {
                if big_button(ui, "Suivant ▸", true, 180.0).on_hover_text("Entrée").clicked() || enter {
                    c.ui.intro_line += 1;
                    *c.click += 1;
                }
                if ui.button("Passer").clicked() {
                    c.ui.intro_line = n;
                }
            });
        } else {
            ui.label(RichText::new(&def.objective).size(15.5).color(OFFWHITE));
            ui.add_space(6.0);
            subheading(ui, "Objectifs");
            for q in &def.quests {
                ui.label(RichText::new(format!("○ {}", q.label)).color(LIGHTGRAY));
            }
            if !def.unlock_text.is_empty() {
                subheading(ui, "Débloqué");
                for u in &def.unlock_text {
                    ui.label(RichText::new(format!("🔓 {u}")).color(GREEN));
                }
            }
            muted(ui, format!("Risque : {}", def.failure_risk));
            ui.add_space(10.0);
            if big_button(ui, "C'est parti !", true, 240.0).on_hover_text("Entrée").clicked() || enter {
                c.run(Command::DismissIntro);
                c.ui.intro_line = 0;
                if c.clock.speed == 0 {
                    c.clock.set_speed(1);
                }
            }
        }
    });
}

fn event(ctx: &egui::Context, c: &mut Ctx) {
    let Some(def) = c.game.0.events.pending_def(&c.content.0).cloned() else { return };
    let boss = c.game.0.events.pending.as_ref().is_some_and(|p| p.boss);
    // Les touches 1-4 choisissent une option.
    let keys = [egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4];
    let key_choice = ctx.input(|i| keys.iter().position(|k| i.key_pressed(*k)));
    modal(ctx, "event", 640.0, |ui| {
        ui.label(RichText::new(if boss { "ÉVÉNEMENT MAJEUR" } else { "ÉVÉNEMENT" }).font(pixel(11.0)).color(if boss { ORANGE } else { MUTED }));
        ui.horizontal_top(|ui| {
            if !def.speaker.is_empty() {
                portrait(ui, c.tex, c.atlas, portrait_for(&def.speaker), 72.0);
            }
            ui.vertical(|ui| {
                ui.label(RichText::new(&def.title).font(pixel(22.0)).color(GREEN));
                if !def.speaker.is_empty() {
                    ui.label(RichText::new(&def.speaker).size(12.5).color(MUTED));
                }
            });
        });
        ui.add_space(6.0);
        ui.add(egui::Label::new(RichText::new(&def.description).size(15.0)).wrap());
        if !def.immediate.is_empty() {
            ui.add_space(4.0);
            effects_list(ui, &def.immediate);
        }
        ui.add_space(8.0);
        for (i, ch) in def.choices.iter().enumerate() {
            let allowed = c.game.0.choice_allowed(&c.content.0, i);
            let frame = if allowed { card() } else { card().fill(GRAPHITE) };
            let r = frame
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        if i < keys.len() {
                            ui.label(RichText::new(format!("{}", i + 1)).font(pixel(13.0)).color(if allowed { GREEN } else { MUTED }));
                        }
                        ui.label(RichText::new(&ch.label).strong().size(15.5).color(if allowed { OFFWHITE } else { MUTED }));
                    });
                    if !ch.description.is_empty() {
                        ui.add(egui::Label::new(RichText::new(&ch.description).size(12.5).color(LIGHTGRAY)).wrap());
                    }
                    ui.horizontal_wrapped(|ui| {
                        for (t, good) in effects_inline(&ch.effects) {
                            ui.label(RichText::new(t).size(12.0).color(if good { GREEN } else { RED }));
                        }
                        for d in &ch.delayed {
                            for (t, good) in effects_inline(&d.effects) {
                                let chance = if d.chance < 1.0 { format!("{:.0} % de risque : ", d.chance * 100.0) } else { String::new() };
                                ui.label(RichText::new(format!("⏳ {chance}{t} (dans {} j)", d.days)).size(12.0).color(if good { GREEN } else { ORANGE }));
                            }
                        }
                    });
                    if !allowed {
                        let reqs: Vec<String> = ch.requires.iter().map(|r| cond_text(c, r)).collect();
                        ui.label(RichText::new(format!("Nécessite : {}", reqs.join(", "))).size(12.0).color(ORANGE));
                    }
                })
                .response
                .interact(egui::Sense::click());
            let r = if allowed { r.on_hover_cursor(egui::CursorIcon::PointingHand) } else { r };
            if allowed && (r.clicked() || key_choice == Some(i)) {
                c.run(Command::ResolveEvent { choice: i });
                if !ch.outcome.is_empty() {
                    c.toasts.push_good(ch.outcome.clone());
                }
            }
            ui.add_space(4.0);
        }
    });
}

pub fn mission_complete(ctx: &egui::Context, c: &mut Ctx) {
    let level = c.game.0.campaign.level;
    let Some(def) = c.content.0.level(level).cloned() else { return };
    let next = c.content.0.level(level + 1).cloned();
    modal(ctx, "mission_complete", 660.0, |ui| {
        ui.label(RichText::new("MISSION ACCOMPLIE").font(pixel(11.0)).color(GREEN));
        ui.label(RichText::new(&def.title).font(pixel(33.0)).color(GREEN));
        ui.add_space(6.0);
        for line in &def.outro {
            speaker_line(ui, c, line);
            ui.add_space(4.0);
        }
        if let Some(r) = c.game.0.campaign.results.last().cloned() {
            card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.columns(3, |cols| {
                    cols[0].label(RichText::new(format!("{} jours", r.days)).strong());
                    cols[1].label(RichText::new(format!("MRR {}", money(r.mrr_end))).strong().color(GREEN));
                    cols[2].label(RichText::new(format!("Trésorerie {}", money(r.cash_end))).strong());
                });
            });
        }
        if let Some(n) = next {
            ui.add_space(8.0);
            ui.label(RichText::new(format!("Prochain niveau : {}", n.title)).font(pixel(22.0)).color(OFFWHITE));
            muted(ui, n.objective.clone());
        }
        ui.add_space(10.0);
        if big_button(ui, "Niveau suivant ▸", true, 260.0).clicked()
            && c.run(Command::AdvanceLevel) {
                c.ui.intro_line = 0;
                c.clock.set_speed(1);
                c.next_state.set(AppState::Playing);
            }
    });
}

pub fn game_over(ctx: &egui::Context, c: &mut Ctx) {
    let g = &c.game.0;
    let day = g.day;
    let hired = g.stats.hired;
    let pages = (g.stats.paperwork * 0.8).round();
    let level = g.campaign.level;
    modal(ctx, "game_over", 600.0, |ui| {
        ui.label(RichText::new("REDRESSEMENT JUDICIAIRE").font(pixel(33.0)).color(RED));
        ui.add_space(6.0);
        ui.add(egui::Label::new(RichText::new(
            "Le tribunal de commerce prononce la liquidation de VERIDIX. Votre dernier post LinkedIn (« Ce n'est qu'un au revoir 🙏 Tout ce qu'on a appris ») fait 12 000 likes, dont 40 de recruteurs.",
        ).size(15.0)).wrap());
        ui.add_space(6.0);
        muted(ui, format!("Niveau {level} · {day} jours · {hired} recrutements · {pages} pages de dossiers administratifs remplies."));
        muted(ui, "Conseil : surveillez le runway, recrutez au rythme du revenu, et n'oubliez pas qu'une aide accordée n'est pas une aide versée.");
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let has_auto = c.slots.0.iter().any(|(s, m)| s == "autosave" && m.is_ok());
            if big_button(ui, "Charger la sauvegarde auto", has_auto, 300.0).clicked() {
                match crate::saves::load_from(c.store, "autosave", c.content) {
                    Ok(s) => {
                        c.game.0 = s;
                        c.clock.set_speed(0);
                        c.next_state.set(AppState::Playing);
                    }
                    Err(e) => c.toasts.push_bad(e),
                }
            }
            if big_button(ui, "Menu principal", true, 220.0).clicked() {
                super::menu::to_main_menu(c);
            }
        });
    });
}

pub fn victory(ctx: &egui::Context, c: &mut Ctx) {
    let ending_id = c.game.0.campaign.ending.clone().unwrap_or_else(|| "acquisition".into());
    let Some(ending) = c.content.0.ending(&ending_id).cloned() else { return };
    let g = &c.game.0;
    let stats = format!(
        "{} jours · MRR final {} · valorisation {} · votre part {} · {} recrutements · {} incidents · {} cafés · {} pages de paperasse",
        g.day,
        money(g.metrics.mrr),
        money(g.metrics.valuation),
        dialogue::pct(g.funding.cap.founder_pct()),
        g.stats.hired,
        g.stats.incidents,
        g.stats.coffees * 37,
        (g.stats.paperwork * 0.8).round()
    );
    modal(ctx, "victory", 720.0, |ui| {
        ui.label(RichText::new("FIN DE LA CAMPAGNE").font(pixel(11.0)).color(MUTED));
        ui.label(RichText::new(&ending.title).font(pixel(33.0)).color(GREEN));
        if let Some(id) = c.tex.endings.get(&ending.art) {
            let w = ui.available_width().min(640.0);
            ui.add(egui::Image::from_texture(egui::load::SizedTexture::new(*id, egui::vec2(w, w * 90.0 / 160.0))).texture_options(egui::TextureOptions::NEAREST));
        }
        ui.add_space(6.0);
        ui.add(egui::Label::new(RichText::new(&ending.headline).italics().size(15.0).color(OFFWHITE)).wrap());
        ui.add_space(6.0);
        for line in &ending.epilogue {
            speaker_line(ui, c, line);
            ui.add_space(4.0);
        }
        muted(ui, stats);
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if big_button(ui, "Crédits", true, 160.0).clicked() {
                c.ui.menu_page = super::MenuPage::Credits;
                super::menu::to_main_menu(c);
            }
            if big_button(ui, "Menu principal", true, 220.0).clicked() {
                super::menu::to_main_menu(c);
            }
        });
    });
}

fn help(ctx: &egui::Context, c: &mut Ctx) {
    let mut open = true;
    egui::Window::new(RichText::new("Aide").font(pixel(22.0)))
        .open(&mut open)
        .default_width(460.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(RichText::new("Boucle de jeu : CONSTRUIRE → LIVRER → ACQUÉRIR → RETENIR → MONÉTISER → SURVIVRE → GRANDIR").strong().color(GREEN));
            ui.add_space(4.0);
            for (k, v) in [
                ("Espace", "Pause / reprise"),
                ("1 · 2 · 3", "Vitesse 1× · 2× · 4×"),
                ("E P V F C B", "Équipe · Produit · Ventes · Financement · Finances · Bureau"),
                ("Clic sur quelqu'un", "Fiche, affectation, salaire"),
                ("Clic sur un objet", "Inspecter / déplacer (tableau blanc → R&D)"),
                ("Glisser / ZQSD / flèches", "Déplacer la caméra"),
                ("Molette, + / -", "Zoom"),
                ("R (placement)", "Pivoter un meuble"),
                ("Échap", "Menu pause"),
            ] {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(k).strong().color(OFFWHITE));
                    ui.label(RichText::new(v).color(LIGHTGRAY));
                });
            }
            ui.add_space(6.0);
            muted(ui, "Chaque personne fait une seule chose par jour : chantier, ventes, marketing, ops, administratif, levée ou management. L'attention du fondateur est votre ressource la plus rare.");
        });
    if !open {
        c.ui.show_help = false;
    }
}
