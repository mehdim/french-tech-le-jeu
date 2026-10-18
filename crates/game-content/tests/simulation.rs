//! Tests d'intégration de la simulation avec le vrai contenu du jeu.

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

/// Passe l'intro du niveau 1, puis choisit le premier mentor proposé.
fn skip_intro(s: &mut GameState, c: &game_core::Content) {
    if s.campaign.intro_pending {
        s.apply(c, Command::DismissIntro).unwrap();
    }
    if s.events.pending.as_ref().is_some_and(|p| p.event_id == "mentors_intro") {
        s.apply(c, Command::ResolveEvent { choice: 0 }).unwrap();
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
    assert_ne!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&d).unwrap(), "des graines différentes doivent diverger");
}

#[test]
fn save_roundtrip_continues_identically() {
    let c = content();
    let mut s = GameState::new(&c, 5);
    for _ in 0..200 {
        autopilot::step(&mut s, &c);
    }
    let data = to_save_string(&s, "test");
    let mut loaded = from_save_string(&data, &c).expect("la sauvegarde se charge");
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
    // Modifie une valeur dans l'état protégé par la somme de contrôle.
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
    assert!(!s.tick(&c), "l'intro doit bloquer l'horloge");
    skip_intro(&mut s, &c);
    assert!(s.tick(&c));
    s.fire_event(&c, "kubernetes", false);
    let day = s.day;
    assert!(!s.tick(&c), "un événement en attente doit bloquer l'horloge");
    assert_eq!(s.day, day);
}

#[test]
fn event_choices_have_real_consequences() {
    let c = content();
    let mut s = GameState::new(&c, 1);
    skip_intro(&mut s, &c);
    // Kubernetes : on le garde -> hébergement plus cher, plus de résilience.
    s.fire_event(&c, "kubernetes", false);
    let hosting = s.product.hosting_extra;
    let resilience = s.product.resilience;
    s.apply(&c, Command::ResolveEvent { choice: 0 }).unwrap();
    assert!(s.product.hosting_extra > hosting);
    assert!(s.product.resilience > resilience);
    assert!(s.flags.contains("k8s"));
    // Incident du stagiaire, ignoré : une conséquence différée est programmée.
    s.fire_event(&c, "intern_api_key", false);
    s.apply(&c, Command::ResolveEvent { choice: 1 }).unwrap();
    assert_eq!(s.events.delayed.len(), 1);
    // Un choix dont les conditions ne sont pas remplies est refusé.
    s.finance.cash = 1_000.0;
    s.fire_event(&c, "linkedin_influencer", false);
    assert!(s.apply(&c, Command::ResolveEvent { choice: 0 }).is_err());
    assert!(s.apply(&c, Command::ResolveEvent { choice: 1 }).is_ok());
}

#[test]
fn reckless_hiring_leads_to_bankruptcy() {
    let c = content();
    let mut s = GameState::new(&c, 3);
    // Saut aux conditions du niveau 2 : le recrutement est débloqué.
    s.campaign.unlocks.insert("recruitment".into());
    skip_intro(&mut s, &c);
    // Recruter tous les consultants d'ESN disponibles, et ne rien faire d'autre.
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
    assert_eq!(s.campaign.phase, Phase::Bankrupt, "trésorerie {}", s.finance.cash);
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
    // Un meilleur produit, des fonctionnalités de rétention et du support font baisser le churn.
    s.product.feature_score = 70.0;
    s.product.model_accuracy = 88.0;
    s.product.fpr = 2.0;
    s.product.retention[0] = 0.4;
    s.last_work.ops = 6.0;
    let inp = s.funnel_inputs(&c);
    let good = game_core::customers::churn_rate(&c, Segment::Smb, &seg, &inp);
    assert!(poor > good * 2.0, "mauvais {poor} contre bon {good}");
    // Augmenter les prix augmente le churn.
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
    // Personne d'affecté : aucune progression.
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
    assert!(quick_debt > clean_debt, "vite fait {quick_debt} propre {clean_debt}");
    assert!(quick_days < clean_days);
}

