use game_core::GameState;

#[test]
fn content_parses_and_validates() {
    let c = game_content::try_load().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(c.levels.len(), 12);
    assert!(c.events.iter().filter(|e| !e.scripted).count() >= 40);
    assert!(c.tasks.len() >= 40);
}

#[test]
fn starter_offices_are_valid_layouts() {
    let c = game_content::load();
    for tier in &c.offices {
        let mut office = game_core::office::OfficeState::new(tier.id);
        office.move_to(&c, tier.id);
        assert_eq!(office.items.len(), tier.starter.len(), "bureau {} : agencement de départ invalide", tier.id);
    }
}

#[test]
fn new_game_starts_level_one() {
    let c = game_content::load();
    let s = GameState::new(&c, 42);
    assert_eq!(s.campaign.level, 1);
    assert_eq!(s.finance.cash, 30_000.0);
    assert_eq!(s.employees.len(), 1);
    assert!((s.metrics.accuracy - 53.0).abs() < 1e-9);
}
