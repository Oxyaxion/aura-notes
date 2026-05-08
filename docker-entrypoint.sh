#!/bin/sh
set -e

CONFIG=/data/clef-note.toml
mkdir -p /data

if [ ! -f "$CONFIG" ]; then
    if [ -n "$PASSWORD_HASH" ]; then
        printf 'password = "%s"\n' "$PASSWORD_HASH" > "$CONFIG"
    elif [ -n "$PASSWORD" ]; then
        echo "Hashing password…"
        HASH=$(/app/clef-note --hash-password "$PASSWORD")
        printf 'password = "%s"\n' "$HASH" > "$CONFIG"
        echo "Password hash written to $CONFIG"
    else
        echo "ERROR: set a Fly secret before deploying:"
        echo "  fly secrets set PASSWORD=yourpassword"
        exit 1
    fi
fi

exec /app/clef-note --storage /data --port "${PORT:-8080}"
