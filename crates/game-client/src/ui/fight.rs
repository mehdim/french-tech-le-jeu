//! Face-à-face contre le patron des acquisitions d'un géant de la tech (niveaux 14 et 15).
//!
//! Quand la défense anti-rachat a réuni ses 1 000 points de levée de fonds, le fondateur
//! affronte le ou la responsable des acquisitions de l'entreprise : dialogues d'avant-combat,
//! puis combat au tour par tour avec une jauge de visée. Défaite = rachat, fin de partie.
//!
//! Les entreprises sont réelles ; les personnages sont fictifs (aucun vrai dirigeant n'est
//! représenté ni cité).

use super::Ctx;
use super::style::*;
use bevy_egui::egui::{self, Align2, Color32, Pos2, Rect, Stroke, pos2, vec2};
use game_core::Command;

/// Allure d'un personnage en pixel art (grille de 16 × 20 cases).
#[derive(Clone, Copy)]
struct Look {
    skin: Color32,
    hair: Color32,
    /// 0 courts, 1 longs, 2 chignon, 3 dégarni, 4 en pétard, 5 bouclés.
    hair_style: u8,
    suit: Color32,
    shirt: Color32,
    accent: Color32,
    glasses: bool,
    headset: bool,
    beard: bool,
}

struct Boss {
    company: &'static str,
    name: &'static str,
    title: &'static str,
    hp: f32,
    look: Look,
    arena: (Color32, Color32),
    intro: [(&'static str, &'static str); 4],
    attacks: [(&'static str, &'static str); 4],
    hurt: [&'static str; 3],
    defeat: &'static str,
    victory: &'static str,
}

const FOUNDER: Look = Look {
    skin: Color32::from_rgb(240, 200, 170),
    hair: Color32::from_rgb(90, 60, 40),
    hair_style: 0,
    suit: Color32::from_rgb(38, 125, 75),
    shirt: Color32::from_rgb(20, 24, 22),
    accent: Color32::from_rgb(124, 255, 82),
    glasses: false,
    headset: false,
    beard: false,
};

/// Teint, cheveux et coiffure du héros selon le type choisi (mêmes couleurs que tools/art).
fn hero_look(f: &game_core::simulation::FounderProfile) -> Look {
    // (teint, cheveux, coiffure homme, coiffure femme, barbe)
    const TYPES: [((u8, u8, u8), (u8, u8, u8), u8, u8, bool); 7] = [
        ((244, 208, 178), (112, 74, 42), 0, 1, false),
        ((212, 164, 120), (34, 26, 22), 0, 1, true),
        ((170, 114, 76), (22, 20, 24), 0, 2, false),
        ((192, 140, 100), (58, 38, 26), 5, 5, false),
        ((104, 68, 46), (20, 18, 18), 5, 5, false),
        ((238, 204, 162), (24, 22, 26), 0, 1, false),
        ((178, 120, 82), (16, 16, 20), 1, 1, false),
    ];
    let Some(e) = f.ethnicity else {
        return if f.female { Look { hair_style: 1, hair: Color32::from_rgb(150, 70, 40), ..FOUNDER } } else { FOUNDER };
    };
    let (skin, hair, sm, sf, beard) = TYPES[e as usize % TYPES.len()];
    Look {
        skin: Color32::from_rgb(skin.0, skin.1, skin.2),
        hair: Color32::from_rgb(hair.0, hair.1, hair.2),
        hair_style: if f.female { sf } else { sm },
        beard: beard && !f.female,
        ..FOUNDER
    }
}

fn boss(company: &str) -> Boss {
    match company {
        "google" => Boss {
            company: "Google",
            name: "Ada Rankwell",
            title: "Vice-présidente des acquisitions",
            hp: 160.0,
            look: Look {
                skin: Color32::from_rgb(170, 120, 90),
                hair: Color32::from_rgb(30, 24, 22),
                hair_style: 1,
                suit: Color32::from_rgb(40, 130, 140),
                shirt: OFFWHITE,
                accent: Color32::from_rgb(250, 206, 72),
                glasses: true,
                headset: false,
                beard: false,
            },
            arena: (Color32::from_rgb(232, 240, 254), Color32::from_rgb(180, 210, 250)),
            intro: [
                ("Ada Rankwell", "Tu as refusé un déjeuner étoilé. Personne ne refuse un déjeuner étoilé."),
                ("Vous", "Je préfère le kebab de Belleville. Et mon indépendance."),
                ("Ada Rankwell", "J'ai indexé chacune de tes faiblesses. Elles tiennent sur une page. La première page des résultats."),
                ("Vous", "Alors cherche « défaite » : tu tomberas sur ta propre photo."),
            ],
            attacks: [
                ("Indexation totale", "J'ai crawlé ton code source. Tes commentaires sont… émouvants."),
                ("Offre à 40 fois l'ARR", "Quarante fois ! Même ta mère te dirait de signer."),
                ("Concurrent gratuit", "Nous lançons la même chose, gratuite, intégrée partout. Mardi."),
                ("Débauchage", "Ta meilleure chercheuse vient de recevoir une offre. Et un vélo coloré."),
            ],
            hurt: ["Aïe ! Ça va me coûter un OKR.", "Impossible. Ce n'était pas dans mes prédictions.", "Je vais devoir en parler à mon N+3."],
            defeat: "Très bien… Veridix reste indépendante. Pour l'instant. Je mets un rappel dans mon agenda.",
            victory: "Signé. Bienvenue dans l'écosystème. Il y a une cantine gratuite.",
        },
        "microsoft" => Boss {
            company: "Microsoft",
            name: "Gordon Ribbon",
            title: "Président des partenariats stratégiques",
            hp: 170.0,
            look: Look {
                skin: Color32::from_rgb(236, 196, 170),
                hair: Color32::from_rgb(150, 150, 150),
                hair_style: 3,
                suit: Color32::from_rgb(30, 50, 100),
                shirt: OFFWHITE,
                accent: Color32::from_rgb(70, 140, 230),
                glasses: false,
                headset: false,
                beard: true,
            },
            arena: (Color32::from_rgb(220, 232, 248), Color32::from_rgb(120, 160, 220)),
            intro: [
                ("Gordon Ribbon", "Nous avons racheté des entreprises plus grosses que ta ville natale."),
                ("Vous", "Ma ville natale, c'est Belleville. Elle ne se vend pas."),
                ("Gordon Ribbon", "Tout se vend. Il suffit d'ajouter un abonnement annuel."),
                ("Vous", "Mets à jour tes certitudes. Redémarrage requis."),
            ],
            attacks: [
                ("Intégration forcée", "Veridix sera préinstallé partout. Que tu le veuilles ou non."),
                ("Licence entreprise", "Tes grands comptes ont déjà un contrat avec nous. Ils signeront ce que je dis."),
                ("Réunion surprise de 3 heures", "Rejoins-moi : réunion sur la synergie. Caméra obligatoire."),
                ("Rachat du concurrent", "J'ai racheté Veridupe ce matin. Par curiosité."),
            ],
            hurt: ["Erreur inattendue. Veuillez réessayer plus tard.", "Mon tableur ne prévoyait pas ça.", "Ce n'est pas un bug… si, c'est un bug."],
            defeat: "Bien joué. Veridix reste libre. Je retourne au siège. En hélicoptère. Seul.",
            victory: "Contrat signé. Ton produit sera disponible dans la prochaine version. Ou la suivante.",
        },
        "amazon" => Boss {
            company: "Amazon",
            name: "Brenda Prime",
            title: "Directrice des « acquisitions du jour »",
            hp: 155.0,
            look: Look {
                skin: Color32::from_rgb(120, 80, 55),
                hair: Color32::from_rgb(25, 20, 18),
                hair_style: 5,
                suit: Color32::from_rgb(240, 140, 40),
                shirt: Color32::from_rgb(30, 30, 30),
                accent: OFFWHITE,
                glasses: false,
                headset: false,
                beard: false,
            },
            arena: (Color32::from_rgb(250, 236, 214), Color32::from_rgb(230, 170, 100)),
            intro: [
                ("Brenda Prime", "Chez nous, on dit « le client d'abord ». Aujourd'hui, le client, c'est moi."),
                ("Vous", "Retour à l'expéditeur."),
                ("Brenda Prime", "J'ai lu ton dernier mémo. Il faisait deux pages. Amateur."),
                ("Vous", "Le mien tient en une phrase : Veridix n'est pas à vendre."),
            ],
            attacks: [
                ("Livraison en une heure", "Mon offre est déjà dans ton hall. Avec du papier bulle."),
                ("Guerre des prix", "Nous vendrons la détection à prix coûtant. Puis en dessous."),
                ("Mémo de six pages", "Silence. Lis mon mémo. Les six pages. Maintenant."),
                ("Facture cloud", "Ton hébergement est chez nous. Ce serait dommage qu'il… ralentisse."),
            ],
            hurt: ["Colis endommagé ! Je demande un remboursement.", "Ce n'était pas dans le mémo.", "Une étoile. Je mets une étoile."],
            defeat: "Commande annulée. Veridix reste indépendante. Je laisse un avis cinq étoiles. À contrecœur.",
            victory: "Livré ! Signature reçue. Merci de noter votre expérience de rachat.",
        },
        "meta" => Boss {
            company: "Meta",
            name: "Ingrid Horizon",
            title: "Vice-présidente des acquisitions",
            hp: 150.0,
            look: Look {
                skin: Color32::from_rgb(244, 210, 186),
                hair: Color32::from_rgb(230, 200, 110),
                hair_style: 2,
                suit: Color32::from_rgb(110, 116, 128),
                shirt: OFFWHITE,
                accent: Color32::from_rgb(80, 130, 240),
                glasses: false,
                headset: true,
                beard: false,
            },
            arena: (Color32::from_rgb(40, 30, 90), Color32::from_rgb(100, 60, 170)),
            intro: [
                ("Ingrid Horizon", "Bienvenue dans mon métavers. Ici, c'est moi qui fixe la gravité."),
                ("Vous", "Tu as des jambes, au moins, dans ce monde ?"),
                ("Ingrid Horizon", "Les jambes, c'est surfait. Les données, non. Donne-moi les tiennes."),
                ("Vous", "Mes données restent en Europe. Et moi, je reste debout."),
            ],
            attacks: [
                ("Pivot vers le métavers", "Ton bureau est désormais virtuel. Ton moral aussi."),
                ("Algorithme du fil", "Je viens de rendre tes posts invisibles. Tous."),
                ("Rachat de l'équipe", "J'offre à tes ingénieurs des lunettes connectées et un salaire à six chiffres."),
                ("Copie conforme", "Nous avons copié ton produit. En story de 24 heures."),
            ],
            hurt: ["Lag ! Je… lag !", "Erreur de rendu : mon avatar a perdu la tête.", "Ça ne fera pas une bonne publication."],
            defeat: "Déconnexion… Veridix reste hors de mon monde. Pour cette version du métavers.",
            victory: "Bienvenue dans la famille ! Pensez à recharger votre casque.",
        },
        _ => Boss {
            company: "NVIDIA",
            name: "Tess Tensor",
            title: "Directrice du développement",
            hp: 165.0,
            look: Look {
                skin: Color32::from_rgb(226, 182, 150),
                hair: Color32::from_rgb(200, 70, 50),
                hair_style: 4,
                suit: Color32::from_rgb(90, 150, 20),
                shirt: Color32::from_rgb(25, 25, 25),
                accent: Color32::from_rgb(250, 206, 72),
                glasses: true,
                headset: false,
                beard: false,
            },
            arena: (Color32::from_rgb(10, 20, 10), Color32::from_rgb(40, 70, 20)),
            intro: [
                ("Tess Tensor", "Chaque calcul que tu fais passe par mes puces. Je suis déjà chez toi."),
                ("Vous", "Alors tu sais qu'on vient de diviser notre consommation par deux."),
                ("Tess Tensor", "Mignon. Mes GPU chauffent plus que ta piscine municipale."),
                ("Vous", "Pas besoin de puissance brute pour gagner. Juste d'efficacité."),
            ],
            attacks: [
                ("Pénurie de GPU", "Ta commande de puces ? Repoussée à 2031."),
                ("Offre en actions", "Je te paie en actions. Elles ont triplé ce matin. Et hier."),
                ("Surcadençage", "Je pousse l'horloge à fond. Tout chauffe. Surtout toi."),
                ("Écosystème propriétaire", "Ton code tourne sur ma plateforme. Sans elle, il ne tourne plus."),
            ],
            hurt: ["Bridage thermique !", "Mes ventilateurs… s'emballent !", "Ce n'était pas optimisé pour ça."],
            defeat: "Bien joué. Veridix reste indépendante. Mais tes GPU, tu les achèteras quand même chez moi.",
            victory: "Marché conclu. Bienvenue dans la plateforme. Ton premier cadeau : un rack de GPU.",
        },
    }
}

/// Les quatre coups du fondateur.
const MOVES: [(&str, &str); 4] = [
    ("Pitch de vision", "Dégâts moyens, fiables."),
    ("Les chiffres", "Dégâts selon votre MRR."),
    ("Appel aux mentors", "Soigne et renforce le coup suivant (2 fois)."),
    ("Pilule empoisonnée", "Énormes dégâts, peut rater (1 fois)."),
];

const PITCH_LINES: [&str; 3] = [
    "Notre mission : rendre la confiance aux textes. Vous, vous voulez juste nos clients.",
    "Nous sommes la seule équipe qui sait mesurer ce qu'elle ne sait pas.",
    "On est partis d'un salon à Belleville. On ne finira pas dans votre tableur.",
];
const MENTOR_LINES: [&str; 3] = [
    "Mehdi : « Sans notre écosystème, leur offre ne vaut rien ! »",
    "Steeve : « Montre-leur ta facture GPU. La vraie. Elle a baissé. »",
    "Jean-Baptiste : « On pose un cône de chantier et on tient la position. »",
];
const PILL_HIT: &str = "Clause anti-OPA activée : chaque action rachetée vaudra le double pour les salariés !";
const PILL_MISS: &str = "Clause anti-OPA… rejetée par le tribunal de commerce. Vos avocats rougissent.";

#[derive(Clone, Copy, PartialEq, Debug, Default)]
enum Phase {
    #[default]
    Intro,
    Choose,
    Aim,
    PlayerHit,
    Enemy,
    End,
}

#[derive(Clone, Debug, Default)]
struct Fight {
    key: String,
    phase: Phase,
    line: usize,
    t: f32,
    last_time: f64,
    rng: u64,
    hp: f32,
    hp_max: f32,
    boss_hp: f32,
    boss_max: f32,
    mentors_left: u8,
    pill_left: u8,
    buff: f32,
    debuff: f32,
    chosen: usize,
    cursor: f32,
    speaker: String,
    text: String,
    popup: String,
    shake_boss: f32,
    shake_hero: f32,
    won: bool,
}

impl Fight {
    fn rand(&mut self) -> f32 {
        let mut x = self.rng.max(1);
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        ((x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 40) as f32) / (1u64 << 24) as f32
    }
}

/// Affiche le combat. Ne fait rien s'il n'y a pas de défense prête.
pub fn show(ctx: &egui::Context, c: &mut Ctx) {
    let Some(t) = c.game.0.takeover.clone() else { return };
    if !t.ready {
        return;
    }
    let b = boss(&t.company);
    let key = format!("{}:{}", t.company, t.deadline_day);
    let id = egui::Id::new("takeover_fight");
    let mut f: Fight = ctx.data(|d| d.get_temp::<Fight>(id)).unwrap_or_default();
    if f.key != key {
        let g = &c.game.0;
        let hp = (100.0 + g.reputation as f32 * 0.4).min(150.0);
        f = Fight {
            key,
            rng: g.seed ^ g.day as u64 ^ 0xA5A5_5A5A,
            hp,
            hp_max: hp,
            boss_hp: b.hp,
            boss_max: b.hp,
            mentors_left: 2,
            pill_left: 1,
            buff: 1.0,
            debuff: 1.0,
            speaker: b.intro[0].0.to_string(),
            text: b.intro[0].1.to_string(),
            ..Default::default()
        };
    }
    let founder = c.game.0.founder.clone();
    let hero_look = hero_look(&founder);
    let now = ctx.input(|i| i.time);
    let dt = if f.last_time > 0.0 { (now - f.last_time).clamp(0.0, 0.1) as f32 } else { 0.0 };
    f.last_time = now;
    f.t += dt;
    f.shake_boss = (f.shake_boss - dt).max(0.0);
    f.shake_hero = (f.shake_hero - dt).max(0.0);
    ctx.request_repaint();

    let mrr = c.game.0.metrics.mrr;
    let mut resolve: Option<bool> = None;
    super::widgets::dim(ctx, 220);
    let cr = ctx.content_rect();
    let w = (cr.width() - 40.0).clamp(320.0, 880.0);
    let h = (w * 9.0 / 16.0).min(cr.height() - 190.0).max(200.0);
    let w = w.min(h * 16.0 / 9.0);
    egui::Area::new(egui::Id::new("fight"))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            modal_frame().show(ui, |ui| {
                ui.set_width(w);
                ui.label(egui::RichText::new(format!("FACE-À-FACE · OFFRE DE RACHAT DE {}", b.company.to_uppercase())).font(pixel(13.0)).color(ORANGE));
                let (rect, resp) = ui.allocate_exact_size(vec2(w, h), egui::Sense::click());
                let p = ui.painter_at(rect);
                let clicked = resp.clicked();
                let keys = ctx.input(|i| {
                    let n = [egui::Key::Num1, egui::Key::Num2, egui::Key::Num3, egui::Key::Num4].iter().position(|k| i.key_pressed(*k));
                    (n, i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space))
                });
                let advance = clicked || keys.1;

                // --- logique -------------------------------------------------
                match f.phase {
                    Phase::Intro => {
                        if advance {
                            f.line += 1;
                            if f.line < b.intro.len() {
                                f.speaker = b.intro[f.line].0.to_string();
                                f.text = b.intro[f.line].1.to_string();
                            } else {
                                f.phase = Phase::Choose;
                                f.speaker = "Narrateur".into();
                                f.text = "Le combat commence ! Choisissez un coup (1-4), puis visez au centre de la jauge.".into();
                            }
                        }
                    }
                    Phase::Aim => {
                        // Le curseur fait des allers-retours ; on vise le centre.
                        f.cursor = 0.5 + 0.5 * (f.t * 3.2).sin();
                        if advance {
                            let accuracy = 1.0 - (f.cursor - 0.5).abs() * 2.0;
                            let mult = 0.55 + accuracy * 0.95;
                            let (dmg, line) = match f.chosen {
                                0 => (18.0 + f.rand() * 8.0, PITCH_LINES[(f.rand() * 3.0) as usize % 3].to_string()),
                                1 => {
                                    let base = 12.0 + ((mrr / 250_000.0) as f32).min(22.0);
                                    (base + f.rand() * 4.0, format!("{} de MRR, une équipe soudée, et des clients qui restent. Trouvez mieux.", game_content::dialogue::money(mrr)))
                                }
                                2 => (0.0, MENTOR_LINES[(f.rand() * 3.0) as usize % 3].to_string()),
                                _ => {
                                    if f.rand() < 0.65 { (36.0 + f.rand() * 10.0, PILL_HIT.to_string()) } else { (0.0, PILL_MISS.to_string()) }
                                }
                            };
                            if f.chosen == 2 {
                                let heal = 18.0 + 10.0 * accuracy;
                                f.hp = (f.hp + heal).min(f.hp_max);
                                f.buff = 1.5;
                                f.popup = format!("+{heal:.0} d'indépendance · prochain coup ×1,5");
                            } else {
                                let total = dmg * mult * f.buff * f.debuff;
                                f.buff = 1.0;
                                f.debuff = 1.0;
                                f.boss_hp = (f.boss_hp - total).max(0.0);
                                if total > 0.0 {
                                    f.shake_boss = 0.5;
                                    f.popup = format!("-{total:.0}{}", if accuracy > 0.85 { " · COUP PARFAIT !" } else { "" });
                                } else {
                                    f.popup = "Raté !".into();
                                }
                            }
                            f.speaker = "Vous".into();
                            f.text = line;
                            f.phase = Phase::PlayerHit;
                            f.t = 0.0;
                        }
                    }
                    Phase::PlayerHit => {
                        if f.t > 1.6 || (advance && f.t > 0.3) {
                            if f.boss_hp <= 0.0 {
                                f.won = true;
                                f.phase = Phase::End;
                                f.speaker = b.name.to_string();
                                f.text = b.defeat.to_string();
                            } else {
                                f.phase = Phase::Enemy;
                                f.t = 0.0;
                                let hurt = b.hurt[(f.rand() * 3.0) as usize % 3];
                                f.speaker = b.name.to_string();
                                f.text = hurt.to_string();
                                f.popup.clear();
                            }
                        }
                    }
                    Phase::Enemy => {
                        if f.t > 0.9 && f.speaker == b.name && f.popup.is_empty() {
                            let k = (f.rand() * 4.0) as usize % 4;
                            let (name, line) = b.attacks[k];
                            let dmg = 13.0 + f.rand() * 11.0;
                            f.hp = (f.hp - dmg).max(0.0);
                            f.shake_hero = 0.5;
                            if k == 3 {
                                f.debuff = 0.7;
                            }
                            f.popup = format!("{name} : -{dmg:.0}{}", if k == 3 { " · votre prochain coup ×0,7" } else { "" });
                            f.text = line.to_string();
                            f.t = 0.9;
                        }
                        if f.t > 2.8 || (advance && f.t > 1.3) {
                            if f.hp <= 0.0 {
                                f.won = false;
                                f.phase = Phase::End;
                                f.speaker = b.name.to_string();
                                f.text = b.victory.to_string();
                            } else {
                                f.phase = Phase::Choose;
                                f.speaker = "Narrateur".into();
                                f.text = "À vous ! Choisissez un coup (1-4).".into();
                                f.popup.clear();
                            }
                        }
                    }
                    Phase::Choose | Phase::End => {}
                }

                // --- dessin ---------------------------------------------------
                let (top, bottom) = b.arena;
                let n = 20;
                for i in 0..n {
                    let k = i as f32 / n as f32;
                    let y0 = rect.min.y + rect.height() * k;
                    p.rect_filled(Rect::from_min_max(pos2(rect.min.x, y0), pos2(rect.max.x, y0 + rect.height() / n as f32 + 1.0)), 0.0, lerp(top, bottom, k));
                }
                // Sol de l'arène.
                p.rect_filled(Rect::from_min_max(pos2(rect.min.x, rect.min.y + rect.height() * 0.62), rect.max), 0.0, Color32::from_black_alpha(60));
                let cell = (rect.height() * 0.44 / 20.0).floor().max(3.0);
                let bob = |phase: f32| (f.t * 2.0 + phase).sin() * 2.0;
                let shake = |s: f32| if s > 0.0 { (s * 60.0).sin() * 6.0 } else { 0.0 };
                let hero_o = pos2(rect.min.x + rect.width() * 0.16 + shake(f.shake_hero), rect.min.y + rect.height() * 0.62 - cell * 20.0 + bob(0.0));
                let boss_o = pos2(rect.max.x - rect.width() * 0.16 - cell * 16.0 + shake(f.shake_boss), rect.min.y + rect.height() * 0.62 - cell * 20.0 + bob(1.5));
                draw_character(&p, hero_o, cell, &hero_look, f.shake_hero > 0.0, false);
                draw_character(&p, boss_o, cell, &b.look, f.shake_boss > 0.0, true);
                // Barres de vie.
                hp_bar(&p, Rect::from_min_size(rect.min + vec2(14.0, 12.0), vec2(rect.width() * 0.38, 16.0)), f.hp / f.hp_max, "Vous · Indépendance", GREEN);
                hp_bar(&p, Rect::from_min_size(pos2(rect.max.x - 14.0 - rect.width() * 0.38, rect.min.y + 12.0), vec2(rect.width() * 0.38, 16.0)), f.boss_hp / f.boss_max,
                    &format!("{} · {}", b.name, b.title), RED);
                // Texte flottant du dernier coup.
                if !f.popup.is_empty() {
                    let at = if matches!(f.phase, Phase::Enemy) { hero_o + vec2(cell * 8.0, -12.0) } else { boss_o + vec2(cell * 8.0, -12.0) };
                    p.text(at + vec2(1.0, 1.0), Align2::CENTER_BOTTOM, &f.popup, pixel(15.0), Color32::from_black_alpha(200));
                    p.text(at, Align2::CENTER_BOTTOM, &f.popup, pixel(15.0), if matches!(f.phase, Phase::Enemy) { RED } else { Color32::from_rgb(250, 206, 72) });
                }
                // Jauge de visée.
                if f.phase == Phase::Aim {
                    let bar = Rect::from_center_size(pos2(rect.center().x, rect.min.y + rect.height() * 0.55), vec2(rect.width() * 0.5, 18.0));
                    p.rect_filled(bar, 3.0, NEAR_BLACK);
                    p.rect_filled(Rect::from_center_size(bar.center(), vec2(bar.width() * 0.15, bar.height())), 0.0, DEEP_GREEN);
                    p.rect_filled(Rect::from_center_size(bar.center(), vec2(bar.width() * 0.05, bar.height())), 0.0, GREEN);
                    let x = bar.min.x + bar.width() * f.cursor;
                    p.rect_filled(Rect::from_center_size(pos2(x, bar.center().y), vec2(5.0, bar.height() + 10.0)), 1.0, OFFWHITE);
                    p.text(bar.center_top() - vec2(0.0, 6.0), Align2::CENTER_BOTTOM, "VISEZ ! (clic ou Espace)", pixel(13.0), OFFWHITE);
                }
                // Boîte de dialogue.
                let dlg = Rect::from_min_max(pos2(rect.min.x + 10.0, rect.min.y + rect.height() * 0.66), pos2(rect.max.x - 10.0, rect.max.y - 10.0));
                p.rect_filled(dlg, 4.0, Color32::from_rgba_unmultiplied(16, 20, 18, 235));
                p.rect_stroke(dlg, 4.0, Stroke::new(2.0, DEEP_GREEN), egui::StrokeKind::Inside);
                p.text(dlg.min + vec2(12.0, 8.0), Align2::LEFT_TOP, game_content::hero::speaker(&f.speaker, &founder), pixel(13.0), GREEN);
                let galley = p.layout(f.text.clone(), egui::FontId::proportional(16.0), OFFWHITE, dlg.width() - 24.0);
                p.galley(dlg.min + vec2(12.0, 30.0), galley, OFFWHITE);
                if matches!(f.phase, Phase::Intro) {
                    p.text(dlg.right_bottom() - vec2(10.0, 8.0), Align2::RIGHT_BOTTOM, "Clic ou Entrée ▸", pixel(11.0), MUTED);
                }

                // --- commandes ------------------------------------------------
                ui.add_space(6.0);
                match f.phase {
                    Phase::Choose => {
                        ui.horizontal_wrapped(|ui| {
                            for (i, (name, help)) in MOVES.iter().enumerate() {
                                let left = match i {
                                    2 => format!(" ({})", f.mentors_left),
                                    3 => format!(" ({})", f.pill_left),
                                    _ => String::new(),
                                };
                                let ok = match i {
                                    2 => f.mentors_left > 0,
                                    3 => f.pill_left > 0,
                                    _ => true,
                                };
                                let r = ui.add_enabled(ok, egui::Button::new(egui::RichText::new(format!("{} · {name}{left}", i + 1)).size(15.0)).min_size(vec2(150.0, 34.0)))
                                    .on_hover_text(*help);
                                if ok && (r.clicked() || keys.0 == Some(i)) {
                                    f.chosen = i;
                                    match i {
                                        2 => f.mentors_left -= 1,
                                        3 => f.pill_left -= 1,
                                        _ => {}
                                    }
                                    f.phase = Phase::Aim;
                                    f.t = 0.0;
                                    f.speaker = "Narrateur".into();
                                    f.text = format!("« {name} » : visez au centre de la jauge !");
                                }
                            }
                        });
                        if ui.small_button("Abandonner et accepter le rachat").clicked() {
                            f.won = false;
                            f.phase = Phase::End;
                            f.speaker = b.name.to_string();
                            f.text = b.victory.to_string();
                        }
                    }
                    Phase::End => {
                        let label = if f.won { "Veridix reste indépendante ! Continuer" } else { "Le rachat est signé… Voir la fin" };
                        if super::widgets::big_button(ui, label, true, 420.0).clicked() {
                            resolve = Some(f.won);
                        }
                    }
                    _ => {
                        ui.label(egui::RichText::new(format!("{} : {}", b.company, b.title)).size(12.5).color(MUTED));
                    }
                }
            });
        });
    if let Some(won) = resolve {
        c.run(Command::ResolveTakeover { won });
        ctx.data_mut(|d| d.remove::<Fight>(id));
    } else {
        ctx.data_mut(|d| d.insert_temp(id, f));
    }
}

