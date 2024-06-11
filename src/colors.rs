use image::{DynamicImage, GenericImageView, ImageResult, ImageBuffer, RgbaImage, Rgb, Rgba};
use std::collections::HashMap;
use std::hash::Hash;
use std::{borrow::BorrowMut, io::Cursor};
use env_logger::{Builder, Env};
use log::{info, warn, debug, Level::Debug, set_max_level};
use image::io::Reader as ImageReader;

pub fn get_image() -> ImageResult<DynamicImage> {
    
    ImageReader::open("test.png")?.decode()
}
// implement clone
#[derive(Clone, Debug)]
pub struct NordOptions {
    pub invert: bool,
    pub hue_rotate: f32,
    pub sepia: bool,
    pub nord: bool,
    pub erase_most_present_color: bool,
    pub erase_when_percentage: f64,
}

impl NordOptions {
    pub fn new() -> Self {
        NordOptions {
            invert: false,
            hue_rotate: 0.0,
            sepia: false,
            nord: false,
            erase_most_present_color: false,
            erase_when_percentage: 0.3,
        }
    }
    pub fn default() -> Self {
        NordOptions {
            invert: true,
            hue_rotate: 180.0,
            sepia: true,
            nord: true,
            erase_most_present_color: true, 
            erase_when_percentage: 0.3,  // if met: all other filters are ignored
        }
    }
}

#[derive(Clone, Debug)]
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
    fn color_distance(&self, c2: (u8, u8, u8)) -> f32 {
        let (r1, g1, b1) = (self.r, self.g, self.b);
        let (r2, g2, b2) = c2;
        let dr = r1 as f32 - r2 as f32;
        let dg = g1 as f32 - g2 as f32;
        let db = b1 as f32 - b2 as f32;
        (dr * dr + dg * dg + db * db).sqrt()
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



pub fn apply_nord(mut _image: DynamicImage, options: NordOptions) -> DynamicImage {
    let mut image = _image.clone();
    println!("{:?}", image.dimensions());
    //image = image.grayscale();
    let brightness = calculate_average_brightness(&image.to_rgba8());
    println!("Brightness of image is: {:.3}", brightness);

    let mut mod_image = image.to_rgba8();


    if options.erase_most_present_color {
        // Remove most present color if above threshold
        let (most_present_color, percentage) = get_most_present_colors(&mut mod_image);
        println!("Most present color: {:?} with percentage {:.3}", most_present_color, percentage);
        if percentage >= options.erase_when_percentage {
            remove_most_present_colors(&mut mod_image, most_present_color, 0.3);
            return DynamicImage::from(mod_image);
        }
    }

    if  brightness > 0.65 && options.invert {
        image.invert();
    }

    // Define RGB values in the range 0-255
    let tint_r = 46;
    let tint_g = 52;
    let tint_b = 64;

    //image = image.adjust_contrast(-4.5);
    //image = image.blur(0.2);
    //image = image.adjust_contrast(5.);
    
    if options.sepia {
        apply_sepia(&mut mod_image);
    }
    image = DynamicImage::from(mod_image);
    image = image.huerotate(options.hue_rotate as i32);

    if options.nord {
        mod_image = image.to_rgba8();
        apply_nord_filter(&mut mod_image, 1.);
        DynamicImage::from(mod_image)
    } else {
        image
    }
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
    let (width, height) = image.dimensions();
    let sample_distance = (width / 50).max(10) as usize;
    let resized_image = DynamicImage::ImageRgba8(image.clone());
    // Proceed with brightness calculation
    let mut total_brightness = 0.0;
    let num_pixels = resized_image.width() * resized_image.height() / sample_distance.max(1 as usize) as u32;

    for (i, Rgba([r, g, b, _])) in resized_image.to_rgba8().pixels().enumerate() {
        if i % sample_distance != 0 {
            continue;
        }
        let brightness = calculate_avg_pixel_brightness(*r, *g, *b);
        total_brightness += brightness;
    }
    total_brightness / num_pixels as f32
}

pub fn calculate_avg_pixel_brightness(r: u8, g: u8, b: u8) -> f32 {
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0
}

pub fn apply_nord_filter(image: &mut RgbaImage, blend_factor: f32) {
    let mut smallest_grey = f32::MAX;
    let mut biggest_grey = f32::MIN;

    let contrast_colors = vec![
        PolarNight::a, PolarNight::b, PolarNight::c, PolarNight::d,
    ];

    let colorful_colors = vec![
        Frost::a, Frost::b, Frost::c, Frost::d,
    ];

    for color in &contrast_colors {
        println!("{} {} {} has brightness {:.3}", color.r, color.g, color.b, color.brightness());
    }

    fn get_nearest_color<'a>(color: &RgbColor, all_colors: &'a [RgbColor]) -> &'a RgbColor {
        let mut min_distance = f32::MAX;
        let mut nearest_color = &all_colors[0];
        let br = color.brightness();
        for c in all_colors.iter() {
            let dist = (c.brightness() - br).abs();
            if dist < min_distance {
                min_distance = dist;
                nearest_color = c;
            }
        }
        nearest_color
    }

    let mut cache: HashMap<(u8, u8, u8), (u8, u8, u8)> = HashMap::new();

    for Rgba([r, g, b, _]) in image.pixels_mut() {
        let key = (*r, *g, *b);
        if let Some(&(cached_r, cached_g, cached_b)) = cache.get(&key) {
            *r = cached_r;
            *g = cached_g;
            *b = cached_b;
            continue;
        }

        let color = RgbColor { r: *r, g: *g, b: *b };
        let br = color.brightness();
        let grayscale_similarity = color.calculate_grayscale_similarity();

        if grayscale_similarity < smallest_grey {
            smallest_grey = grayscale_similarity;
        }
        if grayscale_similarity > biggest_grey {
            biggest_grey = grayscale_similarity;
        }

        let darken_by = (br - 0.85).max(0.0);
        let adjusted_color = if darken_by > 0.0 {
            color.darken_rgb(darken_by)
        } else {
            color
        };

        let nearest_color = if grayscale_similarity < 0.25 {
            get_nearest_color(&adjusted_color, &contrast_colors)
        } else {
            get_nearest_color(&adjusted_color, &colorful_colors)
        };

        let strength = 1.0 - (br - nearest_color.brightness()).abs() * 0.8;

        let blended_r = (adjusted_color.rn() * (1.0 - strength) + nearest_color.rn() * strength) * 255.0;
        let blended_g = (adjusted_color.gn() * (1.0 - strength) + nearest_color.gn() * strength) * 255.0;
        let blended_b = (adjusted_color.bn() * (1.0 - strength) + nearest_color.bn() * strength) * 255.0;

        let final_r = blended_r.min(255.0) as u8;
        let final_g = blended_g.min(255.0) as u8;
        let final_b = blended_b.min(255.0) as u8;

        cache.insert(key, (final_r, final_g, final_b));

        *r = final_r;
        *g = final_g;
        *b = final_b;
    }

    println!("greyscale: {:.3} - {:.3}", smallest_grey, biggest_grey);
}

