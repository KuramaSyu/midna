use ab_glyph::{FontRef, PxScale};
use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, Canvas};
use image::imageops::overlay;
use lazy_static::lazy_static;
use std::sync::Mutex;



lazy_static! {
    static ref TP_IMAGE: Mutex<RgbaImage> = Mutex::new(
        image::open("assets/tp.png")
            .unwrap()
            .to_rgba8()
    );
}


/// Generates an image with a marker and text overlay based on the given percentage, minimum, and maximum values.
/// 
/// # Arguments
/// 
/// * `percentage` - A float value representing the percentage to determine the position of the marker.
/// * `min` - A float value representing the minimum value for the text overlay.
/// * `max` - A float value representing the maximum value for the text overlay.
/// 
/// # Returns
/// 
/// * `DynamicImage` - The generated image with the marker and text overlay.
/// 
/// # Panics
/// 
/// This function will panic if the `tp.png` image or the font file cannot be loaded.
/// 
/// # Example
/// 
/// ```
/// let image = generate_tp_image(0.5, 0.0, 100.0);
/// ```
pub fn generate_tp_image(
    percentage: f32,
    min: f32,
    max: f32,
) -> DynamicImage {

    // time for debug
    let start = std::time::Instant::now();

    // load image tp.png
    let image = TP_IMAGE.lock().unwrap().clone();

    // load font, set color 
    let font = FontRef::try_from_slice(include_bytes!("../../../assets/font.ttf")).unwrap();
    let color = Rgba([222u8, 162u8, 5u8, 255u8]);

    // font settings (size, x-y scale ..)
    let height = 78.;
    let scale = PxScale {
        x: height * 1.2,
        y: height * 1.2,
    };
    let scale_big = PxScale {
        x: height * 2.,
        y: height * 2.,
    };
    // marker which is drawn below the bar + text below the marker
    let marker = "^";
    let text = format!("{:.1}", (max-min) * percentage + min);

    // calculate bar position by percentage
    let bar_pos = get_bar_position(140, 535, percentage) as u32;
    // offset since every font looks different
    let y_font_offset: i32 = -10;

    // new image where marker and text will be drawn
    let mut text_overlay = RgbaImage::new(70, 100);
    draw_text_mut(&mut text_overlay, color, 10, 0, scale_big, &font, marker);
    draw_text_mut(&mut text_overlay, color, 0, 28, scale, &font, &text);

    // draw text_overlay to image
    let mut image = DynamicImage::ImageRgba8(image);
    let text_overlay = DynamicImage::ImageRgba8(text_overlay);
    // determine y position by moving from bottom to top
    let start_y = find_first_dyed_y_position(
        &image, 
        (bar_pos as f32 + (text_overlay.width() as f32 * 0.5)) as u32, 
        color
    ).unwrap() as i32 + y_font_offset;

    print!("start y: {} ", start_y);
    // put text image onto TP image
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