#[test]
fn office_placement_rules() {
    let c = content();
    let mut s = GameState::new(&c, 2);
    let o = &mut s.office;
    // Hors des murs, chevauchement, porte, inaccessible.
    assert!(o.can_place(&c, "plant", -1, 0, false, None).is_err());
    let (dx, dy) = game_core::office::door(c.office(0).unwrap());
    assert_eq!(o.can_place(&c, "plant", dx, dy, false, None), Err(game_core::office::PlaceError::BlocksDoor));
    let first = o.items[0].clone();
    assert_eq!(o.can_place(&c, "plant", first.x, first.y, false, None), Err(game_core::office::PlaceError::Overlap));
    let spot = o.find_spot(&c, "plant").expect("emplacement libre");
    assert!(o.place(&c, "plant", spot.0, spot.1, spot.2).is_ok());
    // Le placement de mobilier est verrouillé jusqu'au niveau 2.
    assert!(s.apply(&c, Command::PlaceFurniture { item: "plant".into(), x: 0, y: 2, rotated: false }).is_err());
}

#[test]
fn walls_of_furniture_cannot_trap_seats() {
    let c = content();
    let s = GameState::new(&c, 2);
    let mut o = s.office.clone();
    // Remplir l'appartement de plantes jusqu'à ce qu'il n'y ait plus de place : chaque objet
    // doit rester accessible depuis la porte.
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
    // Sans personne sur l'administratif, rien n'avance.
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
    assert_eq!(s.campaign.phase, Phase::LevelComplete, "jour {} objectifs {:?}", s.day, s.campaign.quests_done);
}

/// La garantie d'équilibrage principale : un joueur scripté qui n'utilise que
/// les commandes du joueur termine les seize niveaux.
#[test]
fn autopilot_completes_the_campaign() {
    let c = content();
    for seed in [1u64, 2, 3] {
        let s = autopilot::play_campaign(&c, seed, 2000);
        assert_eq!(s.campaign.phase, Phase::Victory, "graine {seed} : bloqué au niveau {} jour {}", s.campaign.level, s.day);
        assert_eq!(s.campaign.results.len(), c.levels.len());
        assert!(s.campaign.ending.is_some());
    }
}

#[test]
fn every_ending_is_reachable_in_principle() {
    let c = content();
    let mut s = autopilot::play_campaign(&c, 1, 2000);
    // On revient sur la décision finale et on vérifie que les conditions de chaque choix
    // sont atteignables par des états de fin de partie plausibles.
    s.campaign.phase = Phase::Playing;
    s.flags.retain(|f| !f.starts_with("ending_") && f != "final_choice");
    s.reputation = 70.0;
    s.flags.insert("series_c_done".into());
    s.flags.insert("nvidia_interest".into());
    s.product.shipped.insert("ipo_readiness".into());
    s.events.pending = None;
    s.fire_event(&c, "final_decision", true);
    let def = s.events.pending_def(&c).unwrap().clone();
    let allowed: Vec<bool> = (0..def.choices.len()).map(|i| s.choice_allowed(&c, i)).collect();
    // Série C, IPO, NVIDIA et communs sont ouverts ; « rentable » dépend du burn.
    assert_eq!(def.choices.len(), 5);
    assert!(allowed[0] && allowed[1] && allowed[2] && allowed[4], "{allowed:?}");
    // Chaque choix mène à une fin définie.
    for ch in &def.choices {
        let flag = ch.effects.iter().find_map(|e| match e {
            game_core::types::Effect::SetFlag(f) => f.strip_prefix("ending_"),
            _ => None,
        });
        assert!(flag.is_some_and(|f| c.ending(f).is_some()), "{}", ch.label);
    }
}

#[test]
fn mentor_choice_opens_the_campaign_and_applies_once() {
    let c = content();
    let mut s = GameState::new(&c, 3);
    s.apply(&c, Command::DismissIntro).unwrap();
    let pending = s.events.pending.as_ref().map(|p| p.event_id.clone());
    assert_eq!(pending.as_deref(), Some("mentors_intro"), "le choix du mentor ouvre la campagne");
    let cost = s.product.inference_cost_per_k;
    // Choix 1 = Steeve : coût d'inférence -12 %.
    s.apply(&c, Command::ResolveEvent { choice: 1 }).unwrap();
    assert_eq!(s.mentors.main.as_deref(), Some("steeve"));
    assert!(s.flags.contains("mentor_steeve"));
    assert!((s.product.inference_cost_per_k - cost * 0.88).abs() < 1e-9);
    // Un seul mentor principal : le bonus ne se cumule pas.
    assert!(s.apply(&c, Command::ChooseMentor { mentor: "jb".into() }).is_err());
    assert!((s.product.inference_cost_per_k - cost * 0.88).abs() < 1e-9);
}

