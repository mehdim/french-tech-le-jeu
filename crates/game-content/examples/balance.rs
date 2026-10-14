//! Balancing report: plays the campaign with the autopilot and prints a
//! monthly trace. Usage: cargo run -p game-content --example balance -- [seed] [quiet]
use game_content::{autopilot, dialogue};
use game_core::missions::Phase;
use game_core::types::Segment;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let quiet = args.iter().any(|a| a == "quiet");
    let c = game_content::load();
    let mut s = game_core::GameState::new(&c, seed);
    let mut last_level = 0;
    let mut log_seen = 0;
    while s.day < 3000 && autopilot::step(&mut s, &c) {
        if s.campaign.level != last_level {
            println!("=== day {:4}  LEVEL {} {}", s.day, s.campaign.level, c.level(s.campaign.level).unwrap().title);
            last_level = s.campaign.level;
        }
        if !quiet {
            for e in &s.log[log_seen.min(s.log.len())..] {
                let t = dialogue::format(&e.msg);
                if t.starts_with("Objectif") || t.contains("Tour bouclé") || t.contains("Accordé") || t.contains("Refusé") || t.contains("Déménagement") || t.contains("Trésorerie négative") {
                    println!("      d{:4} {}", e.day, t);
                }
            }
        }
        log_seen = s.log.len();
        if s.day % 28 == 0 && !quiet {
            let m = &s.metrics;
            println!(
                "d{:4} L{:2} cash {:>10} mrr {:>9} cust {:>4}/{:>3}/{:>3} churn {:4.1} q {:4.1} acc {:4.1} fpr {:4.1} debt {:4.1} mor {:4.1} fat {:4.1} emp {:2} run {:5.1} gm {:4.2} val {:>9} comp {:.2} drift {:.1} cpk {:.2}",
                s.day, s.campaign.level, dialogue::money(s.finance.cash), dialogue::money(m.mrr),
                s.market.seg(Segment::Smb).customers, s.market.seg(Segment::Mid).customers, s.market.seg(Segment::Enterprise).customers,
                m.churn, m.quality, m.accuracy, m.fpr, m.tech_debt, m.morale, m.fatigue, m.employees, m.runway.min(99.0), m.gross_margin,
                dialogue::money(m.valuation), s.competition(&c), s.product.drift, m.inference_cost_per_k
            );
        }
    }
    println!("END day {} phase {:?} level {} ending {:?}", s.day, s.campaign.phase, s.campaign.level, s.campaign.ending);
    for r in &s.campaign.results {
        println!("  level {:2}: {:4} days  cash {:>10}  mrr {:>10}", r.level, r.days, dialogue::money(r.cash_end), dialogue::money(r.mrr_end));
    }
    if s.campaign.phase != Phase::Victory {
        let lvl = c.level(s.campaign.level).unwrap();
        for (i, q) in lvl.quests.iter().enumerate() {
            println!("  quest {:30} done={} now={}", q.label, s.campaign.quests_done[i], s.check(&q.condition));
        }
    }
    println!("stats {:?}", s.stats);
}
