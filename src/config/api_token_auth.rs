use super::env;

///checks if API token authentication middleware is enabled
pub fn is_enabled() -> bool {
    env::get().middleware.api_token_auth_enabled
}
