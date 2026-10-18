//! # game-content
//!
//! Toutes les données du jeu vivent dans `assets/data/*.ron` et sont intégrées à la compilation :
//! le build navigateur n'a besoin d'aucune requête réseau supplémentaire et les tests voient
//! exactement ce que voient les joueurs. Cette crate les convertit en [`game_core::Content`],
//! vérifie les références croisées et met en forme les messages de la simulation en français.

pub mod autopilot;
pub mod balancing;
pub mod characters;
pub mod dialogue;
pub mod hero;
pub mod levels;

use game_core::content::{
    MentorDef, ArchetypeDef, ChannelDef, Content, EndingDef, EventDef, FurnitureDef, InvestorDef, LevelDef, MarketDef,
    NamePools, OfficeTierDef, ProgramDef, RoundDef, SegmentDef, TaskDef,
};
use serde::Deserialize;

const TASKS: &str = include_str!("../../../assets/data/tasks.ron");
const EMPLOYEES: &str = include_str!("../../../assets/data/employees.ron");
const OFFICE: &str = include_str!("../../../assets/data/office.ron");
const FUNDING: &str = include_str!("../../../assets/data/funding.ron");
const MARKET: &str = include_str!("../../../assets/data/market.ron");
const EVENTS: &str = include_str!("../../../assets/data/events.ron");
const LEVELS: &str = include_str!("../../../assets/data/levels.ron");
const BALANCING: &str = include_str!("../../../assets/data/balancing.ron");
const MENTORS: &str = include_str!("../../../assets/data/mentors.ron");

#[derive(Deserialize)]
struct EmployeesFile {
    archetypes: Vec<ArchetypeDef>,
    channels: Vec<ChannelDef>,
    names: NamePools,
}

#[derive(Deserialize)]
struct OfficeFile {
    tiers: Vec<OfficeTierDef>,
    furniture: Vec<FurnitureDef>,
}

#[derive(Deserialize)]
struct FundingFile {
    programs: Vec<ProgramDef>,
    rounds: Vec<RoundDef>,
    investors: Vec<InvestorDef>,
}

#[derive(Deserialize)]
struct MarketFile {
    segments: Vec<SegmentDef>,
    markets: Vec<MarketDef>,
    endings: Vec<EndingDef>,
}

#[derive(Debug)]
pub struct ContentError(pub String);

impl std::fmt::Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ContentError {}

fn parse<T: for<'de> Deserialize<'de>>(name: &str, src: &str) -> Result<T, ContentError> {
    ron::from_str(src).map_err(|e| ContentError(format!("{name}: {e}")))
}

/// Analyse chaque fichier de données intégré et vérifie les références croisées.
pub fn try_load() -> Result<Content, ContentError> {
    let tasks: Vec<TaskDef> = parse("tasks.ron", TASKS)?;
    let emp: EmployeesFile = parse("employees.ron", EMPLOYEES)?;
    let office: OfficeFile = parse("office.ron", OFFICE)?;
    let funding: FundingFile = parse("funding.ron", FUNDING)?;
    let market: MarketFile = parse("market.ron", MARKET)?;
    let events: Vec<EventDef> = parse("events.ron", EVENTS)?;
    let levels: Vec<LevelDef> = parse("levels.ron", LEVELS)?;
    let balancing = balancing::parse(BALANCING)?;
    let mentors: Vec<MentorDef> = parse("mentors.ron", MENTORS)?;
    let content = Content {
        balancing,
        mentors,
        tasks,
        archetypes: emp.archetypes,
        channels: emp.channels,
        offices: office.tiers,
        furniture: office.furniture,
        programs: funding.programs,
        investors: funding.investors,
        rounds: funding.rounds,
        events,
        levels,
        markets: market.markets,
        endings: market.endings,
        segments: market.segments,
        names: emp.names,
    };
    let problems = levels::validate(&content);
    if !problems.is_empty() {
        return Err(ContentError(problems.join("\n")));
    }
    Ok(content)
}

/// Charge le contenu, en paniquant avec un message lisible si les données sont mal formées.
/// Les données sont intégrées et couvertes par les tests : cela ne peut pas échouer à
/// l'exécution pour un build qui a passé `cargo test`.
pub fn load() -> Content {
    try_load().unwrap_or_else(|e| panic!("contenu de jeu invalide :\n{e}"))
}
