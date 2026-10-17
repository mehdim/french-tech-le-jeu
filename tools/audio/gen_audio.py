"""Thème d'« À la French » en 16 bits, airs traditionnels français en 8-bit, effets sonores.

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


# --------------------------------------------------------------- thème À la French
# Le thème du générique de la chaîne « À la French » (0:12 à 0:27), relevé à
# l'oreille et au spectrogramme, puis réarrangé façon console 16 bits : lead
# « guitare synthé » saw + pulse avec vibrato et écho, basse glissée façon 808,
# nappes de cordes, arpèges, batterie échantillonnée. Ré mineur, 89 bpm.

THEME_BPM = 89
# Mélodie : (double croche de départ, note MIDI, durée en doubles croches), 6 mesures.
THEME_LEAD = [
    (0, 62, 1), (2, 62, 2), (4, 57, 1), (6, 57, 2), (9, 67, 1), (10, 65, 1), (11, 65, 1), (12, 67, 2), (14, 65, 2),
    (16, 62, 1), (17, 62, 1), (18, 62, 2), (24, 65, 1), (25, 64, 1), (26, 64, 1), (27, 65, 2), (30, 62, 1), (31, 62, 1),
    (32, 62, 1), (33, 62, 2), (41, 62, 1), (42, 62, 2),
    (49, 65, 1), (50, 65, 1), (51, 67, 1), (52, 65, 2), (55, 62, 1), (56, 62, 1), (57, 62, 1), (58, 62, 2),
    (61, 52, 1), (62, 53, 1), (63, 54, 1),
    (64, 55, 1), (65, 62, 1), (66, 62, 1), (67, 67, 1), (68, 65, 1), (70, 67, 1), (71, 67, 1), (73, 60, 1), (74, 60, 1),
    (76, 64, 1), (77, 62, 1), (78, 62, 1), (79, 62, 1),
    (80, 62, 1), (81, 60, 1), (82, 60, 1), (84, 65, 1), (85, 64, 1), (86, 64, 1), (87, 62, 1), (88, 62, 1), (89, 62, 1),
    (90, 62, 1), (91, 62, 1), (93, 62, 1), (94, 62, 2),
]
# Basse : (départ, durée, note de départ, note d'arrivée du glissando).
THEME_BASS = [
    (0, 8, 38, 38), (9, 7, 45, 48),
    (16, 8, 38, 38), (24, 6, 41, 44), (30, 2, 40, 38),
    (32, 4, 39, 38), (36, 2, 46, 47), (38, 3, 43, 40), (41, 7, 38, 38),
    (48, 4, 36, 36), (52, 8, 38, 38), (60, 2, 38, 38), (62, 2, 51, 51),
    (64, 2, 48, 48), (66, 4, 44, 41), (71, 3, 36, 36), (75, 2, 39, 39), (78, 2, 38, 38),
    (80, 3, 38, 38), (83, 2, 41, 41), (85, 4, 38, 38), (90, 2, 36, 36), (92, 4, 38, 38),
]
# Un accord par demi-mesure (ré m, si♭, sol m, do…).
DM, BB, GM, CM = [50, 53, 57, 62], [46, 50, 53, 58], [43, 46, 50, 55], [48, 52, 55, 60]
THEME_CHORDS = [DM, BB, DM, GM, DM, DM, CM, DM, GM, CM, DM, DM]


def onepole(x, alpha):
    """Passe-bas à un pôle, vectorisé si scipy est là."""
    try:
        from scipy.signal import lfilter
        return lfilter([alpha], [1, alpha - 1], x)
    except ImportError:
        return lowpass(x, alpha)


def glide_osc(f0, f1, dur, shape="saw", vib=0.0):
    """Oscillateur à fréquence glissante (portamento exponentiel) avec vibrato optionnel."""
    t = t_axis(dur)
    k = np.clip(t / max(dur * 0.8, 1e-3), 0, 1)
    f = f0 * (f1 / f0) ** k
    if vib:
        f = f * (1 + vib * np.sin(2 * np.pi * 5.5 * t) * np.clip((t - 0.12) * 6, 0, 1))
    ph = np.cumsum(f) / SR % 1
    if shape == "saw":
        return 2 * ph - 1
    if shape == "pulse":
        return np.where(ph < 0.3, 1.0, -1.0)
    return 2 * np.abs(2 * ph - 1) - 1


def echo(x, delay, fb, mix):
    """Écho à réinjection, comme le tampon d'écho des consoles 16 bits."""
    d = int(delay * SR)
    y = x.copy()
    for i in range(d, len(y), d):
        j = min(len(y), i + d)
        y[i:j] += fb * y[i - d:i - d + (j - i)]
    return x + (y - x) * mix


def lead16(n, dur, vol=0.36):
    f = note(n)
    sig = 0.55 * glide_osc(f, f, dur, "saw", vib=0.006) + 0.35 * glide_osc(f * 1.004, f * 1.004, dur, "pulse", vib=0.006)
    sig += 0.2 * glide_osc(f * 2, f * 2, dur, "tri")
    t = t_axis(dur)
    sig = onepole(sig * (0.45 + 0.55 * np.exp(-t * 9)), 0.45)
    return env(sig, 0.003, 0.12, 0.55, min(0.05, dur / 3)) * vol


