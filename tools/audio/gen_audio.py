"""Airs traditionnels français revisités en 8-bit moderne, et effets sonores originaux.

Synthèse en numpy pur -> WAV -> Ogg Vorbis (ffmpeg). Tout est généré
de zéro : le jeu n'embarque aucun audio tiers.
Usage : python3 gen_audio.py <dossier_de_sortie>
"""
import os
import subprocess
import sys
import tempfile
import wave

import numpy as np

SR = 32000


def note(n):
    """Note MIDI -> Hz."""
    return 440.0 * 2 ** ((n - 69) / 12)


def t_axis(dur):
    return np.arange(int(SR * dur)) / SR


def square(f, dur, duty=0.5):
    t = t_axis(dur)
    return np.where((t * f) % 1 < duty, 1.0, -1.0)


def tri(f, dur):
    t = t_axis(dur)
    return 2 * np.abs(2 * ((t * f) % 1) - 1) - 1


def saw(f, dur):
    t = t_axis(dur)
    return 2 * ((t * f) % 1) - 1


def noise(dur, seed=0):
    return np.random.default_rng(seed).uniform(-1, 1, int(SR * dur))


def env(x, a=0.005, d=0.05, s=0.6, r=0.05):
    n = len(x)
    e = np.ones(n) * s
    ai, di, ri = int(a * SR), int(d * SR), int(r * SR)
    ai = min(ai, n)
    e[:ai] = np.linspace(0, 1, ai) if ai else e[:ai]
    if di and ai + di < n:
        e[ai:ai + di] = np.linspace(1, s, di)
    if ri and ri < n:
        e[-ri:] *= np.linspace(1, 0, ri)
    return x * e


def lowpass(x, alpha):
    y = np.zeros_like(x)
    acc = 0.0
    for i, v in enumerate(x):
        acc += alpha * (v - acc)
        y[i] = acc
    return y


def mix_into(buf, sig, start):
    i = int(start * SR)
    end = min(len(buf), i + len(sig))
    buf[i:end] += sig[: end - i]


