//! Bottom panel: activity feed (newest first) with filters.

use super::Ctx;
use super::hud::date_label;
use super::style::*;
use bevy_egui::egui::{self, RichText};
use game_content::dialogue;
use game_core::messages::Tone;

pub fn bottom(root: &mut egui::Ui, c: &mut Ctx) {
    egui::Panel::bottom("feed")
        .resizable(true)
        .default_size(128.0)
        .min_size(70.0)
        .max_size(320.0)
        .frame(egui::Frame::new().fill(NEAR_BLACK).inner_margin(egui::Margin::symmetric(10, 6)))
        .show(root, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("FIL D'ACTUALITÉ").font(pixel(11.0)).color(MUTED));
                for (i, label) in ["Tout", "Argent", "Équipe & produit", "Histoire"].iter().enumerate() {
                    if ui.selectable_label(c.ui.feed_filter == i as u8, RichText::new(*label).size(12.0)).clicked() {
                        c.ui.feed_filter = i as u8;
                    }
                }
            });
            egui::ScrollArea::vertical().auto_shrink([false, false]).stick_to_bottom(false).show(ui, |ui| {
                let filter = c.ui.feed_filter;
                for e in c.game.0.log.iter().rev().take(120) {
                    let keep = match filter {
                        1 => e.tone == Tone::Money,
                        2 => matches!(e.tone, Tone::Good | Tone::Bad | Tone::Info),
                        3 => e.tone == Tone::Story,
                        _ => true,
                    };
                    if !keep {
                        continue;
                    }
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(date_label(e.day)).size(11.5).color(MUTED));
                        ui.label(RichText::new(dialogue::format(&e.msg)).size(13.0).color(tone_color(e.tone)));
                    });
                }
            });
        });
}