def bass16(n0, n1, dur, vol=0.3):
    sig = glide_osc(note(n0), note(n1), dur, "saw") * 0.7 + glide_osc(note(n0) / 2, note(n1) / 2, dur, "tri") * 0.6
    t = t_axis(dur)
    sig = onepole(sig, 0.05 + 0.08 * np.exp(-t * 12).mean())
    return env(sig, 0.004, 0.08, 0.75, 0.03) * vol


def pad16(chord, dur, vol=0.06):
    sig = sum(saw(note(n + 12), dur) + saw(note(n + 12) * 1.006, dur) for n in chord)
    return env(onepole(sig, 0.035), 0.12, 0.2, 0.8, 0.15) * vol


def kick16(vol=0.8):
    t = t_axis(0.22)
    return np.sin(2 * np.pi * (52 * t + 90 * (1 - np.exp(-t * 28)) / 28)) * np.exp(-t * 14) * vol


def snare16(seed, vol=0.3):
    t = t_axis(0.16)
    body = np.sin(2 * np.pi * 185 * t) * np.exp(-t * 30)
    n = onepole(noise(0.16, seed), 0.55) * np.exp(-t * 18)
    return (body * 0.5 + n) * vol


def hat16(seed, open_=False, vol=0.07):
    dur = 0.18 if open_ else 0.04
    t = t_axis(dur)
    n = noise(dur, seed)
    n = n - onepole(n, 0.6)   # passe-haut grossier
    return n * np.exp(-t * (14 if open_ else 70)) * vol


def music_alafrench():
    """Le thème « À la French » en 16 bits : 2 passages de 6 mesures + 1 mesure de relance, en boucle."""
    beat = 60 / THEME_BPM
    s16 = beat / 4
    bars = 13
    total = bars * 16 * s16
    lead_bus = np.zeros(int(SR * total) + SR)
    buf = np.zeros_like(lead_bus)
    for rep in range(2):
        base = rep * 96 * s16
        for st, n, ln in THEME_LEAD:
            mix_into(lead_bus, lead16(n, ln * s16 * 0.92), base + st * s16)
            if rep == 1:   # second passage : doublure à l'octave et tierce en pulse fine
                mix_into(lead_bus, lead16(n + 12, ln * s16 * 0.92, 0.07), base + st * s16)
        for st, ln, n0, n1 in THEME_BASS:
            vol = 0.3 if (rep == 1 or st >= 9) else 0.18
            mix_into(buf, bass16(n0, n1, ln * s16 * 0.95, vol), base + st * s16)
        for h, chord in enumerate(THEME_CHORDS):
            t0 = base + h * 8 * s16
            mix_into(buf, pad16(chord, 8 * s16, 0.012 if rep == 0 else 0.022), t0)
            if rep == 1:   # arpèges en doubles croches, typiques des consoles 16 bits
                for k in range(8):
                    n = chord[[0, 1, 2, 3, 2, 1, 2, 3][k]] + 12
                    mix_into(lead_bus, pulse_note(n, s16 * 0.8, 0.125, 0.035, d=0.03, s=0.3), t0 + k * s16)
        # Batterie : au premier passage, coups d'ouverture puis montée ; au second, groove complet.
        for bar in range(6):
            b0 = base + bar * 16 * s16
            full = rep == 1 or bar >= 4
            if rep == 0 and bar == 0:
                for p in (0, 4, 5, 8, 12, 13):
                    mix_into(buf, kick16(0.8), b0 + p * s16)
                    mix_into(buf, snare16(p, 0.22), b0 + p * s16)
                continue
            if rep == 0 and bar == 3:
                for p in range(8, 16):   # roulement de caisse claire
                    mix_into(buf, snare16(100 + p, 0.08 + 0.025 * (p - 8)), b0 + p * s16)
            if full:
                for p in (0, 6, 8, 11, 14):
                    mix_into(buf, kick16(0.75), b0 + p * s16)
                for p in (4, 12):
                    mix_into(buf, snare16(bar * 16 + p + rep * 7, 0.3), b0 + p * s16)
                mix_into(buf, snare16(bar + 300, 0.06), b0 + 15 * s16)   # note fantôme
            for p in range(0, 16, 2 if not full else 1):
                mix_into(buf, hat16(bar * 16 + p, open_=(full and p % 8 == 6), vol=0.05 if p % 2 else 0.07),
                         b0 + p * s16)
    # Mesure de relance : basse en ré, roulement, retour au début de la boucle.
    b0 = 12 * 16 * s16
    mix_into(buf, bass16(38, 38, 8 * s16), b0)
    mix_into(buf, bass16(45, 49, 8 * s16), b0 + 8 * s16)
    mix_into(buf, pad16(DM, 16 * s16, 0.02), b0)
    for p in range(16):
        mix_into(buf, snare16(500 + p, 0.05 + 0.015 * p), b0 + p * s16)
    mix_into(buf, kick16(0.8), b0)
    out = buf + echo(lead_bus, 3 * s16, 0.35, 0.5)
    # Couleur 16 bits : léger passe-bas (interpolation gaussienne) puis réduction à 32 kHz.
    out = onepole(out, 0.6)
    return out[: int(SR * total)]


# Le thème sert au menu et ouvre la playlist du bureau ; les airs traditionnels suivent.
MUSIC = {
    "music_theme": music_alafrench,
    "music_office_1": music_frere_jacques,
    "music_office_2": music_ah_vous_dirai_je,
    "music_office_3": music_marseillaise,
    "music_office_4": music_au_clair,
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