def write(path, x, vol=0.8):
    x = np.clip(x / max(1e-9, np.max(np.abs(x))) * vol, -1, 1)
    pcm = (x * 32767).astype(np.int16)
    with tempfile.NamedTemporaryFile(suffix=".wav", delete=False) as tmp:
        wav_path = tmp.name
    with wave.open(wav_path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(pcm.tobytes())
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-i", wav_path, "-c:a", "libvorbis", "-q:a", "3", path], check=True)
    os.unlink(wav_path)


# --------------------------------------------------------------- musique
# Airs traditionnels français (domaine public) revisités en 8-bit moderne :
# ondes carrées façon console, basse triangle, batterie en bruit blanc, et
# pour le menu des accords filtrés qui « pompent » façon French touch.

C4 = 60


def pulse_note(n, dur, duty=0.5, vol=0.15, a=0.004, d=0.06, s=0.55, r=0.03):
    return env(square(note(n), dur, duty), a, d, s, r) * vol


def tri_note(n, dur, vol=0.4):
    return env(tri(note(n), dur), 0.003, 0.05, 0.75, 0.02) * vol


def play_line(buf, line, t0, beat, voice, gap=0.9):
    """Joue une mélodie [(note MIDI ou None, durée en temps)] à partir de t0."""
    t = t0
    for n, d in line:
        if n is not None:
            mix_into(buf, voice(n, d * beat * gap), t)
        t += d * beat
    return t


def drums(buf, bar_t0, beat, bar, kick=(0, 2), snare=(1, 3), hats=8, vol=1.0, seed=0):
    for k in range(hats):
        mix_into(buf, env(noise(0.03, seed + bar * 16 + k), 0.001, 0.02, 0.0, 0.0) * (0.10 if k % 2 else 0.05) * vol,
                 bar_t0 + k * 4 * beat / hats)
    for k in kick:
        t = t_axis(0.14)
        mix_into(buf, np.sin(2 * np.pi * (105 * t - 260 * t * t)) * np.exp(-t * 26) * 0.6 * vol, bar_t0 + k * beat)
    for k in snare:
        mix_into(buf, env(noise(0.09, seed + 500 + bar), 0.001, 0.08, 0.0, 0.0) * 0.17 * vol, bar_t0 + k * beat)


def bass_bar(buf, bar_t0, beat, root, pattern=(0, 0, 12, 0, 0, 0, 12, 7), vol=0.38):
    """Basse triangle en croches, sauts d'octave et de quinte."""
    for k, off in enumerate(pattern):
        mix_into(buf, tri_note(root - 12 + off, beat / 2 * 0.9, vol), bar_t0 + k * beat / 2)


# « Frère Jacques » (fa majeur) : huit mesures, chantées en canon.
FRERE_JACQUES = [
    (65, 1), (67, 1), (69, 1), (65, 1), (65, 1), (67, 1), (69, 1), (65, 1),
    (69, 1), (70, 1), (72, 2), (69, 1), (70, 1), (72, 2),
    (72, .5), (74, .5), (72, .5), (70, .5), (69, 1), (65, 1),
    (72, .5), (74, .5), (72, .5), (70, .5), (69, 1), (65, 1),
    (65, 1), (60, 1), (65, 2), (65, 1), (60, 1), (65, 2),
]


def music_frere_jacques():
    """« Frère Jacques » en canon chiptune à quatre voix, 128 bpm, 24 mesures."""
    bpm = 128
    beat = 60 / bpm
    bars = 24
    buf = np.zeros(int(SR * beat * 4 * bars) + SR)
    voices = [
        (0, lambda n, d: pulse_note(n, d, 0.5, 0.15)),            # meneur
        (2, lambda n, d: pulse_note(n + 12, d, 0.25, 0.09)),      # une octave au-dessus
        (4, lambda n, d: tri_note(n - 12, d, 0.30)),              # en bas, au triangle
        (6, lambda n, d: pulse_note(n + 12, d, 0.125, 0.08)),     # scintillement NES
    ]
    for start, voice in voices:
        t = start * 4 * beat
        for _ in range(2):
            t = play_line(buf, FRERE_JACQUES, t, beat, voice)
    for bar in range(bars):
        t0 = bar * 4 * beat
        bass_bar(buf, t0, beat, 53)   # pédale de fa
        if bar >= 2:
            drums(buf, t0, beat, bar, vol=0.9 if bar < 22 else 0.5, seed=10)
    return buf[: int(SR * beat * 4 * bars)]


# « Ah ! vous dirai-je, maman » (do majeur), l'air des variations de Mozart.
AH_VOUS_DIRAI_JE = [
    (60, 1), (60, 1), (67, 1), (67, 1), (69, 1), (69, 1), (67, 2),
    (65, 1), (65, 1), (64, 1), (64, 1), (62, 1), (62, 1), (60, 2),
    (67, 1), (67, 1), (65, 1), (65, 1), (64, 1), (64, 1), (62, 2),
    (67, 1), (67, 1), (65, 1), (65, 1), (64, 1), (64, 1), (62, 2),
    (60, 1), (60, 1), (67, 1), (67, 1), (69, 1), (69, 1), (67, 2),
    (65, 1), (65, 1), (64, 1), (64, 1), (62, 1), (62, 1), (60, 2),
]
# Une fondamentale par demi-mesure (do, fa, sol…), pour la basse et les arpèges.
AH_ROOTS = [48, 48, 53, 48, 53, 48, 55, 48, 48, 53, 48, 55, 48, 53, 48, 55,
            48, 48, 53, 48, 53, 48, 55, 48]


def music_ah_vous_dirai_je():
    """« Ah ! vous dirai-je, maman » : le thème, puis une variation en doubles croches. 120 bpm."""
    bpm = 120
    beat = 60 / bpm
    bars = 24
    buf = np.zeros(int(SR * beat * 4 * bars) + SR)
    # Thème (mesures 0 à 11), puis reprise doublée à l'octave (12 à 23).
    play_line(buf, AH_VOUS_DIRAI_JE, 0, beat, lambda n, d: pulse_note(n + 12, d, 0.5, 0.15))
    t = 12 * 4 * beat
    play_line(buf, AH_VOUS_DIRAI_JE, t, beat, lambda n, d: pulse_note(n + 12, d, 0.5, 0.12))
    play_line(buf, AH_VOUS_DIRAI_JE, t, beat, lambda n, d: pulse_note(n + 24, d, 0.125, 0.06))
    for bar in range(bars):
        t0 = bar * 4 * beat
        for half in range(2):
            root = AH_ROOTS[(bar * 2 + half) % len(AH_ROOTS)]
            third = 4
            chord = [root, root + third, root + 7]
            th = t0 + half * 2 * beat
            for k in range(4):
                mix_into(buf, tri_note(root - 12 + (12 if k == 2 else 0), beat / 2 * 0.9, 0.36), th + k * beat / 2)
            if bar >= 12:   # la variation : arpèges en doubles croches, à la Mozart en 8-bit
                for k in range(8):
                    n = chord[[0, 1, 2, 1][k % 4]] + 24
                    mix_into(buf, pulse_note(n, beat / 4 * 0.85, 0.25, 0.06, d=0.03, s=0.3), th + k * beat / 4)
        if bar >= 4:
            drums(buf, t0, beat, bar, snare=(1, 3) if bar >= 12 else (2,), hats=8 if bar >= 12 else 4, vol=0.8, seed=20)
    return buf[: int(SR * beat * 4 * bars)]


# Début de « La Marseillaise » (sol majeur), en marche 8-bit.
MARSEILLAISE_A = [
    (62, .75), (62, .25),                                   # Al-lons (levée)
    (62, 1), (67, 1), (67, 1), (69, 1),                    # en-fants de la
    (69, 1), (74, 1.5), (71, .5), (67, 1),                 # Pa-tri-i-e
    (67, .75), (71, .25), (67, .75), (64, .25), (72, 2),   # le jour de gloi-re
    (69, .75), (66, .25), (67, 2),                          # est ar-ri-vé
]
MARSEILLAISE_ROOTS = [43, 43, 48, 50]   # sol, sol, do, ré : une harmonie par mesure de la phrase


def music_marseillaise():
    """« La Marseillaise » en version console de salon : intro au roulement de caisse claire, cinq passages. 116 bpm."""
    bpm = 116
    beat = 60 / bpm
    phrase_beats = sum(d for _, d in MARSEILLAISE_A)
    bars = 24
    buf = np.zeros(int(SR * beat * 4 * bars) + SR)
    # Intro : 4 mesures de roulement et de basse.
    for bar in range(4):
        t0 = bar * 4 * beat
        for k in range(16):
            mix_into(buf, env(noise(0.04, 900 + bar * 16 + k), 0.001, 0.03, 0.0, 0.0) * (0.06 + 0.01 * k / 2), t0 + k * beat / 4)
    t = 4 * 4 * beat - beat      # la levée tombe sur le dernier temps de l'intro
    for rep in range(5):
        lead = (lambda n, d: pulse_note(n + 12, d, 0.5, 0.15)) if rep % 2 == 0 else (lambda n, d: pulse_note(n + 12, d, 0.25, 0.14))
        play_line(buf, MARSEILLAISE_A, t, beat, lead)
        if rep >= 3:   # derniers passages : contre-chant à la tierce
            play_line(buf, [(n + 4 if n else None, d) for n, d in MARSEILLAISE_A], t, beat,
                      lambda n, d: pulse_note(n + 12, d, 0.125, 0.06))
        t += phrase_beats * beat
    for bar in range(4, bars):
        t0 = bar * 4 * beat
        root = MARSEILLAISE_ROOTS[(bar - 4) % len(MARSEILLAISE_ROOTS)]
        bass_bar(buf, t0, beat, root + 12, pattern=(0, 7, 12, 7, 0, 7, 12, 7), vol=0.34)
        drums(buf, t0, beat, bar, kick=(0, 1, 2, 3), snare=(1, 3), hats=8, vol=0.75, seed=30)
    return buf[: int(SR * beat * 4 * bars)]


# « Au clair de la lune » (do majeur) : la mélodie, une note par temps.
AU_CLAIR = [
    (72, 1), (72, 1), (72, 1), (74, 1), (76, 2), (74, 2), (72, 1), (76, 1), (74, 1), (74, 1), (72, 4),
    (72, 1), (72, 1), (72, 1), (74, 1), (76, 2), (74, 2), (72, 1), (76, 1), (74, 1), (74, 1), (72, 4),
    (74, 1), (74, 1), (74, 1), (74, 1), (69, 2), (69, 2), (74, 1), (72, 1), (71, 1), (69, 1), (67, 4),
    (72, 1), (72, 1), (72, 1), (74, 1), (76, 2), (74, 2), (72, 1), (76, 1), (74, 1), (74, 1), (72, 4),
]
# Un accord par mesure (Cmaj7, G6, Am7, D7…), joué en nappe filtrée.
AU_CLAIR_CHORDS = [
    [48, 52, 55, 59], [48, 52, 55, 59], [43, 47, 50, 52], [48, 52, 55, 59],
    [48, 52, 55, 59], [48, 52, 55, 59], [43, 47, 50, 52], [48, 52, 55, 59],
    [43, 47, 50, 53], [45, 48, 52, 55], [50, 54, 57, 60], [43, 47, 50, 53],
    [48, 52, 55, 59], [48, 52, 55, 59], [43, 47, 50, 52], [48, 52, 55, 59],
]


def music_au_clair():
    """Menu : « Au clair de la lune » en French touch 8-bit : nappe filtrée qui pompe, kick, mélodie carrée. 112 bpm."""
    bpm = 112
    beat = 60 / bpm
    bars = 24
    buf = np.zeros(int(SR * beat * 4 * bars) + SR)
    for bar in range(bars):
        t0 = bar * 4 * beat
        chord = AU_CLAIR_CHORDS[bar % 8 if bar < 8 else (bar - 8) % 16]
        cutoff = 0.04 + 0.22 * min(bar, 8) / 8
        for k in range(4):
            stab = lowpass(sum(saw(note(n + 12), beat * 0.45) for n in chord) / 4, cutoff)
            mix_into(buf, env(stab, 0.01, 0.15, 0.4, 0.05) * 0.42, t0 + k * beat + beat / 2)
            b = lowpass(saw(note(chord[0] - 12), beat * 0.4), 0.08)
            mix_into(buf, env(b, 0.005, 0.1, 0.6, 0.05) * 0.5, t0 + k * beat + beat / 2)
            t = t_axis(0.18)
            mix_into(buf, np.sin(2 * np.pi * (95 * t - 200 * t * t)) * np.exp(-t * 22) * 0.7, t0 + k * beat)
            mix_into(buf, env(noise(0.04, bar * 4 + k), 0.001, 0.03, 0.0, 0.0) * 0.08, t0 + k * beat + beat / 2)
    # La mélodie entre après 8 mesures d'intro, en onde carrée avec un écho.
    t = 8 * 4 * beat
    lead = lambda n, d: pulse_note(n, d, 0.25, 0.13, d=0.1, s=0.45, r=0.08)
    play_line(buf, AU_CLAIR, t, beat, lead)
    play_line(buf, AU_CLAIR, t + beat * 0.75, beat, lambda n, d: lead(n, d) * 0.35)
    return buf[: int(SR * beat * 4 * bars)]


MUSIC = {
    "music_menu": music_au_clair,
    "music_office_1": music_frere_jacques,
    "music_office_2": music_ah_vous_dirai_je,
    "music_office_3": music_marseillaise,
}


# --------------------------------------------------------------- effets
def sfx_cash():
    buf = np.zeros(int(SR * 0.6))
    mix_into(buf, env(square(note(88), 0.07, 0.25), 0.001, 0.03, 0.5, 0.02), 0)
    mix_into(buf, env(square(note(93), 0.25, 0.25), 0.001, 0.08, 0.3, 0.15), 0.07)
    bell = sum(np.sin(2 * np.pi * f * t_axis(0.5)) for f in (2637, 3520, 4186)) * np.exp(-t_axis(0.5) * 8)
    mix_into(buf, bell * 0.3, 0.08)
    return buf


def sfx_alert():
    buf = np.zeros(int(SR * 0.7))
    for i, n in enumerate([81, 74, 81, 74]):
        mix_into(buf, env(square(note(n), 0.12, 0.5), 0.002, 0.02, 0.8, 0.02) * 0.6, i * 0.15)
    return buf


def sfx_typing():
    buf = np.zeros(int(SR * 0.5))
    rng = np.random.default_rng(3)
    for i in range(7):
        click = noise(0.015, i) * np.exp(-t_axis(0.015) * 300)
        mix_into(buf, click * rng.uniform(0.4, 1.0), i * 0.06 + rng.uniform(0, 0.02))
    return buf


def sfx_coffee():
    d = 0.9
    t = t_axis(d)
    gurgle = noise(d, 7) * (0.5 + 0.5 * np.sin(2 * np.pi * 9 * t)) * np.exp(-t * 2)
    gurgle = lowpass(gurgle, 0.06)
    hiss = lowpass(noise(d, 8), 0.5) * 0.15 * np.linspace(0, 1, len(t))
    return gurgle + hiss


def sfx_funding():
    buf = np.zeros(int(SR * 1.0))
    for i, n in enumerate([72, 76, 79, 84, 88]):
        mix_into(buf, env(square(note(n), 0.12, 0.25), 0.002, 0.05, 0.5, 0.05) * 0.5, i * 0.07)
    sparkle = sum(np.sin(2 * np.pi * f * t_axis(0.6)) for f in (3136, 3951, 4699)) * np.exp(-t_axis(0.6) * 5)
    mix_into(buf, sparkle * 0.25, 0.35)
    return buf


def sfx_fail():
    buf = np.zeros(int(SR * 2.2))
    for i, n in enumerate([62, 61, 60, 59]):
        d = 0.35 if i < 3 else 1.0
        t = t_axis(d)
        f = note(n) * (1 + 0.01 * np.sin(2 * np.pi * 6 * t))
        tone = lowpass(2 * ((np.cumsum(f) / SR) % 1) - 1, 0.15)
        mix_into(buf, env(tone, 0.02, 0.1, 0.7, 0.1) * 0.7, i * 0.38)
    return buf


def sfx_click():
    return env(square(note(96), 0.03, 0.25), 0.001, 0.02, 0.0, 0.0)


def sfx_level():
    buf = np.zeros(int(SR * 1.4))
    for i, n in enumerate([72, 76, 79]):
        mix_into(buf, env(square(note(n), 0.14, 0.5), 0.002, 0.05, 0.6, 0.03) * 0.4, i * 0.13)
    for n in (72, 76, 79, 84):
        mix_into(buf, env(square(note(n), 0.8, 0.25), 0.002, 0.2, 0.5, 0.4) * 0.18, 0.42)
    return buf


def sfx_popup():
    t = t_axis(0.5)
    return (np.sin(2 * np.pi * 1318 * t) + 0.5 * np.sin(2 * np.pi * 1976 * t)) * np.exp(-t * 9)


def sfx_hire():
    buf = np.zeros(int(SR * 0.4))
    mix_into(buf, env(square(note(79), 0.1, 0.5), 0.002, 0.03, 0.5, 0.02) * 0.5, 0)
    mix_into(buf, env(square(note(84), 0.2, 0.5), 0.002, 0.05, 0.4, 0.1) * 0.5, 0.1)
    return buf


def sfx_incident():
    d = 1.2
    t = t_axis(d)
    f = 700 + 300 * np.sin(2 * np.pi * 2.5 * t)
    siren = np.where((np.cumsum(f) / SR) % 1 < 0.5, 1.0, -1.0)
    return env(siren, 0.02, 0.1, 0.6, 0.2) * 0.5


SFX = {
    "cash": sfx_cash, "alert": sfx_alert, "typing": sfx_typing, "coffee": sfx_coffee, "funding": sfx_funding,
    "fail": sfx_fail, "click": sfx_click, "level": sfx_level, "popup": sfx_popup, "hire": sfx_hire,
    "incident": sfx_incident,
}

if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "."
    os.makedirs(out, exist_ok=True)
    for name, f in MUSIC.items():
        write(os.path.join(out, f"{name}.ogg"), f(), 0.55)
    for name, f in SFX.items():
        write(os.path.join(out, f"sfx_{name}.ogg"), f(), 0.7)
    print(sorted(os.listdir(out)))
