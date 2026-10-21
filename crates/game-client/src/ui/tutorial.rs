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

/// Demande au panneau qui affiche `key` de le faire défiler jusqu'à lui (une fois).
pub fn request_scroll(ctx: &egui::Context, key: &str) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("tutorial_scroll"), key.to_string()));
}

/// Vrai (une seule fois) si une bulle a demandé de faire défiler jusqu'à `key`.
pub fn take_scroll(ctx: &egui::Context, key: &str) -> bool {
    let id = egui::Id::new("tutorial_scroll");
    let hit = ctx.data(|d| d.get_temp::<String>(id)).is_some_and(|k| k == key);
    if hit {
        ctx.data_mut(|d| d.remove::<String>(id));
    }
    hit
}

/// Section de chantiers qu'une bulle veut garder ouverte (clé `cat_…`).
pub fn wants_open(ctx: &egui::Context, key: &str) -> bool {
    let pass = ctx.cumulative_pass_nr();
    ctx.data(|d| d.get_temp::<(String, u64)>(egui::Id::new("tutorial_open")))
        .is_some_and(|(k, p)| k == key && pass.saturating_sub(p) <= 1)
}

fn keep_open(ctx: &egui::Context, key: &str) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| d.insert_temp(egui::Id::new("tutorial_open"), (key.to_string(), pass)));
}