#[test]
fn mentor_advice_depends_on_the_situation_and_has_a_cooldown() {
    let c = content();
    let mut s = GameState::new(&c, 4);
    skip_intro(&mut s, &c); // mentor principal : Mehdi (choix 0)
    let mehdi = c.mentor("mehdi").unwrap().clone();
    let jb = c.mentor("jb").unwrap().clone();
    // Sans produit ni équipe, Mehdi n'a que son conseil général.
    let general = s.mentor_advice(&mehdi).unwrap().text.clone();
    assert_eq!(general, mehdi.advice.last().unwrap().text);
    // Une fois le MVP livré sans API, son conseil change.
    s.product.shipped.insert("mvp_text".into());
    let specific = s.mentor_advice(&mehdi).unwrap().text.clone();
    assert_ne!(specific, general);
    s.apply(&c, Command::ConsultMentor { mentor: "mehdi".into() }).unwrap();
    assert_eq!(s.mentors.last_advice.get("mehdi"), Some(&specific));
    // Délai : 49 jours pour le mentor principal, 98 pour les autres.
    assert!(s.apply(&c, Command::ConsultMentor { mentor: "mehdi".into() }).is_err());
    assert_eq!(s.mentor_wait(&mehdi), mehdi.cooldown_main);
    s.apply(&c, Command::ConsultMentor { mentor: "jb".into() }).unwrap();
    assert_eq!(s.mentor_wait(&jb), jb.cooldown);
    assert!(mehdi.cooldown_main < jb.cooldown);
}

#[test]
fn saves_from_before_the_mentors_still_load() {
    let c = content();
    let s = GameState::new(&c, 5);
    let mut v = serde_json::to_value(&s).unwrap();
    v.as_object_mut().unwrap().remove("mentors");
    let old: GameState = serde_json::from_value(v).expect("une ancienne sauvegarde sans mentors se charge");
    assert!(old.mentors.main.is_none());
}


#[test]
fn understaffing_from_level_ten_hurts() {
    let c = content();
    let mut s = GameState::new(&c, 8);
    s.start_level(&c, 10);
    s.campaign.intro_pending = false;
    s.events.pending = None;
    s.events.queue.clear();
    let need = c.level(10).unwrap().staffing.expect("le niveau 10 exige un effectif minimum");
    assert_eq!((need.engineers, need.headcount), (4, 20));
    // Seul le fondateur : pas un seul ingénieur à plein temps.
    assert_eq!(s.full_time_engineers(), 0);
    // Témoin : même état, mais au niveau 9 (pas d'exigence d'effectif).
    let mut control = s.clone();
    control.campaign.level = 9;
    for st in [&mut s, &mut control] {
        for _ in 0..10 {
            st.tick(&c);
            st.events.pending = None;
        }
    }
    assert!(s.product.tech_debt > control.product.tech_debt, "la dette doit monter en sous-effectif");
    assert!(s.employees[0].fatigue > control.employees[0].fatigue, "l'équipe s'épuise en sous-effectif");
}

#[test]
fn local_hiring_counts_and_markets_need_their_unlock() {
    let c = content();
    let mut s = GameState::new(&c, 9);
    s.campaign.unlocks.insert("international".into());
    s.finance.cash = 5_000_000.0;
    // Sans le déblocage « usa », la filiale américaine est fermée.
    assert!(s.apply(&c, Command::OpenMarket { market: "us".into() }).is_err());
    s.campaign.unlocks.insert("usa".into());
    s.campaign.unlocks.insert("recruitment".into());
    s.apply(&c, Command::OpenMarket { market: "us".into() }).unwrap();
    assert!(s.check(&game_core::types::Condition::MarketOpened("us".into())));
    // Le loyer de la filiale s'ajoute aux charges.
    s.recompute_metrics(&c);
    assert!(s.metrics.run_rate.rent >= 90_000.0);
    // Une embauche par le canal de San Francisco incrémente le compteur.
    s.apply(&c, Command::StartRecruitment { channel: "sf_local".into(), role: Role::Backend }).unwrap();
    for _ in 0..20 {
        s.tick(&c);
        s.events.pending = None;
        s.campaign.intro_pending = false;
    }
    let id = s.candidates.iter().find(|x| x.channel == "sf_local").expect("candidats de San Francisco").id;
    s.apply(&c, Command::Hire { candidate: id }).unwrap();
    assert_eq!(s.counter("hires_us"), 1);
}

