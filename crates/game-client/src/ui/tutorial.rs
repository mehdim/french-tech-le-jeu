//! Tutoriel guidé des niveaux 1 et 2 : des bulles explicatives pointent vers les
//! éléments de l'interface et avancent quand le joueur a fait ce qu'elles demandent.
//!
//! Les panneaux enregistrent la position de leurs éléments clés avec [`mark`] ;
//! le tutoriel lit ces positions (de l'image en cours ou de la précédente).

use super::style::*;
use super::{Ctx, Tab};
use bevy_egui::egui::{self, Color32, FontId, Pos2, Rect, Stroke, pos2, vec2};
use game_core::employees::Assignment;
use game_core::types::Role;

/// Enregistre la position d'un élément de l'interface que le tutoriel peut désigner.
/// Seule la partie visible compte : un élément sorti de sa zone de défilement n'est pas désigné.
pub fn mark(ui: &egui::Ui, key: &str, rect: Rect) {
    let visible = rect.intersect(ui.clip_rect());
    if visible.height() < 12.0 || visible.width() < 12.0 {
        return;
    }
    let ctx = ui.ctx();
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(("tutorial_anchor", key)), (visible, pass)));
}

fn anchor(ctx: &egui::Context, key: &str) -> Option<Rect> {
    let pass = ctx.cumulative_pass_nr();
    ctx.data(|d| d.get_temp::<(Rect, u64)>(egui::Id::new(("tutorial_anchor", key))))
        .filter(|(_, p)| pass.saturating_sub(*p) <= 1)
        .map(|(r, _)| r)
}

/// Ce que le joueur doit faire pour passer à l'étape suivante.
enum Done {
    /// Un bouton « Compris ».
    Ok,
    /// Une condition sur la partie ou l'interface.
    When(fn(&Ctx) -> bool),
}

struct Step {
    /// L'étape attend que cette condition soit vraie pour s'afficher.
    show_when: fn(&Ctx) -> bool,
    title: &'static str,
    text: &'static str,
    /// Éléments à désigner, par ordre de préférence (le premier visible gagne).
    anchors: fn(&Ctx) -> Vec<String>,
    done: Done,
}

fn founder_id(c: &Ctx) -> Option<u32> {
    c.game.0.employees.iter().find(|e| e.role == Role::Founder).map(|e| e.id)
}

fn founder_assignment(c: &Ctx) -> Option<Assignment> {
    c.game.0.employees.iter().find(|e| e.role == Role::Founder).map(|e| e.assignment)
}

fn has_task(c: &Ctx, id: &str) -> bool {
    c.game.0.product.has(id) || c.game.0.tasks.iter().any(|t| t.task_id == id)
}

fn tab_or(c: &Ctx, tab: Tab, inner: &str) -> Vec<String> {
    if c.ui.tab == tab && c.ui.right_open { vec![inner.to_string(), format!("tab_{tab:?}")] } else { vec![format!("tab_{tab:?}"), "panels".into()] }
}

fn employee_anchor(c: &Ctx) -> Vec<String> {
    let card = founder_id(c).map(|id| format!("employee_{id}")).unwrap_or_default();
    tab_or(c, Tab::Team, &card)
}

fn always(_: &Ctx) -> bool {
    true
}

fn level2(c: &Ctx) -> bool {
    c.game.0.campaign.level >= 2
}

