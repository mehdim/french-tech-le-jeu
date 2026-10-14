"""Boucles musicales chiptune / électro à la française et effets sonores originaux.

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
def music_office():
    """Chiptune sautillant pour le bureau : la mineur / do majeur, 124 bpm, 16 mesures."""
    bpm = 124
    beat = 60 / bpm
    bars = 16
    buf = np.zeros(int(SR * beat * 4 * bars) + SR)
    prog = [(57, [57, 60, 64]), (53, [53, 57, 60]), (48, [48, 52, 55]), (55, [55, 59, 62])]  # Am F C G
    lead = [76, 74, 72, 74, 76, 79, 76, 74, 72, 69, 72, 74, 72, 69, 67, 69,
            76, 79, 81, 79, 76, 74, 72, 74, 76, 74, 72, 71, 72, -1, 69, -1]
    for bar in range(bars):
        root, chord = prog[bar % 4]
        t0 = bar * 4 * beat
        # basse triangle : fondamentale à chaque croche, sauts d'octave
        for k in range(8):
            n = root - 12 + (12 if k % 4 == 3 else 0)
            mix_into(buf, env(tri(note(n), beat / 2 * 0.95), 0.003, 0.05, 0.7, 0.02) * 0.45, t0 + k * beat / 2)
        # arpège en onde carrée (rapport cyclique de 12,5 % pour le scintillement NES)
        for k in range(16):
            n = chord[k % 3] + 12
            mix_into(buf, env(square(note(n), beat / 4 * 0.9, 0.125), 0.002, 0.03, 0.3, 0.01) * 0.12, t0 + k * beat / 4)
        # mélodie (à partir de la mesure 4)
        if bar >= 4:
            for k in range(8):
                n = lead[((bar - 4) % 4) * 8 + k]
                if n > 0:
                    mix_into(buf, env(square(note(n), beat / 2 * 0.9, 0.5), 0.005, 0.08, 0.5, 0.04) * 0.16, t0 + k * beat / 2)
        # batterie : charleston en bruit + grosse caisse en blips
        for k in range(8):
            mix_into(buf, env(noise(0.03, bar * 8 + k), 0.001, 0.02, 0.0, 0.0) * (0.12 if k % 2 else 0.06), t0 + k * beat / 2)
        for k in (0, 2):
            t = t_axis(0.12)
            kick = np.sin(2 * np.pi * (110 * t - 300 * t * t)) * np.exp(-t * 30)
            mix_into(buf, kick * 0.55, t0 + k * beat)
        for k in (1, 3):
            mix_into(buf, env(noise(0.08, 99 + bar), 0.001, 0.07, 0.0, 0.0) * 0.18, t0 + k * beat)
    return buf[: int(SR * beat * 4 * bars)]


def music_menu():
    """Boucle de menu façon French house : accords filtrés, four-on-the-floor, 118 bpm."""
    bpm = 118
    beat = 60 / bpm
    bars = 16
    buf = np.zeros(int(SR * beat * 4 * bars) + SR)
    prog = [[57, 60, 64, 67], [53, 57, 60, 64], [50, 53, 57, 60], [52, 55, 59, 62]]  # Am7 Fmaj7 Dm7 Em7
    for bar in range(bars):
        t0 = bar * 4 * beat
        chord = prog[bar % 4]
        # nappe qui pompe : accords plaqués sur les contretemps (effet sidechain)
        for k in range(4):
            stab = sum(saw(note(n), beat * 0.45) for n in chord) / 4
            cutoff = 0.05 + 0.25 * (bar % 8) / 8
            stab = lowpass(stab, cutoff)
            mix_into(buf, env(stab, 0.01, 0.15, 0.4, 0.05) * 0.5, t0 + k * beat + beat / 2)
        # basse
        for k in range(4):
            n = chord[0] - 24
            b = lowpass(saw(note(n), beat * 0.4), 0.08)
            mix_into(buf, env(b, 0.005, 0.1, 0.6, 0.05) * 0.6, t0 + k * beat + beat / 2)
        # grosse caisse à chaque temps
        for k in range(4):
            t = t_axis(0.18)
            kick = np.sin(2 * np.pi * (95 * t - 200 * t * t)) * np.exp(-t * 22)
            mix_into(buf, kick * 0.8, t0 + k * beat)
            mix_into(buf, env(noise(0.04, bar * 4 + k), 0.001, 0.03, 0.0, 0.0) * 0.1, t0 + k * beat + beat / 2)
        if bar >= 8:   # un petit motif en onde carrée façon vocal chop
            motif = [76, -1, 74, 72, -1, 72, 74, -1]
            for k, n in enumerate(motif):
                if n > 0:
                    mix_into(buf, env(square(note(n), beat / 2 * 0.6, 0.25), 0.005, 0.05, 0.3, 0.05) * 0.12, t0 + k * beat / 2)
    return buf[: int(SR * beat * 4 * bars)]


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
    write(os.path.join(out, "music_office.ogg"), music_office(), 0.55)
    write(os.path.join(out, "music_menu.ogg"), music_menu(), 0.55)
    for name, f in SFX.items():
        write(os.path.join(out, f"sfx_{name}.ogg"), f(), 0.7)
    print(sorted(os.listdir(out)))
