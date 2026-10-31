//! Draws the tray icon: the current temperature as big Western digits on a
//! colour-coded rounded square, or a white umbrella in their place when rain is
//! likely. Everything is rendered by hand so no image or font crate is needed.

use tray_icon::Icon;

const SIZE: usize = 64;

/// 5x7 glyphs, one byte per row, bit 4 is the leftmost column.
fn glyph(c: char) -> [u8; 7] {
    match c {
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        _ => [0; 7],
    }
}

fn background(celsius_equiv: f64) -> [u8; 3] {
    match celsius_equiv {
        t if t <= 0.0 => [52, 101, 190],
        t if t < 15.0 => [20, 140, 160],
        t if t < 25.0 => [46, 150, 80],
        t if t < 32.0 => [222, 130, 20],
        _ => [200, 55, 45],
    }
}

fn inside_rounded_square(x: usize, y: usize) -> bool {
    const R: i32 = 12;
    let (x, y) = (x as i32, y as i32);
    let max = SIZE as i32 - 1;
    let cx = x.clamp(R, max - R);
    let cy = y.clamp(R, max - R);
    let (dx, dy) = (x - cx, y - cy);
    dx * dx + dy * dy <= R * R
}

/// A white umbrella that fills the square: the tray shows 16 px, so only one
/// bold silhouette reads — no digits share the icon while it rains.
fn draw_umbrella(px: &mut [u8]) {
    let (cx, cy, rx, ry) = (32.0, 32.0, 26.0, 12.0);
    for y in 0..SIZE {
        for x in 0..SIZE {
            if !inside_rounded_square(x, y) {
                continue;
            }
            let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
            let dome = fy <= cy && ((fx - cx) / rx).powi(2) + ((fy - cy) / ry).powi(2) <= 1.0;
            // three hanging panels, so the canopy edge does not read as a plain dome
            let panel = fy >= cy
                && [19.0, cx, 45.0]
                    .iter()
                    .any(|p| (fx - p).powi(2) + (fy - cy).powi(2) <= 49.0);
            let tip = (31..=32).contains(&x) && (15..=20).contains(&y);
            let stem = (31..=33).contains(&x) && (32..=51).contains(&y);
            let (hx, hy) = (fx - 24.0, fy - 44.0);
            let hook = fx >= 24.0 && fy >= 44.0 && {
                let d = hx * hx + hy * hy;
                d >= 25.0 && d <= 81.0
            };
            if dome || panel || tip || stem || hook {
                let i = (y * SIZE + x) * 4;
                px[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }
}

pub fn render(temp: f64, celsius: bool, rain_likely: bool) -> (Vec<u8>, u32, u32) {
    let bg = background(if celsius { temp } else { (temp - 32.0) / 1.8 });
    let mut px = vec![0u8; SIZE * SIZE * 4];

    for y in 0..SIZE {
        for x in 0..SIZE {
            if inside_rounded_square(x, y) {
                let i = (y * SIZE + x) * 4;
                px[i..i + 4].copy_from_slice(&[bg[0], bg[1], bg[2], 255]);
            }
        }
    }

    if rain_likely {
        draw_umbrella(&mut px);
        return (px, SIZE as u32, SIZE as u32);
    }

    let text = format!("{:.0}", temp);
    let text = if text == "-0" { "0".to_string() } else { text };
    let n = text.chars().count();
    let cols = n * 5 + (n - 1);
    let scale = ((SIZE - 6) / cols).min((SIZE - 6) / 7).clamp(1, 8);
    let (w, h) = (cols * scale, 7 * scale);
    let (x0, y0) = ((SIZE - w) / 2, (SIZE - h) / 2);

    for (ci, ch) in text.chars().enumerate() {
        for (row, bits) in glyph(ch).iter().enumerate() {
            for col in 0..5 {
                if bits & (0b10000 >> col) == 0 {
                    continue;
                }
                let gx = x0 + (ci * 6 + col) * scale;
                let gy = y0 + row * scale;
                for dy in 0..scale {
                    for dx in 0..scale {
                        let i = ((gy + dy) * SIZE + gx + dx) * 4;
                        px[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
                    }
                }
            }
        }
    }
    (px, SIZE as u32, SIZE as u32)
}

pub fn temperature_icon(temp: f64, celsius: bool, rain_likely: bool) -> Option<Icon> {
    let (px, w, h) = render(temp, celsius, rain_likely);
    Icon::from_rgba(px, w, h).ok()
}

/// The icon bundled into the EXE by build.rs (resource id 1).
pub fn app_icon() -> Option<Icon> {
    Icon::from_resource(1, Some((32, 32))).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_expected_buffer_size_for_all_widths() {
        for t in [-15.0, -0.4, 0.0, 7.0, 72.4, 104.0, 123.0] {
            let (px, w, h) = render(t, false, t > 50.0);
            assert_eq!(px.len(), (w * h * 4) as usize);
            assert!(px.chunks(4).any(|p| p == [255, 255, 255, 255]), "no digits drawn for {t}");
        }
    }

    #[test]
    fn rain_replaces_the_digits_with_an_umbrella() {
        let (px, ..) = render(20.0, true, true);
        let white = |x: usize, y: usize| px[(y * SIZE + x) * 4] == 255;
        assert!(white(32, 25), "no canopy");
        assert!(white(31, 45), "no stem");
        assert!(white(24, 52), "no hook");
        assert!(!white(10, 20), "canopy leaks outside the dome");
        assert!(!px.chunks(4).any(|p| p == [120, 190, 255, 255]), "old rain strip still drawn");
    }
}
