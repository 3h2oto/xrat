mod check;
mod errors;

pub use check::{DownloadResult, download_speed_check, make_proxied_download_with_client};

#[cfg(test)]
mod tests;
