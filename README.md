# French Tech Tycoon — From France Travail to Unicorn

> « La France est une startup nation. À vous de payer l'URSSAF. »

A satirical startup-management game about **VERIDIX**, a Paris AI startup that
claims to detect AI-generated content (its first model scores 53 % on its own
benchmark). Take it from a two-laptop apartment and a France Travail
appointment to a European HQ. Along the way: Bpifrance dossiers, business
angels from the Minitel era, a Series A pitched in PowerPoint, enterprise
procurement, a GPU bill that explodes, and one final choice: **unicorn,
profitable company, acquisition, or open-source commons**.

Written in Rust with Bevy, compiled to WebAssembly, and played in the browser.
The game text is in French.

![Title screen](docs/screenshots/menu.png)

| Level 2 — the garage | Level 6 — Station F | Level 12 — European HQ |
|---|---|---|
| ![Early office with the Product & R&D panel](docs/screenshots/garage.png) | ![Mid-game office with the Team panel](docs/screenshots/station-f.png) | ![Late-game headquarters with the Funding panel](docs/screenshots/hq.png) |

## Play

The browser build is a folder of static files (`dist/`). Serve it with any
static web server; it does not run from `file://` because wasm and
assets are fetched over HTTP:

```bash
cd dist && python3 -m http.server 8080
# open http://localhost:8080
```

Players need no Rust toolchain or plugins, only a browser with WebAssembly and
WebGPU or WebGL2.

### Supported browsers

| Browser | Renderer | Status |
|---|---|---|
| Chrome / Edge 113+ (desktop) | WebGPU, automatic WebGL2 fallback | WebGL2 path smoke-tested in headless Chromium (see below) |
| Firefox 115+ | WebGL2 (WebGPU in 141+ on Windows) | Expected to work; not automatically tested |
| Safari 17+ (macOS) | WebGL2 (WebGPU in Safari 26) | Expected to work; not automatically tested |
| Mobile browsers | WebGL2 | Playable with touch, but the UI targets ≥ 1280×720 |

The loader (`web/loader.js`) asks the browser for a WebGPU adapter. If one
exists it loads the WebGPU build, otherwise it uses the WebGL2 build. If the
WebGPU build fails to start (lost device, Bevy quitting on a render error, or
not ready within 25 s), the loader remembers that in `localStorage` and reloads
with WebGL2. If neither API or WebAssembly is available, it shows a
compatibility message explaining what is missing instead of a black canvas. A
GPU failure after the game has started shows a "reload" screen rather than a
frozen canvas. You can force a renderer with `?renderer=webgl2` or
`?renderer=webgpu`; a forced WebGPU run reports errors instead of falling back.

**What was verified, and what wasn't.** In headless Chromium (SwiftShader,
no physical GPU):

- The WebGL2 build plays: new game, events, autosave, reload, continue.
- The no-GPU compatibility screen appears when WebGL and WebGPU are disabled.
- The WebGPU build gets an adapter, but Chromium's experimental
  software WebGPU then reports a lost device. That is what exercised the
  automatic fallback, which worked (WebGL2, playable in ~5 s).

The WebGPU build has **not** been checked on a real GPU. On a desktop with
working WebGPU it should run; if not, players silently get WebGL2.

### Controls

| Action | Mouse / touch | Keyboard |
|---|---|---|
| Pan the office | drag (any mouse button), one-finger drag | WASD / arrow keys |
| Zoom | wheel, pinch | `+` / `-` |
| Select an employee or furniture | click | |
| Pause / resume | ⏸ in the top bar | `Space` |
| Speed 1× / 2× / 4× | top bar | `1` `2` `3`/`4` |
| Panels: Team, Product & R&D, Sales, Funding, Finances, Office | right panel tabs | `E` `P` `V` `F` `C` `B` |
| Rotate furniture while placing | | `R` |
| Keep placing copies | | hold `Shift` |
| Cancel placement | right click | `Esc` |
| Next story line / start level | « Suivant », « C'est parti ! » | `Enter` |
| Answer an event | click a choice | `1`–`4` |
| Pause menu (save / load / settings) | ☰ | `Esc` |

## What's in the game

- **12-level campaign**, each level with a story intro, 2–4 measurable
  quests, a unique "boss" event, an unlock and a humorous outro. It runs from
  « France Travail Simulator » to « Licorne ou Poney ? ».
- **4 illustrated endings**: Unicorn, Profitable Company, Acquisition, Commons.
- **Deterministic daily simulation** (1 tick = 1 day, 28-day accounting
  months). It covers leads → conversion → subscriptions → churn by customer
  segment, MRR/ARR, COGS (inference + hosting), gross margin, burn, runway,
  CAC, LTV, ARPA, valuation and a cap table.
- **AI R&D that never "solves" detection**: 52 product, research,
  enterprise and pivot tasks with dependencies, engineering and research
  points, cost, tech-debt risk, and quick-vs-clean approaches. Concept drift
  and new foundation models keep eroding benchmark accuracy and the
  false-positive rate.
