//! File des chantiers de développement et de R&D.

use crate::content::{Content, TaskDef};
use crate::product::ProductState;
use serde::{Deserialize, Serialize};

/// Le soin apporté à un chantier : vitesse contre dette technique.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Approach {
    /// « On verra en prod » : rapide, beaucoup de dette, risqué.
    Quick,
    Standard,
    /// Tests, docs, revue de code : lent, peu de dette.
    Clean,
}

impl Approach {
    pub const ALL: [Approach; 3] = [Approach::Quick, Approach::Standard, Approach::Clean];
    pub fn effort_mult(self) -> f64 {
        match self {
            Approach::Quick => 0.7,
            Approach::Standard => 1.0,
            Approach::Clean => 1.3,
        }
    }
    pub fn debt_mult(self) -> f64 {
        match self {
            Approach::Quick => 2.2,
            Approach::Standard => 1.0,
            Approach::Clean => 0.3,
        }
    }
    pub fn risk_mult(self) -> f64 {
        match self {
            Approach::Quick => 1.6,
            Approach::Standard => 1.0,
            Approach::Clean => 0.5,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Approach::Quick => "Vite fait (on verra en prod)",
            Approach::Standard => "Standard",
            Approach::Clean => "Propre (tests + revue)",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveTask {
    pub instance: u32,
    pub task_id: String,
    pub approach: Approach,
    pub eng_total: f64,
    pub research_total: f64,
    pub eng_done: f64,
    pub research_done: f64,
    pub started_day: u32,
}

impl ActiveTask {
    pub fn progress(&self) -> f64 {
        let total = self.eng_total + self.research_total;
        if total <= 0.0 {
            return 1.0;
        }
        ((self.eng_done.min(self.eng_total) + self.research_done.min(self.research_total)) / total)
            .clamp(0.0, 1.0)
    }
    pub fn is_complete(&self) -> bool {
        self.eng_done >= self.eng_total - 1e-6 && self.research_done >= self.research_total - 1e-6
    }
    pub fn eng_remaining(&self) -> f64 {
        (self.eng_total - self.eng_done).max(0.0)
    }
    pub fn research_remaining(&self) -> f64 {
        (self.research_total - self.research_done).max(0.0)
    }
}

/// Pourquoi un chantier ne peut pas démarrer maintenant.
#[derive(Clone, Debug, PartialEq)]
pub enum TaskBlock {
    Locked(String),
    MissingDep(String),
    AlreadyShipped,
    AlreadyActive,
    NotEnoughCash,
}

pub fn task_availability(
    def: &TaskDef,
    product: &ProductState,
    unlocks: &std::collections::BTreeSet<String>,
    active: &[ActiveTask],
    cash: f64,
) -> Result<(), TaskBlock> {
    if let Some(u) = &def.requires_unlock
        && !unlocks.contains(u) {
            return Err(TaskBlock::Locked(u.clone()));
        }
    for d in &def.deps {
        if !product.has(d) {
            return Err(TaskBlock::MissingDep(d.clone()));
        }
    }
    if !def.repeatable && product.has(&def.id) {
        return Err(TaskBlock::AlreadyShipped);
    }
    if active.iter().any(|t| t.task_id == def.id) {
        return Err(TaskBlock::AlreadyActive);
    }
    if cash < def.cost {
        return Err(TaskBlock::NotEnoughCash);
    }
    Ok(())
}

/// Chantiers visibles dans l'arbre (des dépendances peuvent encore manquer).
pub fn visible_tasks<'a>(
    content: &'a Content,
    unlocks: &std::collections::BTreeSet<String>,
) -> Vec<&'a TaskDef> {
    content
        .tasks
        .iter()
        .filter(|t| t.requires_unlock.as_ref().is_none_or(|u| unlocks.contains(u)))
        .collect()
}
