#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
CARGO_BIN="${HOME}/.cargo/bin"

mkdir -p "${BIN_DIR}"

cat > "${BIN_DIR}/arcadia" <<EOF
#!/usr/bin/env bash
set -euo pipefail

export PATH="${CARGO_BIN}:\${PATH}"
CONFIG_ROOT="\${HOME}/Arcadia/Configuration"

open_config_file() {
  local config_name="\$1"
  local config_file="\${config_name}"
  if [[ "\${config_file}" != *.toml ]]; then
    config_file="\${config_file}.toml"
  fi

  mkdir -p "\${CONFIG_ROOT}"
  touch "\${CONFIG_ROOT}/\${config_file}"
  open "\${CONFIG_ROOT}/\${config_file}"
}

if [[ "\${1:-}" == "configuration" ]]; then
  if [[ -z "\${2:-}" ]]; then
    echo "Usage: arcadia configuration <config file>"
    exit 1
  fi
  open_config_file "\${2}"
  exit 0
fi

cd "${ROOT_DIR}"
cargo build -p arcadia --no-default-features --features headless >/dev/null
exec "${ROOT_DIR}/target/debug/arcadia" "\$@"
EOF

cat > "${BIN_DIR}/arcadia-gui" <<EOF
#!/usr/bin/env bash
set -euo pipefail

export PATH="${CARGO_BIN}:\${PATH}"
CONFIG_ROOT="\${HOME}/Arcadia/Configuration"

open_config_file() {
  local config_name="\$1"
  local config_file="\${config_name}"
  if [[ "\${config_file}" != *.toml ]]; then
    config_file="\${config_file}.toml"
  fi

  mkdir -p "\${CONFIG_ROOT}"
  touch "\${CONFIG_ROOT}/\${config_file}"
  open "\${CONFIG_ROOT}/\${config_file}"
}

if [[ "\${1:-}" == "configuration" ]]; then
  if [[ -z "\${2:-}" ]]; then
    echo "Usage: arcadia-gui configuration <config file>"
    exit 1
  fi
  open_config_file "\${2}"
  exit 0
fi

cd "${ROOT_DIR}"
cargo build -p arcadia --no-default-features --features gui >/dev/null
exec "${ROOT_DIR}/target/debug/arcadia" "\$@"
EOF

chmod +x "${BIN_DIR}/arcadia" "${BIN_DIR}/arcadia-gui"

if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
  echo "Installed commands, but ${BIN_DIR} is not currently on PATH in this shell."
  echo "Add this to your shell profile (~/.zshrc):"
  echo "  export PATH=\"${BIN_DIR}:\$PATH\""
else
  echo "Installed commands to ${BIN_DIR}"
fi

echo "Use:"
echo "  arcadia <args>"
echo "  arcadia-gui <args>"
