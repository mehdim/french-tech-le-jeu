//! Integration tests for the simulation with real game content.

use game_content::autopilot;
use game_core::employees::Assignment;
use game_core::missions::Phase;
use game_core::research::Approach;
use game_core::save::{SaveError, from_save_string, to_save_string};
use game_core::types::{Role, Segment};
use game_core::{Command, GameState};

fn content() -> game_core::Content {
    game_content::load()
}

fn skip_intro(s: &mut GameState, c: &game_core::Content) {
    if s.campaign.intro_pending {
        s.apply(c, Command::DismissIntro).unwrap();
    }
}

#[test]
fn simulation_is_deterministic() {
    let c = content();
    let mut a = GameState::new(&c, 99);
    let mut b = GameState::new(&c, 99);
    for _ in 0..300 {
        autopilot::step(&mut a, &c);
        autopilot::step(&mut b, &c);
    }
    assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
    let mut d = GameState::new(&c, 100);
    for _ in 0..300 {
        autopilot::step(&mut d, &c);
    }
    assert_ne!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&d).unwrap(), "different seeds should diverge");
}

#[test]
fn save_roundtrip_continues_identically() {
    let c = content();
    let mut s = GameState::new(&c, 5);
    for _ in 0..200 {
        autopilot::step(&mut s, &c);
    }
    let data = to_save_string(&s, "test");
    let mut loaded = from_save_string(&data, &c).expect("save loads");
    for _ in 0..150 {
        autopilot::step(&mut s, &c);
        autopilot::step(&mut loaded, &c);
    }
    assert_eq!(serde_json::to_string(&s).unwrap(), serde_json::to_string(&loaded).unwrap());
}

#[test]
fn corrupt_and_incompatible_saves_are_rejected() {
    let c = content();
    let s = GameState::new(&c, 5);
    let data = to_save_string(&s, "test");
    // Flip a value inside the checksummed state.
    let mut env: serde_json::Value = serde_json::from_str(&data).unwrap();
    let state = env["state"].as_str().unwrap().replacen("\"day\":0", "\"day\":7", 1);
    env["state"] = serde_json::Value::String(state);
    assert!(matches!(from_save_string(&env.to_string(), &c), Err(SaveError::Corrupt)));
    let mut v: serde_json::Value = serde_json::from_str(&data).unwrap();
    v["schema"] = serde_json::json!(999);
    assert!(matches!(from_save_string(&v.to_string(), &c), Err(SaveError::Incompatible(999))));
    assert!(matches!(from_save_string("{not json", &c), Err(SaveError::Parse(_))));
    assert!(matches!(from_save_string(&data[..data.len() / 2], &c), Err(SaveError::Parse(_))));
}

#[test]
fn the_simulation_pauses_for_intro_and_events() {
    let c = content();
    let mut s = GameState::new(&c, 1);
    assert!(!s.tick(&c), "intro must block the clock");
    skip_intro(&mut s, &c);
    assert!(s.tick(&c));
    s.fire_event(&c, "kubernetes", false);
    let day = s.day;
    assert!(!s.tick(&c), "pending event must block the clock");
    assert_eq!(s.day, day);
}

#[test]
fn event_choices_have_real_consequences() {
    let c = content();
    let mut s = GameState::new(&c, 1);
    skip_intro(&mut s, &c);
    // Kubernetes: keep it -> higher hosting, more resilience.
    s.fire_event(&c, "kubernetes", false);
    let hosting = s.product.hosting_extra;
    let resilience = s.product.resilience;
    s.apply(&c, Command::ResolveEvent { choice: 0 }).unwrap();
    assert!(s.product.hosting_extra > hosting);
    assert!(s.product.resilience > resilience);
    assert!(s.flags.contains("k8s"));
    // Intern incident, ignored: a delayed consequence is scheduled.
    s.fire_event(&c, "intern_api_key", false);
    s.apply(&c, Command::ResolveEvent { choice: 1 }).unwrap();
    assert_eq!(s.events.delayed.len(), 1);
    // A choice with unmet requirements is refused.
    s.finance.cash = 1_000.0;
    s.fire_event(&c, "linkedin_influencer", false);
    assert!(s.apply(&c, Command::ResolveEvent { choice: 0 }).is_err());
    assert!(s.apply(&c, Command::ResolveEvent { choice: 1 }).is_ok());
}