- **Employees as entities**: 12+ archetypes (Polytechnicien, 42 developer,
  Rust evangelist, ex-GAFAM, ESN consultant, PhD researcher, growth hacker,
  Jira-happy PM…) with skills, morale, fatigue, loyalty and traits. They are
  hired through 7 channels (LinkedIn, France Travail, grandes écoles, 42,
  open source, recruiters, referrals). Firing costs severance and morale.
- **French ecosystem as mechanics**: France Travail support, satirical
  Bpifrance programmes (grants, innovation aid, loans, deeptech, export) with
  dossiers, paperwork, waiting time and staged disbursement; JEI/CIR-style
  tax relief; four recurring business angels (Jean-Michel Exit 2001, Patrick
  Blockchain, Bernard EBITDA, Sophie Term Sheet); seed to Series B VCs with
  dilution.
- **60 data-driven random events** with conditions, weights, cooldowns and
  choices with immediate and delayed effects (the Kubernetes incident, the
  Chinese open-weight model dilemma, the LinkedIn influencer, the intern's
  leaked API key…).
- **Isometric pixel-art office** that evolves through 6 tiers, from the
  apartment to the European HQ. Animated employees walk, sit, type, drink
  coffee, meet, panic and burn out. A grid furniture system supports placement
  previews, validity checks, rotation and moving or storing furniture. Every
  furniture item affects morale, capacity, efficiency or infrastructure.
- **Saves**: autosave every in-game month and on level transitions, plus 3
  manual slots. Saves carry a schema version and checksum, corrupt saves are
  detected, and campaign progress can be reset.
- **Original audio**: synthesised chiptune and French-electro loops plus
  sound effects (cash register, alerts, funding, typing, coffee machine,
  bankruptcy sting). Volume settings and a 30 FPS low-power mode are
  included.

## Build from source

### Requirements

