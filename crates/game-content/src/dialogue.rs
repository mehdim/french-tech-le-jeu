//! Transforme les messages structurés de la simulation en texte français.

use crate::characters::{role_name, segment_name};
use game_core::messages::Msg;

/// « 12 345 € », « 1,25 M€ », « -3 400 € ».
pub fn money(v: f64) -> String {
    let neg = v < 0.0;
    let a = v.abs();
    let body = if a >= 1_000_000.0 {
        format!("{} M€", decimal(a / 1_000_000.0, 2))
    } else if a >= 100_000.0 {
        format!("{} k€", thousands(a / 1000.0))
    } else {
        format!("{} €", thousands(a))
    };
    if neg { format!("-{body}") } else { body }
}

/// Lit un montant saisi par le joueur : « 49 », « 1 250 € », « 2,5 k », « 1,2 M€ ».
/// Accepte la virgule ou le point décimal et ignore espaces et symbole €.
pub fn parse_money(text: &str) -> Option<f64> {
    let t: String = text.chars().filter(|c| !c.is_whitespace() && *c != '€').collect();
    let t = t.to_lowercase().replace(',', ".");
    let (num, mult) = if let Some(n) = t.strip_suffix('k') {
        (n, 1_000.0)
    } else if let Some(n) = t.strip_suffix('m') {
        (n, 1_000_000.0)
    } else {
        (t.as_str(), 1.0)
    };
    let v: f64 = num.parse().ok()?;
    v.is_finite().then_some(v * mult)
}

/// Entier avec séparateurs de milliers à la française (espace insécable).
pub fn thousands(v: f64) -> String {
    let n = v.round() as i64;
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('\u{a0}');
        }
        out.push(ch);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// Décimal avec une virgule à la française, sans zéros superflus.
pub fn decimal(v: f64, places: usize) -> String {
    let s = format!("{v:.places$}");
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    s.replace('.', ",")
}

pub fn pct(v: f64) -> String {
    format!("{} %", decimal(v, 1))
}

pub fn months(v: f64) -> String {
    if !v.is_finite() || v > 99.0 {
        "∞".into()
    } else {
        format!("{} mois", decimal(v, 1))
    }
}