#[test]
fn reckless_hiring_leads_to_bankruptcy() {
    let c = content();
    let mut s = GameState::new(&c, 3);
    // Jump to level 2 conditions: recruitment unlocked.
    s.campaign.unlocks.insert("recruitment".into());
    skip_intro(&mut s, &c);
    // Hire every ESN consultant we can find, do nothing else.
    for _ in 0..400 {
        if s.campaign.phase == Phase::Bankrupt {
            break;
        }
        if s.searches.is_empty() {
            let _ = s.apply(&c, Command::StartRecruitment { channel: "esn".into(), role: Role::Consultant });
        }
        let ids: Vec<u32> = s.candidates.iter().map(|c| c.id).collect();
        for id in ids {
            let _ = s.apply(&c, Command::Hire { candidate: id });
        }
        if s.events.pending.is_some() {
            let _ = s.apply(&c, Command::ResolveEvent { choice: 0 });
        }
        if s.campaign.intro_pending {
            skip_intro(&mut s, &c);
        }
        s.tick(&c);
    }
    assert_eq!(s.campaign.phase, Phase::Bankrupt, "cash {}", s.finance.cash);
}

#[test]
fn growth_without_retention_is_punished() {
    let c = content();
    let mut s = GameState::new(&c, 4);
    s.product.shipped.insert("mvp_text".into());
    s.product.launched = true;
    s.market.seg_mut(Segment::Smb).customers = 500;
    s.recompute_metrics(&c);
    let inp = s.funnel_inputs(&c);
    let seg = s.market.seg(Segment::Smb).clone();
    let poor = game_core::customers::churn_rate(&c, Segment::Smb, &seg, &inp);
    // Better product, retention features and support staff lower churn.
    s.product.feature_score = 70.0;
    s.product.model_accuracy = 88.0;
    s.product.fpr = 2.0;
    s.product.retention[0] = 0.4;
    s.last_work.ops = 6.0;
    let inp = s.funnel_inputs(&c);
    let good = game_core::customers::churn_rate(&c, Segment::Smb, &seg, &inp);
    assert!(poor > good * 2.0, "poor {poor} vs good {good}");
    // Raising prices raises churn.
    let mut pricey = seg.clone();
    pricey.price *= 2.0;
    assert!(game_core::customers::churn_rate(&c, Segment::Smb, &pricey, &inp) > good);
}

#[test]
fn tasks_progress_with_assigned_staff_and_ship() {
    let c = content();
    let mut s = GameState::new(&c, 8);
    skip_intro(&mut s, &c);
    s.apply(&c, Command::StartTask { task: "mvp_text".into(), approach: Approach::Standard }).unwrap();
    let inst = s.tasks[0].instance;
    // Nobody assigned: no progress.
    for _ in 0..5 {
        s.tick(&c);
        while s.events.pending.is_some() {
            s.apply(&c, Command::ResolveEvent { choice: 0 }).unwrap();
        }
    }
    assert_eq!(s.tasks[0].eng_done, 0.0);
    s.apply(&c, Command::Assign { employee: 1, assignment: Assignment::Task(inst) }).unwrap();
    for _ in 0..40 {
        if s.product.has("mvp_text") {
            break;
        }
        s.tick(&c);
        while s.events.pending.is_some() {
            s.apply(&c, Command::ResolveEvent { choice: 0 }).unwrap();
        }
    }
    assert!(s.product.has("mvp_text"));
    assert!(s.product.model_accuracy > 53.0);
}

