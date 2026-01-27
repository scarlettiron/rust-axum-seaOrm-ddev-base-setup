# DDEV Commands

This project includes custom DDEV commands for Rust development and service management.

## Rust Commands

### `ddev cargo`

Run cargo commands inside the web container with Rust environment loaded.

```bash
# Build the project
ddev cargo build

# Build for release
ddev cargo build --release

# Run the server
ddev cargo run

# Run tests
ddev cargo test

# Check code without building
ddev cargo check

# Format code
ddev cargo fmt

# Run clippy linter
ddev cargo clippy
```

### `ddev sea-orm-cli`

Run SeaORM CLI commands for database migrations and entity generation.

```bash
# Run pending migrations
ddev sea-orm-cli migrate up

# Rollback last migration
ddev sea-orm-cli migrate down

# Check migration status
ddev sea-orm-cli migrate status

# Generate entities from database
ddev sea-orm-cli generate entity -o entity/src

# Create a new migration
ddev sea-orm-cli migrate generate <migration_name>
```

## Redis Commands

### `ddev redis-cli`

Access the Redis CLI inside the Redis container.

```bash
# Interactive mode
ddev redis-cli

# Get all keys
ddev redis-cli KEYS '*'

# Get server info
ddev redis-cli INFO

# Check Redis version
ddev redis-cli --version

# Set a value
ddev redis-cli SET mykey "myvalue"

# Get a value
ddev redis-cli GET mykey
```

### `ddev redis-flush`

Flush all data from Redis cache.

```bash
ddev redis-flush
```

**Warning**: This deletes all Redis data. Use with caution.

## Database Commands

### `ddev adminer`

Open Adminer, a web-based database management tool.

```bash
ddev adminer
```

Opens in your default browser. Login credentials:
- **System**: PostgreSQL
- **Server**: db
- **Username**: db
- **Password**: db
- **Database**: db

### Direct PostgreSQL Access

```bash
# Connect to PostgreSQL CLI
ddev exec psql -U db -d db

# Run a SQL file
ddev exec psql -U db -d db -f /path/to/file.sql
```

## Service Management Commands

### `ddev redis-commander`

Open Redis Commander, a web-based Redis management tool.

```bash
ddev redis-commander
```

### `ddev redis-backend`

Access the Redis backend directly (useful for debugging).

```bash
ddev redis-backend
```

## Standard DDEV Commands

### Container Management

```bash
# Start containers
ddev start

# Stop containers
ddev stop

# Restart containers
ddev restart

# Remove containers (keeps data)
ddev remove

# Show container status
ddev describe

# Show running containers
ddev list
```

### Logging

```bash
# View all logs
ddev logs

# Follow logs in real-time
ddev logs -f

# View web container logs only
ddev logs -s web
```

### SSH Access

```bash
# SSH into web container
ddev ssh

# SSH into database container
ddev ssh -s db

# SSH into Redis container
ddev ssh -s redis
```

### Database Operations

```bash
# Export database
ddev export-db > backup.sql.gz

# Import database
ddev import-db < backup.sql.gz

# Snapshot database
ddev snapshot

# Restore snapshot
ddev snapshot restore <snapshot-name>
```

## Environment Variables

DDEV sets these environment variables automatically:

| Variable | Value | Description |
|----------|-------|-------------|
| `DATABASE_URL` | `postgres://db:db@db:5432/db` | PostgreSQL connection string |
| `RUST_LOG` | `debug` | Rust logging level |
| `PORT` | `3000` | Server port |

See [environment.md](environment.md) for all configurable options.
