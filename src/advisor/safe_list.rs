const BLACKLIST: &[&str] = &[
    // macOS
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
    // Linux
    "/proc",
    "/sys",
    "/dev",
    "/boot",
    "/etc",
    "/usr/share",
    "/lib",
    "/lib64",
    "/snap/bin",
    "/run",
];

pub fn is_blacklisted(path: &str) -> bool {
    BLACKLIST.iter().any(|banned| path.contains(banned))
}

pub fn is_safe_to_suggest(path: &str) -> bool {
    !is_blacklisted(path)
}
