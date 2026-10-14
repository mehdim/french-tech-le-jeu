//! Validation du contenu : chaque identifiant référencé doit exister, pour qu'une
//! faute de frappe dans un fichier RON fasse échouer `cargo test` au lieu de casser un niveau en silence.

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

/// Renvoie la liste des problèmes, lisibles par un humain (vide = valide).
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
                problems.push(format!("identifiant en double ({what}) : `{id}`"));
            }
        }
    };
    check_unique("tâche", content.tasks.iter().map(|t| t.id.as_str()).collect());
    check_unique("événement", content.events.iter().map(|t| t.id.as_str()).collect());
    check_unique("profil", content.archetypes.iter().map(|t| t.id.as_str()).collect());
    check_unique("meuble", content.furniture.iter().map(|t| t.id.as_str()).collect());
    check_unique("dispositif", content.programs.iter().map(|t| t.id.as_str()).collect());
    check_unique("investisseur", content.investors.iter().map(|t| t.id.as_str()).collect());

    let mut conditions: Vec<(String, &Condition)> = Vec::new();
    let mut effects: Vec<(String, &Effect)> = Vec::new();

    for t in &content.tasks {
        for d in &t.deps {
            if !task_ids.contains(d.as_str()) {
                problems.push(format!("la tâche `{}` dépend d'une tâche inconnue `{d}`", t.id));
            }
        }
        if let Some(u) = &t.requires_unlock
            && !unlocks.contains(u.as_str()) {
                problems.push(format!("la tâche `{}` exige le déblocage `{u}`, qu'aucun niveau n'accorde", t.id));
            }
        effects.extend(t.effects.iter().map(|e| (format!("tâche {}", t.id), e)));
    }
    for e in &content.events {
        if e.choices.is_empty() {
            problems.push(format!("l'événement `{}` n'a aucun choix", e.id));
        }
        conditions.extend(e.conditions.iter().map(|c| (format!("événement {}", e.id), c)));
        effects.extend(e.immediate.iter().map(|x| (format!("événement {}", e.id), x)));
        for ch in &e.choices {
            conditions.extend(ch.requires.iter().map(|c| (format!("événement {}", e.id), c)));
            effects.extend(ch.effects.iter().map(|x| (format!("événement {}", e.id), x)));
            for d in &ch.delayed {
                effects.extend(d.effects.iter().map(|x| (format!("événement {}", e.id), x)));
            }
        }
    }
    let mut ids: Vec<u32> = content.levels.iter().map(|l| l.id).collect();
    ids.sort();
    if ids != (1..=12).collect::<Vec<u32>>() {
        problems.push(format!("les niveaux doivent être numérotés de 1 à 12, trouvé {ids:?}"));
    }
    for l in &content.levels {
        if !event_ids.contains(l.boss_event.as_str()) {
            problems.push(format!("niveau {} : événement majeur `{}` introuvable", l.id, l.boss_event));
        }
        if !(2..=4).contains(&l.quests.len()) {
            problems.push(format!("le niveau {} doit avoir de 2 à 4 objectifs", l.id));
        }
        for q in &l.quests {
            conditions.push((format!("niveau {} objectif {}", l.id, q.id), &q.condition));
        }
        if let BossTrigger::When(c) = &l.boss_trigger {
            conditions.push((format!("niveau {} déclencheur de l'événement majeur", l.id), c));
        }
        effects.extend(l.start_effects.iter().map(|x| (format!("niveau {}", l.id), x)));
        if let Some(t) = l.min_office
            && content.office(t).is_none() {
                problems.push(format!("le niveau {} déménage vers un bureau inconnu {t}", l.id));
            }
    }
    for p in &content.programs {
        conditions.extend(p.requires.iter().map(|c| (format!("dispositif {}", p.id), c)));
        effects.extend(p.on_approve.iter().map(|x| (format!("dispositif {}", p.id), x)));
        if let Some(u) = &p.requires_unlock
            && !unlocks.contains(u.as_str()) {
                problems.push(format!("le dispositif `{}` exige le déblocage `{u}`, qu'aucun niveau n'accorde", p.id));
            }
    }
    for i in &content.investors {
        conditions.extend(i.requires.iter().map(|c| (format!("investisseur {}", i.id), c)));
        effects.extend(i.on_accept.iter().map(|x| (format!("investisseur {}", i.id), x)));
    }
    for e in &content.endings {
        conditions.extend(e.requires.iter().map(|c| (format!("fin {}", e.id), c)));
    }
    for c in &content.channels {
        for a in &c.archetypes {
            if content.archetype(a).is_none() {
                problems.push(format!("le canal `{}` cite un profil inconnu `{a}`", c.id));
            }
        }
    }
    for o in &content.offices {
        for (_, _, item) in &o.starter {
            if content.furniture(item).is_none() {
                problems.push(format!("bureau {} : meuble de départ inconnu `{item}`", o.id));
            }
        }
        if let Some(u) = &o.requires_unlock
            && !unlocks.contains(u.as_str()) {
                problems.push(format!("le bureau {} exige le déblocage `{u}`, qu'aucun niveau n'accorde", o.id));
            }
    }
    for s in &content.segments {
        for f in &s.required_features {
            if !task_ids.contains(f.as_str()) {
                problems.push(format!("le segment {:?} exige une fonctionnalité inconnue `{f}`", s.segment));
            }
        }
    }
    if content.archetype("founder").is_none() {
        problems.push("le profil `founder` est obligatoire".into());
    }

    let mut flat = Vec::new();
    for (ctx, c) in &conditions {
        flat.clear();
        walk_condition(c, &mut flat);
        for c in &flat {
            match c {
                Condition::Shipped(t) | Condition::NotShipped(t) | Condition::TaskActive(t) => {
                    if !task_ids.contains(t.as_str()) {
                        problems.push(format!("{ctx} : tâche inconnue `{t}`"));
                    }
                }
                Condition::Unlocked(u)
                    if !unlocks.contains(u.as_str()) => {
                        problems.push(format!("{ctx} : déblocage inconnu `{u}`"));
                    }
                _ => {}
            }
        }
    }
    for (ctx, e) in &effects {
        match e {
            Effect::TriggerEvent(id) => {
                if !event_ids.contains(id.as_str()) {
                    problems.push(format!("{ctx} : déclenche un événement inconnu `{id}`"));
                }
            }
            Effect::Unlock(u)
                if !unlocks.contains(u.as_str()) => {
                    problems.push(format!("{ctx} : déblocage inconnu `{u}`"));
                }
            _ => {}
        }
    }
    let random = content.events.iter().filter(|e| !e.scripted).count();
    if random < 40 {
        problems.push(format!("il faut au moins 40 événements aléatoires, il y en a {random}"));
    }
    problems
}
