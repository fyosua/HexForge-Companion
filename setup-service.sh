#!/usr/bin/env bash
set -euo pipefail

echo "=== HexForge Proxy Service Setup ==="

# Check if proxy is already running
if systemctl --user is-active hexforge-proxy.service &>/dev/null; then
    echo "Restarting service..."
    systemctl --user restart hexforge-proxy.service
else
    echo "Starting service..."
    systemctl --user daemon-reload
    systemctl --user enable hexforge-proxy.service
    systemctl --user start hexforge-proxy.service
fi

echo ""
echo "Checking status..."
sleep 1
systemctl --user status hexforge-proxy.service --no-pager

echo ""
echo "=== Testing routes ==="
sleep 1
curl -s http://raspberrypi.local:1421/ | head -c 100
echo ""
echo ""

echo "=== Service logs ==="
journalctl --user -u hexforge-proxy.service -n 10 --no-pager