/// Fait défiler une seule fois par bulle (et non à chaque image).
fn scroll_once(ctx: &egui::Context, bubble: &str, key: &str) {
    let id = egui::Id::new("tutorial_scrolled");
    if ctx.data(|d| d.get_temp::<String>(id)).as_deref() != Some(bubble) {
        ctx.data_mut(|d| d.insert_temp(id, bubble.to_string()));
        request_scroll(ctx, key);
    }
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
    /// Chantier vers lequel faire défiler le panneau Produit (clé `task_…`).
    scroll: Option<&'static str>,
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
        text: "C'est moi qui t'accompagne pendant les deux premiers niveaux (tu peux couper le tutoriel à tout moment). À gauche, tes objectifs : survole-les pour un indice.",
        anchors: |_| vec!["quests".into()],
        done: Done::Ok,
        scroll: None,
    },
    Step {
        show_when: always,
        title: "Le temps qui passe",
        text: "Le jeu avance jour après jour. ⏸ met en pause (Espace), 1× 2× 4× accélèrent. Mets en pause dès que tu veux réfléchir : rien ne presse… à part la trésorerie.",
        anchors: |_| vec!["speed".into()],
        done: Done::Ok,
        scroll: None,
    },
    Step {
        show_when: always,
        title: "Construire le MVP",
        text: "Ouvre l'onglet « Produit & R&D » (touche P). Je fais défiler jusqu'à la section « MVP », dans la feuille de route : clique sur le bouton vert « Lancer » de la carte « Détection de texte (MVP) ».",
        anchors: |c| {
            let mut v = tab_or(c, Tab::Product, "launch_mvp_text");
            if c.ui.tab == Tab::Product {
                v.insert(1, "task_mvp_text".into());
            }
            v
        },
        done: Done::When(|c| has_task(c, "mvp_text")),
        scroll: Some("task_mvp_text"),
    },
    Step {
        show_when: always,
        title: "Affecte-toi au MVP",
        text: "Le chantier est lancé… mais personne n'y travaille ! Dans l'onglet « Équipe » (touche E), clique sur le menu « Sans affectation » sous ton nom, puis choisis « Détection de texte (MVP) ». Astuce : cliquer sur ton personnage dans le bureau ouvre aussi sa fiche.",
        anchors: |c| {
            let assign = founder_id(c).map(|id| format!("assign_{id}")).unwrap_or_default();
            let mut v = tab_or(c, Tab::Team, &assign);
            if c.ui.tab == Tab::Team {
                v.insert(1, founder_id(c).map(|id| format!("employee_{id}")).unwrap_or_default());
            }
            v
        },
        done: Done::When(|c| matches!(founder_assignment(c), Some(Assignment::Task(_))) || c.game.0.product.has("mvp_text")),
        scroll: None,
    },
    Step {
        show_when: always,
        title: "France Travail",
        text: "Ouvre l'onglet « Financement » (touche F) et clique sur « Constituer le dossier » pour le Dossier créateur d'entreprise. Accompagnement et ARE : ne crache pas dessus.",
        anchors: |c| tab_or(c, Tab::Funding, "program_france_travail"),
        done: Done::When(|c| c.game.0.funding.applications.iter().any(|a| a.program == "france_travail") || c.game.0.funding.approved.iter().any(|p| p == "france_travail")),
        scroll: None,
    },
    Step {
        show_when: always,
        title: "La paperasse",
        text: "Un dossier avance avec des points « Administratif ». Chacun ne fait qu'une chose par jour : alterne entre le MVP et l'administratif dans ton menu d'affectation. Ton attention est la ressource la plus rare de Veridix !",
        anchors: employee_anchor,
        done: Done::Ok,
        scroll: None,
    },
    Step {
        show_when: always,
        title: "Accélère",
        text: "Tout est lancé. Passe en 2× ou 4× et regarde les barres de progression avancer, à gauche et dans les panneaux.",
        anchors: |_| vec!["speed".into()],
        done: Done::When(|c| c.clock.speed >= 2),
        scroll: None,
    },
    Step {
        show_when: |c| c.game.0.product.has("mvp_text"),
        title: "MVP livré !",
        text: "Bravo ! Il te faut maintenant 3 bêta-testeurs : affecte-toi à « Ventes » ou « Marketing » dans l'onglet Équipe. Et n'oublie pas le dossier France Travail.",
        anchors: employee_anchor,
        done: Done::When(|c| matches!(founder_assignment(c), Some(Assignment::Sales | Assignment::Marketing)) || c.game.0.metrics.customers >= 3),
        scroll: None,
    },
    Step {
        show_when: always,
        title: "Tes mentors",
        text: "On est trois à te conseiller, avec les deux autres d'À la French : onglet « Mentors » (touche M). Chaque conseil se recharge au fil des jours, le mien deux fois plus vite.",
        anchors: |_| vec!["tab_Mentors".into(), "panels".into()],
        done: Done::When(|c| c.ui.tab == Tab::Mentors && c.ui.right_open),
        scroll: None,
    },
    Step {
        show_when: always,
        title: "La trésorerie",
        text: "En haut : trésorerie, MRR (revenu mensuel) et runway (mois avant la faillite). Garde plus de 5 000 € en banque : c'est un objectif du niveau. Une aide accordée n'est pas une aide versée !",
        anchors: |_| vec!["runway".into()],
        done: Done::Ok,
        scroll: None,
    },
    // ------------------------------------------------------------------ niveau 2
    Step {
        show_when: |c| level2(c) && !c.game.0.campaign.intro_pending,
        title: "Niveau 2 : recruter",
        text: "Un vrai bureau ! Dans l'onglet « Équipe », section Recrutement (en bas du panneau) : choisis un profil (Backend, par exemple) puis un canal, et clique sur « Lancer ». LinkedIn est rapide, France Travail gratuit mais lent.",
        anchors: |c| tab_or(c, Tab::Team, "recruit"),
        done: Done::When(|c| !c.game.0.searches.is_empty() || !c.game.0.candidates.is_empty() || c.game.0.employees.len() >= 2),
        scroll: None,
    },
    Step {
        show_when: level2,
        title: "Embaucher",
        text: "Les candidats arrivent au bout de quelques jours, en bas de l'onglet Équipe. Compare salaire et compétences, puis clique sur « Embaucher ».",
        anchors: |c| tab_or(c, Tab::Team, "hire"),
        done: Done::When(|c| c.game.0.employees.len() >= 2),
        scroll: None,
    },
    Step {
        show_when: level2,
        title: "Le lancement",
        text: "Pour faire payer tes clients : dans « Produit & R&D », livre le Dashboard, puis le « Lancement public ». Affecte ta nouvelle recrue dessus.",
        anchors: |c| tab_or(c, Tab::Product, if c.game.0.product.has("mvp_dashboard") { "task_launch" } else { "task_mvp_dashboard" }),
        done: Done::When(|c| has_task(c, "launch")),
        scroll: Some("task_mvp_dashboard"),
    },
    Step {
        show_when: level2,
        title: "Vendre",
        text: "Onglet « Ventes » (touche V) : prix, budget marketing et marchés. MRR = clients payants × prix mensuel. Objectif du niveau : 500 € de MRR.",
        anchors: |_| vec!["tab_Sales".into(), "panels".into()],
        done: Done::When(|c| c.ui.tab == Tab::Sales && c.ui.right_open),
        scroll: None,
    },
    Step {
        show_when: level2,
        title: "À toi de jouer",
        text: "Tu sais l'essentiel. La suite : subventions, levées de fonds, Station F, la Chine, Davos… Bonne chance, {nom}. Et pense à mettre un cône de chantier dans le bureau.",
        anchors: |_| vec![],
        done: Done::Ok,
        scroll: None,
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
        // Les chantiers des niveaux 1 et 2 ont été présentés par le tutoriel.
        c.game.0.tasks_seen_level = c.game.0.tasks_seen_level.max(c.game.0.campaign.level);
        return;
    }
    c.game.0.tutorial = Some(idx as u32);
    let step = &STEPS[idx];
    if !(step.show_when)(c) {
        return;
    }
    if let Some(key) = step.scroll
        && c.ui.tab == Tab::Product
        && c.ui.right_open
    {
        scroll_once(ctx, &format!("step{idx}"), key);
    }
    let target = (step.anchors)(c).iter().find_map(|k| anchor(ctx, k));
    let founder = c.game.0.founder.clone();
    let title = game_content::hero::personalize(step.title, &founder);
    let mut text = game_content::hero::personalize(step.text, &founder);
    let mentor_id = c.game.0.mentors.main.clone().unwrap_or_else(|| "alf".into());
    if idx + 1 == STEPS.len() {
        text.push(' ');
        text.push_str(match mentor_id.as_str() {
            "mehdi" => "Et rappelle-toi : si ça n'a pas d'API, ça n'existe pas.",
            "steeve" => "Et montre-moi ta facture GPU de temps en temps. La vraie.",
            "jb" => "Et publie ton code. Le cône de chantier, c'est cadeau.",
            _ => "",
        });
    }
    let header = format!("TUTORIEL {}/{}", idx + 1, STEPS.len());
    let (ok, skip) = bubble(ctx, c, target, &header, &title, &text, matches!(step.done, Done::Ok), "Passer le tutoriel");
    if ok {
        *c.click += 1;
        c.game.0.tutorial = Some(idx as u32 + 1);
    }
    if skip {
        c.game.0.tutorial = None;
        c.game.0.tasks_seen_level = c.game.0.campaign.level;
        c.toasts.push_good("Tutoriel désactivé. L'aide reste disponible avec « ? ».");
    }
}

