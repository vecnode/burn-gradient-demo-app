#!/bin/bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

cleanup() {
    echo ""
    echo "Stopping servers"
    pkill -f "dx serve" 2>/dev/null || true
    sleep 1
    exit 0
}

trap cleanup SIGINT SIGTERM

echo "Starting pattern-clock"
echo "Web: http://127.0.0.1:8080"
echo "HTTP Server: http://127.0.0.1:8080"
echo "Press Ctrl+C to stop"
echo ""

# Start web server first (required for desktop app to send signals)
echo "=========================================="
echo "Step 1: Starting WEB SERVER"
echo "=========================================="
echo "This will start:"
echo "  - HTTP server on port 8080"
echo ""
if command -v gnome-terminal >/dev/null 2>&1; then
    gnome-terminal --title "Web Server" -- bash -c "cd '$SCRIPT_DIR' && dx serve 2>&1 | grep -v '\[200\] /api/messages/stream' | tee /tmp/pattern-clock-web.log; exec bash" &
elif command -v xterm >/dev/null 2>&1; then
    xterm -T "Web Server" -e "cd '$SCRIPT_DIR' && dx serve 2>&1 | grep -v '\[200\] /api/messages/stream' | tee /tmp/pattern-clock-web.log; exec bash" &
else
    dx serve 2>&1 | grep -v '\[200\] /api/messages/stream' | tee /tmp/pattern-clock-web.log &
fi

# Wait for web server to fully start
echo "Waiting for web server to start..."
sleep 3
echo "✓ Web server should be ready"
echo ""

# Start desktop app (connects to web server)
echo "=========================================="
echo "Step 2: Starting DESKTOP APP"
echo "=========================================="
echo "This will:"
echo "  - Open desktop window"
echo "  - Connect to web server on port 8080"
echo ""
if command -v gnome-terminal >/dev/null 2>&1; then
    gnome-terminal --title "Desktop App" -- bash -c "cd '$SCRIPT_DIR' && dx serve --platform desktop 2>&1 | tee /tmp/pattern-clock-desktop.log; exec bash" &
elif command -v xterm >/dev/null 2>&1; then
    xterm -T "Desktop App" -e "cd '$SCRIPT_DIR' && dx serve --platform desktop 2>&1 | tee /tmp/pattern-clock-desktop.log; exec bash" &
else
    dx serve --platform desktop 2>&1 | tee /tmp/pattern-clock-desktop.log &
fi

echo ""
echo "=========================================="
echo "✓ Both servers started!"
echo "=========================================="
echo "Web Interface: http://127.0.0.1:8080"
echo "HTTP Server:  http://127.0.0.1:8080"
echo ""
echo "Logs:"
echo "  Web server:    /tmp/pattern-clock-web.log"
echo "  Desktop app:   /tmp/pattern-clock-desktop.log"
echo ""
echo "Check the terminals for:"
echo "  ✓ HTTP server listening on http://127.0.0.1:8080"
echo ""

while true; do
    sleep 1
    if ! pgrep -f "dx serve" >/dev/null 2>&1; then
        break
    fi
done
