#!/usr/bin/env bash
# Compile la version navigateur d'A la French (le jeu) dans ./dist
#
#   tools/build-web.sh                   # les deux rendus (WebGPU + repli WebGL2)
#   RENDERERS=webgl2 tools/build-web.sh  # un seul (itération plus rapide)
#   PROFILE=dev tools/build-web.sh       # non optimisé, compilation plus rapide
#
# Prérequis : rustup target add wasm32-unknown-unknown
#             cargo install wasm-bindgen-cli --version <celle de Cargo.lock>
# Optionnel : wasm-opt (version native de binaryen, sur GitHub) dans le PATH pour
#             un binaire plus petit. Évitez le paquet npm « binaryen » : c'est un portage JS lent.
#
# Si la cible wasm32 ne peut pas être installée (hors ligne / réseau restreint)
# mais que le composant `rust-src` est présent, le script se rabat sur la
# compilation de la bibliothèque standard depuis les sources (-Zbuild-std).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OUT="${OUT:-$ROOT/dist}"
RENDERERS="${RENDERERS:-webgl2 webgpu}"
PROFILE="${PROFILE:-wasm-release}"
TARGET=wasm32-unknown-unknown
BIN=french-tech-tycoon

log() { printf '\033[1;32m==>\033[0m %s\n' "$*"; }
die() { printf '\033[1;31merreur :\033[0m %s\n' "$*" >&2; exit 1; }

command -v wasm-bindgen >/dev/null || die "wasm-bindgen introuvable (cargo install wasm-bindgen-cli --version $(grep -A1 'name = "wasm-bindgen"$' Cargo.lock | sed -n 's/version = "\(.*\)"/\1/p'))"

lock_ver="$(grep -A1 'name = "wasm-bindgen"$' Cargo.lock | sed -n 's/version = "\(.*\)"/\1/p')"
cli_ver="$(wasm-bindgen --version | awk '{print $2}')"
[ "$lock_ver" = "$cli_ver" ] || die "wasm-bindgen-cli $cli_ver ne correspond pas à wasm-bindgen $lock_ver dans Cargo.lock"

SYSROOT="$(rustc --print sysroot)"
BUILD_STD=()
if [ -d "$SYSROOT/lib/rustlib/$TARGET/lib" ]; then
  log "Bibliothèque standard $TARGET installée : utilisée"
elif [ -d "$SYSROOT/lib/rustlib/src/rust/library/std" ]; then
  log "std $TARGET non installée : compilation depuis rust-src (-Zbuild-std)"
  export RUSTC_BOOTSTRAP=1
  BUILD_STD=(-Zbuild-std=std,panic_abort)
else
  die "Lancez : rustup target add $TARGET"
fi

case "$PROFILE" in
  dev) PROFILE_DIR=debug ;;
  release) PROFILE_DIR=release ;;
  *) PROFILE_DIR="$PROFILE" ;;
esac

rm -rf "$OUT"
mkdir -p "$OUT"

for r in $RENDERERS; do
  case "$r" in webgl2|webgpu) ;; *) die "rendu inconnu $r" ;; esac
  log "Compilation ($r, profil $PROFILE)…"
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

# Si un seul rendu a été compilé, le chargeur l'utilise pour les deux chemins.
for r in webgl2 webgpu; do
  if [ ! -d "$OUT/pkg-$r" ]; then
    other=$([ "$r" = webgl2 ] && echo webgpu || echo webgl2)
    if [ "$r" = webgpu ] && [ -d "$OUT/pkg-webgl2" ]; then
      # WebGL2 fonctionne partout où WebGPU fonctionne.
      cp -r "$OUT/pkg-webgl2" "$OUT/pkg-webgpu"
    elif [ -d "$OUT/pkg-$other" ]; then
      log "attention : seul pkg-$other a été compilé ; pkg-$r manque"
    fi
  fi
done

log "Copie de la page web et des ressources"
cp web/index.html web/style.css web/loader.js "$OUT/"
[ -f web/favicon.png ] && cp web/favicon.png "$OUT/"
[ -f web/_headers ] && cp web/_headers "$OUT/"
mkdir -p "$OUT/assets"
for d in audio backgrounds characters fonts sprites tilesets ui; do
  cp -r "assets/$d" "$OUT/assets/"
done

log "Terminé → $OUT"
du -sh "$OUT" "$OUT"/pkg-* 2>/dev/null
