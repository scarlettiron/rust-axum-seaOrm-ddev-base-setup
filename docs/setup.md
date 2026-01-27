# Setup Guide

This guide walks you through setting up the ERP Proxy Server from scratch.

## Prerequisites

### Install DDEV

DDEV is required for local development. Follow the installation guide for your operating system:

- **macOS**: https://ddev.readthedocs.io/en/stable/users/install/ddev-installation/#macos
- **Windows (WSL2)**: https://ddev.readthedocs.io/en/stable/users/install/ddev-installation/#wsl2-docker-desktop
- **Linux**: https://ddev.readthedocs.io/en/stable/users/install/ddev-installation/#linux

### Install Docker

DDEV requires Docker to run containers:

- **Docker Desktop**: https://www.docker.com/products/docker-desktop/
- **Colima (macOS alternative)**: https://github.com/abiosoft/colima

## Installation

### 1. Clone the Repository

```bash
git clone <repository-url>
cd erp-proxy-server
```

### 2. Start DDEV

```bash
ddev start
```

This will:
- Create and start Docker containers for the web server, PostgreSQL, and Redis
- Install Rust and required build tools in the web container
- Configure nginx to proxy requests to the Rust server

### 3. Verify Rust Installation

```bash
ddev cargo --version
```

You should see the Rust cargo version output.

### 4. Build the Project

```bash
ddev cargo build
```

This compiles the Rust project and all dependencies.

### 5. Run Database Migrations

```bash
ddev sea-orm-cli migrate up
```

This creates the required database tables:
- `api_token` - For API token authentication
- `allowed_ip_address` - For IP address whitelisting

### 6. Seed Initial Data (Optional)

If you need to add initial API tokens or allowed IP addresses, connect to the database:

```bash
ddev adminer
```

Or use psql directly:

```bash
ddev exec psql -U db -d db
```

Example SQL to add an API token:

```sql
INSERT INTO api_token (token, description, is_active, created_at, updated_at)
VALUES ('your-api-token-here', 'Development token', true, NOW(), NOW());
```

Example SQL to add an allowed IP address:

```sql
INSERT INTO allowed_ip_address (ip_address, description, is_active, created_at, updated_at)
VALUES ('192.168.1.1', 'Development machine', true, NOW(), NOW());
```

### 7. Start the Server

```bash
ddev cargo run
```

The server will start on port 3000 inside the container and be accessible at:
- `https://erp-proxy-server.ddev.site`

## Accessing the Application

### Main Endpoints

| URL | Description |
|-----|-------------|
| `https://erp-proxy-server.ddev.site/` | Root healthcheck |
| `https://erp-proxy-server.ddev.site/healthcheck` | Healthcheck endpoint |
| `https://erp-proxy-server.ddev.site/local/swagger-ui/` | Swagger UI |
| `https://erp-proxy-server.ddev.site/api-doc/openapi.json` | OpenAPI spec |

### Admin Tools

| Command | Description |
|---------|-------------|
| `ddev adminer` | Open Adminer database GUI |
| `ddev redis-commander` | Open Redis Commander GUI |

## Troubleshooting

### Rust not found

If you see "cargo not found", restart the DDEV containers:

```bash
ddev restart
```

### Database connection errors

Verify the database is running:

```bash
ddev describe
```

Check the DATABASE_URL in `.ddev/config.yaml`:

```yaml
web_environment:
  - DATABASE_URL=postgres://db:db@db:5432/db
```

### Permission errors

If you encounter permission errors with Rust packages:

```bash
ddev exec sudo chown -R $(id -u):$(id -g) /home/.cargo
```

### Port conflicts

If port 3000 is already in use:

```bash
ddev exec lsof -i :3000
```

Kill any conflicting processes or change the PORT environment variable.
