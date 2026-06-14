const BLACKLIST: &[&str] = &[
    "/System",
    "/bin",
    "/sbin",
    "/usr/bin",
    "/usr/lib",
    "/usr/sbin",
    "/private/var/folders",
    ".git/objects",
    ".git/refs",
    ".git/logs",
];

pub fn is_blacklisted(path: &str) -> bool {
    BLACKLIST.iter().any(|banned| path.contains(banned))
}

pub fn is_safe_to_suggest(path: &str) -> bool {
    !is_blacklisted(path)
}
