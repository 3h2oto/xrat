use super::routes::{ROUTES, page_for_path, path_for_page};

#[test]
fn every_page_round_trips_through_its_url() {
    for (page, path) in ROUTES {
        assert_eq!(page_for_path(path), *page);
        assert_eq!(path_for_page(page), Some(*path));
    }
    assert_eq!(page_for_path("/ui/configs/"), "Configs");
    assert_eq!(page_for_path("/ui/logs/engine"), "Engine logs");
    assert_eq!(page_for_path("/not-a-page"), "Page not found");
    assert_eq!(path_for_page("Page not found"), None);
}
