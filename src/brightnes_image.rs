use ab_glyph::{FontRef, PxScale};
use image::{imageops::FilterType, DynamicImage, Rgb, RgbImage, Rgba, RgbaImage};
use imageproc::{drawing::{draw_filled_rect_mut, draw_line_segment_mut, draw_text_mut, text_size, Canvas}, rect::Rect};
use image::imageops::{overlay};
use lazy_static::lazy_static;
use std::sync::Mutex;
use crate::colors::ImageType;



lazy_static! {
    static ref TP_IMAGE: Mutex<RgbaImage> = Mutex::new(
        image::open("tp.png")
            .unwrap()
            .to_rgba8()
    );
}

//let mut image_buffer: Option<RgbaImage> = None;
pub fn generate_image(
    percentage: f32,
    min: f32,
    max: f32,
) -> DynamicImage {
    // include tp.image


    let start = std::time::Instant::now();
    // load image tp.png
    let image = TP_IMAGE.lock().unwrap().clone();
    println!("Load image {:?}", start.elapsed());

    //let mut image = RgbImage::new(200, 200);
    let font = FontRef::try_from_slice(include_bytes!("../font.ttf")).unwrap();
    let color = Rgba([222u8, 162u8, 5u8, 255u8]);
    let height = 78.;
    let scale = PxScale {
        x: height * 1.2,
        y: height * 1.2,
    };
    let scale_big = PxScale {
        x: height * 2.,
        y: height * 2.,
    };
    let marker = "^";
    let text = format!("{:.1}", (max-min) * percentage + min);
    let bar_pos = get_bar_position(140, 535, percentage) as u32;
    let y_font_offset: i32 = -10;
    // make new full transparent image 200x200
    let mut text_overlay = RgbaImage::new(70, 100);
    draw_text_mut(&mut text_overlay, Rgba([222u8, 162u8, 5u8, 255u8]), 10, 0, scale_big, &font, marker);
    draw_text_mut(&mut text_overlay, Rgba([222u8, 162u8, 5u8, 255u8]), 0, 28, scale, &font, &text);
    //let (_w, h) = text_size(scale, &font, &text);

    // draw text_overlay to image
    let mut image = DynamicImage::ImageRgba8(image);
    let text_overlay = DynamicImage::ImageRgba8(text_overlay);
    let start_y = find_first_dyed_y_position(&image, (bar_pos as f32 + (text_overlay.width() as f32 * 0.5)) as u32, color).unwrap() as i32 + y_font_offset;
    // draw small debug rect
    // let rect = Rect::at(bar_pos as i32, start_y as i32).of_size(10, 10);
    // draw_filled_rect_mut(&mut image, rect, color);
    print!("start y: {} ", start_y);
    overlay(&mut image, &text_overlay, (bar_pos).into(), start_y.into());
    println!("Creating image {:?}", start.elapsed());
    image
}

fn get_bar_position(bar_start: u32, bar_end: u32, percentage: f32) -> i32 {
    let bar_length = bar_end - bar_start;
    let position = bar_start + (bar_length as f32 * percentage) as u32;
    position as i32
}

fn find_first_dyed_y_position(image: &DynamicImage, x: u32, _dye: Rgba<u8>) -> Option<u32> {
    let (_, height) = image.dimensions();
    // Iterate from bottom to top in the specified column (x)
    for y in (0..height).rev() {
        // Get RGBA color of the pixel at (x, y)
        let pixel = image.get_pixel(x, y);
        // Check if it's yellow (255, 255, 0)
        if pixel[3] > 200 {
            return Some(y);
        }
    }

    None // No dyed pixel found in the specified column (x)
}