# Swagger / OpenAPI Documentation

The ERP Proxy Server provides interactive API documentation using Swagger UI and OpenAPI 3.0 specification.

## Accessing Swagger UI

Once the server is running, access Swagger UI at:

```
https://erp-proxy-server.ddev.site/local/swagger-ui/
```

The raw OpenAPI JSON specification is available at:

```
https://erp-proxy-server.ddev.site/api-doc/openapi.json
```

## Configuration

### OpenAPI Setup

**File**: `src/openapi.rs`

The OpenAPI documentation is configured using the `utoipa` crate:

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::healthcheck,
        crate::auth::services::health_check,
        crate::admin::services::health_check,
    ),
    components(schemas(
        HealthCheckResponse,
        AuthHealthResponse,
        AdminHealthResponse,
    )),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Auth", description = "Authentication module endpoints"),
        (name = "Admin", description = "Admin module endpoints"),
    ),
    info(
        title = "ERP Proxy Server API",
        description = "API documentation for the ERP Proxy Server",
        version = "1.0.0"
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://erp-proxy-server.ddev.site", description = "DDEV development server"),
    )
)]
pub struct ApiDoc;
```

### Router Integration

**File**: `src/routes.rs`

Swagger UI is integrated into the main router:

```rust
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::openapi::ApiDoc;

pub fn create_router(state: AppState) -> Router {
    let swagger_ui = SwaggerUi::new("/local/swagger-ui")
        .url("/api-doc/openapi.json", ApiDoc::openapi());

    Router::new()
        .merge(swagger_ui)
        // ... other routes
}
```

## Adding New Endpoints to Swagger

### Step 1: Annotate the Handler

Add `#[utoipa::path]` attribute to your handler function:

```rust
use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct MyResponse {
    pub message: String,
    pub data: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/my-endpoint",
    tag = "MyModule",
    responses(
        (status = 200, description = "Success", body = MyResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("id" = i32, Path, description = "Resource ID"),
        ("filter" = Option<String>, Query, description = "Optional filter")
    )
)]
pub async fn my_handler() -> (StatusCode, Json<MyResponse>) {
    // ... implementation
}
```

### Step 2: Register in OpenAPI Config

Add the path and schema to `src/openapi.rs`:

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        // ... existing paths
        crate::mymodule::services::my_handler,
    ),
    components(schemas(
        // ... existing schemas
        MyResponse,
    )),
    tags(
        // ... existing tags
        (name = "MyModule", description = "My module endpoints"),
    ),
    // ...
)]
pub struct ApiDoc;
```

## Common Annotations

### Request Body

```rust
#[utoipa::path(
    post,
    path = "/users",
    tag = "Users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created", body = User),
    )
)]
pub async fn create_user(Json(body): Json<CreateUserRequest>) -> impl IntoResponse {
    // ...
}
```

### Path Parameters

```rust
#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "Users",
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
    )
)]
pub async fn get_user(Path(id): Path<i32>) -> impl IntoResponse {
    // ...
}
```

### Query Parameters

```rust
#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    params(
        ("page" = Option<i32>, Query, description = "Page number"),
        ("limit" = Option<i32>, Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
    )
)]
pub async fn list_users(Query(params): Query<PaginationParams>) -> impl IntoResponse {
    // ...
}
```

### Security (API Key)

```rust
#[utoipa::path(
    get,
    path = "/protected",
    tag = "Protected",
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, description = "Success"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn protected_endpoint() -> impl IntoResponse {
    // ...
}
```

Then add the security scheme to `openapi.rs`:

```rust
#[openapi(
    // ...
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
        );
    }
}
```

## Dependencies

The following crates are used for OpenAPI support:

```toml
# Cargo.toml
[dependencies]
utoipa = { version = "5.4.0", features = ["axum_extras", "chrono"] }
utoipa-swagger-ui = { version = "6.0.0", features = ["axum"] }
```

## Public Routes

Swagger UI and OpenAPI spec are configured as public routes and do not require authentication:

- `/local/swagger-ui/*` - Swagger UI interface
- `/api-doc/openapi.json` - OpenAPI JSON specification

## Customization

### Change Swagger UI Path

Modify in `src/routes.rs`:

```rust
let swagger_ui = SwaggerUi::new("/docs")  // Change path here
    .url("/api-doc/openapi.json", ApiDoc::openapi());
```

Remember to update the public routes in middleware if you change the path.

### Add Custom Servers

In `src/openapi.rs`:

```rust
servers(
    (url = "http://localhost:3000", description = "Local development"),
    (url = "https://staging.example.com", description = "Staging"),
    (url = "https://api.example.com", description = "Production"),
)
```
