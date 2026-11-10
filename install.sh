#!/usr/bin/env bash
set -euo pipefail

BINARY_NAME="lorianworkspace"
INSTALL_DIR="${HOME}/.local/bin"
SERVICE_DIR="${HOME}/.config/systemd/user"
SERVICE_SRC="./lorianworkspace.service"

echo "==> Building ${BINARY_NAME} (release)..."
cargo build --release

echo "==> Installing binary to ${INSTALL_DIR}/"
mkdir -p "${INSTALL_DIR}"
cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/"

echo "==> Installing systemd user service..."
mkdir -p "${SERVICE_DIR}"
cp "${SERVICE_SRC}" "${SERVICE_DIR}/"

echo "==> Reloading systemd daemon..."
systemctl --user daemon-reload

echo ""
echo "=== Installation complete ==="
echo ""
echo "To start the daemon now:"
echo "  systemctl --user start lorianworkspace"
echo ""
echo "To enable on login:"
echo "  systemctl --user enable lorianworkspace"
echo ""
echo "Or use the built-in subcommands:"
echo "  ${INSTALL_DIR}/lorianworkspace start"
echo "  ${INSTALL_DIR}/lorianworkspace stop"
echo "  ${INSTALL_DIR}/lorianworkspace status"
echo "  ${INSTALL_DIR}/lorianworkspace pause"
echo "  ${INSTALL_DIR}/lorianworkspace resume"
echo "  ${INSTALL_DIR}/lorianworkspace next"
echo "  ${INSTALL_DIR}/lorianworkspace reload"
echo "  ${INSTALL_DIR}/lorianworkspace autostart"
echo ""
echo "Interactive mode (foreground):"
echo "  ${INSTALL_DIR}/lorianworkspace"
