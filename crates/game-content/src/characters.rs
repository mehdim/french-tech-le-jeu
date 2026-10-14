//! Noms affichés des rôles, des traits et des personnages récurrents, plus la
//! clé de portrait que le client utilise pour chaque personnage qui parle.

use game_core::content::Trait;
use game_core::types::{Role, Segment};

pub fn role_name(role: Role) -> &'static str {
    match role {
        Role::Founder => "CTO fondateur",
        Role::Backend => "Dev backend",
        Role::Frontend => "Dev frontend",
        Role::AiResearcher => "Chercheur·se IA",
        Role::DevOps => "DevOps / SRE",
        Role::Sales => "Commercial·e",
        Role::Growth => "Growth",
        Role::ProductManager => "Product manager",
        Role::Hr => "RH",
        Role::Cfo => "CFO",
        Role::Intern => "Stagiaire",
        Role::Consultant => "Consultant·e ESN",
    }
}

pub fn trait_name(t: Trait) -> &'static str {
    match t {
        Trait::Standard => "Profil classique",
        Trait::Polytechnicien => "Polytechnicien·ne",
        Trait::Dev42 => "Ancien·ne de 42",
        Trait::RustEvangelist => "Évangéliste Rust",
        Trait::ExGafam => "Ex-GAFAM",
        Trait::EsnConsultant => "Consultant·e ESN",
        Trait::GrowthHacker => "Growth hacker",
        Trait::PhdResearcher => "Docteur·e",
        Trait::JiraMaster => "Maître·sse de Jira",
        Trait::Stagiaire => "Stagiaire",
        Trait::Founder => "Fondateur·rice",
    }
}

pub fn trait_effect(t: Trait) -> &'static str {
    match t {
        Trait::Standard => "Aucun effet particulier.",
        Trait::Polytechnicien => "Recherche ×1,4, ingénierie ×1,1. Démontre parfois que le business model est impossible.",
        Trait::Dev42 => "Ingénierie ×1,3 — tombe à ×0,8 s'il y a plus d'un PM pour 4 ingénieurs.",
        Trait::RustEvangelist => "Dette technique et incidents -15 % pour toute l'équipe. Veut tout réécrire en Rust.",
        Trait::ExGafam => "Ingénierie ×1,5 si au moins un poste pro pour deux ingénieurs, sinon ×1,1 et moral en baisse.",
        Trait::EsnConsultant => "Facturé au TJM. Part au bout de 3 mois avec le contexte (avancement -10 %, dette +3).",
        Trait::GrowthHacker => "Marketing ×1,7, mais la conversion globale baisse de 7 % (leads de moindre qualité).",
        Trait::PhdResearcher => "Recherche ×1,5. Moral en chute s'il est affecté aux ventes ou au marketing.",
        Trait::JiraMaster => "En Management : +7 % sur les tâches par PM utile (1 pour 5 ingénieurs), -4 % par PM en trop.",
        Trait::Stagiaire => "Bon marché, progresse chaque jour de travail. Contrat de 6 mois.",
        Trait::Founder => "Ne démissionne jamais. Moral +8.",
    }
}

pub fn segment_name(s: Segment) -> &'static str {
    match s {
        Segment::Smb => "PME & écoles",
        Segment::Mid => "Médias & universités",
        Segment::Enterprise => "Grands comptes",
    }
}

/// Clé du sprite de portrait pour un nom de personnage utilisé dans les dialogues.
pub fn portrait_for(speaker: &str) -> &'static str {
    let s = speaker.to_lowercase();
    if s.contains("vous") {
        "founder"
    } else if s.contains("france travail") || s.contains("conseill") {
        "advisor"
    } else if s.contains("bpi") || s.contains("nathalie") {
        "bpi"
    } else if s.contains("jean-michel") {
        "jm"
    } else if s.contains("patrick") {
        "patrick"
    } else if s.contains("bernard") {
        "bernard"
    } else if s.contains("sophie") {
        "sophie"
    } else if s.contains("narrat") {
        "narrator"
    } else if s.contains("chercheu") {
        "researcher"
    } else if s.contains("cfo") || s.contains("financ") {
        "cfo"
    } else if s.contains("ministre") || s.contains("commissaire") || s.contains("administration") || s.contains("urssaf") {
        "official"
    } else if s.contains("fonds") || s.contains("associé") || s.contains("board") || s.contains("vc") || s.contains("sand hill") {
        "vc"
    } else {
        "generic"
    }
}
