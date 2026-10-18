use game_core::GameState;

#[test]
fn content_parses_and_validates() {
    let c = game_content::try_load().unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(c.levels.len(), 18);
    // 16 niveaux pour tout le monde, puis un niveau bonus selon la fin choisie.
    let mut flags = std::collections::BTreeSet::new();
    assert_eq!(c.campaign_length(&flags), 16);
    assert_eq!(c.next_level(16, &flags), None);
    flags.insert("ending_ipo".to_string());
    assert_eq!(c.campaign_length(&flags), 17);
    assert_eq!(c.next_level(16, &flags), Some(17));
    assert_eq!(c.next_level(17, &flags), None);
    flags.remove("ending_ipo");
    flags.insert("ending_acquisition".to_string());
    assert_eq!(c.next_level(16, &flags), Some(18));
    assert_eq!(c.next_level(18, &flags), None);
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