const STEPS: &[Step] = &[
    // ------------------------------------------------------------------ niveau 1
    Step {
        show_when: always,
        title: "Bienvenue, {nom} !",
        text: "Je vous accompagne pendant les deux premiers niveaux (vous pouvez couper le tutoriel à tout moment). À gauche, vos objectifs : survolez-les pour un indice.",
        anchors: |_| vec!["quests".into()],
        done: Done::Ok,
    },
    Step {
        show_when: always,
        title: "Le temps qui passe",
        text: "Le jeu avance jour après jour. ⏸ met en pause (Espace), 1× 2× 4× accélèrent. Mettez en pause dès que vous voulez réfléchir : rien ne presse… à part la trésorerie.",
        anchors: |_| vec!["speed".into()],
        done: Done::Ok,
    },
    Step {
        show_when: always,
        title: "Construire le MVP",
        text: "Ouvrez l'onglet « Produit & R&D » (touche P), puis cliquez sur « Lancer » à côté de « Détection de texte (MVP) », dans la feuille de route (faites défiler le panneau).",
        anchors: |c| tab_or(c, Tab::Product, "task_mvp_text"),
        done: Done::When(|c| has_task(c, "mvp_text")),
    },
    Step {
        show_when: always,
        title: "Au travail !",
        text: "Un chantier n'avance que si quelqu'un y travaille. Dans l'onglet « Équipe » (touche E), choisissez le chantier MVP dans votre menu d'affectation.",
        anchors: employee_anchor,
        done: Done::When(|c| matches!(founder_assignment(c), Some(Assignment::Task(_))) || c.game.0.product.has("mvp_text")),
    },
    Step {
        show_when: always,
        title: "France Travail",
        text: "Ouvrez l'onglet « Financement » (touche F) et cliquez sur « Constituer le dossier » pour le Dossier créateur d'entreprise. Il apporte l'accompagnement et l'ARE.",
        anchors: |c| tab_or(c, Tab::Funding, "program_france_travail"),
        done: Done::When(|c| c.game.0.funding.applications.iter().any(|a| a.program == "france_travail") || c.game.0.funding.approved.iter().any(|p| p == "france_travail")),
    },
    Step {
        show_when: always,
        title: "La paperasse",
        text: "Un dossier avance avec des points « Administratif ». Chaque personne ne fait qu'une chose par jour : alternez entre le MVP et l'administratif dans votre menu d'affectation. Votre attention est la ressource la plus rare de Veridix !",
        anchors: employee_anchor,
        done: Done::Ok,
    },
    Step {
        show_when: always,
        title: "Accélérez",
        text: "Tout est lancé. Passez en 2× ou 4× et regardez les barres de progression avancer, à gauche et dans les panneaux.",
        anchors: |_| vec!["speed".into()],
        done: Done::When(|c| c.clock.speed >= 2),
    },
    Step {
        show_when: |c| c.game.0.product.has("mvp_text"),
        title: "MVP livré !",
        text: "Bravo ! Il vous faut maintenant 3 bêta-testeurs : affectez-vous à « Ventes » ou « Marketing » dans l'onglet Équipe. Et n'oubliez pas le dossier France Travail.",
        anchors: employee_anchor,
        done: Done::When(|c| matches!(founder_assignment(c), Some(Assignment::Sales | Assignment::Marketing)) || c.game.0.metrics.customers >= 3),
    },
    Step {
        show_when: always,
        title: "Vos mentors",
        text: "Mehdi, Steeve et Jean-Baptiste, d'À la French, vous conseillent : onglet « Mentors » (touche M). Chaque conseil se recharge au fil des jours.",
        anchors: |_| vec!["tab_Mentors".into(), "panels".into()],
        done: Done::When(|c| c.ui.tab == Tab::Mentors && c.ui.right_open),
    },
    Step {
        show_when: always,
        title: "La trésorerie",
        text: "En haut : trésorerie, MRR (revenu mensuel) et runway (mois avant la faillite). Gardez plus de 5 000 € en banque : c'est un objectif du niveau. Une aide accordée n'est pas une aide versée !",
        anchors: |_| vec!["runway".into()],
        done: Done::Ok,
    },
    // ------------------------------------------------------------------ niveau 2
    Step {
        show_when: |c| level2(c) && !c.game.0.campaign.intro_pending,
        title: "Niveau 2 : recruter",
        text: "Un vrai bureau ! Dans l'onglet « Équipe », section Recrutement (en bas du panneau) : choisissez un profil (Backend, par exemple) puis un canal, et cliquez sur « Lancer ». LinkedIn est rapide, France Travail gratuit mais lent.",
        anchors: |c| tab_or(c, Tab::Team, "recruit"),
        done: Done::When(|c| !c.game.0.searches.is_empty() || !c.game.0.candidates.is_empty() || c.game.0.employees.len() >= 2),
    },
    Step {
        show_when: level2,
        title: "Embaucher",
        text: "Les candidats arrivent au bout de quelques jours, en bas de l'onglet Équipe. Comparez le salaire et les compétences, puis cliquez sur « Embaucher ».",
        anchors: |c| tab_or(c, Tab::Team, "hire"),
        done: Done::When(|c| c.game.0.employees.len() >= 2),
    },
    Step {
        show_when: level2,
        title: "Le lancement",
        text: "Pour faire payer vos clients : dans « Produit & R&D », livrez le Dashboard, puis le « Lancement public ». Pensez à y affecter votre nouvelle recrue.",
        anchors: |c| tab_or(c, Tab::Product, if c.game.0.product.has("mvp_dashboard") { "task_launch" } else { "task_mvp_dashboard" }),
        done: Done::When(|c| has_task(c, "launch")),
    },
    Step {
        show_when: level2,
        title: "Vendre",
        text: "Onglet « Ventes » (touche V) : prix, budget marketing et marchés. MRR = clients payants × prix mensuel. Objectif du niveau : 500 € de MRR.",
        anchors: |_| vec!["tab_Sales".into(), "panels".into()],
        done: Done::When(|c| c.ui.tab == Tab::Sales && c.ui.right_open),
    },
    Step {
        show_when: level2,
        title: "À vous de jouer",
        text: "Vous savez l'essentiel. La suite : subventions, levées de fonds, Station F, la Chine, Davos… Bonne chance, {nom}. Et pensez à mettre un cône de chantier dans le bureau.",
        anchors: |_| vec![],
        done: Done::Ok,
    },
];

