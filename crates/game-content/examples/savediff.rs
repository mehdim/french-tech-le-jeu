use game_core::save::{from_save_string, to_save_string};
fn main() {
    let c = game_content::load();
    let mut s = game_core::GameState::new(&c, 5);
    for _ in 0..200 { game_content::autopilot::step(&mut s, &c); }
    let data = to_save_string(&s, "t");
    let mut l = from_save_string(&data, &c).unwrap();
    let a0 = serde_json::to_string(&s).unwrap();
    let b0 = serde_json::to_string(&l).unwrap();
    println!("identique juste après le chargement : {}", a0 == b0);
    for step in 0..150 {
        game_content::autopilot::step(&mut s, &c);
        game_content::autopilot::step(&mut l, &c);
        let a = serde_json::to_string(&s).unwrap();
        let b = serde_json::to_string(&l).unwrap();
        if a != b {
            let i = a.bytes().zip(b.bytes()).position(|(x, y)| x != y).unwrap_or(0);
            println!("divergence à l'étape {step}, jour {} : ...{}...\n contre ...{}...", s.day, &a[i.saturating_sub(150)..(i + 80).min(a.len())], &b[i.saturating_sub(150)..(i + 80).min(b.len())]);
            break;
        }
    }
}
