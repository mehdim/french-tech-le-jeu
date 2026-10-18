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
    // Le tutoriel au début du niveau 1, avec une héroïne.
    let mut t = game_core::GameState::new(&c, 7);
    t.set_founder("Camille", true);
    t.tutorial = Some(0);
    t.choose_starting_mentor(&c, "steeve").unwrap();
    let _ = t.apply(&c, game_core::Command::DismissIntro);
    t.recompute_metrics(&c);
    std::fs::write(format!("{out}/tutorial.json"), to_save_string(&t, "démo")).unwrap();
    t.tutorial = Some(2);
    std::fs::write(format!("{out}/tutorial2.json"), to_save_string(&t, "démo")).unwrap();
    let mut t2 = game_core::GameState::new(&c, 7);
    t2.set_founder("Camille", true);
    t2.start_level(&c, 2);
    t2.campaign.intro_pending = false;
    t2.events.pending = None;
    t2.events.queue.clear();
    t2.office.move_to(&c, 1);
    t2.tutorial = Some(10);
    t2.choose_starting_mentor(&c, "jb").unwrap();
    t2.recompute_metrics(&c);
    std::fs::write(format!("{out}/tutorial_l2.json"), to_save_string(&t2, "démo")).unwrap();
    // Les niveaux bonus : 17 après l'IPO, 18 après le rachat par NVIDIA.
    for (pref, name) in [("ending_ipo", "level17"), ("ending_acquisition", "level18")] {
        autopilot::prefer_ending(Some(pref));
        let mut b = game_core::GameState::new(&c, 1);
        b.set_founder("Camille", true);
        while b.campaign.level < 17 && autopilot::step(&mut b, &c) {}
        for _ in 0..40 {
            autopilot::step(&mut b, &c);
        }
        b.recompute_metrics(&c);
        std::fs::write(format!("{out}/{name}.json"), to_save_string(&b, "démo")).unwrap();
        let mut i = b.clone();
        i.start_level(&c, i.campaign.level);
        std::fs::write(format!("{out}/{name}_intro.json"), to_save_string(&i, "démo")).unwrap();
    }
    autopilot::prefer_ending(None);
    let mut s = game_core::GameState::new(&c, 1);
    while s.campaign.level < 16 && autopilot::step(&mut s, &c) {}
    for _ in 0..12 {
        autopilot::step(&mut s, &c);
    }
    s.campaign.intro_pending = false;
    s.recompute_metrics(&c);
    std::fs::write(format!("{out}/level16.json"), to_save_string(&s, "démo")).unwrap();
    // Une défense anti-rachat prête, par entreprise, pour tester les combats.
    for company in ["google", "microsoft", "amazon", "meta", "nvidia"] {
        let mut e = s.clone();
        e.takeover = Some(game_core::simulation::Takeover {
            company: company.to_string(),
            points: 1000.0,
            required: 1000.0,
            deadline_day: e.day + 30,
            ready: true,
        });
        std::fs::write(format!("{out}/fight_{company}.json"), to_save_string(&e, "démo")).unwrap();
    }
    // Une sauvegarde par fin, pour tester les animations et les mini-jeux.
    for ending in ["unicorn", "ipo", "acquisition", "profitable", "commons"] {
        let mut e = s.clone();
        e.campaign.phase = game_core::missions::Phase::Victory;
        e.campaign.ending = Some(ending.to_string());
        std::fs::write(format!("{out}/ending_{ending}.json"), to_save_string(&e, "démo")).unwrap();
    }
    println!("niveau {} jour {} énergie {:.0} MWh", s.campaign.level, s.day, s.metrics.energy_mwh);
}