pub fn format(msg: &Msg) -> String {
    match msg {
        Msg::Text(t) => t.clone(),
        Msg::MentorChosen { mentor } => format!("{mentor} devient votre mentor principal."),
        Msg::MentorAdvice { mentor, text } => format!("{mentor} : « {text} »"),
        Msg::Hired { name, role } => format!("{name} rejoint l'équipe ({}). Bienvenue ! 🎉", role_name(*role)),
        Msg::Resigned { name } => format!("{name} a démissionné. Le pot de départ est prévu jeudi."),
        Msg::Fired { name, severance } => {
            format!("{name} quitte l'entreprise. Indemnités : {}. L'ambiance est lourde.", money(*severance))
        }
        Msg::ContractEnded { name, consultant } => {
            if *consultant {
                format!("{name} (ESN) termine sa mission et repart avec tout le contexte du projet.")
            } else {
                format!("Fin de contrat pour {name}. Merci pour tout !")
            }
        }
        Msg::BurnOut { name, days } => format!("{name} est en arrêt pour épuisement ({days} jours). Prenez soin de l'équipe."),
        Msg::TaskStarted { task } => format!("Chantier lancé : {task}."),
        Msg::TaskShipped { task } => format!("Livré en production : {task} ✅"),
        Msg::TaskComplication { task, debt } => {
            format!("Complication sur « {task} » : dette technique +{}.", decimal(*debt, 0))
        }
        Msg::TaskCancelled { task, refund } => {
            if *refund > 0.0 {
                format!("Chantier annulé : {task} (remboursé {}).", money(*refund))
            } else {
                format!("Chantier annulé : {task}.")
            }
        }
        Msg::Incident { cost } => format!("🔥 Incident de production ! Crédits SLA et nuit blanche : {}.", money(*cost)),
        Msg::CustomersWeek { new, churned } => format!("Semaine : +{new} clients, -{churned} départs."),
        Msg::FirstCustomer { segment } => format!("Premier client payant dans le segment « {} » !", segment_name(*segment)),
        Msg::MonthClosed { month, net_burn, mrr } => {
            if *net_burn > 0.0 {
                format!("Clôture du mois {month} : MRR {}, burn net {}.", money(*mrr), money(*net_burn))
            } else {
                format!("Clôture du mois {month} : MRR {}, bénéfice {} !", money(*mrr), money(-net_burn))
            }
        }
        Msg::CandidatesArrived { channel, count } => format!("{count} candidature(s) via {channel}."),
        Msg::NoCandidates { channel } => format!("Aucun profil pertinent via {channel}."),
        Msg::ApplicationStarted { program } => format!("Dossier ouvert : {program}. Affectez quelqu'un à l'administratif."),
        Msg::ApplicationSubmitted { program } => format!("Dossier déposé : {program}. Réponse dans quelques semaines."),
        Msg::ApplicationApproved { program, amount } => {
            if *amount > 0.0 {
                format!("Accordé : {program} ({}).", money(*amount))
            } else {
                format!("Accordé : {program}.")
            }
        }
        Msg::ApplicationRejected { program } => format!("Refusé : {program}. Vous pourrez retenter plus tard."),
        Msg::MoneyIn { label, amount } => format!("💶 Versement reçu : {} ({label}).", money(*amount)),
        Msg::LoanRepaid { label } => format!("Prêt intégralement remboursé : {label}."),
        Msg::PitchStarted { round } => format!("Levée lancée ({round}). Affectez-vous à « Levée de fonds »."),
        Msg::OffersReceived { count } => format!("{count} offre(s) d'investissement sur la table !"),
        Msg::NoOffers => "Aucun investisseur n'a mordu. Revenez avec de meilleures métriques.".into(),
        Msg::RoundClosed { investor, amount, dilution } => {
            format!("Tour bouclé avec {investor} : {} pour {} du capital.", money(*amount), pct(*dilution))
        }
        Msg::OffersExpired => "Les offres ont expiré. Les investisseurs ont la mémoire courte.".into(),
        Msg::LowRunway { months: m } => format!("⚠️ Runway critique : {}.", months(*m)),
        Msg::NegativeCash { days_left } => {
            format!("🚨 Trésorerie négative ! Redressement judiciaire dans {days_left} jours si rien ne change.")
        }
        Msg::QuestDone { label } => format!("Objectif atteint : {label}"),
        Msg::LevelComplete { level } => format!("Niveau {level} terminé !"),
        Msg::OfficeMoved { name } => format!("Déménagement : bienvenue à « {name} »."),
        Msg::FurniturePlaced { name } => format!("Installé : {name}."),
        Msg::MarketOpened { name } => format!("Nouveau marché ouvert : {name} !"),
        Msg::PriceChanged { segment, price } => format!("Nouveau prix « {} » : {}/mois.", segment_name(*segment), money(*price)),
        Msg::IncomeEnded { label } => format!("Fin du versement : {label}."),
        Msg::Launched => "Veridix est officiellement lancé. Les plans payants sont en ligne !".into(),
        Msg::DriftWarning { accuracy } => {
            format!("La précision dérive ({}). Un réentraînement serait bienvenu.", pct(*accuracy))
        }
    }
}

use game_core::types::{Condition, Effect, ModKind, Segment};

fn signed(v: f64, unit: &str) -> String {
    let s = decimal(v.abs(), 1);
    if v >= 0.0 { format!("+{s}{unit}") } else { format!("-{s}{unit}") }
}

fn seg(s: &Segment) -> &'static str {
    segment_name(*s)
}

