//! Musique et effets sonores. Tout l'audio est généré par `tools/audio/gen_audio.py`.

use crate::saves::Settings;
use crate::sim::Cues;
use crate::states::AppState;
use bevy::audio::Volume;
use bevy::prelude::*;
use game_core::messages::Cue;

#[derive(Resource)]
struct Sounds {
    music_office: Handle<AudioSource>,
    music_menu: Handle<AudioSource>,
    cash: Handle<AudioSource>,
    alert: Handle<AudioSource>,
    typing: Handle<AudioSource>,
    coffee: Handle<AudioSource>,
    funding: Handle<AudioSource>,
    fail: Handle<AudioSource>,
    click: Handle<AudioSource>,
    level: Handle<AudioSource>,
    popup: Handle<AudioSource>,
    hire: Handle<AudioSource>,
    incident: Handle<AudioSource>,
}

#[derive(Component)]
struct Music(&'static str);

/// Le code d'interface demande un son de clic via cette ressource.
#[derive(Resource, Default)]
pub struct UiSound(pub u32);

#[derive(Resource, Default)]
struct Throttle {
    cash: f32,
    typing: f32,
}

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiSound>()
            .init_resource::<Throttle>()
            .add_systems(Startup, load)
            .add_systems(Update, (music, cue_sounds, ui_clicks, apply_volume));
    }
}

fn load(mut commands: Commands, server: Res<AssetServer>) {
    let l = |n: &str| server.load(format!("audio/{n}.ogg"));
    commands.insert_resource(Sounds {
        music_office: l("music_office"),
        music_menu: l("music_menu"),
        cash: l("sfx_cash"),
        alert: l("sfx_alert"),
        typing: l("sfx_typing"),
        coffee: l("sfx_coffee"),
        funding: l("sfx_funding"),
        fail: l("sfx_fail"),
        click: l("sfx_click"),
        level: l("sfx_level"),
        popup: l("sfx_popup"),
        hire: l("sfx_hire"),
        incident: l("sfx_incident"),
    });
}

fn music(
    mut commands: Commands,
    state: Res<State<AppState>>,
    sounds: Option<Res<Sounds>>,
    settings: Res<Settings>,
    current: Query<(Entity, &Music)>,
) {
    let Some(sounds) = sounds else { return };
    let want = match state.get() {
        AppState::MainMenu | AppState::Victory | AppState::GameOver => "menu",
        _ => "office",
    };
    if current.iter().any(|(_, m)| m.0 == want) {
        return;
    }
    for (e, _) in &current {
        commands.entity(e).despawn();
    }
    let handle = if want == "menu" { sounds.music_menu.clone() } else { sounds.music_office.clone() };
    commands.spawn((
        AudioPlayer::new(handle),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(settings.music_volume * 0.6)),
        Music(want),
    ));
}

fn play(commands: &mut Commands, h: &Handle<AudioSource>, vol: f32) {
    if vol <= 0.0 {
        return;
    }
    commands.spawn((AudioPlayer::new(h.clone()), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(vol))));
}

fn cue_sounds(
    mut commands: Commands,
    cues: Res<Cues>,
    sounds: Option<Res<Sounds>>,
    settings: Res<Settings>,
    time: Res<Time>,
    mut throttle: ResMut<Throttle>,
    state: Res<State<AppState>>,
) {
    let Some(s) = sounds else { return };
    if *state.get() == AppState::MainMenu {
        return; // l'entreprise du mode démo reste silencieuse
    }
    let v = settings.sfx_volume;
    throttle.cash -= time.delta_secs();
    throttle.typing -= time.delta_secs();
    for cue in &cues.0 {
        match cue {
            Cue::NewCustomers(_) if throttle.cash <= 0.0 => {
                play(&mut commands, &s.cash, v * 0.6);
                throttle.cash = 1.2;
            }
            Cue::Shipped(_) if throttle.typing <= 0.0 => {
                play(&mut commands, &s.typing, v * 0.7);
                throttle.typing = 1.0;
            }
            Cue::Incident => play(&mut commands, &s.incident, v * 0.6),
            Cue::Funding(_) => play(&mut commands, &s.funding, v),
            Cue::Alert => play(&mut commands, &s.alert, v * 0.7),
            Cue::Hired(_) => play(&mut commands, &s.hire, v * 0.7),
            Cue::LevelComplete | Cue::Victory => play(&mut commands, &s.level, v),
            Cue::EventPopup => play(&mut commands, &s.popup, v * 0.8),
            Cue::Bankrupt => play(&mut commands, &s.fail, v),
            Cue::Coffee => play(&mut commands, &s.coffee, v * 0.25),
            _ => {}
        }
    }
}

fn ui_clicks(mut commands: Commands, mut ui: ResMut<UiSound>, sounds: Option<Res<Sounds>>, settings: Res<Settings>) {
    if ui.0 == 0 {
        return;
    }
    if let Some(s) = sounds {
        play(&mut commands, &s.click, settings.sfx_volume * 0.5);
    }
    ui.0 = 0;
}

fn apply_volume(settings: Res<Settings>, mut q: Query<&mut AudioSink, With<Music>>) {
    if !settings.is_changed() {
        return;
    }
    for mut sink in &mut q {
        sink.set_volume(Volume::Linear(settings.music_volume * 0.6));
    }
}
