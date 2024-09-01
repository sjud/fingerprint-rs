use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Window};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CanvasFingerPrint {
    pub winding: bool,
    pub geometry_hash: u64,
    pub text_hash: u64,
}
impl CanvasFingerPrint {
    pub fn new(window: &Window) -> Option<Self> {
        let canvas = window
            .document()?
            .create_element("canvas")
            .ok()?
            .dyn_into::<HtmlCanvasElement>()
            .ok()?;
        let ctx = canvas
            .get_context("2d")
            .ok()??
            .dyn_into::<CanvasRenderingContext2d>()
            .ok()?;
        let winding = supports_winding(&ctx);
        render_text_image(&canvas, &ctx);
        // ignore browsers that add noise to image data.
        let text = canvas.to_data_url().ok()?;
        let text_2 = canvas.to_data_url().ok()?;
        if text != text_2 {
            return None;
        }
        render_geometry_image(&canvas, &ctx);
        let geometry = canvas.to_data_url().ok()?;
        let mut hasher = DefaultHasher::new();
        hasher.write(geometry.as_bytes());
        let geometry_hash = hasher.finish();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hasher.write(text.as_bytes());
        let text_hash = hasher.finish();
        Some(Self {
            winding,
            geometry_hash,
            text_hash,
        })
    }
}
fn render_text_image(canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) {
    // Resizing the canvas cleans it
    canvas.set_width(240);
    canvas.set_height(60);

    ctx.set_text_baseline("alphabetic");
    ctx.set_fill_style(&JsValue::from_str("#60"));
    ctx.fill_rect(100., 1., 62., 20.);
    ctx.set_fill_style(&JsValue::from_str("#069"));
    // It's important to use explicit built-in fonts in order to exclude the affect of font preferences
    // (there is a separate entropy source for them).
    ctx.set_font(r#"11pt "Times New Roman""#);
    let printed_text = "Cwm fjordbank gly \u{1F60D}";
    ctx.fill_text(printed_text, 2., 15.).unwrap();
    ctx.set_fill_style(&JsValue::from_str("rgba(102, 204, 0, 0.2)"));
    ctx.set_font("18pt Arial");
    ctx.fill_text(&printed_text, 4., 45.).unwrap();
}

fn supports_winding(ctx: &CanvasRenderingContext2d) -> bool {
    ctx.rect(0., 0., 10., 10.);
    ctx.rect(2., 2., 6., 6.);
    ctx.is_point_in_path_with_f64_and_canvas_winding_rule(
        5.,
        5.,
        web_sys::CanvasWindingRule::Evenodd,
    )
}

fn render_geometry_image(canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) {
    // clear canvas by resizing
    canvas.set_width(122);
    canvas.set_height(110);

    // Set global composite operation to 'multiply'
    ctx.set_global_composite_operation("multiply").unwrap();

    // Draw the circles with different colors
    let colors_and_positions = [
        ("#f2f", 40.0, 40.0),
        ("#2ff", 80.0, 40.0),
        ("#ff2", 60.0, 80.0),
    ];

    // draw three different color circles at different positions so their parts of each overlap at the center of the image.
    for (color, x, y) in colors_and_positions.iter() {
        ctx.set_fill_style(&JsValue::from_str(color));
        ctx.begin_path();
        ctx.arc(*x, *y, 40.0, 0.0, std::f64::consts::PI * 2.0)
            .unwrap();
        ctx.close_path();
        ctx.fill();
    }

    // Draw the winding rule example
    ctx.set_fill_style(&JsValue::from_str("#f9c"));
    ctx.begin_path();
    ctx.arc(60.0, 60.0, 60.0, 0.0, std::f64::consts::PI * 2.0)
        .unwrap();
    ctx.arc(60.0, 60.0, 20.0, 0.0, std::f64::consts::PI * 2.0)
        .unwrap();
    ctx.fill_with_canvas_winding_rule(web_sys::CanvasWindingRule::Evenodd);
}