pub fn get_most_present_colors(image: &mut RgbaImage) -> (RgbColor, f64) {
    let mut color_map: HashMap<(u8, u8, u8), u64> = HashMap::new();
    static SAMPLES_PER_LINE: usize = 50;

    for (i, Rgba([r, g, b, a])) in image.pixels_mut().enumerate() {
        if i % SAMPLES_PER_LINE != 0 || *a <= 128 {
            // ignore transparent pixels
            continue;
        }
        let key = (*r, *g, *b);
        color_map.entry(key).and_modify(|e| *e += 1).or_insert(1);
    }
    
    let mut most_present_color = (0, 0, 0);
    let mut total_count = 0.;
    let mut max_count = 0.;
    for (color, count) in color_map.iter() {
        if *count as f64 > max_count {
            max_count = *count as f64;
            most_present_color = *color;
        }
        total_count += *count as f64;
    }
    (RgbColor { r: most_present_color.0, g: most_present_color.1, b: most_present_color.2 }, max_count / total_count)

}


fn map_distance_to_transparency(distance: f32, max_distance: f32) -> u8 {
    if distance >= max_distance {
        0
    } else {
        let alpha = 1.0 - (distance / max_distance);
        (alpha * 255.0) as u8
    }
}

pub fn remove_most_present_colors(image: &mut RgbaImage, most_present_color: RgbColor, max_distance: f32) {
    for pixel in image.pixels_mut() {
        let Rgba([r, g, b, a]) = *pixel;
        let rgb = (r, g, b);
        let mut min_distance = most_present_color.color_distance(rgb);

        if min_distance < max_distance {
            let new_alpha = map_distance_to_transparency(min_distance, max_distance);
            *pixel = Rgba([r, g, b, new_alpha]);
        }
    }
}