/// À chaque nouveau niveau, une bulle présente les chantiers qu'il débloque : la
/// section du panneau Produit où ils apparaissent, et le premier à lancer.
pub fn new_tasks(ctx: &egui::Context, c: &mut Ctx) {
    let g = &c.game.0;
    if g.tutorial.is_some() || g.tasks_seen_level >= g.campaign.level {
        return;
    }
    if g.campaign.intro_pending || g.events.pending.is_some() || g.takeover.as_ref().is_some_and(|t| t.ready) {
        return;
    }
    let level = g.campaign.level;
    let unlocks = c.content.0.level(level).map(|l| l.unlocks.clone()).unwrap_or_default();
    let fresh: Vec<game_core::content::TaskDef> = c
        .content
        .0
        .tasks
        .iter()
        .filter(|t| t.requires_unlock.as_ref().is_some_and(|u| unlocks.contains(u)))
        .filter(|t| {
            !matches!(
                game_core::research::task_availability(t, &g.product, &g.campaign.unlocks, &g.tasks, g.finance.cash),
                Err(game_core::research::TaskBlock::Locked(_)) | Err(game_core::research::TaskBlock::AlreadyShipped)
            )
        })
        .cloned()
        .collect();
    if fresh.is_empty() {
        c.game.0.tasks_seen_level = level;
        return;
    }
    // Les catégories concernées, dans l'ordre du panneau ; la bulle désigne la première.
    let mut cats: Vec<game_core::content::TaskCategory> = Vec::new();
    for t in &fresh {
        if !cats.contains(&t.category) {
            cats.push(t.category);
        }
    }
    let first = fresh.iter().find(|t| t.category == cats[0]).map(|t| t.id.clone()).unwrap_or_default();
    let cat_key = format!("cat_{:?}", cats[0]);
    let on_product = c.ui.tab == Tab::Product && c.ui.right_open;
    if on_product {
        keep_open(ctx, &cat_key);
        scroll_once(ctx, &format!("level{level}"), &format!("task_{first}"));
    }
    let anchors: Vec<String> = if on_product {
        vec![format!("launch_{first}"), format!("task_{first}"), cat_key.clone(), "tab_Product".into()]
    } else {
        vec!["tab_Product".into(), "panels".into()]
    };
    let target = anchors.iter().find_map(|k| anchor(ctx, k));
    let labels: Vec<&str> = cats.iter().map(|k| super::product::category_label(*k)).collect();
    let mut names: Vec<String> = fresh.iter().take(5).map(|t| format!("• {}", t.name)).collect();
    if fresh.len() > 5 {
        names.push(format!("… et {} autres", fresh.len() - 5));
    }
    let lead = if on_product {
        "Ils sont dans la feuille de route, sections : ".to_string()
    } else {
        "Ouvre l'onglet « Produit & R&D » (touche P) pour les voir, sections : ".to_string()
    };
    let text = format!("{lead}{}.\n{}", labels.join(", "), names.join("\n"));
    let title = if fresh.len() == 1 { "Nouveau chantier débloqué !".to_string() } else { format!("{} nouveaux chantiers débloqués !", fresh.len()) };
    let header = format!("NIVEAU {level}");
    let (ok, skip) = bubble(ctx, c, target, &header, &title, &text, true, "Ne plus afficher");
    if ok || skip {
        *c.click += 1;
        c.game.0.tasks_seen_level = level;
    }
    if skip {
        c.game.0.tasks_seen_level = u32::MAX;
    }
}

