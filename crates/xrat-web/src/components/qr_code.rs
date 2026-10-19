use base64::{Engine, engine::general_purpose::STANDARD};
use dioxus::prelude::*;
use qrcode::{Color, EcLevel, QrCode};
use std::fmt::Write;

pub fn svg(payload: &str) -> Result<String, &'static str> {
    if payload.is_empty() {
        return Err("No configurations to encode.");
    }
    let qr = QrCode::with_error_correction_level(payload, EcLevel::H)
        .map_err(|_| "This content is too large for one QR code.")?;
    let width = qr.width();
    let size = width + 8;
    let mut path = String::new();
    for (index, color) in qr.to_colors().into_iter().enumerate() {
        if color == Color::Dark {
            let _ = write!(path, "M{} {}h1v1h-1z", index % width + 4, index / width + 4);
        }
    }
    let logo_size = width as f64 * 0.16;
    let logo_at = (size as f64 - logo_size) / 2.0;
    let logo = STANDARD.encode(include_bytes!("../../assets/qr-logo.png"));
    Ok(format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {size} {size}" width="1024" height="1024" shape-rendering="crispEdges"><rect width="100%" height="100%" fill="white"/><path d="{path}" fill="black"/><image x="{logo_at}" y="{logo_at}" width="{logo_size}" height="{logo_size}" href="data:image/png;base64,{logo}"/></svg>"#
    ))
}

#[component]
pub fn QrCodeImage(
    payload: String,
    #[props(default = "QR code containing raw configuration links".to_string())] label: String,
    #[props(default = "xrat-configs.svg".to_string())] filename: String,
) -> Element {
    let rendered = use_memo(move || svg(&payload));
    match rendered() {
        Ok(svg) => {
            let encoded = svg
                .bytes()
                .map(|byte| format!("%{byte:02X}"))
                .collect::<String>();
            let uri = format!("data:image/svg+xml,{encoded}");
            rsx! {
                div { class: "grid justify-items-center gap-3",
                    img { class: "w-full max-w-[340px] aspect-square rounded-md bg-white", src: uri.clone(), alt: label }
                    a { class: "text-xs text-primary underline underline-offset-4", href: uri, download: filename, "Download QR" }
                }
            }
        }
        Err(message) => {
            rsx! { p { role: "status", class: "text-sm text-muted-foreground py-6", "{message}" } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn qr_rejects_overflow_and_preserves_quiet_zone() {
        assert!(svg(&"x".repeat(10_000)).is_err());
        assert!(svg("").is_err());
        let output = svg("vless://example\nss://other").unwrap();
        assert!(output.contains("M4 "));
        assert!(output.contains("fill=\"white\""));
        assert!(!output.contains("vless"));
        assert!(output.contains("href=\"data:image/png;base64,"));
    }
}