/// Court résumé en français d'un effet, et s'il est bon pour le joueur.
/// Renvoie None pour les effets purement internes (indicateurs, enchaînements).
pub fn effect_text(e: &Effect) -> Option<(String, bool)> {
    Some(match e {
        Effect::Cash(v) => (format!("Trésorerie {}", if *v >= 0.0 { format!("+{}", money(*v)) } else { money(*v) }), *v >= 0.0),
        Effect::Reputation(v) => (format!("Réputation {}", signed(*v, "")), *v >= 0.0),
        Effect::MoraleAll(v) => (format!("Moral {}", signed(*v, "")), *v >= 0.0),
        Effect::FatigueAll(v) => (format!("Fatigue {}", signed(*v, "")), *v <= 0.0),
        Effect::MoraleRole(r, v) => (format!("Moral {} {}", role_name(*r), signed(*v, "")), *v >= 0.0),
        Effect::TechDebt(v) => (format!("Dette technique {}", signed(*v, "")), *v <= 0.0),
        Effect::Accuracy(v) => (format!("Précision {}", signed(*v, " pt")), *v >= 0.0),
        Effect::Fpr(v) => (format!("Faux positifs {}", signed(*v, " pt")), *v <= 0.0),
        Effect::Drift(v) => (format!("Dérive du benchmark {}", signed(*v, " pt")), *v <= 0.0),
        Effect::DriftMul(f) => (format!("Dérive -{} %", decimal((1.0 - f) * 100.0, 0)), *f <= 1.0),
        Effect::Robustness(v) => (format!("Robustesse {}", signed(v * 100.0, " %")), *v >= 0.0),
        Effect::FeatureScore(v) => (format!("Fonctionnalités {}", signed(*v, "")), *v >= 0.0),
        Effect::Leads(s, v) => (format!("{} leads {}", signed(*v, ""), seg(s)), *v >= 0.0),
        Effect::CustomersPct(s, f) => (format!("Clients {} {}", seg(s), signed(f * 100.0, " %")), *f >= 0.0),
        Effect::AddCustomers(s, n) => (format!("+{n} client(s) {}", seg(s)), true),
        Effect::CompetitorPressure(v) => (format!("Pression concurrentielle {}", signed(v * 100.0, "")), *v <= 0.0),
        Effect::InferenceCostMul(f) => (format!("Coût d'inférence {}", signed((f - 1.0) * 100.0, " %")), *f <= 1.0),
        Effect::HostingMonthly(v) => (format!("Hébergement {}/mois", if *v >= 0.0 { format!("+{}", money(*v)) } else { money(*v) }), *v <= 0.0),
        Effect::UsageMul(f) => (format!("Usage client {}", signed((f - 1.0) * 100.0, " %")), *f <= 1.0),
        Effect::Retention(s, v) => (format!("Rétention {} {}", seg(s), signed(v * 100.0, " %")), *v >= 0.0),
        Effect::ConversionBonus(s, v) => (format!("Conversion {} {}", seg(s), signed(v * 100.0, " %")), *v >= 0.0),
        Effect::Languages(n) => (format!("+{n} langue(s)"), true),
        Effect::AdminPoints(v) => (format!("Paperasse {}", signed(*v, " pts")), *v <= 0.0),
        Effect::DelayDisbursements(d) => (format!("Versements retardés de {d} j"), false),
        Effect::Modifier { kind, value, days } => {
            let (label, good) = match kind {
                ModKind::LeadMul => (format!("Leads ×{}", decimal(*value, 2)), *value >= 1.0),
                ModKind::ConversionMul => (format!("Conversion ×{}", decimal(*value, 2)), *value >= 1.0),
                ModKind::ChurnAdd => (format!("Churn {}", signed(*value, " pt")), *value <= 0.0),
                ModKind::ProductivityMul => (format!("Productivité ×{}", decimal(*value, 2)), *value >= 1.0),
                ModKind::MoraleTarget => (format!("Moral {}", signed(*value, "")), *value >= 0.0),
                ModKind::InfraCostMul => (format!("Coûts d'infra ×{}", decimal(*value, 2)), *value <= 1.0),
                ModKind::SalesMul => (format!("Ventes ×{}", decimal(*value, 2)), *value >= 1.0),
                ModKind::ResearchMul => (format!("Recherche ×{}", decimal(*value, 2)), *value >= 1.0),
                ModKind::IncidentRiskMul => (format!("Risque d'incident ×{}", decimal(*value, 2)), *value <= 1.0),
                ModKind::CompetitionAdd => (format!("Concurrence {}", signed(value * 100.0, "")), *value <= 0.0),
            };
            (format!("{label} ({days} j)"), good)
        }
        Effect::Incident => ("Incident de production".into(), false),
        Effect::QuitRole(r) => (format!("Départ : {}", role_name(*r)), false),
        Effect::TaskSetback(f) => (format!("Avancement des chantiers -{} %", decimal(f * 100.0, 0)), false),
        Effect::SalaryPct(f) => (format!("Salaires {}", signed(f * 100.0, " %")), *f <= 0.0),
        Effect::FounderProductivity(f) => (format!("Productivité fondateur {}", signed((f - 1.0) * 100.0, " %")), *f >= 1.0),
        Effect::Income { monthly, months, label } => (format!("{label} : {}/mois pendant {months} mois", money(*monthly)), true),
        Effect::Resilience(v) => (format!("Incidents -{} %", decimal(v * 100.0, 0)), true),
        Effect::RentMonthly(v) => (format!("Loyer +{}/mois", money(*v)), *v <= 0.0),
        Effect::PriceAll(f) => (format!("Prix {}", signed(f * 100.0, " %")), *f >= 0.0),
        Effect::Launch => ("Lancement commercial".into(), true),
        Effect::SetFlag(_) | Effect::ClearFlag(_) | Effect::TriggerEvent(_) | Effect::Unlock(_) => return None,
        Effect::ChooseMentor(_) => ("Mentor principal".into(), true),
    })
}

