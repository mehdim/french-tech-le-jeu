//! # game-content
//!
//! All game data lives in `assets/data/*.ron` and is embedded at compile time,
//! so the browser build needs no extra network requests and tests see exactly
//! what players see. This crate parses it into [`game_core::Content`], checks
//! cross-references, and formats simulation messages into French prose.

pub mod autopilot;
pub mod balancing;
pub mod characters;
pub mod dialogue;
pub mod levels;

use game_core::content::{
    ArchetypeDef, ChannelDef, Content, EndingDef, EventDef, FurnitureDef, InvestorDef, LevelDef, MarketDef,
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

/// Parse every embedded data file and validate cross-references.
pub fn try_load() -> Result<Content, ContentError> {
    let tasks: Vec<TaskDef> = parse("tasks.ron", TASKS)?;
    let emp: EmployeesFile = parse("employees.ron", EMPLOYEES)?;
    let office: OfficeFile = parse("office.ron", OFFICE)?;
    let funding: FundingFile = parse("funding.ron", FUNDING)?;
    let market: MarketFile = parse("market.ron", MARKET)?;
    let events: Vec<EventDef> = parse("events.ron", EVENTS)?;
    let levels: Vec<LevelDef> = parse("levels.ron", LEVELS)?;
    let balancing = balancing::parse(BALANCING)?;
    let content = Content {
        balancing,
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

/// Load content, panicking with a readable message on malformed data.
/// Data is embedded and covered by tests, so this cannot fail at runtime
/// for a build that passed `cargo test`.
pub fn load() -> Content {
    try_load().unwrap_or_else(|e| panic!("invalid game content:\n{e}"))
}
