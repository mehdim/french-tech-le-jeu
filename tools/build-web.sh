#!/usr/bin/env bash
# Build the browser version of French Tech Tycoon into ./dist
#
#   tools/build-web.sh                 # both renderers (WebGPU + WebGL2 fallback)
#   RENDERERS=webgl2 tools/build-web.sh  # only one (faster iteration)
#   PROFILE=dev tools/build-web.sh     # unoptimised, faster compile
#
# Requirements: rustup target add wasm32-unknown-unknown
#               cargo install wasm-bindgen-cli --version <same as Cargo.lock>
# Optional:     wasm-opt (native binaryen release from GitHub) on PATH for a
#               smaller binary. Avoid the npm "binaryen" package: it is a slow JS port.
#
# If the wasm32 target cannot be installed (offline / restricted network)
# but the `rust-src` component is present, the script falls back to
# building the standard library from source (-Zbuild-std).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUT="${OUT:-$ROOT/dist}"
RENDERERS="${RENDERERS:-webgl2 webgpu}"
PROFILE="${PROFILE:-wasm-release}"
TARGET=wasm32-unknown-unknown
BIN=french-tech-tycoon

log() { printf '\033[1;32m==>\033[0m %s\n' "$*"; }
die() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

command -v wasm-bindgen >/dev/null || die "wasm-bindgen not found (cargo install wasm-bindgen-cli --version $(grep -A1 'name = "wasm-bindgen"$' Cargo.lock | sed -n 's/version = "\(.*\)"/\1/p'))"

lock_ver="$(grep -A1 'name = "wasm-bindgen"$' Cargo.lock | sed -n 's/version = "\(.*\)"/\1/p')"
cli_ver="$(wasm-bindgen --version | awk '{print $2}')"
[ "$lock_ver" = "$cli_ver" ] || die "wasm-bindgen-cli $cli_ver does not match wasm-bindgen $lock_ver in Cargo.lock"

SYSROOT="$(rustc --print sysroot)"
BUILD_STD=()
if [ -d "$SYSROOT/lib/rustlib/$TARGET/lib" ]; then
  log "Using installed $TARGET standard library"
elif [ -d "$SYSROOT/lib/rustlib/src/rust/library/std" ]; then
  log "$TARGET std not installed: building it from rust-src (-Zbuild-std)"
  export RUSTC_BOOTSTRAP=1
  BUILD_STD=(-Zbuild-std=std,panic_abort)
else
  die "Run: rustup target add $TARGET"
fi

case "$PROFILE" in
  dev) PROFILE_DIR=debug ;;
  release) PROFILE_DIR=release ;;
  *) PROFILE_DIR="$PROFILE" ;;
esac

rm -rf "$OUT"
mkdir -p "$OUT"

for r in $RENDERERS; do
  case "$r" in webgl2|webgpu) ;; *) die "unknown renderer $r" ;; esac
  log "Compiling ($r, profile $PROFILE)…"
  cargo build -p game-client --bin "$BIN" --target "$TARGET" --profile "$PROFILE" \
    --no-default-features --features "$r,audio" ${BUILD_STD[@]+"${BUILD_STD[@]}"}

  wasm="target/$TARGET/$PROFILE_DIR/$BIN.wasm"
  log "wasm-bindgen → pkg-$r"
  wasm-bindgen --target web --no-typescript --out-dir "$OUT/pkg-$r" --out-name ftt "$wasm"

  if command -v wasm-opt >/dev/null && [ "$PROFILE" != dev ]; then
    log "wasm-opt -Oz"
    wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
      --enable-mutable-globals --enable-reference-types --enable-multivalue \
      "$OUT/pkg-$r/ftt_bg.wasm" -o "$OUT/pkg-$r/ftt_bg.opt.wasm"
    mv "$OUT/pkg-$r/ftt_bg.opt.wasm" "$OUT/pkg-$r/ftt_bg.wasm"
  fi
done

# If only one renderer was built, let the loader use it for both paths.
for r in webgl2 webgpu; do
  if [ ! -d "$OUT/pkg-$r" ]; then
    other=$([ "$r" = webgl2 ] && echo webgpu || echo webgl2)
    if [ "$r" = webgpu ] && [ -d "$OUT/pkg-webgl2" ]; then
      # WebGL2 works everywhere WebGPU does.
      cp -r "$OUT/pkg-webgl2" "$OUT/pkg-webgpu"
    elif [ -d "$OUT/pkg-$other" ]; then
      log "warning: only pkg-$other built; pkg-$r is missing"
    fi
  fi
done

log "Copying web shell and assets"
cp web/index.html web/style.css web/loader.js "$OUT/"
[ -f web/favicon.png ] && cp web/favicon.png "$OUT/"
[ -f web/_headers ] && cp web/_headers "$OUT/"
mkdir -p "$OUT/assets"
for d in audio backgrounds characters fonts sprites tilesets ui; do
  cp -r "assets/$d" "$OUT/assets/"
done

log "Done → $OUT"
du -sh "$OUT" "$OUT"/pkg-* 2>/dev/null