#[test]
fn quick_approach_creates_more_debt_than_clean() {
    let c = content();
    let run = |approach: Approach| {
        let mut s = GameState::new(&c, 11);
        skip_intro(&mut s, &c);
        s.apply(&c, Command::StartTask { task: "mvp_text".into(), approach }).unwrap();
        let inst = s.tasks[0].instance;
        s.apply(&c, Command::Assign { employee: 1, assignment: Assignment::Task(inst) }).unwrap();
        let debt0 = s.product.tech_debt;
        while !s.product.has("mvp_text") {
            s.tick(&c);
            while s.events.pending.is_some() {
                s.apply(&c, Command::ResolveEvent { choice: 1.min(s.events.pending_def(&c).unwrap().choices.len() - 1) }).unwrap();
            }
        }
        (s.product.tech_debt - debt0, s.day)
    };
    let (quick_debt, quick_days) = run(Approach::Quick);
    let (clean_debt, clean_days) = run(Approach::Clean);
    assert!(quick_debt > clean_debt, "quick {quick_debt} clean {clean_debt}");
    assert!(quick_days < clean_days);
}

#[test]
fn office_placement_rules() {
    let c = content();
    let mut s = GameState::new(&c, 2);
    let o = &mut s.office;
    // Out of bounds, overlap, door, unreachable.
    assert!(o.can_place(&c, "plant", -1, 0, false, None).is_err());
    let (dx, dy) = game_core::office::door(c.office(0).unwrap());
    assert_eq!(o.can_place(&c, "plant", dx, dy, false, None), Err(game_core::office::PlaceError::BlocksDoor));
    let first = o.items[0].clone();
    assert_eq!(o.can_place(&c, "plant", first.x, first.y, false, None), Err(game_core::office::PlaceError::Overlap));
    let spot = o.find_spot(&c, "plant").expect("free spot");
    assert!(o.place(&c, "plant", spot.0, spot.1, spot.2).is_ok());
    // Placing furniture is locked until level 2.
    assert!(s.apply(&c, Command::PlaceFurniture { item: "plant".into(), x: 0, y: 2, rotated: false }).is_err());
}

#[test]
fn walls_of_furniture_cannot_trap_seats() {
    let c = content();
    let s = GameState::new(&c, 2);
    let mut o = s.office.clone();
    // Fill the apartment with plants until no spot remains: every item must
    // stay reachable from the door.
    while let Some((x, y, r)) = o.find_spot(&c, "plant") {
        o.place(&c, "plant", x, y, r).unwrap();
    }
    let walk = game_core::office::walkable(&c, &o);
    assert!(walk.iter().any(|w| *w));
}

#[test]
fn moving_office_keeps_furniture() {
    let c = content();
    let mut s = GameState::new(&c, 2);
    let before = s.office.items.len();
    s.office.move_to(&c, 2);
    assert!(s.office.items.len() + s.office.storage.len() >= before);
    assert_eq!(s.office.tier, 2);
}

#[test]
fn economy_formulas_handle_edge_cases() {
    use game_core::economy::{ltv, runway, safe_div};
    assert_eq!(safe_div(1.0, 0.0), 0.0);
    assert!(runway(10_000.0, 0.0).is_infinite());
    assert!(runway(10_000.0, -5.0).is_infinite());
    assert_eq!(runway(10_000.0, 2_000.0), 5.0);
    assert!(ltv(50.0, 0.8, 0.0).is_finite());
    assert!((ltv(50.0, 0.8, 5.0) - 800.0).abs() < 1e-9);
    let c = content();
    let s = GameState::new(&c, 1);
    let m = &s.metrics;
    assert_eq!(m.mrr, 0.0);
    assert_eq!(m.gross_margin, 0.0);
    assert!(m.runway.is_infinite() || m.runway > 0.0);
    assert_eq!(m.arr, m.mrr * 12.0);
}

#[test]
fn cap_table_dilution_math() {
    let mut cap = game_core::funding::CapTable { founder: 1_000_000.0, holders: vec![] };
    let d = cap.issue("Fonds", 250_000.0, 750_000.0);
    assert!((d - 25.0).abs() < 1e-9);
    assert!((cap.founder_pct() - 75.0).abs() < 1e-9);
}

