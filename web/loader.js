// French Tech Tycoon — browser loader.
//
// 1. Detects WebAssembly and a usable GPU API (WebGPU first, then WebGL2).
// 2. Downloads the matching wasm build with a progress bar.
// 3. Starts Bevy and fades the boot screen out when the game signals
//    `ftt-ready` (textures loaded, first frames drawn).
//
// Override the renderer with ?renderer=webgl2 or ?renderer=webgpu.
//
// WebGPU fallback: if the WebGPU build fails to start (device lost, no
// adapter, or not ready in time), the failure is remembered and the page
// reloads with the WebGL2 build. A forced ?renderer=webgpu reports the error
// instead, for debugging.

const $ = (id) => document.getElementById(id);
const fill = $("fill");
const statusEl = $("status");
const progress = $("progress");

function setProgress(pct, text) {
  const p = Math.max(0, Math.min(100, pct));
  fill.style.width = p + "%";
  progress.setAttribute("aria-valuenow", String(Math.round(p)));
  if (text) statusEl.textContent = text;
}

function showIncompatible(reason) {
  progress.hidden = true;
  $("incompatible").hidden = false;
  $("incompatible-reason").textContent = reason;
}

let crashed = false;
function showCrash(detail) {
  if (crashed) return;
  crashed = true;
  const boot = $("boot");
  boot.classList.remove("done");
  progress.hidden = true;
  $("crash").hidden = false;
  $("crash-detail").textContent = String(detail || "").slice(0, 2000);
}

// winit uses a JS exception to hand control to the browser event loop.
function isControlFlowException(e) {
  const msg = String((e && e.message) || e || "");
  return msg.includes("Using exceptions for control flow");
}

window.addEventListener("error", (ev) => {
  if (isControlFlowException(ev.error || ev.message)) {
    ev.preventDefault();
    return;
  }
  const msg = String(ev.message || "");
  if (msg.includes("unreachable") || msg.includes("panicked") || msg.includes("RuntimeError")) {
    showCrash(msg);
  }
});
window.addEventListener("unhandledrejection", (ev) => {
  if (isControlFlowException(ev.reason)) {
    ev.preventDefault();
  }
});

// Browsers keep AudioContexts suspended until a user gesture. Track every
// context the engine creates and resume them on the first interaction.
(function unlockAudio() {
  const Ctx = window.AudioContext || window.webkitAudioContext;
  if (!Ctx) return;
  const contexts = [];
  const Patched = function (...args) {
    const ctx = new Ctx(...args);
    contexts.push(ctx);
    return ctx;
  };
  Patched.prototype = Ctx.prototype;
  window.AudioContext = Patched;
  if (window.webkitAudioContext) window.webkitAudioContext = Patched;
  const resume = () => contexts.forEach((c) => c.state === "suspended" && c.resume());
  ["pointerdown", "keydown", "touchend"].forEach((t) =>
    window.addEventListener(t, resume, { capture: true, passive: true }),
  );
})();

async function hasWebGPU() {
  if (!("gpu" in navigator)) return false;
  try {
    const adapter = await Promise.race([
      navigator.gpu.requestAdapter(),
      new Promise((r) => setTimeout(() => r(null), 3000)),
    ]);
    return !!adapter;
  } catch {
    return false;
  }
}

function hasWebGL2() {
  try {
    const c = document.createElement("canvas");
    return !!c.getContext("webgl2");
  } catch {
    return false;
  }
}

const params = new URLSearchParams(location.search);
const forcedRenderer = params.get("renderer");
const WEBGPU_FAILED_KEY = "french-tech-tycoon:webgpu-failed";

function webgpuPreviouslyFailed() {
  try {
    return localStorage.getItem(WEBGPU_FAILED_KEY) === "1";
  } catch {
    return false;
  }
}

async function pickRenderer() {
  if (forcedRenderer === "webgl2" || forcedRenderer === "webgpu") return forcedRenderer;
  if (!webgpuPreviouslyFailed() && (await hasWebGPU())) return "webgpu";
  if (hasWebGL2()) return "webgl2";
  return null;
}