/// Dessine une bulle d'aide (portrait du mentor principal, titre, texte) et sa flèche.
/// Renvoie (Compris, Passer).
#[allow(clippy::too_many_arguments)]
fn bubble(ctx: &egui::Context, c: &Ctx, target: Option<Rect>, header: &str, title: &str, text: &str, ok_button: bool, skip_label: &str) -> (bool, bool) {
    let mentor_id = c.game.0.mentors.main.clone().unwrap_or_else(|| "alf".into());
    let mentor_name = c.content.0.mentor(&mentor_id).map(|m| m.name.clone()).unwrap_or_else(|| "À la French".into());
    let screen = ctx.content_rect();
    let w = 400.0_f32.min(screen.width() - 32.0);
    const FACE: f32 = 64.0;
    let body = ctx.fonts_mut(|f| f.layout(text.to_string(), FontId::proportional(14.5), OFFWHITE, w - 24.0 - FACE - 10.0));
    let h = (body.size().y + 86.0).max(FACE + 60.0);
    let size = vec2(w, h);
    let pos = place(screen, target, size);
    let rect = Rect::from_min_size(pos, size);

    let t = ctx.input(|i| i.time) as f32;
    let layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("tutorial_layer"));
    let p = ctx.layer_painter(layer);
    if let Some(r) = target {
        // Contour qui pulse autour de l'élément désigné, et flèche depuis la bulle.
        let pulse = (t * 4.0).sin() * 0.5 + 0.5;
        let col = Color32::from_rgba_unmultiplied(124, 255, 82, (140.0 + 115.0 * pulse) as u8);
        p.rect_stroke(r.expand(3.0 + 2.0 * pulse), 4.0, Stroke::new(2.5, col), egui::StrokeKind::Outside);
        let from = closest_point(rect, r.center());
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
                    ui.horizontal_top(|ui| {
                        ui.vertical(|ui| {
                            ui.set_width(FACE);
                            super::widgets::portrait(ui, c.tex, c.atlas, &mentor_id, FACE);
                            ui.label(egui::RichText::new(&mentor_name).font(pixel(10.0)).color(GREEN));
                        });
                        ui.vertical(|ui| {
                            ui.set_width(w - 24.0 - FACE - 10.0);
                            ui.label(egui::RichText::new(header).font(pixel(10.0)).color(MUTED));
                            ui.label(egui::RichText::new(title).font(pixel(15.0)).color(GREEN));
                            ui.add(egui::Label::new(egui::RichText::new(text).size(14.5).color(OFFWHITE)).wrap());
                        });
                    });
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ok_button {
                            if ui.button(egui::RichText::new("Compris ▸").color(GREEN).strong()).clicked() {
                                ok = true;
                            }
                        } else {
                            ui.label(egui::RichText::new("⟳ à toi de jouer…").size(12.5).italics().color(LIGHTGRAY));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button(egui::RichText::new(skip_label).size(11.5).color(MUTED)).clicked() {
                                skip = true;
                            }
                        });
                    });
                });
        });
    (ok, skip)
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
