//! Visual identity: À la French black / green / off-white, pixel headings.

use bevy_egui::egui::{self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke, TextStyle};
use std::sync::Arc;

pub const NEAR_BLACK: Color32 = Color32::from_rgb(16, 20, 18);
pub const GRAPHITE: Color32 = Color32::from_rgb(32, 40, 34);
pub const GRAPHITE_2: Color32 = Color32::from_rgb(44, 54, 47);
pub const GRAPHITE_3: Color32 = Color32::from_rgb(58, 70, 62);
pub const GREEN: Color32 = Color32::from_rgb(124, 255, 82);
pub const DEEP_GREEN: Color32 = Color32::from_rgb(38, 125, 75);
pub const OFFWHITE: Color32 = Color32::from_rgb(244, 241, 232);
pub const LIGHTGRAY: Color32 = Color32::from_rgb(190, 197, 187);
pub const MUTED: Color32 = Color32::from_rgb(140, 150, 142);
pub const ORANGE: Color32 = Color32::from_rgb(246, 162, 60);
pub const RED: Color32 = Color32::from_rgb(228, 91, 91);
pub const BLUE: Color32 = Color32::from_rgb(110, 160, 230);

const PIXEL: &[u8] = include_bytes!("../../../../assets/fonts/VeridixPixel.ttf");
const BODY: &[u8] = include_bytes!("../../../../assets/fonts/DejaVuSans-Subset.ttf");
const BODY_BOLD: &[u8] = include_bytes!("../../../../assets/fonts/DejaVuSans-Bold-Subset.ttf");

pub fn pixel(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("pixel".into()))
}

pub fn bold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert("pixel".into(), Arc::new(FontData::from_static(PIXEL)));
    fonts.font_data.insert("body".into(), Arc::new(FontData::from_static(BODY)));
    fonts.font_data.insert("body_bold".into(), Arc::new(FontData::from_static(BODY_BOLD)));
    // Keep egui's bundled fonts (emoji, symbols) as fallbacks.
    let fallbacks: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut prop = vec!["body".to_string()];
    prop.extend(fallbacks.iter().cloned());
    fonts.families.insert(FontFamily::Proportional, prop);
    let mut px = vec!["pixel".to_string(), "body".to_string()];
    px.extend(fallbacks.iter().cloned());
    fonts.families.insert(FontFamily::Name("pixel".into()), px);
    let mut b = vec!["body_bold".to_string()];
    b.extend(fallbacks.iter().cloned());
    fonts.families.insert(FontFamily::Name("bold".into()), b);
    ctx.set_fonts(fonts);

    ctx.all_styles_mut(|style| {
        style.text_styles = [
            (TextStyle::Heading, pixel(22.0)),
            (TextStyle::Body, FontId::proportional(14.5)),
            (TextStyle::Button, FontId::proportional(14.5)),
            (TextStyle::Small, FontId::proportional(12.0)),
            (TextStyle::Monospace, FontId::monospace(13.5)),
        ]
        .into();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size.y = 26.0;
        style.spacing.slider_width = 160.0;
        let v = &mut style.visuals;
        v.dark_mode = true;
        v.override_text_color = Some(OFFWHITE);
        v.panel_fill = GRAPHITE;
        v.window_fill = Color32::from_rgb(24, 30, 26);
        v.extreme_bg_color = NEAR_BLACK;
        v.faint_bg_color = GRAPHITE_2;
        v.code_bg_color = NEAR_BLACK;
        v.hyperlink_color = GREEN;
        v.selection.bg_fill = DEEP_GREEN;
        v.selection.stroke = Stroke::new(1.0, GREEN);
        v.window_stroke = Stroke::new(2.0, DEEP_GREEN);
        v.window_corner_radius = CornerRadius::same(3);
        v.menu_corner_radius = CornerRadius::same(3);
        v.warn_fg_color = ORANGE;
        v.error_fg_color = RED;
        let r = CornerRadius::same(2);
        v.widgets.noninteractive.bg_fill = GRAPHITE;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, GRAPHITE_3);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, LIGHTGRAY);
        v.widgets.noninteractive.corner_radius = r;
        v.widgets.inactive.bg_fill = GRAPHITE_2;
        v.widgets.inactive.weak_bg_fill = GRAPHITE_2;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, GRAPHITE_3);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, OFFWHITE);
        v.widgets.inactive.corner_radius = r;
        v.widgets.hovered.bg_fill = DEEP_GREEN;
        v.widgets.hovered.weak_bg_fill = Color32::from_rgb(38, 90, 60);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, GREEN);
        v.widgets.hovered.fg_stroke = Stroke::new(1.5, OFFWHITE);
        v.widgets.hovered.corner_radius = r;
        v.widgets.active.bg_fill = Color32::from_rgb(60, 160, 90);
        v.widgets.active.weak_bg_fill = DEEP_GREEN;
        v.widgets.active.bg_stroke = Stroke::new(1.0, GREEN);
        // egui also uses the active text colour for `RichText::strong()`
        // labels, so it must stay light: dark text vanished on dark cards.
        v.widgets.active.fg_stroke = Stroke::new(2.0, OFFWHITE);
        v.widgets.active.corner_radius = r;
        v.widgets.open.bg_fill = GRAPHITE_3;
        v.widgets.open.weak_bg_fill = GRAPHITE_3;
        v.widgets.open.corner_radius = r;
    });
}

pub fn panel_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(GRAPHITE)
        .inner_margin(Margin::same(10))
        .stroke(Stroke::new(1.0, Color32::from_rgb(28, 36, 30)))
}

pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(GRAPHITE_2)
        .inner_margin(Margin::same(8))
        .corner_radius(CornerRadius::same(3))
        .stroke(Stroke::new(1.0, GRAPHITE_3))
}

pub fn modal_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(22, 28, 24))
        .inner_margin(Margin::same(18))
        .corner_radius(CornerRadius::same(4))
        .stroke(Stroke::new(2.0, DEEP_GREEN))
        .shadow(egui::Shadow { offset: [0, 6], blur: 24, spread: 2, color: Color32::from_black_alpha(160) })
}

pub fn tone_color(tone: game_core::messages::Tone) -> Color32 {
    use game_core::messages::Tone;
    match tone {
        Tone::Info => LIGHTGRAY,
        Tone::Good => GREEN,
        Tone::Bad => RED,
        Tone::Money => Color32::from_rgb(250, 214, 92),
        Tone::Story => OFFWHITE,
    }
}
