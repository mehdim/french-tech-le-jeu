//! Les mentors d'A la French : Mehdi, Steeve et Jean-Baptiste.
//!
//! * Au début de la campagne, le joueur choisit un **mentor principal**
//!   (événement `mentors_intro`, effet [`Effect::ChooseMentor`]). Son bonus
//!   permanent s'applique une fois, et ses conseils reviennent plus vite.
//! * Pendant la partie, chaque mentor peut être **consulté** (commande
//!   `ConsultMentor`). Il donne le premier conseil de sa liste dont les
//!   conditions sont remplies : un conseil adapté à la situation, pas un bonus
//!   fixe. Puis il faut attendre la fin de son délai avant de le solliciter.
//!
//! [`Effect::ChooseMentor`]: crate::types::Effect::ChooseMentor

use crate::content::{Content, MentorDef};
use crate::messages::{Cue, Msg, Tone};
use crate::simulation::{CmdError, GameState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MentorState {
    /// Identifiant du mentor principal, une fois choisi.
    #[serde(default)]
    pub main: Option<String>,
    /// id du mentor -> jour de la dernière consultation.
    #[serde(default)]
    pub last_consult: BTreeMap<String, u32>,
    /// Dernier conseil reçu de chaque mentor (affiché dans le panneau).
    #[serde(default)]
    pub last_advice: BTreeMap<String, String>,
    #[serde(default)]
    pub consults: u32,
}

impl GameState {
    /// Délai entre deux consultations de ce mentor (plus court pour le mentor principal).
    pub fn mentor_cooldown(&self, def: &MentorDef) -> u32 {
        if self.mentors.main.as_deref() == Some(def.id.as_str()) {
            def.cooldown_main
        } else {
            def.cooldown
        }
    }

    /// Jours restants avant de pouvoir consulter ce mentor (0 = disponible).
    pub fn mentor_wait(&self, def: &MentorDef) -> u32 {
        match self.mentors.last_consult.get(&def.id) {
            Some(&d) => (d + self.mentor_cooldown(def)).saturating_sub(self.day),
            None => 0,
        }
    }

    /// Le conseil que ce mentor donnerait maintenant (le premier dont les conditions tiennent).
    pub fn mentor_advice<'a>(&self, def: &'a MentorDef) -> Option<&'a crate::content::AdviceDef> {
        def.advice.iter().find(|a| a.conditions.iter().all(|c| self.check(c)))
    }

    pub(crate) fn choose_mentor(&mut self, content: &Content, id: &str) -> Result<(), CmdError> {
        if self.mentors.main.is_some() {
            return Err(CmdError::Invalid("Vous avez déjà un mentor principal"));
        }
        let def = content.mentor(id).ok_or(CmdError::NotFound)?.clone();
        self.mentors.main = Some(def.id.clone());
        self.flags.insert(format!("mentor_{}", def.id));
        self.apply_effects(content, &def.passive, &def.name);
        self.log(Tone::Good, Msg::MentorChosen { mentor: def.name.clone() });
        Ok(())
    }

    pub(crate) fn consult_mentor(&mut self, content: &Content, id: &str) -> Result<(), CmdError> {
        let def = content.mentor(id).ok_or(CmdError::NotFound)?.clone();
        if self.mentor_wait(&def) > 0 {
            return Err(CmdError::Invalid("Ce mentor n'est pas encore disponible"));
        }
        let Some(advice) = self.mentor_advice(&def).cloned() else {
            return Err(CmdError::Invalid("Ce mentor n'a rien à vous dire pour l'instant"));
        };
        let cost: f64 = advice
            .effects
            .iter()
            .filter_map(|e| match e {
                crate::types::Effect::Cash(v) if *v < 0.0 => Some(-v),
                _ => None,
            })
            .sum();
        if cost > 0.0 && self.finance.cash < cost {
            return Err(CmdError::NotEnoughCash);
        }
        self.mentors.last_consult.insert(def.id.clone(), self.day);
        self.mentors.last_advice.insert(def.id.clone(), advice.text.clone());
        self.mentors.consults += 1;
        self.apply_effects(content, &advice.effects, &def.name);
        self.log(Tone::Good, Msg::MentorAdvice { mentor: def.name.clone(), text: advice.text.clone() });
        self.cues.push(Cue::EventPopup);
        Ok(())
    }
}
