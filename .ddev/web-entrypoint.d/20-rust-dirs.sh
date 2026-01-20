#!/usr/bin/env bash
set -euo pipefail

# Create parent dir (bind point) and ensure the container user can write
mkdir -p /var/lib/rust

# Ensure the mountpoints exist (harmless if volumes already mounted)
mkdir -p /var/lib/rust/.cargo /var/lib/rust/.rustup /var/lib/rust/target

# DDEV web user is typically uid 1000; make it writable.
# Use numeric IDs to avoid needing to know the username.
chown -R 1000:1000 /var/lib/rust || true
chmod -R u+rwX,g+rwX /var/lib/rust || true