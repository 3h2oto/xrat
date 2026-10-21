#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Placement {
    Select,
    Menu { outside: bool },
    Tooltip(Side),
}

#[derive(Clone, Copy)]
struct Rect {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
    width: f64,
    height: f64,
}

impl From<web_sys::DomRect> for Rect {
    fn from(rect: web_sys::DomRect) -> Self {
        Self {
            left: rect.left(),
            right: rect.right(),
            top: rect.top(),
            bottom: rect.bottom(),
            width: rect.width(),
            height: rect.height(),
        }
    }
}

pub(super) fn position(
    content: &web_sys::HtmlElement,
    trigger: &web_sys::HtmlElement,
    placement: Placement,
) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let viewport = (
        window
            .inner_width()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        window
            .inner_height()
            .ok()
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
    );
    let rect = trigger.get_bounding_client_rect().into();
    let style = match placement {
        Placement::Tooltip(side) => {
            let size = content.get_bounding_client_rect();
            tooltip_style(rect, viewport, (size.width(), size.height()), side)
        }
        _ => {
            let width = preferred_width(rect, viewport.0, placement);
            let _ = content.style().set_property("width", &format!("{width}px"));
            popup_style(
                rect,
                viewport,
                f64::from(content.scroll_height()).min(300.0),
                placement,
            )
        }
    };
    content.style().set_css_text(&style);
}

fn preferred_width(rect: Rect, viewport_width: f64, placement: Placement) -> f64 {
    let preferred = if placement == Placement::Select {
        rect.width
    } else {
        rect.width.max(180.0)
    };
    preferred.min((viewport_width - 16.0).max(0.0))
}

fn popup_style(rect: Rect, viewport: (f64, f64), desired: f64, placement: Placement) -> String {
    let (viewport_width, viewport_height) = viewport;
    let width = preferred_width(rect, viewport_width, placement);
    let below = viewport_height - rect.bottom;
    let above = below < desired && rect.top > below;
    let left = rect.left.min(viewport_width - width - 8.0).max(8.0);
    let height = (if above { rect.top } else { below } - 12.0).clamp(60.0, 300.0);
    let vertical = if above {
        format!("top:auto;bottom:{}px", viewport_height - rect.top + 4.0)
    } else {
        format!("top:{}px;bottom:auto", rect.bottom + 4.0)
    };
    let (left, vertical, height) = if placement == (Placement::Menu { outside: true }) {
        let left = if rect.right + 4.0 + width <= viewport_width - 8.0 {
            rect.right + 4.0
        } else {
            rect.left - width - 4.0
        };
        (
            left.max(8.0),
            format!(
                "top:{}px;bottom:auto",
                rect.top.min(viewport_height - desired - 8.0).max(8.0)
            ),
            (viewport_height - 16.0).max(0.0),
        )
    } else {
        (left, vertical, height)
    };
    format!(
        "pointer-events:auto;position:fixed;width:{width}px;min-width:0;margin:0;left:{left}px;right:auto;{vertical};max-height:{height}px;overflow-y:auto;overflow-x:hidden"
    )
}

fn tooltip_style(rect: Rect, viewport: (f64, f64), size: (f64, f64), side: Side) -> String {
    let (width, height) = size;
    let left = match side {
        Side::Left => rect.left - width - 10.0,
        Side::Right => rect.right + 10.0,
        _ => rect.left + (rect.width - width) / 2.0,
    };
    let top = match side {
        Side::Top => rect.top - height - 8.0,
        Side::Bottom => rect.bottom + 8.0,
        _ => rect.top + (rect.height - height) / 2.0,
    };
    let left = left.min(viewport.0 - width - 8.0).max(8.0);
    let top = top.min(viewport.1 - height - 8.0).max(8.0);
    format!(
        "opacity:1;visibility:visible;position:fixed;left:{left}px;top:{top}px;bottom:auto;right:auto;margin:0;pointer-events:none"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trigger() -> Rect {
        Rect {
            left: 250.0,
            right: 350.0,
            top: 300.0,
            bottom: 330.0,
            width: 100.0,
            height: 30.0,
        }
    }

    #[test]
    fn popup_flips_above_and_clamps_to_viewport() {
        let style = popup_style(
            trigger(),
            (390.0, 400.0),
            200.0,
            Placement::Menu { outside: false },
        );
        assert!(style.contains("left:202px"));
        assert!(style.contains("top:auto;bottom:104px"));
        assert!(style.contains("max-height:288px"));
        let style = popup_style(
            trigger(),
            (390.0, 400.0),
            200.0,
            Placement::Menu { outside: true },
        );
        assert!(style.contains("left:66px"));
        assert!(style.contains("top:192px;bottom:auto"));
    }

    #[test]
    fn tooltips_stay_inside_viewport() {
        let style = tooltip_style(trigger(), (390.0, 400.0), (180.0, 60.0), Side::Right);
        assert!(style.contains("left:202px;top:285px"));
    }
}