/// Affiche la bulle de l'étape en cours (sans rien faire si le tutoriel est fini ou désactivé).
pub fn show(ctx: &egui::Context, c: &mut Ctx) {
    let Some(mut idx) = c.game.0.tutorial.map(|i| i as usize) else { return };
    let g = &c.game.0;
    // Pas de bulle par-dessus une scène d'histoire, un événement ou un combat.
    if g.campaign.intro_pending || g.events.pending.is_some() || g.takeover.as_ref().is_some_and(|t| t.ready) {
        return;
    }
    // Les étapes déjà accomplies (le joueur a pris de l'avance) sont sautées d'office.
    while let Some(step) = STEPS.get(idx) {
        match step.done {
            Done::When(f) if (step.show_when)(c) && f(c) => idx += 1,
            _ => break,
        }
    }
    if g.campaign.level > 2 && idx < STEPS.len() {
        idx = STEPS.len();
    }
    if idx >= STEPS.len() {
        c.game.0.tutorial = None;
        return;
    }
    c.game.0.tutorial = Some(idx as u32);
    let step = &STEPS[idx];
    if !(step.show_when)(c) {
        return;
    }
    let target = (step.anchors)(c).iter().find_map(|k| anchor(ctx, k));
    let founder = c.game.0.founder.clone();
    let title = game_content::hero::personalize(step.title, &founder);
    let text = game_content::hero::personalize(step.text, &founder);

    let screen = ctx.content_rect();
    let w = 330.0_f32.min(screen.width() - 32.0);
    // Taille de la bulle : mesurée sur le texte.
    let body = ctx.fonts_mut(|f| f.layout(text.clone(), FontId::proportional(14.5), OFFWHITE, w - 24.0));
    let h = body.size().y + 86.0;
    let size = vec2(w, h);
    let pos = place(screen, target, size);
    let bubble = Rect::from_min_size(pos, size);

    let t = ctx.input(|i| i.time) as f32;
    let layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("tutorial_layer"));
    let p = ctx.layer_painter(layer);
    if let Some(r) = target {
        // Contour qui pulse autour de l'élément désigné, et flèche depuis la bulle.
        let pulse = (t * 4.0).sin() * 0.5 + 0.5;
        let col = Color32::from_rgba_unmultiplied(124, 255, 82, (140.0 + 115.0 * pulse) as u8);
        p.rect_stroke(r.expand(3.0 + 2.0 * pulse), 4.0, Stroke::new(2.5, col), egui::StrokeKind::Outside);
        let from = closest_point(bubble, r.center());
        let to = closest_point(r.expand(6.0), from);
        p.line_segment([from, to], Stroke::new(3.0, GREEN));
        let dir = (to - from).normalized();
        let n = vec2(-dir.y, dir.x);
        p.add(egui::Shape::convex_polygon(vec![to, to - dir * 12.0 + n * 7.0, to - dir * 12.0 - n * 7.0], GREEN, Stroke::NONE));
    }

    let mut skip = false;
    let mut ok = false;
    egui::Area::new(egui::Id::new("tutorial_bubble"))
        .order(egui::Order::Foreground)
        .fixed_pos(pos)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(18, 30, 22))
                .stroke(Stroke::new(2.0, GREEN))
                .corner_radius(8.0)
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(w - 24.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("TUTORIEL {}/{}", idx + 1, STEPS.len())).font(pixel(10.0)).color(MUTED));
                    });
                    ui.label(egui::RichText::new(title).font(pixel(15.0)).color(GREEN));
                    ui.add(egui::Label::new(egui::RichText::new(text).size(14.5).color(OFFWHITE)).wrap());
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if matches!(step.done, Done::Ok) {
                            if ui.button(egui::RichText::new("Compris ▸").color(GREEN).strong()).clicked() {
                                ok = true;
                            }
                        } else {
                            ui.label(egui::RichText::new("⟳ à vous de jouer…").size(12.5).italics().color(LIGHTGRAY));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button(egui::RichText::new("Passer le tutoriel").size(11.5).color(MUTED)).clicked() {
                                skip = true;
                            }
                        });
                    });
                });
        });
    if ok {
        *c.click += 1;
        c.game.0.tutorial = Some(idx as u32 + 1);
    }
    if skip {
        c.game.0.tutorial = None;
        c.toasts.push_good("Tutoriel désactivé. L'aide reste disponible avec « ? ».");
    }
}