- Rust **1.95+** (verified with 1.97.0); `rust-toolchain.toml` selects stable
- For the browser build:
  - `rustup target add wasm32-unknown-unknown`
  - `cargo install wasm-bindgen-cli --version 0.2.129` (must match `wasm-bindgen` in `Cargo.lock`)
  - optional: `wasm-opt` from [binaryen](https://github.com/WebAssembly/binaryen/releases) on `PATH` (smaller binary)
- On Linux, `cargo test --workspace` and the native client also build the Bevy
  client, whose audio backend links ALSA: `sudo apt install pkg-config libasound2-dev`.
  (X11/Wayland/xkbcommon are loaded at runtime; a desktop session provides them.)

### Tests

```bash
cargo test --workspace
```

The simulation tests live in the pure-Rust crates (`game-core`,
`game-content`, `platform`) and need no GPU. `cargo test` without
`--workspace` runs only those crates and needs no system libraries. The suite
includes a scripted autopilot that plays the **whole 12-level campaign to
victory** on several seeds, plus tests for
determinism, save round-trips, corrupt-save rejection, bankruptcy from
reckless hiring, churn punishing growth without retention, tech debt,
office placement rules, cap-table maths and grant paperwork.

```bash
cargo run -p game-content --example balance -- 42 quiet   # campaign balancing report
```

### Browser build

```bash
tools/build-web.sh                      # → dist/ (WebGPU + WebGL2 builds)
RENDERERS=webgl2 tools/build-web.sh     # single renderer, half the build time
cd dist && python3 -m http.server 8080
```

Bevy selects WebGPU or WebGL2 at compile time, so the script compiles
the client twice (`--features webgpu` and `--features webgl2`) and runs
`wasm-bindgen --target web` on each. It then runs `wasm-opt -Oz` if available,
and copies the HTML shell from `web/` and the runtime assets into `dist/`. The
size-oriented `wasm-release` profile is defined in `Cargo.toml`.

If the `wasm32-unknown-unknown` target cannot be installed (restricted network)
but the `rust-src` component is present, the script falls back to building the
standard library from source with `-Zbuild-std`. That is how the build was
produced in the sandbox used for development.

### Browser smoke test

```bash
pip install playwright && python -m playwright install chromium
python tools/smoke_test.py dist --shots smoke-shots
```

This starts headless Chromium against `dist/` the way a visitor arrives (the
loader chooses the renderer; `--renderer webgl2|webgpu` forces one). It waits
for the game to load, starts a new campaign with `Enter`, runs time at 4×,
answers events with the number keys, and checks that the monthly autosave lands
in `localStorage`. It then reloads the page, continues from the save, and fails
on any uncaught exception or Rust panic. Keys are held for 120 ms like a
person's: software rendering can draw only a few frames per second, and an
instantaneous key press can fall between frames.

### Native desktop

```bash
cargo run -p game-client --release
```

The native client is useful for development. It also has automation hooks for
screenshots and CI (`FTT_START=new|auto:<days>`, `FTT_SHOTS`, `FTT_EXIT`,
`FTT_TAB`, `FTT_SPEED`; see `crates/game-client/src/dev.rs`).

## Deploy (GitHub Pages)

Push the repository to GitHub, then in **Settings → Pages** set the source to
**GitHub Actions**. `.github/workflows/pages.yml` builds the game on every
push to `main` and publishes it at `https://<owner>.github.io/<repo>/`. All
paths in the build are relative, so it works under that sub-path.

## Deploy (Cloudflare Pages)

`dist/` is a plain static site. `web/_headers` (copied into `dist/`) sets the
`application/wasm` content type and caching.

```bash
npx wrangler pages deploy dist --project-name french-tech-tycoon
```

Or connect the repository in the Cloudflare dashboard with:

- Build command: `tools/build-web.sh` (the build image needs Rust; using the GitHub Action below is simpler)
- Output directory: `dist`

`.github/workflows/ci.yml` runs tests and clippy, builds `dist/`, runs the
browser smoke test, uploads `dist/` as an artifact, and deploys to Cloudflare
Pages on `main` when the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`
secrets are set.

Each wasm file is ~21 MB (~7 MB gzipped), under Cloudflare Pages' 25 MiB
per-file limit. For a host with a smaller cap, `tools/split-wasm.sh dist 12`
splits each module into ≤ 12 MiB parts plus a `parts.json`. The loader then
downloads the parts and joins them. A visitor downloads only one of the two builds. The
`wasm-release` profile uses `opt-level = "z"` with thin LTO, followed by
`wasm-opt -Oz`. Fat LTO is smaller still but needs more than 7 GB of RAM to
link.

## Architecture

```
crates/
  game-core/      Deterministic simulation. No Bevy, no browser APIs.
                  GameState is the single authoritative state; Command is the
                  only way to mutate it (the UI and the autopilot use the same
                  commands). economy, customers, employees, product/research,
                  funding, events, missions, office grid, save format.
  game-content/   Parses assets/data/*.ron (embedded at compile time) into
                  Content, validates cross-references, formats simulation
                  messages into French prose, and hosts the autopilot.
  platform/       Adapters: Storage (localStorage on web, files on desktop,
                  memory in tests) and clock. A mobile port adds another
                  Storage implementation.
  game-client/    Bevy 0.19 presentation: isometric renderer, animation
                  state machine, camera, input, egui management panels,
                  audio, saves, menus, explicit AppState (MainMenu, Playing,
                  Paused, MissionComplete, GameOver, Victory).
assets/
  data/*.ron      levels, events, tasks, employees, funding, market, office,
                  balancing. All tuning numbers live here, not in code.
  sprites, tilesets, characters, ui, backgrounds, fonts, audio
tools/
  art/            Python generators for every sprite atlas, portrait, ending
                  illustration and the pixel font (outputs are committed).
  audio/          numpy synthesiser for music and sound effects (→ Ogg Vorbis).
  build-web.sh    browser build
  split-wasm.sh   optional: split the wasm for hosts with a per-file size cap
  smoke_test.py   Playwright browser test
web/              index.html, style.css, loader.js, _headers, favicon
```

**Simulation, presentation and content are separate.** Revenue is never
computed in UI code, dialogue strings never sit next to economic formulas, and
missions and events are data with typed conditions and effects. The
simulation advances in fixed whole-day ticks. The client converts real time ×
speed into ticks with an accumulator, so frame rate never changes results.
Saves serialise the authoritative `GameState`, including the ChaCha RNG state,
never rendering entities, so a loaded game continues exactly as it would have.

**Saves on the web** use `localStorage`. It is synchronous, available in every
browser, and a full campaign save (~40–80 KB) is far below the per-origin
quota. Errors (private mode, quota) are surfaced in the UI. IndexedDB would
only be needed for much larger saves; the `Storage` trait allows swapping it in.

**Why egui for the panels:** the management UI is data-dense (tables, progress
bars, tooltips, dropdowns, scrolling feeds), which egui handles well with
little code. The office itself is rendered with Bevy sprites from texture
atlases, sorted by isometric depth.

## Art and audio

All visual and audio assets are original and generated by the scripts in
`tools/`. No third-party art, music or sound is included. The UI text font is
a subset of DejaVu Sans (Bitstream Vera / DejaVu licence); the pixel display
font *VeridixPixel* is original.

The palette follows an "À la French" black / electric green / off-white
identity: near-black `#101412`, graphite `#202822`, electric green `#7CFF52`,
deep green `#267D4B`, off-white `#F4F1E8`, light grey `#BEC5BB`. Orange `#F6A23C` and red
`#E45B5B` are reserved for financial danger, burnout and crises.

*Note:* the brief mentions a local reference video
(`Le passage du SaaS à l'IA native [ATc5dtUdosY].webm`). It was not present
in the development environment, so the art direction follows the written
palette and references instead of frames from that video.

## Disclaimer

This is satire. France Travail, Bpifrance, Station F and the other
institutions appear as simplified, fictionalised game mechanics. Nothing here
is legal, tax or funding advice, and every person, investor and company in the
game is fictional.

## License

MIT OR Apache-2.0.
