//! Debug: run the autopilot N days and dump team/tasks.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(400);
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let c = game_content::load();
    let mut s = game_core::GameState::new(&c, seed);
    while s.day < days && game_content::autopilot::step(&mut s, &c) {}
    println!("day {} level {} cash {:.0} mrr {:.0} seats {}", s.day, s.campaign.level, s.finance.cash, s.metrics.mrr, s.metrics.seats);
    for e in &s.employees {
        println!("  {:3} {:22} {:?} {:?} mor {:.0} fat {:.0} {:?}", e.id, e.name, e.role, e.assignment, e.morale, e.fatigue, e.activity);
    }
    for t in &s.tasks {
        println!("  task {} {} {:.0}/{:.0} {:.0}/{:.0}", t.instance, t.task_id, t.eng_done, t.eng_total, t.research_done, t.research_total);
    }
    println!("shipped: {:?}", s.product.shipped);
    println!("unlocks: {:?}", s.campaign.unlocks);
    println!("searches {:?} candidates {}", s.searches.iter().map(|x| (&x.channel, x.role)).collect::<Vec<_>>(), s.candidates.len());
}