/// Place la bulle à côté de l'élément désigné, du côté où il y a de la place.
fn place(screen: Rect, target: Option<Rect>, size: egui::Vec2) -> Pos2 {
    let margin = 16.0;
    let clamp = |p: Pos2| {
        pos2(
            p.x.clamp(screen.min.x + margin, (screen.max.x - size.x - margin).max(screen.min.x + margin)),
            p.y.clamp(screen.min.y + margin, (screen.max.y - size.y - margin).max(screen.min.y + margin)),
        )
    };
    let Some(r) = target else {
        return clamp(pos2(screen.center().x - size.x / 2.0, screen.max.y - size.y - 150.0));
    };
    let gap = 40.0;
    // Barre du haut : en dessous. Panneau de droite : à gauche. Panneau de gauche : à droite.
    if r.max.y < screen.min.y + 70.0 {
        return clamp(pos2(r.center().x - size.x / 2.0, r.max.y + gap));
    }
    if r.center().x > screen.center().x {
        return clamp(pos2(r.min.x - size.x - gap, r.center().y - size.y / 2.0));
    }
    clamp(pos2(r.max.x + gap, r.center().y - size.y / 2.0))
}

fn closest_point(r: Rect, p: Pos2) -> Pos2 {
    pos2(p.x.clamp(r.min.x, r.max.x), p.y.clamp(r.min.y, r.max.y))
}
