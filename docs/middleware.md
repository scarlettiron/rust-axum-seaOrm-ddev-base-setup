# Middleware Documentation

The ERP Proxy Server uses several middleware layers for security and logging. Middleware is applied in order, with each layer processing requests before passing them to the next.

## Middleware Stack Order

```
Request → Logging → Allowed Hosts → IP Auth → API Token Auth → Route Handler
```

## Request Logging Middleware

**File**: `src/middleware/logging.rs`

Logs all incoming requests and outgoing responses with timing information.

### Features

- Logs request method, path, client IP, and timestamp
- Logs response status code and duration
- Filters sensitive headers from logs
- Can be disabled via environment variable

### Sensitive Headers (Never Logged)

The following headers are automatically filtered from logs:

- `authorization`
- `cookie`
- `set-cookie`
- `x-api-key`
- `x-auth-token`
- `x-access-token`
- `x-refresh-token`
- `proxy-authorization`

### Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `REQUEST_LOGGING` | `true` | Set to `false` or `0` to disable logging |

### Log Output Example

```
INFO direction="incoming" method="GET" path="/healthcheck" client_ip="192.168.1.1" timestamp="2025-01-26 12:00:00 UTC" headers=[...] "Request received"
INFO direction="outgoing" method="GET" path="/healthcheck" client_ip="192.168.1.1" status=200 duration_ms=5 headers=[...] "Response sent"
```

### Client IP Detection

The middleware extracts client IP from headers in this priority:
1. `X-Forwarded-For` (first IP in chain)
2. `X-Real-IP`
3. Falls back to "unknown"

---

## Allowed Hosts Middleware

**File**: `src/config/hosts.rs`

Validates the `Host` header against a whitelist to prevent host header attacks.

### Features

- Validates incoming Host header
- Supports exact match and wildcard subdomains
- Returns 400 Bad Request for invalid hosts

### Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `ALLOWED_HOSTS` | `erp-proxy-server.ddev.site` | Comma-separated list of allowed hosts |

### Host Matching Rules

1. **Exact match**: `example.com` matches `example.com`
2. **Subdomain match**: `example.com` matches `api.example.com`
3. **Wildcard**: `.example.com` matches `anything.example.com`

### Example Configuration

```bash
# Single host
ALLOWED_HOSTS=api.example.com

# Multiple hosts
ALLOWED_HOSTS=api.example.com,admin.example.com,localhost

# Wildcard subdomain
ALLOWED_HOSTS=.example.com
```

---

## IP Address Authentication Middleware

**File**: `src/middleware/ip_auth.rs`

Restricts access to requests from whitelisted IP addresses stored in the database.

### Features

- Validates client IP against `allowed_ip_address` database table
- Logs unauthorized attempts with full request details (CRITICAL level)
- Skips validation for public routes
- Returns 403 Forbidden for unauthorized IPs

### Enable/Disable

The middleware is controlled by a hardcoded flag in `src/config/ip_address_auth.rs`:

```rust
const ENABLED: bool = true;  // Set to false to disable
```

### Public Routes (No IP Check Required)

- `/`
- `/healthcheck`
- `/local/swagger-ui`
- `/api-doc/openapi.json`

### Database Table: `allowed_ip_address`

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `ip_address` | VARCHAR | IP address to allow |
| `description` | VARCHAR | Optional description |
| `is_active` | BOOLEAN | Whether the IP is currently allowed |
| `created_at` | TIMESTAMP | Creation timestamp |
| `updated_at` | TIMESTAMP | Last update timestamp |

### Adding Allowed IPs

```sql
INSERT INTO allowed_ip_address (ip_address, description, is_active, created_at, updated_at)
VALUES ('192.168.1.100', 'Office network', true, NOW(), NOW());
```

### Unauthorized Access Log Example

```
ERROR severity="CRITICAL" event="unauthorized_ip_address_attempt" client_ip="10.0.0.1" route="/api/data" method="GET" headers="..." body="..." "Unauthorized IP address attempt detected"
```

---

## API Token Authentication Middleware

**File**: `src/middleware/api_token_auth.rs`

Validates API tokens provided in request headers against the database.

### Features

- Accepts tokens via `Authorization: Bearer <token>` header
- Accepts tokens via `X-API-Key: <token>` header
- Validates against `api_token` database table
- Logs unauthorized attempts with full request details (CRITICAL level)
- Skips validation for public routes
- Returns 401 Unauthorized for missing/invalid tokens

### Enable/Disable

The middleware is controlled by a hardcoded flag in `src/config/api_token_auth.rs`:

```rust
const ENABLED: bool = true;  // Set to false to disable
```

### Token Header Formats

```bash
# Bearer token format
curl -H "Authorization: Bearer your-api-token" https://erp-proxy-server.ddev.site/api/endpoint

# X-API-Key format
curl -H "X-API-Key: your-api-token" https://erp-proxy-server.ddev.site/api/endpoint
```

### Public Routes (No Token Required)

- `/`
- `/healthcheck`
- `/local/swagger-ui`
- `/api-doc/openapi.json`

### Database Table: `api_token`

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER | Primary key |
| `token` | VARCHAR | The API token value |
| `description` | VARCHAR | Optional description |
| `is_active` | BOOLEAN | Whether the token is currently valid |
| `created_at` | TIMESTAMP | Creation timestamp |
| `updated_at` | TIMESTAMP | Last update timestamp |

### Adding API Tokens

```sql
INSERT INTO api_token (token, description, is_active, created_at, updated_at)
VALUES ('sk_live_abc123xyz789', 'Production API key', true, NOW(), NOW());
```

### Unauthorized Access Log Examples

Missing token:
```
ERROR severity="CRITICAL" event="unauthorized_api_token_missing" client_ip="192.168.1.1" route="/api/data" method="POST" headers="..." body="..." "Unauthorized request: No API token provided"
```

Invalid token:
```
ERROR severity="CRITICAL" event="unauthorized_api_token_attempt" api_token="invalid-token" client_ip="192.168.1.1" route="/api/data" method="POST" headers="..." body="..." "Unauthorized API token attempt detected"
```

---

## Security Considerations

### Log Data

Unauthorized access attempts are logged with:
- Full request headers (sensitive headers filtered)
- Request body content
- Client IP address
- Route and method

This data is valuable for security auditing but may contain sensitive information. Ensure logs are stored securely and rotated appropriately.

### Token Generation

API tokens should be:
- Generated with cryptographically secure random generators
- At least 32 characters long
- Unique per client/application

### IP Whitelisting

Consider that:
- IP addresses can be spoofed (use in combination with API tokens)
- NAT and proxy servers may cause legitimate clients to appear from different IPs
- IPv6 addresses should be accounted for if applicable