fn lerp(a: Color32, b: Color32, k: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

fn hp_bar(p: &egui::Painter, r: Rect, frac: f32, label: &str, col: Color32) {
    p.rect_filled(r.expand(2.0), 3.0, NEAR_BLACK);
    p.rect_filled(Rect::from_min_size(r.min, vec2(r.width() * frac.clamp(0.0, 1.0), r.height())), 2.0, col);
    // Étiquette sur fond sombre, lisible quel que soit le décor de l'arène.
    let galley = p.layout_no_wrap(label.to_string(), pixel(13.0), OFFWHITE);
    let bg = Rect::from_min_size(r.left_bottom() + vec2(0.0, 4.0), galley.size() + vec2(10.0, 4.0));
    p.rect_filled(bg, 2.0, Color32::from_black_alpha(190));
    p.galley(bg.min + vec2(5.0, 2.0), galley, OFFWHITE);
}

/// Dessine un personnage en pixel art : grille de 16 × 20 cases de `cell` pixels.
fn draw_character(p: &egui::Painter, o: Pos2, cell: f32, l: &Look, hit: bool, facing_left: bool) {
    let ink = Color32::from_rgb(20, 18, 16);
    let px = |x: i32, y: i32, w: i32, h: i32, c: Color32| {
        // Les personnages se font face : l'adversaire est dessiné en miroir.
        let x = if facing_left { 16 - x - w } else { x };
        let r = Rect::from_min_size(o + vec2(x as f32 * cell, y as f32 * cell), vec2(w as f32 * cell, h as f32 * cell));
        p.rect_filled(r, 0.0, if hit { lerp(c, OFFWHITE, 0.6) } else { c });
    };
    // Contours (une case de plus autour de la tête et du corps).
    px(3, 2, 10, 10, ink);
    px(1, 11, 14, 9, ink);
    // Corps, chemise, accessoire (cravate, badge…).
    px(2, 12, 12, 8, l.suit);
    px(6, 12, 4, 4, l.shirt);
    px(7, 13, 2, 4, l.accent);
    px(2, 16, 2, 4, lerp(l.suit, ink, 0.25));
    px(12, 16, 2, 4, lerp(l.suit, ink, 0.25));
    // Cou et tête.
    px(7, 11, 2, 1, l.skin);
    px(4, 3, 8, 8, l.skin);
    // Cheveux.
    match l.hair_style {
        1 => {
            px(4, 2, 8, 2, l.hair);
            px(3, 3, 2, 9, l.hair);
            px(11, 3, 2, 9, l.hair);
        }
        2 => {
            px(4, 2, 8, 2, l.hair);
            px(6, 0, 4, 2, l.hair);
            px(4, 4, 1, 2, l.hair);
            px(11, 4, 1, 2, l.hair);
        }
        3 => {
            px(3, 5, 1, 4, l.hair);
            px(12, 5, 1, 4, l.hair);
        }
        4 => {
            px(4, 2, 8, 2, l.hair);
            for i in 0..4 {
                px(4 + i * 2, 1, 1, 1, l.hair);
            }
            px(4, 4, 1, 1, l.hair);
        }
        5 => {
            px(3, 1, 10, 3, l.hair);
            px(2, 3, 2, 5, l.hair);
            px(12, 3, 2, 5, l.hair);
        }
        _ => {
            px(4, 2, 8, 2, l.hair);
            px(4, 4, 1, 2, l.hair);
            px(11, 4, 1, 2, l.hair);
        }
    }
    // Barbe.
    if l.beard {
        px(4, 9, 8, 2, l.hair);
        px(5, 11, 6, 1, l.hair);
    }
    // Yeux, lunettes ou casque de réalité virtuelle.
    if l.headset {
        px(4, 5, 8, 3, Color32::from_rgb(40, 40, 46));
        px(5, 6, 6, 1, l.accent);
    } else {
        px(6, 6, 1, 1, ink);
        px(9, 6, 1, 1, ink);
        if l.glasses {
            px(5, 5, 3, 1, ink);
            px(8, 5, 3, 1, ink);
            px(5, 7, 3, 1, ink);
            px(8, 7, 3, 1, ink);
        }
    }
    // Bouche (sourire en coin).
    px(6, 9, 3, 1, Color32::from_rgb(140, 60, 60));
    px(9, 8, 1, 1, Color32::from_rgb(140, 60, 60));
}
