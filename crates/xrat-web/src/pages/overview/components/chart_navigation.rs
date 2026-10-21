use dioxus::prelude::*;
use wasm_bindgen::JsCast;

pub(super) fn navigate_samples(event: KeyboardEvent) {
    let Some(native) = event.data().downcast::<web_sys::KeyboardEvent>().cloned() else {
        return;
    };
    let Some(sample) = native
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
        .and_then(|target| target.closest(".chart-sample").ok().flatten())
    else {
        return;
    };
    let Some(samples) = sample
        .closest("svg")
        .ok()
        .flatten()
        .and_then(|svg| svg.query_selector_all("g.chart-sample").ok())
    else {
        return;
    };
    let Some(index) = (0..samples.length()).find(|index| {
        samples
            .item(*index)
            .is_some_and(|node| node.is_same_node(Some(&sample)))
    }) else {
        return;
    };
    let Some(next) = next_sample(&event.key(), index, samples.length()) else {
        return;
    };
    event.prevent_default();
    if let Some(next) = samples
        .item(next)
        .and_then(|node| node.dyn_into::<web_sys::SvgElement>().ok())
    {
        let _ = next.focus();
    }
}

fn next_sample(key: &Key, index: u32, count: u32) -> Option<u32> {
    let last = count.checked_sub(1)?;
    match key {
        Key::Home => Some(0),
        Key::End => Some(last),
        Key::ArrowLeft => Some(index.saturating_sub(1).min(last)),
        Key::ArrowRight => Some(index.saturating_add(1).min(last)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_clamps_without_wrapping() {
        assert_eq!(next_sample(&Key::ArrowLeft, 0, 3), Some(0));
        assert_eq!(next_sample(&Key::ArrowRight, 2, 3), Some(2));
        assert_eq!(next_sample(&Key::ArrowRight, 0, 3), Some(1));
        assert_eq!(next_sample(&Key::Home, 2, 3), Some(0));
        assert_eq!(next_sample(&Key::End, 0, 3), Some(2));
        assert_eq!(next_sample(&Key::End, 0, 0), None);
        assert_eq!(next_sample(&Key::Tab, 0, 3), None);
    }
}
