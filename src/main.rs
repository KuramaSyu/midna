use image::{DynamicImage, GenericImageView, ImageResult, ImageBuffer, RgbaImage, Rgb, Rgba};
use std::{borrow::BorrowMut, io::Cursor};
use env_logger::{Builder, Env};
use log::{info, warn, debug, Level::Debug, set_max_level};
use image::io::Reader as ImageReader;

fn get_image() -> ImageResult<DynamicImage> {
    
    ImageReader::open("test.png")?.decode()
}

struct RgbColor {
    r: u8,
    g: u8,
    b: u8,
}

impl RgbColor {
    pub fn rn(&self) -> f32 {
        self.r as f32 / 255.0
    }

    pub fn gn(&self) -> f32 {
        self.g as f32 / 255.0
    }

    pub fn bn(&self) -> f32 {
        self.b as f32 / 255.0
    }

    pub fn brightness(&self) -> f32 {
        calculate_avg_pixel_brightness(self.r, self.g, self.b)
    }

    pub fn calculate_grayscale_similarity(&self) -> f32 {
        let r_f32 = self.rn();
        let g_f32 = self.gn();
        let b_f32 = self.bn();
    
        // Calculate the mean of the RGB values
        let mean = (r_f32 + g_f32 + b_f32) / 3.0;
    
        // Calculate the squared differences from the mean
        let r_diff = (r_f32 - mean).powi(2);
        let g_diff = (g_f32 - mean).powi(2);
        let b_diff = (b_f32 - mean).powi(2);
    
        // Calculate the variance (mean of squared differences)
        let variance = (r_diff + g_diff + b_diff) / 3.0;
    
        // The standard deviation is the square root of the variance
        let std_dev = variance.sqrt();
    
        std_dev  
    }
}

struct NormalizedRgb {
    r: f32,
    g: f32,
    b: f32,
}

impl RgbColor {
    fn normalize(&self) -> NormalizedRgb {
        let max_value = 255.0; // Maximum value of an 8-bit RGB component
        NormalizedRgb {
            r: self.r as f32 / max_value,
            g: self.g as f32 / max_value,
            b: self.b as f32 / max_value,
        }
    }

}


struct PolarNight {}
impl PolarNight {
    const a: RgbColor = RgbColor {r: 46, g: 52, b: 64};
    const b: RgbColor = RgbColor {r: 59, g: 66, b: 82};
    const c: RgbColor = RgbColor {r: 67, g: 76, b: 94};
    const d: RgbColor = RgbColor {r: 76, g: 86, b: 106};
}

struct SnowStorm {}
// impl for #d8dee9 #e5e9f0 #eceff4
impl SnowStorm {
    const a: RgbColor = RgbColor {r: 216, g: 222, b: 233};
    const b: RgbColor = RgbColor {r: 229, g: 233, b: 240};
    const c: RgbColor = RgbColor {r: 236, g: 239, b: 244};
    const d: RgbColor = RgbColor {r: 236, g: 239, b: 244};
}

struct Frost {}
// impl for #8fbcbb #88c0d0 #81a1c1 #5e81ac
impl Frost {
    const a: RgbColor = RgbColor {r: 143, g: 188, b: 187};
    const b: RgbColor = RgbColor {r: 136, g: 192, b: 208};
    const c: RgbColor = RgbColor {r: 129, g: 161, b: 193};
    const d: RgbColor = RgbColor {r: 94, g: 129, b: 172};
}

struct Nord {
    pub polar_night: PolarNight,
    pub snow_storm: SnowStorm,
    pub frost: Frost,
}

impl Nord {
    const polar_night: PolarNight = PolarNight {};
    const snow_storm: SnowStorm = SnowStorm {};
    const frost: Frost = Frost {};

    fn new() -> Self {
        Nord {
            polar_night: PolarNight {},
            snow_storm: SnowStorm {},
            frost: Frost {},
        }
    }
}

 
fn main() {
    env_logger::init();
    let mut image = get_image().unwrap();
    debug!("{:?}", image.dimensions());
    //image = image.grayscale();
    let brightness = calculate_average_brightness(image.as_rgba8().unwrap());
    info!("Brightness of image is: {:.3}", brightness);
    if  brightness > 0.65 {
        image.invert();
    }

    // Define RGB values in the range 0-255
    let tint_r = 46;
    let tint_g = 52;
    let tint_b = 64;

    // Convert to the range 0.0-1.0
    let tint = Rgb([
        tint_r as f32 / 255.0,
        tint_g as f32 / 255.0,
        tint_b as f32 / 255.0,
    ]);

    // Define the Nord color in RGB format (normalized)
    let nord_color = Rgb([
        67 as f32 / 255.0,
        76 as f32 / 255.0,
        94 as f32 / 255.0,
    ]);
    // image = image.adjust_contrast(-4.5);
    image = image.blur(0.2);
    // image = image.adjust_contrast(5.);
    
    apply_nord_filter(image.as_mut_rgba8().unwrap(), 1.);
    
    // Define a blend factor (between 0.0 and 1.0)
    //let blend_factor = 0.4;
    //apply_tone(image.as_mut_rgba8().unwrap(), tint, blend_factor);
    image.save("result.png");
}


