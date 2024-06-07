use image::{DynamicImage, GenericImageView, ImageResult, ImageBuffer, RgbaImage, Rgb, Rgba};
use std::{borrow::BorrowMut, io::Cursor};
use env_logger::{Builder, Env};
use log::{info, warn, debug, Level::Debug, set_max_level};
use image::io::Reader as ImageReader;

pub fn get_image() -> ImageResult<DynamicImage> {
    
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
    fn darken_rgb(&self, amount: f32) -> RgbColor {
        // Clamp RGB values between 0 and 1
        // Calculate darkened RGB values
        let new_r = self.rn() - amount;
        let new_g = self.gn() - amount;
        let new_b = self.bn() - amount;
    
        // Clamp darkened RGB values between 0 and 1
        let new_r = new_r.max(0.0).min(1.0);
        let new_g = new_g.max(0.0).min(1.0);
        let new_b = new_b.max(0.0).min(1.0);
    
        RgbColor {
            r: (new_r * 255.0) as u8,
            g: (new_g * 255.0) as u8,
            b: (new_b * 255.0) as u8,
        }
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


pub fn apply_nord(mut _image: DynamicImage) -> DynamicImage {
    let mut image = _image.clone();
    println!("{:?}", image.dimensions());
    //image = image.grayscale();
    let brightness = calculate_average_brightness(&image.to_rgba8());
    info!("Brightness of image is: {:.3}", brightness);
    if  brightness > 0.65 {
        image.invert();
    }

    // Define RGB values in the range 0-255
    let tint_r = 46;
    let tint_g = 52;
    let tint_b = 64;

    //image = image.adjust_contrast(-4.5);
    //image = image.blur(0.2);
    //image = image.adjust_contrast(5.);
    let mut mod_image = image.to_rgba8();
    apply_sepia(&mut mod_image);
    image = DynamicImage::from(mod_image);
    image = image.huerotate(180);
    mod_image = image.to_rgba8();
    apply_nord_filter(&mut mod_image, 1.);
    DynamicImage::from(mod_image)
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

pub fn calculate_average_brightness(image: &RgbaImage) -> f32 {
    // Define maximum dimensions for the resized image
    // const MAX_DIMENSION: u32 = 800;

    // // Calculate the aspect ratio and determine new dimensions
    // let (width, height) = image.dimensions();
    // let (new_width, new_height) = if width > height {
    //     let new_width = MAX_DIMENSION;
    //     let new_height = (height as f32 * new_width as f32 / width as f32) as u32;
    //     (new_width, new_height)
    // } else {
    //     let new_height = MAX_DIMENSION;
    //     let new_width = (width as f32 * new_height as f32 / height as f32) as u32;
    //     (new_width, new_height)
    // };

    // // Resize the image if it is larger than the maximum dimensions
    // let resized_image = if width > MAX_DIMENSION || height > MAX_DIMENSION {
    //     DynamicImage::ImageRgba8(image.clone()).resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3)
    // } else {
    //     DynamicImage::ImageRgba8(image.clone())
    // };
    let resized_image = DynamicImage::ImageRgba8(image.clone());
    // Proceed with brightness calculation
    let mut total_brightness = 0.0;
    let num_pixels = resized_image.width() * resized_image.height();

    for Rgba([r, g, b, _]) in resized_image.to_rgba8().pixels() {
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
        PolarNight::a, PolarNight::b, PolarNight::c, PolarNight::d, 
        // SnowStorm::a, SnowStorm::c, SnowStorm::d, SnowStorm::b, 
    ];
    let colorful_colors = vec![
        Frost::a, Frost::b, Frost::c, Frost::d
    ];
    for color in &contrast_colors {
        println!("{} {} {} has brightness {:.3}",color.r, color.g, color.b, color.brightness())
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
        
        let mut color = RgbColor {r: *r, g: *g, b: *b};
        let darken_by = (color.brightness() - 0.85).max(0.);
        if darken_by > 0. {
            color = color.darken_rgb(darken_by);
        }
        let br = color.brightness();
        if color.calculate_grayscale_similarity() < smallest_grey {
            smallest_grey = color.calculate_grayscale_similarity()
        }
        if color.calculate_grayscale_similarity() > biggest_grey {
            biggest_grey = color.calculate_grayscale_similarity()
        }
        let new = {
            if color.calculate_grayscale_similarity() < 0.25 {
                get_nearest_color(&color, &contrast_colors)
            } else {
                get_nearest_color(&color, &colorful_colors)
            }
            
        };
        // let multiplier = {
        //     if color.calculate_grayscale_similarity() < 0.25 {
        //         // grayscale
        //         if color.brightness() > 0.4 {
        //             0.
        //         } else {
        //             1. - color.brightness()
        //         }
        //     } else {
        //         1. - color.brightness()
        //     }  
        // };
        let strength =(1. - (br - new.brightness()).abs());

        let blended_r = (color.rn() * (1.0 - strength) + new.rn() * strength) * 255.0;
        let blended_g = (color.gn() * (1.0 - strength) + new.gn() * strength) * 255.0;
        let blended_b = (color.bn() * (1.0 - strength) + new.bn() * strength) * 255.0;
        // let mut blended_r = (0.131 * *r as f32 + 0.272 * *g as f32 + 0.534 * *b as f32).min(255.0) as u8;
        // let mut blended_g = (0.168 * *r as f32 + 0.349 * *g as f32 + 0.686 * *b as f32).min(255.0) as u8;
        // let mut blended_b = (0.189 * *r as f32 + 0.393 * *g as f32 + 0.769 * *b as f32).min(255.0) as u8;

        // Update pixel values with blended colors
        *r = blended_r.min(255.) as u8;
        *g = blended_g.min(255.) as u8;
        *b = blended_b.min(255.) as u8;
    }
    println!("greyscale: {:.3} - {:.3}", smallest_grey, biggest_grey);
}
    


