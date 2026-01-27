# Environment Variables

This document describes all environment variables used by the ERP Proxy Server.

## Configuration Summary

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3000` | Server listening port |
| `DATABASE_URL` | `postgres://db:db@db:5432/db` | PostgreSQL connection string |
| `RUST_LOG` | `debug` | Logging level |
| `CORS_ALLOWED_ORIGINS` | `https://erp-proxy-server.ddev.site` | Allowed CORS origins |
| `ALLOWED_HOSTS` | `erp-proxy-server.ddev.site` | Allowed Host headers |
| `REQUEST_LOGGING` | `true` | Enable/disable request logging |

## Server Configuration

### PORT

The port the server listens on inside the container.

```bash
PORT=3000
```

**Note**: DDEV's nginx proxy handles external traffic and forwards to this port.

### RUST_LOG

Controls the logging verbosity using the `tracing` crate's filter syntax.

```bash
# Debug level (verbose)
RUST_LOG=debug

# Info level (standard)
RUST_LOG=info

# Warning level (minimal)
RUST_LOG=warn

# Error only
RUST_LOG=error

# Module-specific logging
RUST_LOG=erp_proxy_server=debug,sea_orm=info
```

## Database Configuration

### DATABASE_URL

PostgreSQL connection string in the format:

```
postgres://username:password@host:port/database
```

```bash
# DDEV default
DATABASE_URL=postgres://db:db@db:5432/db

# External database
DATABASE_URL=postgres://user:password@db.example.com:5432/erp_prod
```

**Connection Pool Settings** (hardcoded in `src/config/database.rs`):
- Max connections: 100
- Min connections: 5
- Connect timeout: 8 seconds
- Acquire timeout: 8 seconds
- Idle timeout: 8 seconds
- Max lifetime: 8 seconds

## CORS Configuration

### CORS_ALLOWED_ORIGINS

Comma-separated list of origins allowed to make cross-origin requests.

```bash
# Single origin
CORS_ALLOWED_ORIGINS=https://example.com

# Multiple origins
CORS_ALLOWED_ORIGINS=https://example.com,https://admin.example.com,http://localhost:3000
```

**Default**: `https://erp-proxy-server.ddev.site`

**Allowed Methods**: GET, POST, PUT, DELETE, OPTIONS, PATCH

**Allowed Headers**:
- `authorization`
- `content-type`
- `x-requested-with`
- `x-custom-host`
- `accept`
- `origin`

**Credentials**: Allowed

## Host Validation

### ALLOWED_HOSTS

Comma-separated list of valid Host headers. Requests with other Host headers receive 400 Bad Request.

```bash
# Single host
ALLOWED_HOSTS=api.example.com

# Multiple hosts
ALLOWED_HOSTS=api.example.com,admin.example.com,localhost

# Wildcard subdomain (matches *.example.com)
ALLOWED_HOSTS=.example.com
```

**Default**: `erp-proxy-server.ddev.site`

**Matching Rules**:
1. Exact match
2. Subdomain match (e.g., `example.com` allows `api.example.com`)
3. Wildcard prefix (e.g., `.example.com` allows `anything.example.com`)

## Logging Configuration

### REQUEST_LOGGING

Enable or disable the request logging middleware.

```bash
# Enable (default)
REQUEST_LOGGING=true

# Disable
REQUEST_LOGGING=false
REQUEST_LOGGING=0
```

When enabled, logs:
- Incoming requests (method, path, client IP, headers)
- Outgoing responses (status, duration)

Sensitive headers are automatically filtered from logs.

## DDEV Configuration

DDEV sets environment variables in `.ddev/config.yaml`:

```yaml
web_environment:
  - RUST_LOG=debug
  - DATABASE_URL=postgres://db:db@db:5432/db
  - PORT=3000
```

To add or modify environment variables:

1. Edit `.ddev/config.yaml`
2. Add variables under `web_environment`
3. Restart DDEV: `ddev restart`

## Local Development

For local development outside DDEV, create a `.env` file:

```bash
# .env
PORT=3000
DATABASE_URL=postgres://user:password@localhost:5432/erp_dev
RUST_LOG=debug
CORS_ALLOWED_ORIGINS=http://localhost:3000,http://localhost:5173
ALLOWED_HOSTS=localhost
REQUEST_LOGGING=true
```

**Note**: The application does not automatically load `.env` files. Use a tool like `dotenv` or export variables manually:

```bash
export $(cat .env | xargs) && cargo run
```

## Production Considerations

For production deployments:

1. **Use strong database credentials**:
   ```bash
   DATABASE_URL=postgres://erp_user:strong_password@db.internal:5432/erp_prod
   ```

2. **Restrict CORS origins**:
   ```bash
   CORS_ALLOWED_ORIGINS=https://app.example.com
   ```

3. **Validate production hosts**:
   ```bash
   ALLOWED_HOSTS=api.example.com
   ```

4. **Set appropriate log level**:
   ```bash
   RUST_LOG=info
   ```

5. **Consider disabling request logging in high-traffic scenarios**:
   ```bash
   REQUEST_LOGGING=false
   ```

## Hardcoded Configuration

Some configuration is hardcoded and requires code changes:

| Setting | Location | Current Value |
|---------|----------|---------------|
| API Token Auth Enabled | `src/config/api_token_auth.rs` | `true` |
| IP Address Auth Enabled | `src/config/ip_address_auth.rs` | `true` |
| DB Max Connections | `src/config/database.rs` | `100` |
| DB Min Connections | `src/config/database.rs` | `5` |
| DB Timeouts | `src/config/database.rs` | `8 seconds` |
