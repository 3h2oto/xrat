pub(super) const ROUTES: &[(&str, &str)] = &[
    ("Overview", "/"),
    ("Configs", "/ui/configs"),
    ("Subscriptions", "/ui/subscriptions"),
    ("Jobs", "/ui/jobs"),
    ("XRAT logs", "/ui/logs/xrat"),
    ("Engine logs", "/ui/logs/engine"),
    ("Tools", "/ui/tools"),
    ("TCP scanner", "/ui/tools/tcp-scanner"),
    ("Parse configs", "/ui/tools/parse"),
    ("GeoIP lookup", "/ui/tools/geoip"),
    ("Host administration", "/ui/host"),
    ("Settings", "/ui/settings"),
    ("Users", "/ui/users"),
    ("Account", "/ui/account"),
];
pub(super) fn page_for_path(path: &str) -> &'static str {
    let normalized = path.trim_end_matches('/');
    ROUTES
        .iter()
        .find(|(_, route)| route.trim_end_matches('/') == normalized)
        .map(|(page, _)| *page)
        .unwrap_or("Page not found")
}
pub(super) fn path_for_page(page: &str) -> Option<&'static str> {
    ROUTES
        .iter()
        .find(|(name, _)| *name == page)
        .map(|(_, path)| *path)
}
