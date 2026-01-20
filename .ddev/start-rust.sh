#!/bin/bash
set -e

echo "Waiting for database to be ready..."
# Wait for PostgreSQL to be ready
until PGPASSWORD=db psql -h db -U db -d db -c '\q' 2>/dev/null; do
  echo "PostgreSQL is unavailable - sleeping"
  sleep 1
done

echo "Database is ready!"

# Build the project if needed
echo "Building Rust project..."
cargo build --release

# Run the server
echo "Starting Rust/Axum server..."
exec cargo run --release
