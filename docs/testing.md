# Testing Documentation

This guide covers running tests and using the Postman collection for API testing.

## Rust Tests

### Running Tests

```bash
# Run all tests
ddev cargo test

# Run specific test file
ddev cargo test --test tenant_tests

# Run tests with output
ddev cargo test -- --nocapture

# Run specific test function
ddev cargo test test_tenant_id_format
```

### Test Structure

Tests are located in the `tests/` directory:

```
tests/
└── tenant_tests.rs    # Tenant module tests
```

### Test Categories

#### Unit Tests

Located in `tests/tenant_tests.rs`:

| Test Module | Description |
|-------------|-------------|
| `tenant_service_unit_tests` | Tenant ID format and uniqueness |
| `tenant_request_validation_tests` | Request body serialization |
| `tenant_response_tests` | Response structure validation |
| `tenant_pagination_tests` | Pagination calculations |
| `uuid_validation_tests` | UUID parsing validation |

### Writing New Tests

Example test structure:

```rust
#[cfg(test)]
mod my_tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(1 + 1, 2);
    }

    #[tokio::test]
    async fn test_async_something() {
        // Async test code
    }
}
```

## Postman Collection

### Files

Located in the `postman/` directory:

| File | Description |
|------|-------------|
| `erp-proxy-server.postman_collection.json` | API request collection |
| `erp-proxy-server.postman_environment.json` | Environment variables |

### Importing into Postman

1. Open Postman
2. Click **Import** button
3. Select both files from the `postman/` directory
4. The collection and environment will be imported

### Configuration

After importing, configure the environment:

1. Click the **Environments** tab
2. Select **ERP Proxy Server - Local**
3. Set `api_token` to your valid API token
4. Click **Save**

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `base_url` | Server base URL | `https://erp-proxy-server.ddev.site` |
| `api_token` | Your API token | `your-api-token-here` |
| `tenant_id` | Auto-populated after create | (empty) |

### Available Requests

#### Health Endpoints

| Request | Method | Auth Required |
|---------|--------|---------------|
| Health Check | GET /healthcheck | No |
| Root Health Check | GET / | No |

#### Tenant Endpoints

| Request | Method | Auth Required |
|---------|--------|---------------|
| List Tenants | GET /tenant | Yes |
| List Active Only | GET /tenant?status=active | Yes |
| Create Tenant | POST /tenant | Yes |
| Create Without Name | POST /tenant | Yes |
| Get Tenant | GET /tenant/{tenant_id} | Yes |
| Get Not Found | GET /tenant/{tenant_id} | Yes |
| Update Tenant | PUT /tenant/{tenant_id} | Yes |
| Update Status | PUT /tenant/{tenant_id} | Yes |
| Reactivate | PUT /tenant/{tenant_id} | Yes |
| Delete (Soft) | DELETE /tenant/{tenant_id} | Yes |
| Delete Not Found | DELETE /tenant/{tenant_id} | Yes |

### Running Tests in Postman

Each request includes test scripts that validate:
- Response status codes
- Response body structure
- Field values and types

To run all tests:

1. Open the collection
2. Click **Run** button
3. Select requests to run
4. Click **Run ERP Proxy Server API**

### Workflow Example

Typical testing workflow:

1. **Create Tenant** - Creates a new tenant, stores `tenant_id`
2. **Get Tenant** - Retrieves the created tenant
3. **Update Tenant** - Modifies display name
4. **List Tenants** - Verifies tenant appears in list
5. **Delete Tenant** - Soft deletes the tenant

### Auto-populated Variables

The **Create Tenant** request automatically stores the `tenant_id` in collection variables for use in subsequent requests.

## Integration Testing

For full integration tests with a real database:

```bash
# Ensure database is running
ddev start

# Run migrations
ddev sea-orm-cli migrate up

# Run integration tests
ddev cargo test --test tenant_tests
```

## Test Database

Tests run against the development database by default. For isolated testing:

1. Create a test database
2. Set `DATABASE_URL` environment variable
3. Run migrations on test database
4. Run tests

```bash
# Create test database
ddev exec psql -U db -c "CREATE DATABASE test_db;"

# Run tests with test database
DATABASE_URL=postgres://db:db@db:5432/test_db ddev cargo test
```

## Continuous Integration

Example GitHub Actions workflow:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: test
        ports:
          - 5432:5432

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        env:
          DATABASE_URL: postgres://test:test@localhost:5432/test
        run: cargo test
```

## Code Coverage

To generate code coverage reports:

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
ddev cargo tarpaulin --out Html
```

The report will be generated in `tarpaulin-report.html`.
