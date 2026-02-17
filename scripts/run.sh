#!/usr/bin/env zsh
set -euo pipefail

echo "--- smart-road: environment bootstrap & run helper ---"

# If VCPKG_ROOT is set to a Windows path or otherwise points to vcpkg,
# it will force the sdl2-sys build script down the vcpkg route. Unset it.
if [[ -n "${VCPKG_ROOT-}" ]]; then
  echo "Unsetting VCPKG_ROOT (was: $VCPKG_ROOT) for this run"
  unset VCPKG_ROOT
fi

# Ensure pkg-config is available
if ! command -v pkg-config >/dev/null 2>&1; then
  if command -v brew >/dev/null 2>&1; then
    echo "pkg-config not found: installing via Homebrew..."
    brew install pkg-config
  else
    echo "pkg-config is required but not installed."
    echo "Please install Homebrew (https://brew.sh/) and then run: brew install pkg-config sdl2"
    exit 1
  fi
fi

# Ensure SDL2 is available via pkg-config. Install via Homebrew if missing.
if ! pkg-config --exists sdl2; then
  if command -v brew >/dev/null 2>&1; then
    echo "SDL2 not found via pkg-config: installing via Homebrew..."
    brew install sdl2
  else
    echo "SDL2 not found by pkg-config. Install SDL2 (Homebrew recommended): brew install sdl2"
    exit 1
  fi
fi

# Ensure PKG_CONFIG_PATH includes Homebrew sdl2 if brew is available
if command -v brew >/dev/null 2>&1; then
  BREW_SDL2_PREFIX=$(brew --prefix sdl2 2>/dev/null || true)
  if [[ -n "$BREW_SDL2_PREFIX" ]]; then
    export PKG_CONFIG_PATH="$BREW_SDL2_PREFIX/lib/pkgconfig:${PKG_CONFIG_PATH-}"
    echo "Set PKG_CONFIG_PATH to include: $BREW_SDL2_PREFIX/lib/pkgconfig"
  fi
fi

echo "Running: cargo clean && cargo run"
cargo clean
cargo run

echo "--- finished ---"
