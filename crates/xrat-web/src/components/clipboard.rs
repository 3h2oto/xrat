pub async fn copy(text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("Clipboard unavailable")?;
    let navigator = window.navigator();
    if !window.is_secure_context()
        || js_sys::Reflect::get(
            navigator.as_ref(),
            &wasm_bindgen::JsValue::from_str("clipboard"),
        )
        .map_or(true, |value| value.is_undefined() || value.is_null())
    {
        return Err("Clipboard unavailable. Copy the text below.".into());
    }
    wasm_bindgen_futures::JsFuture::from(navigator.clipboard().write_text(text))
        .await
        .map_err(|_| "Clipboard unavailable. Copy the text below.".to_string())?;
    Ok(())
}