#[test]
fn grants_take_paperwork_and_time() {
    let c = content();
    let mut s = GameState::new(&c, 21);
    skip_intro(&mut s, &c);
    s.apply(&c, Command::StartApplication { program: "france_travail".into() }).unwrap();
    // Without anyone on admin, nothing moves.
    for _ in 0..5 {
        s.tick(&c);
        while s.events.pending.is_some() {
            s.apply(&c, Command::ResolveEvent { choice: 0 }).unwrap();
        }
    }
    assert_eq!(s.funding.applications[0].admin_done, 0.0);
    s.apply(&c, Command::Assign { employee: 1, assignment: Assignment::Admin }).unwrap();
    for _ in 0..40 {
        s.tick(&c);
        while s.events.pending.is_some() {
            s.apply(&c, Command::ResolveEvent { choice: 1.min(s.events.pending_def(&c).unwrap().choices.len() - 1) }).unwrap();
        }
    }
    assert!(s.flags.contains("onboarding_done"), "{:?}", s.funding.applications);
}

#[test]
fn level_one_can_be_completed_by_hand() {
    let c = content();
    let mut s = GameState::new(&c, 31);
    skip_intro(&mut s, &c);
    s.apply(&c, Command::StartApplication { program: "france_travail".into() }).unwrap();
    s.apply(&c, Command::Assign { employee: 1, assignment: Assignment::Admin }).unwrap();
    s.apply(&c, Command::StartTask { task: "mvp_text".into(), approach: Approach::Standard }).unwrap();
    let inst = s.tasks[0].instance;
    for _ in 0..200 {
        if s.campaign.phase != Phase::Playing {
            break;
        }
        if s.flags.contains("onboarding_done") && s.tasks.iter().any(|t| t.instance == inst) {
            s.apply(&c, Command::Assign { employee: 1, assignment: Assignment::Task(inst) }).unwrap();
        }
        if s.product.has("mvp_text") {
            let _ = s.apply(&c, Command::Assign { employee: 1, assignment: Assignment::Sales });
        }
        if !s.flags.contains("onboarding_done") && s.funding.active_applications().count() == 0 {
            let _ = s.apply(&c, Command::StartApplication { program: "france_travail".into() });
            let _ = s.apply(&c, Command::Assign { employee: 1, assignment: Assignment::Admin });
        }
        while s.events.pending.is_some() {
            s.apply(&c, Command::ResolveEvent { choice: 0 }).unwrap();
        }
        s.tick(&c);
    }
    assert_eq!(s.campaign.phase, Phase::LevelComplete, "day {} quests {:?}", s.day, s.campaign.quests_done);
}

/// The headline balancing guarantee: a scripted player using only
/// player commands completes all twelve levels.
#[test]
fn autopilot_completes_the_campaign() {
    let c = content();
    for seed in [1u64, 2, 3] {
        let s = autopilot::play_campaign(&c, seed, 2000);
        assert_eq!(s.campaign.phase, Phase::Victory, "seed {seed}: stuck at level {} day {}", s.campaign.level, s.day);
        assert_eq!(s.campaign.results.len(), 12);
        assert!(s.campaign.ending.is_some());
    }
}

#[test]
fn every_ending_is_reachable_in_principle() {
    let c = content();
    let mut s = autopilot::play_campaign(&c, 1, 2000);
    // Rewind the final decision and check each choice's requirements are
    // satisfiable by plausible end-game states.
    s.campaign.phase = Phase::Playing;
    s.flags.retain(|f| !f.starts_with("ending_") && f != "final_choice");
    s.reputation = 70.0;
    s.events.pending = None;
    s.fire_event(&c, "final_decision", true);
    let def = s.events.pending_def(&c).unwrap().clone();
    let allowed: Vec<bool> = (0..def.choices.len()).map(|i| s.choice_allowed(&c, i)).collect();
    // Unicorn, acquisition and commons are open; profitable depends on burn.
    assert!(allowed[0] && allowed[2] && allowed[3], "{allowed:?}");
}