pub fn tint_image(image: &mut RgbaImage, tint: Rgb<f32>) {
    let Rgb([tint_r, tint_g, tint_b]) = tint;
    for Rgba([r, g, b, _]) in image.pixels_mut() {
        *r = (*r as f32 * tint_r) as u8;
        *g = (*g as f32 * tint_g) as u8;
        *b = (*b as f32 * tint_b) as u8;
    }
}

pub fn apply_sepia(image: &mut RgbaImage) {
    for Rgba([r, g, b, _]) in image.pixels_mut() {
        let tr = (0.393 * *r as f32 + 0.769 * *g as f32 + 0.189 * *b as f32).min(255.0) as u8;
        let tg = (0.349 * *r as f32 + 0.686 * *g as f32 + 0.168 * *b as f32).min(255.0) as u8;
        let tb = (0.272 * *r as f32 + 0.534 * *g as f32 + 0.131 * *b as f32).min(255.0) as u8;
        *r = tr;
        *g = tg;
        *b = tb;
    }
}

pub fn apply_tone(image: &mut RgbaImage, target_color: Rgb<f32>, blend_factor: f32) {
    let Rgb([target_r, target_g, target_b]) = target_color;
    for Rgba([r, g, b, _]) in image.pixels_mut() {
        let orig_r = *r as f32 / 255.0;
        let orig_g = *g as f32 / 255.0;
        let orig_b = *b as f32 / 255.0;

        *r = ((orig_r * (1.0 - blend_factor) + target_r * blend_factor) * 255.0).min(255.0) as u8;
        *g = ((orig_g * (1.0 - blend_factor) + target_g * blend_factor) * 255.0).min(255.0) as u8;
        *b = ((orig_b * (1.0 - blend_factor) + target_b * blend_factor) * 255.0).min(255.0) as u8;
    }
}

fn calculate_average_brightness(image: &RgbaImage) -> f32 {
    let mut total_brightness = 0.0;
    let num_pixels = image.width() * image.height();

    for Rgba([r, g, b, _]) in image.pixels() {
        let brightness = calculate_avg_pixel_brightness(*r, *g, *b);
        total_brightness += brightness;
    }

    total_brightness / num_pixels as f32
}

pub fn calculate_avg_pixel_brightness(r: u8, g: u8, b: u8) -> f32 {
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0
}

pub fn apply_nord_filter(image: &mut RgbaImage, blend_factor: f32) {
    // Define Nord color components normalized to the range 0.0-1.0
    let mut smallest_grey = f32::MAX;
    let mut biggest_grey = f32::MIN;
    let contrast_colors = vec![ 
        PolarNight::c, PolarNight::b, PolarNight::c, PolarNight::d, 
        //SnowStorm::a, // SnowStorm::c, SnowStorm::d, SnowStorm::b, 
    ];
    let colorful_colors = vec![
        Frost::a, Frost::b, Frost::c, Frost::d
    ];
    for color in &contrast_colors {
        debug!("{} {} {} has brightness {:.3}",color.r, color.g, color.b, color.brightness())
    }
    fn get_nearest_color<'a>(color: &RgbColor, all_colors: &'a Vec<RgbColor>) -> &'a RgbColor {
        let mut min_distance = f32::MAX;
        let mut nearest_color = &all_colors[0];
        let br = color.brightness();
        for c in all_colors.iter() {
            if (c.brightness() - br).abs() < min_distance {
                min_distance = c.brightness();
                nearest_color = c;
            }
        }
        nearest_color
    }
    // Loop through each pixel
    for Rgba([r, g, b, _]) in image.pixels_mut() {
        // Convert original RGB values to floats in the range 0.0-1.0
        let orig_r = *r as f32 / 255.0;
        let orig_g = *g as f32 / 255.0;
        let orig_b = *b as f32 / 255.0;
        
        let color = RgbColor {r: *r, g: *g, b: *b};
        let br = color.brightness();
        if color.calculate_grayscale_similarity() < smallest_grey {
            smallest_grey = color.calculate_grayscale_similarity()
        }
        if color.calculate_grayscale_similarity() > biggest_grey {
            biggest_grey = color.calculate_grayscale_similarity()
        }
        let new = {
            if color.calculate_grayscale_similarity() < 0.15 {
                get_nearest_color(&color, &contrast_colors)
            } else {
                get_nearest_color(&color, &colorful_colors)
            }
            
        };
        let multiplier = {
            if color.calculate_grayscale_similarity() < 0.15 {
                if color.brightness() > 0.4 {
                    0.2
                } else {
                    0.85
                }
            } else {
                0.9
            }  
        };
        let strength =(1. - (br - new.brightness()).abs()) * multiplier ;

        let blended_r = (orig_r * (1.0 - strength) + new.rn() * strength) * 255.0;
        let blended_g = (orig_g * (1.0 - strength) + new.gn() * strength) * 255.0;
        let blended_b = (orig_b * (1.0 - strength) + new.bn() * strength) * 255.0;

        // Update pixel values with blended colors
        *r = blended_r.min(255.0) as u8;
        *g = blended_g.min(255.0) as u8;
        *b = blended_b.min(255.0) as u8;
    }
    debug!("greyscale: {:.3} - {:.3}", smallest_grey, biggest_grey);
}

