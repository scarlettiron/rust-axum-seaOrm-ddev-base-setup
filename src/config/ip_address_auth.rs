///hardcoded value for enabling IP address authentication middleware
///set to true to enable, false to disable
const ENABLED: bool = true;

///checks if IP address authentication middleware is enabled
///returns the hardcoded ENABLED value
pub fn is_enabled() -> bool {
    ENABLED
}
