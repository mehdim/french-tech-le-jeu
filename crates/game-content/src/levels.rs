//! Content validation: every id referenced anywhere must exist, so a typo in
//! a RON file fails `cargo test` instead of silently breaking a level.

use game_core::content::{BossTrigger, Content};
use game_core::types::{Condition, Effect};
use std::collections::BTreeSet;

fn walk_condition<'a>(c: &'a Condition, out: &mut Vec<&'a Condition>) {
    out.push(c);
    match c {
        Condition::All(v) | Condition::Any(v) => v.iter().for_each(|c| walk_condition(c, out)),
        Condition::Not(c) => walk_condition(c, out),
        _ => {}
    }
}

/// Returns a list of human-readable problems (empty = valid).
pub fn validate(content: &Content) -> Vec<String> {
    let mut problems = Vec::new();
    let task_ids: BTreeSet<&str> = content.tasks.iter().map(|t| t.id.as_str()).collect();
    let event_ids: BTreeSet<&str> = content.events.iter().map(|e| e.id.as_str()).collect();
    let mut unlocks: BTreeSet<&str> = BTreeSet::new();
    for l in &content.levels {
        for u in &l.unlocks {
            unlocks.insert(u.as_str());
        }
    }

    let mut check_unique = |what: &str, ids: Vec<&str>| {
        let mut seen = BTreeSet::new();
        for id in ids {
            if !seen.insert(id) {
                problems.push(format!("duplicate {what} id `{id}`"));
            }
        }
    };
    check_unique("task", content.tasks.iter().map(|t| t.id.as_str()).collect());
    check_unique("event", content.events.iter().map(|t| t.id.as_str()).collect());
    check_unique("archetype", content.archetypes.iter().map(|t| t.id.as_str()).collect());
    check_unique("furniture", content.furniture.iter().map(|t| t.id.as_str()).collect());
    check_unique("program", content.programs.iter().map(|t| t.id.as_str()).collect());
    check_unique("investor", content.investors.iter().map(|t| t.id.as_str()).collect());

    let mut conditions: Vec<(String, &Condition)> = Vec::new();
    let mut effects: Vec<(String, &Effect)> = Vec::new();

    for t in &content.tasks {
        for d in &t.deps {
            if !task_ids.contains(d.as_str()) {
                problems.push(format!("task `{}` depends on unknown task `{d}`", t.id));
            }
        }
        if let Some(u) = &t.requires_unlock
            && !unlocks.contains(u.as_str()) {
                problems.push(format!("task `{}` requires unlock `{u}` that no level grants", t.id));
            }
        effects.extend(t.effects.iter().map(|e| (format!("task {}", t.id), e)));
    }
    for e in &content.events {
        if e.choices.is_empty() {
            problems.push(format!("event `{}` has no choices", e.id));
        }
        conditions.extend(e.conditions.iter().map(|c| (format!("event {}", e.id), c)));
        effects.extend(e.immediate.iter().map(|x| (format!("event {}", e.id), x)));
        for ch in &e.choices {
            conditions.extend(ch.requires.iter().map(|c| (format!("event {}", e.id), c)));
            effects.extend(ch.effects.iter().map(|x| (format!("event {}", e.id), x)));
            for d in &ch.delayed {
                effects.extend(d.effects.iter().map(|x| (format!("event {}", e.id), x)));
            }
        }
    }
    let mut ids: Vec<u32> = content.levels.iter().map(|l| l.id).collect();
    ids.sort();
    if ids != (1..=12).collect::<Vec<u32>>() {
        problems.push(format!("levels must be numbered 1..=12, got {ids:?}"));
    }
    for l in &content.levels {
        if !event_ids.contains(l.boss_event.as_str()) {
            problems.push(format!("level {} boss event `{}` missing", l.id, l.boss_event));
        }
        if !(2..=4).contains(&l.quests.len()) {
            problems.push(format!("level {} must have 2-4 quests", l.id));
        }
        for q in &l.quests {
            conditions.push((format!("level {} quest {}", l.id, q.id), &q.condition));
        }
        if let BossTrigger::When(c) = &l.boss_trigger {
            conditions.push((format!("level {} boss trigger", l.id), c));
        }
        effects.extend(l.start_effects.iter().map(|x| (format!("level {}", l.id), x)));
        if let Some(t) = l.min_office
            && content.office(t).is_none() {
                problems.push(format!("level {} moves to unknown office {t}", l.id));
            }
    }
    for p in &content.programs {
        conditions.extend(p.requires.iter().map(|c| (format!("program {}", p.id), c)));
        effects.extend(p.on_approve.iter().map(|x| (format!("program {}", p.id), x)));
        if let Some(u) = &p.requires_unlock
            && !unlocks.contains(u.as_str()) {
                problems.push(format!("program `{}` requires unlock `{u}` that no level grants", p.id));
            }
    }
    for i in &content.investors {
        conditions.extend(i.requires.iter().map(|c| (format!("investor {}", i.id), c)));
        effects.extend(i.on_accept.iter().map(|x| (format!("investor {}", i.id), x)));
    }
    for e in &content.endings {
        conditions.extend(e.requires.iter().map(|c| (format!("ending {}", e.id), c)));
    }
    for c in &content.channels {
        for a in &c.archetypes {
            if content.archetype(a).is_none() {
                problems.push(format!("channel `{}` lists unknown archetype `{a}`", c.id));
            }
        }
    }
    for o in &content.offices {
        for (_, _, item) in &o.starter {
            if content.furniture(item).is_none() {
                problems.push(format!("office {} starter item `{item}` unknown", o.id));
            }
        }
        if let Some(u) = &o.requires_unlock
            && !unlocks.contains(u.as_str()) {
                problems.push(format!("office {} requires unlock `{u}` that no level grants", o.id));
            }
    }
    for s in &content.segments {
        for f in &s.required_features {
            if !task_ids.contains(f.as_str()) {
                problems.push(format!("segment {:?} requires unknown feature `{f}`", s.segment));
            }
        }
    }
    if content.archetype("founder").is_none() {
        problems.push("archetype `founder` is required".into());
    }

    let mut flat = Vec::new();
    for (ctx, c) in &conditions {
        flat.clear();
        walk_condition(c, &mut flat);
        for c in &flat {
            match c {
                Condition::Shipped(t) | Condition::NotShipped(t) | Condition::TaskActive(t) => {
                    if !task_ids.contains(t.as_str()) {
                        problems.push(format!("{ctx}: unknown task `{t}`"));
                    }
                }
                Condition::Unlocked(u)
                    if !unlocks.contains(u.as_str()) => {
                        problems.push(format!("{ctx}: unknown unlock `{u}`"));
                    }
                _ => {}
            }
        }
    }
    for (ctx, e) in &effects {
        match e {
            Effect::TriggerEvent(id) => {
                if !event_ids.contains(id.as_str()) {
                    problems.push(format!("{ctx}: triggers unknown event `{id}`"));
                }
            }
            Effect::Unlock(u)
                if !unlocks.contains(u.as_str()) => {
                    problems.push(format!("{ctx}: unknown unlock `{u}`"));
                }
            _ => {}
        }
    }
    let random = content.events.iter().filter(|e| !e.scripted).count();
    if random < 40 {
        problems.push(format!("need at least 40 random events, have {random}"));
    }
    problems
}
