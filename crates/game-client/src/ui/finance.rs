//! Onglet Finances : compte de résultat au rythme actuel, unit economics, bilans mensuels, graphiques.

use super::style::*;
use super::widgets::*;
use super::Ctx;
use bevy_egui::egui::{self, RichText};
use game_content::dialogue;

pub fn show(ui: &mut egui::Ui, c: &mut Ctx) {
    heading(ui, "Finances");
    let g = &c.game.0;
    let m = &g.metrics;
    let rr = &m.run_rate;
    subheading(ui, "Ce mois-ci (rythme actuel)");
    card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        kv(ui, "Revenus (MRR)", money(m.mrr), GREEN, "Abonnements des clients payants.");
        if rr.other_income > 0.0 {
            kv(ui, "Autres revenus récurrents", money(rr.other_income), GREEN, "Aides versées mensuellement.");
        }
        kv(ui, "  Inférence (COGS)", format!("-{}", money(rr.inference)), RED, "Coût des analyses : clients × usage × coût/1000 analyses.");
        kv(ui, "  Hébergement (COGS)", format!("-{}", money(rr.hosting)), RED, "Serveurs, stockage, base + coût par client.");
        kv(ui, "Marge brute", if m.mrr > 0.0 { dialogue::pct(m.gross_margin * 100.0) } else { "—".into() }, if m.gross_margin >= 0.7 { GREEN } else { ORANGE },
            "(MRR − COGS) / MRR.");
        kv(ui, "  Salaires chargés", format!("-{}", money(rr.payroll)), RED, "Brut + charges patronales (≈45 %, 20 % pour la R&D sous statut JEI).");
        kv(ui, "  Loyer", format!("-{}", money(rr.rent)), RED, "");
        kv(ui, "  Marketing", format!("-{}", money(rr.marketing)), RED, "");
        kv(ui, "  Outils SaaS", format!("-{}", money(rr.tools)), RED, "Slack, Notion, Jira, Figma… par personne.");
        if rr.furniture > 0.0 {
            kv(ui, "  Électricité & équipements", format!("-{}", money(rr.furniture)), RED, "Serveurs, machines.");
        }
        if rr.loans > 0.0 {
            kv(ui, "  Remboursements de prêts", format!("-{}", money(rr.loans)), RED, "");
        }
        ui.separator();
        let burn = m.net_burn;
        kv(ui, if burn > 0.0 { "Burn net" } else { "Bénéfice net" }, money(burn.abs()), if burn > 0.0 { RED } else { GREEN },
            "Burn net = dépenses − revenus. Négatif = vous gagnez de l'argent.");
        kv(ui, "Runway", if m.runway.is_infinite() { "∞".into() } else { dialogue::months(m.runway) }, if m.runway < 6.0 { ORANGE } else { OFFWHITE },
            "Trésorerie / burn net.");
        kv(ui, "ARR (run-rate)", money(m.arr), OFFWHITE, "MRR × 12. Ce n'est pas du chiffre d'affaires réalisé.");
    });
    chart(ui, c);
    reports(ui, c);
}

fn chart(ui: &mut egui::Ui, c: &mut Ctx) {
    let h = &c.game.0.finance.history;
    if h.len() < 2 {
        return;
    }
    subheading(ui, "Trésorerie et MRR");
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 120.0), egui::Sense::hover());
    let p = ui.painter_at(rect);
    p.rect_filled(rect, 2.0, NEAR_BLACK);
    let n = h.len();
    let start = n.saturating_sub(80);
    let pts = &h[start..];
    let max_cash = pts.iter().map(|x| x.1).fold(1.0f64, f64::max);
    let min_cash = pts.iter().map(|x| x.1).fold(0.0f64, f64::min);
    let max_mrr = pts.iter().map(|x| x.2).fold(1.0f64, f64::max);
    let to = |i: usize, v: f64, lo: f64, hi: f64| {
        let x = rect.min.x + 6.0 + (rect.width() - 12.0) * i as f32 / (pts.len().max(2) - 1) as f32;
        let y = rect.max.y - 8.0 - (rect.height() - 20.0) * ((v - lo) / (hi - lo).max(1.0)) as f32;
        egui::pos2(x, y)
    };
    if min_cash < 0.0 {
        let zero = to(0, 0.0, min_cash, max_cash).y;
        p.line_segment([egui::pos2(rect.min.x, zero), egui::pos2(rect.max.x, zero)], egui::Stroke::new(1.0, RED));
    }
    let cash: Vec<egui::Pos2> = pts.iter().enumerate().map(|(i, x)| to(i, x.1, min_cash, max_cash)).collect();
    let mrr: Vec<egui::Pos2> = pts.iter().enumerate().map(|(i, x)| to(i, x.2, 0.0, max_mrr)).collect();
    p.add(egui::Shape::line(cash, egui::Stroke::new(2.0, ORANGE)));
    p.add(egui::Shape::line(mrr, egui::Stroke::new(2.0, GREEN)));
    p.text(rect.min + egui::vec2(6.0, 4.0), egui::Align2::LEFT_TOP, format!("Trésorerie (max {})", money(max_cash)), egui::FontId::proportional(11.0), ORANGE);
    p.text(egui::pos2(rect.max.x - 6.0, rect.min.y + 4.0), egui::Align2::RIGHT_TOP, format!("MRR (max {})", money(max_mrr)), egui::FontId::proportional(11.0), GREEN);
}

fn reports(ui: &mut egui::Ui, c: &mut Ctx) {
    let reports: Vec<_> = c.game.0.finance.reports.iter().rev().take(8).cloned().collect();
    if reports.is_empty() {
        muted(ui, "Le premier bilan mensuel arrive au jour 28.");
        return;
    }
    subheading(ui, "Bilans mensuels");
    egui::Grid::new("reports").striped(true).spacing([10.0, 4.0]).show(ui, |ui| {
        for h in ["Mois", "CA", "COGS", "Salaires", "Burn", "Clients", "Churn"] {
            ui.label(RichText::new(h).size(11.5).color(MUTED));
        }
        ui.end_row();
        for r in reports {
            ui.label(RichText::new(format!("M{}", r.month)).size(12.0));
            ui.label(RichText::new(money(r.revenue)).size(12.0).color(GREEN));
            ui.label(RichText::new(money(r.cogs)).size(12.0));
            ui.label(RichText::new(money(r.payroll)).size(12.0));
            ui.label(RichText::new(money(r.net_burn)).size(12.0).color(if r.net_burn > 0.0 { RED } else { GREEN }));
            ui.label(RichText::new(format!("{} (+{} −{})", r.customers_end, r.new_customers, r.churned)).size(12.0));
            ui.label(RichText::new(dialogue::pct(r.churn_pct)).size(12.0));
            ui.end_row();
        }
    });
}
