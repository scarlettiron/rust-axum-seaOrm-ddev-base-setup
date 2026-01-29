# Tenant API Documentation

The Tenant API provides multi-tenant management capabilities with auto-generated tenant IDs.

## Overview

Tenants represent isolated organizational units within the system. Each tenant is assigned a unique identifier in the format `TN_<uuid>` where the UUID has no dashes.

## Authentication

All tenant endpoints require API token authentication via:
- `Authorization: Bearer <token>` header, or
- `X-API-Key: <token>` header

## Endpoints

### List Tenants

```
GET /tenant
```

Returns a paginated list of tenants.

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `page` | integer | 1 | Page number |
| `per_page` | integer | 20 | Items per page |
| `status` | string | - | Filter by status (`active` or `removed`) |
| `display_name` | string | - | Filter by display name (partial match) |
| `tenant_id` | string | - | Filter by tenant ID (partial match) |

**Response:**

```json
{
  "items": [
    {
      "id": 1,
      "uuid": "550e8400-e29b-41d4-a716-446655440000",
      "tenant_id": "TN_550e8400e29b41d4a716446655440000",
      "display_name": "Acme Corp",
      "status": "active",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 20,
  "total_pages": 1
}
```

**Example:**

```bash
curl -X GET "https://erp-proxy-server.ddev.site/tenant?page=1&per_page=10&status=active" \
  -H "X-API-Key: your-api-token"
```

---

### Create Tenant

```
POST /tenant
```

Creates a new tenant with an auto-generated tenant ID.

**Request Body:**

```json
{
  "display_name": "Acme Corp"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `display_name` | string | No | Human-readable name for the tenant |

**Response (201 Created):**

```json
{
  "id": 1,
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "TN_550e8400e29b41d4a716446655440000",
  "display_name": "Acme Corp",
  "status": "active",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

**Example:**

```bash
curl -X POST "https://erp-proxy-server.ddev.site/tenant" \
  -H "X-API-Key: your-api-token" \
  -H "Content-Type: application/json" \
  -d '{"display_name": "Acme Corp"}'
```

---

### Get Tenant

```
GET /tenant/{tenant_id}
```

Retrieves a specific tenant by tenant ID.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `tenant_id` | string | Tenant ID in `TN_xxx` format |

**Response (200 OK):**

```json
{
  "id": 1,
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "TN_550e8400e29b41d4a716446655440000",
  "display_name": "Acme Corp",
  "status": "active",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

**Response (404 Not Found):**

```json
{
  "error": "Tenant not found"
}
```

**Example:**

```bash
curl -X GET "https://erp-proxy-server.ddev.site/tenant/TN_550e8400e29b41d4a716446655440000" \
  -H "X-API-Key: your-api-token"
```

---

### Update Tenant

```
PUT /tenant/{tenant_id}
```

Updates a tenant's display name or status.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `tenant_id` | string | Tenant ID in `TN_xxx` format |

**Request Body:**

```json
{
  "display_name": "Updated Name",
  "status": "active"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `display_name` | string | No | New display name |
| `status` | string | No | New status (`active` or `removed`) |

**Response (200 OK):**

```json
{
  "id": 1,
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "TN_550e8400e29b41d4a716446655440000",
  "display_name": "Updated Name",
  "status": "active",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T12:00:00Z"
}
```

**Example:**

```bash
curl -X PUT "https://erp-proxy-server.ddev.site/tenant/TN_550e8400e29b41d4a716446655440000" \
  -H "X-API-Key: your-api-token" \
  -H "Content-Type: application/json" \
  -d '{"display_name": "Updated Name"}'
```

---

### Delete Tenant (Soft Delete)

```
DELETE /tenant/{tenant_id}
```

Soft deletes a tenant by setting its status to `removed`. The tenant record is preserved in the database.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `tenant_id` | string | Tenant ID in `TN_xxx` format |

**Response (200 OK):**

```json
{
  "message": "Tenant removed successfully"
}
```

**Response (404 Not Found):**

```json
{
  "error": "Tenant not found"
}
```

**Example:**

```bash
curl -X DELETE "https://erp-proxy-server.ddev.site/tenant/TN_550e8400e29b41d4a716446655440000" \
  -H "X-API-Key: your-api-token"
```

---

## Tenant ID Format

Tenant IDs are automatically generated when creating a new tenant:

- Format: `TN_<32-character-hex-string>`
- Example: `TN_550e8400e29b41d4a716446655440000`
- The hex string is a UUID v4 with dashes removed
- Tenant IDs are unique and immutable

## Status Values

| Status | Description |
|--------|-------------|
| `active` | Tenant is active and operational |
| `removed` | Tenant has been soft-deleted |

## Error Responses

All endpoints may return:

| Status Code | Description |
|-------------|-------------|
| 400 | Bad Request - Invalid input |
| 401 | Unauthorized - Missing or invalid API token |
| 404 | Not Found - Tenant does not exist |
| 500 | Internal Server Error - Database or server error |

Error response format:

```json
{
  "error": "Error message description"
}
```

## Database Schema

```sql
CREATE TABLE tenant (
    id BIGSERIAL PRIMARY KEY,
    uuid UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    display_name TEXT,
    tenant_id TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status enum ('active', 'removed') NOT NULL DEFAULT 'active'
);
```
