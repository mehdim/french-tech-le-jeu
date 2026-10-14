//! Veridix, le produit : qualité de détection, dérive, dette technique, coût d'inférence.
//!
//! La détection universelle de texte généré par IA n'est *pas* un problème soluble ici : les
//! gains de précision s'amenuisent près d'un plafond, les benchmarks dérivent à chaque nouveau
//! modèle de fondation, et les faux positifs sur des textes humains inquiètent les clients.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductState {
    /// Identifiants des chantiers livrés au moins une fois.
    pub shipped: BTreeSet<String>,
    /// Étendue et finition du produit, 0..100.
    pub feature_score: f64,
    /// Précision du modèle sur le benchmark de référence avant dérive, en %.
    pub model_accuracy: f64,
    /// Dégradation du benchmark accumulée depuis le dernier réentraînement.
    pub drift: f64,
    /// Robustesse à la dérive et aux paraphrases adverses (0..0,8).
    pub robustness: f64,
    /// Taux de base de faux positifs sur des textes humains, en %.
    pub fpr: f64,
    pub tech_debt: f64,
    /// € pour 1 000 analyses (avant modificateurs temporaires).
    pub inference_cost_per_k: f64,
    /// Multiplicateur de l'usage par client.
    pub usage_mult: f64,
    /// Langues prises en charge (1 = français + anglais).
    pub languages: u32,
    /// Fiabilité dégradée par les incidents récents (s'atténue).
    pub incident_penalty: f64,
    pub launched: bool,
    /// Bonus de rétention relatifs par segment (0,1 = -10 % de churn).
    pub retention: [f64; 3],
    /// Bonus de conversion relatifs par segment.
    pub conversion: [f64; 3],
    /// Hébergement mensuel supplémentaire dû aux choix d'architecture.
    pub hosting_extra: f64,
    /// Réduction de la probabilité d'incident grâce au monitoring, aux audits, au travail SRE.
    #[serde(default)]
    pub resilience: f64,
    /// Croissance organique cumulée de l'usage (plafonnée).
    #[serde(default)]
    pub usage_creep: f64,
}

impl ProductState {
    pub fn new(accuracy: f64, fpr: f64, debt: f64, inference_cost: f64) -> Self {
        Self {
            shipped: BTreeSet::new(),
            feature_score: 0.0,
            model_accuracy: accuracy,
            drift: 0.0,
            robustness: 0.0,
            fpr,
            tech_debt: debt,
            inference_cost_per_k: inference_cost,
            usage_mult: 1.0,
            languages: 1,
            incident_penalty: 0.0,
            launched: false,
            retention: [0.0; 3],
            conversion: [0.0; 3],
            hosting_extra: 0.0,
            resilience: 0.0,
            usage_creep: 0.0,
        }
    }

    /// Précision mesurée sur le trafic du jour (précision du modèle moins la dérive).
    pub fn effective_accuracy(&self) -> f64 {
        (self.model_accuracy - self.drift).clamp(50.0, 99.0)
    }

    /// Les faux positifs augmentent à mesure que le benchmark dérive.
    pub fn effective_fpr(&self) -> f64 {
        (self.fpr + self.drift * 0.15).clamp(0.3, 40.0)
    }

    /// Score de détection 0..100 : précision au-dessus du pile ou face, moins une pénalité
    /// pour avoir signalé des humains (ce que les clients détestent le plus).
    pub fn detection_score(&self) -> f64 {
        ((self.effective_accuracy() - 50.0) * 3.0 - self.effective_fpr() * 1.2).clamp(0.0, 100.0)
    }

    pub fn reliability(&self) -> f64 {
        (100.0 - self.tech_debt * 0.6 - self.incident_penalty).clamp(0.0, 100.0)
    }

    /// Qualité produit perçue par le marché, 0..100.
    /// qualité = 35 % fonctionnalités + 45 % détection + 20 % fiabilité
    pub fn quality(&self) -> f64 {
        (0.35 * self.feature_score.min(100.0)
            + 0.45 * self.detection_score()
            + 0.20 * self.reliability())
        .clamp(0.0, 100.0)
    }

    /// Applique un gain de précision à rendement décroissant : plus on est près du
    /// plafond, plus l'amélioration est faible.
    pub fn gain_accuracy(&mut self, amount: f64, ceiling: f64) {
        let gap = (ceiling - self.model_accuracy).max(0.0);
        let scale = (gap / (ceiling - 53.0)).clamp(0.0, 1.0).sqrt();
        self.model_accuracy = (self.model_accuracy + amount * scale.max(0.05)).min(ceiling);
    }

    pub fn has(&self, task: &str) -> bool {
        self.shipped.contains(task)
    }
}