async function fetchWithProgress(url, onProgress) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: HTTP ${res.status}`);
  const total = Number(res.headers.get("content-length")) || 0;
  if (!res.body || !total) {
    const buf = await res.arrayBuffer();
    onProgress(1);
    return buf;
  }
  const reader = res.body.getReader();
  const chunks = [];
  let got = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    got += value.length;
    onProgress(Math.min(1, got / total));
  }
  const out = new Uint8Array(got);
  let off = 0;
  for (const c of chunks) {
    out.set(c, off);
    off += c.length;
  }
  return out.buffer;
}

// Bevy reports fatal GPU problems through its logger (console.*). Watch for
// them so a dead canvas never sits behind a finished loading screen.
const FATAL = /DeviceLost|Quitting the application due to|Unable to find a GPU|Failed to create (the )?(device|adapter|surface)/i;
let activeRenderer = null;
let gameReady = false;
let fallingBack = false;

function fallbackToWebGL2(reason) {
  if (fallingBack) return;
  console.warn("[loader] WebGPU failed, falling back to WebGL2:", reason);
  let remembered = false;
  try {
    localStorage.setItem(WEBGPU_FAILED_KEY, "1");
    remembered = localStorage.getItem(WEBGPU_FAILED_KEY) === "1";
  } catch {}
  // Some hosts drop the query string on reload; without a stored flag the
  // page would pick WebGPU again and loop. Ask the player instead.
  if (!remembered && !location.search.includes("renderer=")) {
    showCrash("WebGPU n'a pas pu démarrer sur cet appareil (" + reason + "). Ajoutez ?renderer=webgl2 à l'adresse pour utiliser le mode compatibilité.");
    return;
  }
  fallingBack = true;
  const url = new URL(location.href);
  url.searchParams.set("renderer", "webgl2");
  url.searchParams.set("from", "webgpu");
  location.replace(url.toString());
}

function onEngineFatal(text) {
  if (activeRenderer === "webgpu" && !gameReady && forcedRenderer !== "webgpu") {
    fallbackToWebGL2(text);
  } else {
    showCrash("Erreur GPU : " + text);
  }
}

for (const level of ["log", "warn", "error"]) {
  const orig = console[level].bind(console);
  console[level] = (...args) => {
    orig(...args);
    if (fallingBack || crashed) return;
    const text = args.filter((a) => typeof a === "string").join(" ");
    if (FATAL.test(text)) onEngineFatal(text.replace(/%c/g, "").slice(0, 300));
  };
}

async function fetchPartsList(base) {
  try {
    const res = await fetch(base + "parts.json");
    if (!res.ok) return null;
    const list = await res.json();
    return Array.isArray(list.parts) && list.parts.length ? list.parts : null;
  } catch {
    return null;
  }
}

async function fetchParts(base, parts, onProgress) {
  const bufs = [];
  for (let i = 0; i < parts.length; i++) {
    bufs.push(new Uint8Array(await fetchWithProgress(base + parts[i], (f) => onProgress((i + f) / parts.length))));
  }
  const total = bufs.reduce((n, b) => n + b.length, 0);
  const out = new Uint8Array(total);
  let off = 0;
  for (const b of bufs) {
    out.set(b, off);
    off += b.length;
  }
  return out.buffer;
}

const QUIPS = [
  "Envoi du dossier à Bpifrance…",
  "Recherche de la page 4…",
  "Compilation en Rust (le produit était déjà en Rust)…",
  "Négociation avec le cloud provider…",
  "Calcul du Monthly Recurring Optimism…",
];

async function main() {
  if (typeof WebAssembly !== "object" || typeof WebAssembly.instantiate !== "function") {
    showIncompatible("WebAssembly n'est pas disponible dans ce navigateur.");
    return;
  }
  setProgress(2, "Vérification du GPU…");
  const renderer = await pickRenderer();
  if (!renderer) {
    showIncompatible("Ni WebGPU ni WebGL2 ne sont disponibles (GPU désactivé, navigateur trop ancien ou mode restreint).");
    return;
  }
  activeRenderer = renderer;
  const fellBack = params.get("from") === "webgpu" || (renderer === "webgl2" && webgpuPreviouslyFailed());
  $("renderer").textContent =
    "Rendu : " +
    (renderer === "webgpu" ? "WebGPU · ?renderer=webgl2 pour forcer le mode compatibilité" : "WebGL2") +
    (fellBack ? " (WebGPU indisponible sur cet appareil)" : "");

  const base = `./pkg-${renderer}/`;
  let quip = 0;
  const quipTimer = setInterval(() => {
    quip = (quip + 1) % QUIPS.length;
  }, 1400);

  let wasm;
  const onWasmProgress = (f) => setProgress(5 + f * 75, `${QUIPS[quip]} ${Math.round(f * 100)} %`);
  try {
    // Hosts with a per-file size cap can serve the module as numbered parts
    // listed in parts.json (see tools/split-wasm.sh); otherwise one file.
    const parts = await fetchPartsList(base);
    wasm = parts
      ? await fetchParts(base, parts, onWasmProgress)
      : await fetchWithProgress(base + "ftt_bg.wasm", onWasmProgress);
  } catch (e) {
    clearInterval(quipTimer);
    showCrash("Téléchargement impossible : " + e);
    return;
  }
  clearInterval(quipTimer);
  setProgress(82, "Démarrage du moteur…");

  window.addEventListener("ftt-progress", (ev) => {
    const { loaded, total } = ev.detail;
    setProgress(85 + (loaded / Math.max(1, total)) * 15, `Chargement des sprites… ${loaded}/${total}`);
  });
  const ready = new Promise((resolve) => window.addEventListener("ftt-ready", resolve, { once: true }));

  try {
    const mod = await import(base + "ftt.js");
    await mod.default({ module_or_path: wasm });
  } catch (e) {
    if (!isControlFlowException(e)) {
      showCrash(e && e.stack ? e.stack : e);
      return;
    }
  }

  // If the ready signal never arrives: WebGPU → retry with WebGL2;
  // WebGL2 → reveal the canvas anyway after a generous delay.
  const timeoutMs = renderer === "webgpu" ? 25000 : 30000;
  const timedOut = await Promise.race([
    ready.then(() => false),
    new Promise((r) => setTimeout(() => r(true), timeoutMs)),
  ]);
  if (crashed || fallingBack) return;
  if (timedOut && renderer === "webgpu" && forcedRenderer !== "webgpu") {
    fallbackToWebGL2("timeout");
    return;
  }
  gameReady = true;
  setProgress(100, "C'est parti.");
  $("boot").classList.add("done");
  const canvas = $("game-canvas");
  canvas.focus();
  window.__FTT_RENDERER = renderer;
  window.__FTT_READY = true;
}

main();