/// Texte lisible d'une condition requise.
pub fn condition_text(c: &Condition) -> String {
    match c {
        Condition::Always => "—".into(),
        Condition::CashAtLeast(v) => format!("Trésorerie ≥ {}", money(*v)),
        Condition::CashBelow(v) => format!("Trésorerie < {}", money(*v)),
        Condition::MrrAtLeast(v) => format!("MRR ≥ {}", money(*v)),
        Condition::CustomersAtLeast(n) => format!("{n} clients"),
        Condition::PayingCustomersAtLeast(n) => format!("{n} clients payants"),
        Condition::SegmentCustomersAtLeast(s, n) => format!("{n} clients {}", seg(s)),
        Condition::EmployeesAtLeast(n) => format!("{n} personnes dans l'équipe"),
        Condition::EmployeesBelow(n) => format!("moins de {n} personnes"),
        Condition::RoleAtLeast(r, n) => format!("{n} × {}", role_name(*r)),
        Condition::HiredInLevelAtLeast(n) => format!("{n} recrutement(s) ce niveau"),
        Condition::Shipped(t) => format!("fonctionnalité « {t} » livrée"),
        Condition::NotShipped(t) => format!("« {t} » non livrée"),
        Condition::ShippedInLevelAtLeast(n) => format!("{n} améliorations livrées ce niveau"),
        Condition::TaskActive(t) => format!("chantier « {t} » en cours"),
        Condition::Flag(f) => format!("condition « {f} »"),
        Condition::NotFlag(f) => format!("pas « {f} »"),
        Condition::Unlocked(u) => format!("débloqué : {u}"),
        Condition::LevelAtLeast(l) => format!("niveau ≥ {l}"),
        Condition::LevelAtMost(l) => format!("niveau ≤ {l}"),
        Condition::QualityAtLeast(v) => format!("Qualité ≥ {}", decimal(*v, 0)),
        Condition::QualityBelow(v) => format!("Qualité < {}", decimal(*v, 0)),
        Condition::AccuracyAtLeast(v) => format!("Précision ≥ {}", pct(*v)),
        Condition::AccuracyBelow(v) => format!("Précision < {}", pct(*v)),
        Condition::FprAtMost(v) => format!("Faux positifs ≤ {}", pct(*v)),
        Condition::FprAbove(v) => format!("Faux positifs > {}", pct(*v)),
        Condition::TechDebtAtLeast(v) => format!("Dette ≥ {}", decimal(*v, 0)),
        Condition::TechDebtBelow(v) => format!("Dette < {}", decimal(*v, 0)),
        Condition::MoraleAtLeast(v) => format!("Moral ≥ {}", decimal(*v, 0)),
        Condition::MoraleBelow(v) => format!("Moral < {}", decimal(*v, 0)),
        Condition::FatigueAtLeast(v) => format!("Fatigue ≥ {}", decimal(*v, 0)),
        Condition::ReputationAtLeast(v) => format!("Réputation ≥ {}", decimal(*v, 0)),
        Condition::ReputationBelow(v) => format!("Réputation < {}", decimal(*v, 0)),
        Condition::ChurnBelow(v) => format!("Churn < {}", pct(*v)),
        Condition::ChurnAtLeast(v) => format!("Churn ≥ {}", pct(*v)),
        Condition::SegmentChurnBelow(s, v) => format!("Churn {} < {}", seg(s), pct(*v)),
        Condition::RunwayAtLeast(v) => format!("Runway ≥ {}", months(*v)),
        Condition::RunwayBelow(v) => format!("Runway < {}", months(*v)),
        Condition::GrossMarginAtLeast(v) => format!("Marge brute ≥ {}", pct(*v)),
        Condition::FundingSecuredInLevelAtLeast(v) => format!("{} sécurisés ce niveau", money(*v)),
        Condition::ApplicationSubmittedInLevel => "un dossier déposé ce niveau".into(),
        Condition::FounderEquityAtLeast(v) => format!("fondateur ≥ {} du capital", pct(*v)),
        Condition::RoundClosedInLevel { min_amount, max_dilution } => {
            format!("tour ≥ {} avec ≤ {} de dilution", money(*min_amount), pct(*max_dilution))
        }
        Condition::EquityRaisedInLevelAtLeast(v) => format!("{} levés ce niveau", money(*v)),
        Condition::MrrGrowthStreak { months: m, min_pct } => format!("croissance moyenne ≥ {} sur {m} mois", pct(*min_pct)),
        Condition::InferenceCostReducedBy(f) => format!("coût d'inférence -{}", pct(f * 100.0)),
        Condition::CustomersRetainedPct(p) => format!("garder {} des clients", pct(*p)),
        Condition::MarketsAtLeast(n) => format!("{n} marchés"),
        Condition::LanguagesAtLeast(n) => format!("{n} langues"),
        Condition::LtvCacAtLeast(v) => format!("LTV/CAC ≥ {}", decimal(*v, 1)),
        Condition::NetBurnAtMost(v) => if *v <= 0.0 { "être rentable".into() } else { format!("burn ≤ {}", money(*v)) },
        Condition::ValuationAtLeast(v) => format!("valorisation ≥ {}", money(*v)),
        Condition::OfficeTierAtLeast(t) => format!("bureau niveau {t}"),
        Condition::DayInLevelAtLeast(d) => format!("{d} jours dans le niveau"),
        Condition::LeadsAtLeast(v) => format!("{} leads", decimal(*v, 0)),
        Condition::Launched => "produit lancé".into(),
        Condition::All(v) => v.iter().map(condition_text).collect::<Vec<_>>().join(" et "),
        Condition::Any(v) => v.iter().map(condition_text).collect::<Vec<_>>().join(" ou "),
        Condition::Not(c) => format!("pas ({})", condition_text(c)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_amounts_are_read_back() {
        assert_eq!(parse_money("49"), Some(49.0));
        assert_eq!(parse_money("1\u{a0}250 €"), Some(1250.0));
        assert_eq!(parse_money("2,5 k"), Some(2500.0));
        assert_eq!(parse_money("1,25 M€"), Some(1_250_000.0));
        assert_eq!(parse_money("abc"), None);
        // Ce que l'interface affiche se relit à l'identique.
        for v in [49.0, 1250.0, 99_000.0] {
            assert_eq!(parse_money(&money(v)), Some(v));
        }
    }
}
