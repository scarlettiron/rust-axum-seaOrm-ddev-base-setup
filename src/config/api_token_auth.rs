///hardcoded value for enabling API token authentication middleware
///set to true to enable, false to disable
const ENABLED: bool = true;

///checks if API token authentication middleware is enabled
///returns the hardcoded ENABLED value
pub fn is_enabled() -> bool {
    ENABLED
}
