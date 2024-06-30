//! An example of drawing text. Writes to the user-provided target file.

use ab_glyph::{FontRef, PxScale};
use image::{DynamicImage, Rgb, RgbImage, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_line_segment_mut, draw_text_mut, text_size, Canvas};
use image::imageops::{overlay};
use std::env;
use std::path::Path;


fn generate_image(
    percentage: f32,
    min: f32,
    max: f32,
) {
    let arg = if env::args().count() == 2 {
        env::args().nth(1).unwrap()
    } else {
        panic!("Please enter a target file path")
    };

    let path = Path::new(&arg);
    // load image tp.png
    let mut image = image::open("tp.png").unwrap().to_rgba8();
    //let mut image = RgbImage::new(200, 200);

    let font = FontRef::try_from_slice(include_bytes!("../font.ttf")).unwrap();
    let color = Rgba([222u8, 162u8, 5u8, 255u8]);
    let height = 120.;
    let scale = PxScale {
        x: height * 1.2,
        y: height * 1.2,
    };
    let scale_big = PxScale {
        x: height * 2.,
        y: height * 2.,
    };
    let percentage = 0.85;
    let marker = "^";
    let text = format!("{:.1}", 8. * percentage + 1.0);
    let bar_pos = get_bar_position(280, 1070, percentage) as u32;
    // make new full transparent image 200x200
    let mut text_overlay = RgbaImage::new(140, 200);
    draw_text_mut(&mut text_overlay, Rgba([222u8, 162u8, 5u8, 255u8]), 70, 0, scale_big, &font, marker);
    draw_text_mut(&mut text_overlay, Rgba([222u8, 162u8, 5u8, 255u8]), 0, 50, scale, &font, &text);
    let (w, h) = text_size(scale, &font, &text);

    // draw text_overlay to image
    let mut image = DynamicImage::ImageRgba8(image);
    let text_overlay = DynamicImage::ImageRgba8(text_overlay);
    let start_y = find_first_dyed_y_position(&image, (bar_pos as f32 + text_overlay.width() as f32 * 0.8) as u32, color).unwrap() + 10;
    overlay(&mut image, &text_overlay, (bar_pos).into(), start_y.into());

    println!("Text size: {}x{}", w, h);

    image.save(path).unwrap();
}

fn get_bar_position(bar_start: u32, bar_end: u32, percentage: f32) -> i32 {
    let bar_length = bar_end - bar_start;
    let position = bar_start + (bar_length as f32 * percentage) as u32;
    position as i32
}

fn find_first_dyed_y_position(image: &DynamicImage, x: u32, dye: Rgba<u8>) -> Option<u32> {
    let (_, height) = image.dimensions();

    // Iterate from bottom to top in the specified column (x)
    for y in (0..height).rev() {
        // Get RGBA color of the pixel at (x, y)
        let pixel = image.get_pixel(x, y);
        // Check if it's yellow (255, 255, 0)
        if pixel == dye {
            return Some(y);
        }
    }

    None // No dyed pixel found in the specified column (x)
}