use super::dom::document;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

const SLOTS: usize = 24;
const FALLBACK: (&str, &str) = ("#ff8a65", "#ffb74d");

#[derive(Clone, Copy, Default)]
struct Bucket {
    w: f64,
    r: f64,
    g: f64,
    b: f64,
}

/// Picks two distinct, bright colours from an image for a gradient (e.g. the equalizer).
pub async fn cover_gradient(url: &str) -> Option<(String, String)> {
    let img = HtmlImageElement::new().ok()?;
    img.set_src(url);
    JsFuture::from(img.decode()).await.ok()?;

    let size = 48u32;
    let canvas: HtmlCanvasElement = document().create_element("canvas").ok()?.dyn_into().ok()?;
    canvas.set_width(size);
    canvas.set_height(size);
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d").ok()??.dyn_into().ok()?;
    ctx.draw_image_with_html_image_element_and_dw_and_dh(&img, 0.0, 0.0, size as f64, size as f64).ok()?;
    let data = ctx.get_image_data(0.0, 0.0, size as f64, size as f64).ok()?.data();
    Some(pick_colors(&data))
}

/// Hue in degrees (0..360) of an RGB colour whose channel spread is `d` > 0 and maximum is `max`.
fn hue(r: f64, g: f64, b: f64, max: f64, d: f64) -> f64 {
    let h = if max == r {
        ((g - b) / d) % 6.0
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    h * 60.0
}

fn pick_colors(data: &[u8]) -> (String, String) {
    // Bucket pixels by hue, skipping near-black, near-white and grey.
    let mut buckets = [Bucket::default(); SLOTS];
    for px in data.chunks_exact(4) {
        let (r, g, b) = (px[0] as f64, px[1] as f64, px[2] as f64);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let v = max / 255.0;
        let sat = if max == 0.0 { 0.0 } else { (max - min) / max };
        if v < 0.25 || sat < 0.08 {
            continue;
        }
        let d = max - min;
        let h = if d > 0.0 { (hue(r, g, b, max, d) + 360.0) % 360.0 } else { 0.0 };
        let k = ((h / 360.0 * SLOTS as f64).floor() as usize) % SLOTS;
        let w = 0.4 + sat; // favour saturated pixels, but keep pastel covers working
        let bucket = &mut buckets[k];
        bucket.w += w;
        bucket.r += r * w;
        bucket.g += g * w;
        bucket.b += b * w;
    }

    let mut ranked: Vec<(usize, Bucket)> = buckets.into_iter().enumerate().filter(|(_, b)| b.w > 0.0).collect();
    ranked.sort_by(|a, b| b.1.w.total_cmp(&a.1.w));
    let Some(&first) = ranked.first() else {
        return (FALLBACK.0.into(), FALLBACK.1.into());
    };
    let second = ranked
        .iter()
        .find(|(k, b)| {
            let diff = k.abs_diff(first.0);
            diff.min(SLOTS - diff) >= 2 && b.w > first.1.w * 0.08
        })
        .copied()
        .unwrap_or(first);

    let color = |b: Bucket| lift(b.r / b.w, b.g / b.w, b.b / b.w);
    (color(first.1), color(second.1))
}

/// Boost saturation and lightness so the colour reads well on the black island.
fn lift(r: f64, g: f64, b: f64) -> String {
    let (r, g, b) = (r / 255.0, g / 255.0, b / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let mut h = if d > 0.0 { hue(r, g, b, max, d) } else { 0.0 };
    if h < 0.0 {
        h += 360.0;
    }
    let l = (max + min) / 2.0;
    let s = if d == 0.0 { 0.0 } else { d / (1.0 - (2.0 * l - 1.0).abs()) };
    let sat = ((s * 100.0).max(55.0) * 1.1).min(100.0);
    let light = (l * 100.0).max(58.0).min(72.0);
    format!("hsl({} {}% {}%)", h.round(), sat.round(), light.round())
}
