//! Débogage : fait tourner l'autopilote N jours et affiche l'équipe et les chantiers.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(400);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let c = game_content::load();
    let mut s = game_core::GameState::new(&c, seed);
    while s.day < days && game_content::autopilot::step(&mut s, &c) {}
    println!("jour {} niveau {} trésorerie {:.0} mrr {:.0} places {}", s.day, s.campaign.level, s.finance.cash, s.metrics.mrr, s.metrics.seats);
    for e in &s.employees {
        println!("  {:3} {:22} {:?} {:?} moral {:.0} fatigue {:.0} {:?}", e.id, e.name, e.role, e.assignment, e.morale, e.fatigue, e.activity);
    }
    for t in &s.tasks {
        println!("  tâche {} {} {:.0}/{:.0} {:.0}/{:.0}", t.instance, t.task_id, t.eng_done, t.eng_total, t.research_done, t.research_total);
    }
    println!("livré : {:?}", s.product.shipped);
    println!("déblocages : {:?}", s.campaign.unlocks);
    println!("recherches {:?} candidats {}", s.searches.iter().map(|x| (&x.channel, x.role)).collect::<Vec<_>>(), s.candidates.len());
}
