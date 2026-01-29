use super::env;

///gets allowed hosts from central config
pub fn get_allowed_hosts() -> Vec<String> {
    env::get().hosts.allowed.clone()
}

///checks if a host is allowed
pub fn is_host_allowed(host: &str) -> bool {
    let allowed = get_allowed_hosts();

    //strip port if present
    let host_without_port = host.split(':').next().unwrap_or(host);

    allowed.iter().any(|allowed_host| {
        //exact match
        host_without_port == allowed_host
        //wildcard subdomain match (e.g., ".ddev.site" matches "anything.ddev.site")
        || (allowed_host.starts_with('.') && host_without_port.ends_with(allowed_host))
        //match subdomains
        || host_without_port.ends_with(&format!(".{}", allowed_host))
    })
}
