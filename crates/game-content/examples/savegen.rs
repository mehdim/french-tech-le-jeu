//! Génère des sauvegardes de démonstration (un bureau par niveau, et une fin de campagne)
//! pour les captures d'écran du navigateur. Usage : cargo run --example savegen -- <dossier>
use game_content::autopilot;
use game_core::save::to_save_string;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let c = game_content::load();
    for (tier, level) in [(0u32, 1u32), (1, 2), (2, 3), (3, 5), (4, 6), (5, 9)] {
        let mut s = game_core::GameState::new(&c, 7);
        s.start_level(&c, level);
        s.campaign.intro_pending = false;
        s.events.pending = None;
        s.events.queue.clear();
        s.office.move_to(&c, tier);
        s.flags.insert("mentor_mehdi".into());
        s.flags.insert("bpi_certificate".into());
        s.recompute_metrics(&c);
        std::fs::write(format!("{out}/tier{tier}.json"), to_save_string(&s, "démo")).unwrap();
    }
    let mut s = game_core::GameState::new(&c, 1);
    while s.campaign.level < 16 && autopilot::step(&mut s, &c) {}
    for _ in 0..12 {
        autopilot::step(&mut s, &c);
    }
    s.campaign.intro_pending = false;
    s.recompute_metrics(&c);
    std::fs::write(format!("{out}/level16.json"), to_save_string(&s, "démo")).unwrap();
    println!("niveau {} jour {} énergie {:.0} MWh", s.campaign.level, s.day, s.metrics.energy_mwh);
}
