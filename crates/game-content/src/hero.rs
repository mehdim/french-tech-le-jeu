//! Personnalisation des textes selon le héros ou l'héroïne choisi·e à la création de la partie.
//!
//! Les données peuvent contenir `{nom}` (le nom choisi), `{fondateur}` / `{Fondateur}`
//! (fondateur ou fondatrice) et `{e}` (accord au féminin). Pour une héroïne, les tournures
//! existantes « le fondateur », « du fondateur »… sont aussi accordées.

use game_core::simulation::FounderProfile;

/// Remplace les marqueurs et accorde le texte au genre du héros.
pub fn personalize(text: &str, f: &FounderProfile) -> String {
    if !text.contains('{') && !(f.female && text.contains("ondateur")) && !text.contains("Vous (") {
        return text.to_string();
    }
    let (fond, fond_cap) = if f.female { ("fondatrice", "Fondatrice") } else { ("fondateur", "Fondateur") };
    let mut s = text
        .replace("{nom}", &f.name)
        .replace("{fondateur}", fond)
        .replace("{Fondateur}", fond_cap)
        .replace("{e}", if f.female { "e" } else { "" });
    if f.female {
        for (a, b) in [
            ("Le fondateur", "La fondatrice"),
            ("le fondateur", "la fondatrice"),
            ("du fondateur", "de la fondatrice"),
            ("au fondateur", "à la fondatrice"),
            ("un fondateur", "une fondatrice"),
            ("Un fondateur", "Une fondatrice"),
        ] {
            s = s.replace(a, b);
        }
        // « fondateur » isolé (mais pas « fondateurs »).
        let mut out = String::with_capacity(s.len());
        let mut rest = s.as_str();
        while let Some(i) = rest.find("fondateur") {
            let after = &rest[i + "fondateur".len()..];
            out.push_str(&rest[..i]);
            if after.starts_with('s') {
                out.push_str("fondateur");
            } else {
                out.push_str("fondatrice");
            }
            rest = after;
        }
        out.push_str(rest);
        s = out;
    }
    s
}

/// Nom affiché pour un interlocuteur : « Vous » devient « Vous (Camille) ».
pub fn speaker(speaker: &str, f: &FounderProfile) -> String {
    let base = personalize(speaker, f);
    if speaker == "Vous" || speaker.starts_with("Vous,") {
        format!("{base} · {}", f.name)
    } else {
        base
    }
}

/// Clé de portrait, en tenant compte du genre et du type du héros.
pub fn portrait(speaker: &str, f: &FounderProfile) -> String {
    match crate::characters::portrait_for(speaker) {
        "founder" => founder_portrait(f),
        k => k.to_string(),
    }
}

/// Portrait du héros ou de l'héroïne de la partie.
pub fn founder_portrait(f: &FounderProfile) -> String {
    match (f.ethnicity, f.female) {
        (Some(e), true) => format!("hero_f{e}"),
        (Some(e), false) => format!("hero_m{e}"),
        (None, true) => "founder_f".into(),
        (None, false) => "founder".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts_follow_the_hero() {
        let her = FounderProfile { name: "Camille".into(), female: true, ethnicity: None };
        let him = FounderProfile { name: "Hugo".into(), female: false, ethnicity: Some(2) };
        assert_eq!(personalize("{nom} sonne la cloche.", &her), "Camille sonne la cloche.");
        assert_eq!(personalize("Le fondateur et le moral du fondateur", &her), "La fondatrice et le moral de la fondatrice");
        assert_eq!(personalize("Les fondateurs, la {fondateur} prêt{e}", &her), "Les fondateurs, la fondatrice prête");
        assert_eq!(personalize("le {fondateur} prêt{e}", &him), "le fondateur prêt");
        assert_eq!(personalize("Modèle de fondation", &her), "Modèle de fondation");
        assert_eq!(portrait("Vous", &her), "founder_f");
        assert_eq!(portrait("Mehdi", &her), "mehdi");
        assert_eq!(portrait("Vous", &him), "hero_m2");
        assert_eq!(speaker("Vous", &him), "Vous · Hugo");
    }
}
