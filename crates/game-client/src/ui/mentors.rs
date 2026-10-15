//! Onglet Mentors : Mehdi, Steeve et Jean-Baptiste, d'A la French.
//! Choisir un mentor principal (si ce n'est pas encore fait), voir le bonus de
//! chacun, et lui demander conseil quand il est disponible.

use super::style::*;
use super::widgets::*;
use super::Ctx;
use bevy_egui::egui::{self, RichText};
use game_core::Command;

pub fn show(ui: &mut egui::Ui, c: &mut Ctx) {
    heading(ui, "Mentors");
    muted(
        ui,
        "Les systèmes critiques sont triplés pour ne jamais tomber en panne. Votre startup aussi : trois mentors d'A la French. \
         Chacun donne un conseil adapté à votre situation, puis il faut attendre avant de le solliciter à nouveau.",
    );
    let mentors = c.content.0.mentors.clone();
    let main = c.game.0.mentors.main.clone();
    if main.is_none() {
        ui.add_space(4.0);
        ui.label(RichText::new("Choisissez un mentor principal : son bonus est permanent et ses conseils reviennent deux fois plus vite.").color(ORANGE));
    }
    for m in &mentors {
        ui.add_space(6.0);
        let is_main = main.as_deref() == Some(m.id.as_str());
        let frame = if is_main { card().stroke(egui::Stroke::new(1.5, GREEN)) } else { card() };
        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                portrait(ui, c.tex, c.atlas, &m.id, 64.0);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 3.0;
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&m.name).font(pixel(18.0)).color(GREEN));
                        if is_main {
                            ui.label(RichText::new("MENTOR PRINCIPAL").font(pixel(10.0)).color(NEAR_BLACK).background_color(GREEN));
                        }
                    });
                    ui.label(RichText::new(&m.title).size(12.5).color(LIGHTGRAY));
                    ui.horizontal_wrapped(|ui| {
                        for s in &m.skills {
                            ui.label(RichText::new(format!(" {s} ")).size(11.5).color(OFFWHITE).background_color(DEEP_GREEN));
                        }
                    });
                });
            });
            ui.add(egui::Label::new(RichText::new(&m.bio).size(12.5).color(LIGHTGRAY)).wrap());
            let bonus_color = if is_main { GREEN } else { MUTED };
            ui.label(RichText::new(format!("Bonus de mentor principal : {}", m.passive_label)).size(12.5).color(bonus_color));

            if let Some(text) = c.game.0.mentors.last_advice.get(&m.id) {
                ui.add(egui::Label::new(RichText::new(format!("Dernier conseil : « {text} »")).size(12.5).italics().color(OFFWHITE)).wrap());
            }

            ui.horizontal_wrapped(|ui| {
                if main.is_none()
                    && ui
                        .button(RichText::new("Choisir comme mentor principal").color(GREEN))
                        .on_hover_text(&m.passive_label)
                        .clicked()
                {
                    c.run(Command::ChooseMentor { mentor: m.id.clone() });
                    c.toasts.push_good(format!("{} devient votre mentor principal.", m.name));
                }
                let wait = c.game.0.mentor_wait(m);
                let advice = c.game.0.mentor_advice(m).cloned();
                let can = wait == 0 && advice.is_some();
                let label = if wait > 0 { format!("Disponible dans {wait} j") } else { "Demander conseil".to_string() };
                let mut b = ui.add_enabled(can, egui::Button::new(RichText::new(label).color(if can { OFFWHITE } else { MUTED })));
                if let Some(a) = &advice
                    && !a.effects.is_empty()
                {
                    b = b.on_hover_ui(|ui| {
                        ui.label("Ce que ce conseil apporterait maintenant :");
                        effects_list(ui, &a.effects);
                    });
                }
                if b.clicked()
                    && c.run(Command::ConsultMentor { mentor: m.id.clone() })
                    && let Some(text) = c.game.0.mentors.last_advice.get(&m.id)
                {
                    c.toasts.push_good(format!("{} : « {} »", m.name, text));
                }
                let cooldown = c.game.0.mentor_cooldown(m);
                ui.label(RichText::new(format!("tous les {cooldown} j")).size(11.5).color(MUTED));
            });
        });
    }
}
