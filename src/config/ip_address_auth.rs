use super::env;

///checks if IP address authentication middleware is enabled
pub fn is_enabled() -> bool {
    env::get().middleware.ip_address_auth_enabled
}
