//! Paramètres numériques d'équilibrage. Les valeurs par défaut vivent ici pour que la
//! simulation fonctionne sans contenu ; `game-content` les remplace depuis `balancing.ron`.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Balancing {
    // --- conditions de départ --------------------------------------------
    pub start_cash: f64,
    pub start_reputation: f64,
    pub start_accuracy: f64,
    pub start_fpr: f64,
    pub start_tech_debt: f64,

    // --- masse salariale (ordres de grandeur français, ajustés pour le jeu) -----
    /// Cotisations patronales, en fraction du salaire brut.
    pub employer_contrib: f64,
    /// Taux de cotisations réduit pour le personnel R&D sous statut JEI.
    pub jei_contrib: f64,
    /// Outils SaaS par personne et par mois (Slack, Notion, Jira, Figma...).
    pub tools_per_employee: f64,
    /// Indemnités de licenciement en mois de salaire par année d'ancienneté (+ un mois de préavis).
    pub severance_months_per_year: f64,

    // --- infrastructure ---------------------------------------------------
    pub hosting_base: f64,
    pub hosting_per_customer: f64,
    /// Coût d'inférence en euros pour 1 000 analyses au début de la partie.
    pub inference_cost_per_k: f64,
    pub incident_base: f64,
    pub incident_per_debt: f64,
    pub incident_per_kcustomer: f64,

    // --- recherche en IA ---------------------------------------------------
    pub accuracy_ceiling: f64,
    pub drift_per_day: f64,
    pub drift_growth_per_level: f64,

    // --- travail ----------------------------------------------------------
    /// Points produits par point de compétence et par jour (compétence 60 -> 6 points).
    pub output_per_skill: f64,
    pub fatigue_work: f64,
    pub fatigue_weekend_recovery: f64,
    pub fatigue_crunch: f64,
    pub morale_base: f64,
    pub morale_drift: f64,
    pub onboarding_days: u32,
    pub tech_debt_drag: f64,
    pub tech_debt_daily_per_engineer: f64,

    // --- marché -----------------------------------------------------------
    pub organic_leads: f64,
    pub self_serve_rate: f64,
    pub lead_decay: f64,
    pub churn_quality_slope: f64,
    pub churn_fpr_slope: f64,
    pub competition_base: f64,
    pub competition_per_level: f64,

    // --- événements --------------------------------------------------------
    pub event_chance_per_day: f64,
    pub event_min_gap_days: u32,

    // --- échec ------------------------------------------------------------
    pub bankruptcy_grace_days: u32,
    pub bankruptcy_floor: f64,
    pub resign_morale_threshold: f64,

    // --- valorisation ------------------------------------------------------
    pub valuation_arr_multiple: f64,
    pub valuation_floor: f64,
    pub valuation_per_engineer: f64,
    /// Croissance quotidienne de l'usage par client après le lancement (adoption plus profonde).
    pub usage_creep_per_day: f64,
}

impl Default for Balancing {
    fn default() -> Self {
        Self {
            start_cash: 30_000.0,
            start_reputation: 10.0,
            start_accuracy: 53.0,
            start_fpr: 14.0,
            start_tech_debt: 5.0,
            employer_contrib: 0.45,
            jei_contrib: 0.20,
            tools_per_employee: 60.0,
            severance_months_per_year: 0.25,
            hosting_base: 120.0,
            hosting_per_customer: 1.2,
            inference_cost_per_k: 4.0,
            incident_base: 0.002,
            incident_per_debt: 0.00012,
            incident_per_kcustomer: 0.002,
            accuracy_ceiling: 97.0,
            drift_per_day: 0.025,
            drift_growth_per_level: 0.12,
            output_per_skill: 0.1,
            fatigue_work: 1.6,
            fatigue_weekend_recovery: 9.0,
            fatigue_crunch: 2.6,
            morale_base: 58.0,
            morale_drift: 0.06,
            onboarding_days: 14,
            tech_debt_drag: 0.006,
            tech_debt_daily_per_engineer: 0.012,
            organic_leads: 0.5,
            self_serve_rate: 0.025,
            lead_decay: 0.04,
            churn_quality_slope: 0.14,
            churn_fpr_slope: 0.30,
            competition_base: 0.08,
            competition_per_level: 0.03,
            event_chance_per_day: 0.10,
            event_min_gap_days: 5,
            bankruptcy_grace_days: 14,
            bankruptcy_floor: -50_000.0,
            resign_morale_threshold: 25.0,
            valuation_arr_multiple: 6.0,
            valuation_floor: 350_000.0,
            valuation_per_engineer: 60_000.0,
            usage_creep_per_day: 0.0007,
        }
    }
}