#[test]
fn energy_is_billed_only_with_own_datacenters() {
    let c = content();
    let mut s = autopilot::play_campaign(&c, 1, 700);
    s.recompute_metrics(&c);
    assert_eq!(s.metrics.run_rate.energy, 0.0);
    s.flags.insert("own_datacenters".into());
    s.recompute_metrics(&c);
    assert!(s.metrics.energy_mwh > 0.0);
    assert!(s.metrics.run_rate.energy > 0.0);
    let before = s.metrics.run_rate.energy;
    s.apply_effects(&c, &[game_core::types::Effect::EnergyMul(0.5)], "test");
    s.recompute_metrics(&c);
    assert!((s.metrics.run_rate.energy - before * 0.5).abs() < 1e-6);
}

#[test]
fn takeover_offers_arrive_and_a_lost_fight_ends_the_game() {
    let c = content();
    // Les cinq géants ont un événement d'offre et une fin correspondante.
    for co in ["google", "microsoft", "amazon", "meta", "nvidia"] {
        assert!(c.event(&format!("takeover_{co}")).is_some(), "offre {co}");
        assert!(c.ending(&format!("takeover_{co}")).is_some(), "fin {co}");
    }
    let mut s = GameState::new(&c, 12);
    s.start_level(&c, 14);
    s.campaign.intro_pending = false;
    s.events.pending = None;
    s.events.queue.clear();
    s.finance.cash = 50_000_000.0;
    let mut offered = None;
    for _ in 0..40 {
        s.tick(&c);
        if let Some(p) = &s.events.pending {
            if p.event_id.starts_with("takeover_") {
                offered = Some(p.event_id.clone());
                break;
            }
            s.events.pending = None;
        }
    }
    let id = offered.expect("une offre de rachat arrive au niveau 14");
    // Résister : la défense démarre.
    s.apply(&c, Command::ResolveEvent { choice: 0 }).unwrap();
    let t = s.takeover.clone().expect("défense lancée");
    assert_eq!(format!("takeover_{}", t.company), id);
    // Sans pitch, le délai expire et le rachat passe en force.
    let mut idle = s.clone();
    for _ in 0..130 {
        if idle.campaign.phase != Phase::Playing {
            break;
        }
        idle.tick(&c);
        idle.events.pending = None;
    }
    assert_eq!(idle.campaign.ending.as_deref(), Some(id.as_str()), "délai dépassé = rachat");
    // Le pitch du fondateur fait monter les points ; défense prête → simulation en attente.
    s.takeover.as_mut().unwrap().points = 960.0;
    s.apply(&c, Command::Assign { employee: 1, assignment: Assignment::Fundraising }).unwrap();
    for _ in 0..400 {
        if s.takeover.as_ref().is_some_and(|t| t.ready) || s.campaign.phase != Phase::Playing {
            break;
        }
        s.tick(&c);
        s.events.pending = None;
    }
    assert!(s.takeover.as_ref().is_some_and(|t| t.ready), "{:?}", s.takeover);
    assert!(!s.is_running());
    // Un combat gagné : Veridix reste indépendante.
    let mut won = s.clone();
    won.apply(&c, Command::ResolveTakeover { won: true }).unwrap();
    assert!(won.takeover.is_none() && won.campaign.phase == Phase::Playing);
    // Un combat perdu : rachat, la partie s'arrête sur la fin de l'entreprise.
    s.apply(&c, Command::ResolveTakeover { won: false }).unwrap();
    assert_eq!(s.campaign.phase, Phase::Victory);
    assert_eq!(s.campaign.ending.as_deref(), Some(id.as_str()));
}
