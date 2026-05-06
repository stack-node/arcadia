#!/usr/bin/env bash

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
CARGO_BIN="${HOME}/.cargo/bin"

if [[ ":${PATH}:" != *":${CARGO_BIN}:"* ]]; then
  export PATH="${CARGO_BIN}:${PATH}"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "Error: cargo not found. Install Rust via rustup, then retry."
  echo "Hint: https://rustup.rs"
  exit 1
fi

run_arcadia() {
  local build_mode="$1"
  local features="$2"

  echo
  if [[ -n "${build_mode}" ]]; then
    echo "Running: cargo run -p arcadia ${build_mode} --features ${features}"
  else
    echo "Running: cargo run -p arcadia --features ${features}"
  fi
  (
    cd "${ROOT_DIR}" || exit 1
    if [[ -n "${build_mode}" ]]; then
      cargo run -p arcadia "${build_mode}" --features "${features}"
    else
      cargo run -p arcadia --features "${features}"
    fi
  )
}

while true; do
  clear
  echo "=================================="
  echo " Arcadia Launcher"
  echo "=================================="
  echo "Choose option (type two keys, no Enter needed):"
  echo "  1A) Launch GUI Release"
  echo "  1B) Launch GUI Debug"
  echo "  2A) Launch Headless Release"
  echo "  2B) Launch Headless Debug"
  echo "  0X) Exit"
  echo
  printf "Enter choice: "

  read -r -s -n 2 raw_choice
  echo
  choice="$(printf '%s' "$raw_choice" | tr '[:lower:]' '[:upper:]')"

  case "$choice" in
    1A)
      run_arcadia "--release" "gui"
      read -r -p "Press Enter to continue..."
      ;;
    1B)
      run_arcadia "" "gui"
      read -r -p "Press Enter to continue..."
      ;;
    2A)
      run_arcadia "--release" "headless"
      read -r -p "Press Enter to continue..."
      ;;
    2B)
      run_arcadia "" "headless"
      read -r -p "Press Enter to continue..."
      ;;
    0X|00|0A|0B)
      echo "Goodbye."
      exit 0
      ;;
    *)
      echo
      echo "Invalid option. Use 1A, 1B, 2A, 2B, or 0X."
      read -r -p "Press Enter to continue..."
      ;;
  esac
done
