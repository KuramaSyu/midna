use image::{DynamicImage, GenericImageView, ImageResult, ImageBuffer, RgbaImage, Rgb, Rgba, Pixel};
use imageproc::filter::gaussian_blur_f32;
use onnxruntime::session::Session;
use serenity::all::{ButtonStyle, CreateActionRow, CreateButton, ReactionType};
use serenity::model;
use std::collections::HashMap;
use std::hash::Hash;
use std::vec;
use std::{borrow::BorrowMut, io::Cursor};
use env_logger::{Builder, Env};
use log::{info, warn, debug, Level::Debug, set_max_level};
use image::io::Reader as ImageReader;
use onnxruntime::{environment::Environment, ndarray::Array4, tensor::OrtOwnedTensor, GraphOptimizationLevel};
use std::error::Error;
use tokio::task;
use ndarray;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, Debug)]
pub enum ImageType {
    Cartoon,
    Picture
}
pub fn get_image() -> ImageResult<DynamicImage> {
    
    let image = ImageReader::open("test.png")?.decode();

    image
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum Models {
    U2net,
    IsnetAnime,
    IsnetGeneral,
    Algorithm,
}

#[derive(Clone, Debug)]
pub struct Model {
    pub id: usize,
    pub path: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

impl Models {
    pub fn to_struct(&self) -> Model {
        match self {
            Models::U2net => Model {
                id: 0,
                path: String::from("models/u2net.onnx"),
                name: String::from("AI General 2"),
                width: 320,
                height: 320,
            },
            Models::IsnetAnime => Model {
                id: 1,
                path: String::from("models/isnet-anime.onnx"),
                name: String::from("AI Anime"),
                width: 1024,
                height: 1024,
            },
            Models::IsnetGeneral => Model {
                id: 2,
                path: String::from("models/isnet-general-use.onnx"),
                name: String::from("AI General"),
                width: 1024,
                height: 1024,
            },
            Models::Algorithm => Model {    
                id: 3,
                path: String::from("LOCAL"),
                name: String::from("General"),
                width: 320,
                height: 320,
            }
        }
    }

    pub fn from_id(id: usize) -> Self {
        match id {
            0 => Models::U2net,
            1 => Models::IsnetAnime,
            2 => Models::IsnetGeneral,
            3 => Models::Algorithm,
            _ => Models::Algorithm,
        }
    }
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
    pub auto_adjust: bool,
    pub start: bool,
    pub model: Models,
}

impl NordOptions {
    pub fn new() -> Self {
        let mut options = NordOptions::default();
        options.start = true;
        options
    }
    pub fn default() -> Self {
        NordOptions {
            invert: true,
            hue_rotate: 180.0,
            sepia: true,
            nord: true,
            erase_most_present_color: false, 
            erase_when_percentage: 0.3,  // if met: all other filters are ignored
            auto_adjust: true,
            start: false,
            model: Models::Algorithm,
        }
    }

    pub fn from_image_information(image_information: &ImageInformation) -> Self {
        let mut options = NordOptions::default();
        let invert_by_brightness = image_information.brightness.average > 0.5;
        match image_information.image_type {
            Some(ImageType::Cartoon) => {
                options.erase_most_present_color = image_information.color_map.most_present_color_percentage > 0.1;
                options.invert = invert_by_brightness;
                options.hue_rotate = 180.;
                options.sepia = true;
                options.nord = true;
                options.erase_when_percentage = 0.1;
                options.auto_adjust = false;
                options.start = false;
                options.model = Models::Algorithm;
            },
            Some(ImageType::Picture) => {
            if image_information.color_map.most_present_color_percentage > 0.1 {
                options.invert = false;
                options.hue_rotate = 0.;
                options.sepia = false;
                options.nord = false;
                options.erase_most_present_color = true;
                options.erase_when_percentage = 0.1;
                options.auto_adjust = false;
                options.start = false;
                options.model = Models::U2net;
            } else {
                options.invert = false;
                options.hue_rotate = 0.;
                options.sepia = true;
                options.nord = true;
                options.erase_most_present_color = false;
                options.erase_when_percentage = 0.3;
                options.auto_adjust = false;
                options.start = false;
                options.model = Models::IsnetGeneral;
            }},
            None => {}
        }
        options
    }

    pub fn default_erase() -> Self {
        NordOptions {
            invert: false,
            hue_rotate: 0.0,
            sepia: false,
            nord: false,
            erase_most_present_color: true, 
            erase_when_percentage: 0.3,  // if met: all other filters are ignored
            auto_adjust: false,
            start: false,
            model: Models::IsnetGeneral,
        }
    }

    pub fn make_nord_custom_id(&self, message_id: &u64, update: bool) -> String {
        format!(
            "darken-{}-{}-{}-{}-{}-{}-{:.2}-{}-{}-{}-{}", 
            update, self.invert, self.hue_rotate, 
            self.sepia, self.nord, self.erase_most_present_color, 
            self.erase_when_percentage, self.auto_adjust, 
            self.start, self.model.to_struct().id, message_id
        )
    }
    
    pub fn from_custom_id(custom_id: &str) -> Self {
        let mut parts = custom_id.split("-").skip(1);
        let _update = parts.next().unwrap().parse::<bool>().unwrap();
        let invert = parts.next().unwrap().parse::<bool>().unwrap();
        let hue_rotate = parts.next().unwrap().parse::<f32>().unwrap();
        let sepia = parts.next().unwrap().parse::<bool>().unwrap();
        let nord = parts.next().unwrap().parse::<bool>().unwrap();
        let erase_most_present_color = parts.next().unwrap().parse::<bool>().unwrap();
        let erase_when_percentage = parts.next().unwrap().parse::<f64>().unwrap();
        let auto_adjust = parts.next().unwrap().parse::<bool>().unwrap();
        let start = parts.next().unwrap().parse::<bool>().unwrap();
        let model_id: usize = parts.next().unwrap().parse::<usize>().unwrap();
        let model = Models::from_id(model_id);
        NordOptions {
            invert, hue_rotate, sepia, 
            nord, erase_most_present_color, 
            erase_when_percentage, auto_adjust, 
            start, model,
        }
    }
    pub fn build_componets(&self, message_id: u64, update: bool) -> Vec<CreateActionRow> {
        let mut components = Vec::new();
        let mut action_rows = Vec::<Vec<CreateButton>>::new();
        let mut self_no_start = self.clone();
        self_no_start.start = false;

        // make option lists, so that the clicked button is inverted
        let mut option_2d_list = vec![
            // component row
            vec![
                // component
                //name: intert, enabled/disabled, When click, then switch enabled/disabled
                ("Invert", self.invert, NordOptions {invert: !self.invert, ..self_no_start}),
                ("Hue Rotate", if self.hue_rotate == 180. {true} else {false}, NordOptions {hue_rotate: if self.hue_rotate == 180. {0.} else {180.}, ..self_no_start}),
                ("Sepia", self.sepia, NordOptions {sepia: !self.sepia, ..self_no_start}),
                ("Nord", self.nord, NordOptions {nord: !self.nord, ..self_no_start}),
            ],
            vec![
                ("Erase Background", self.erase_most_present_color, NordOptions {erase_most_present_color: !self.erase_most_present_color, ..self_no_start} ),
                ("General", self.model == Models::Algorithm, NordOptions {model: Models::Algorithm, ..self_no_start}),
                ("Anime", self.model == Models::IsnetAnime, NordOptions {model: Models::IsnetAnime, ..self_no_start}),
                ("General Use", self.model == Models::IsnetGeneral, NordOptions {model: Models::IsnetGeneral, ..self_no_start}),
                ("General Use 2", self.model == Models::U2net, NordOptions {model: Models::U2net, ..self_no_start})
            ],
        ];
        if !self.start {
            option_2d_list.push(vec![
                ("Start", self.start, NordOptions {start: !self.start, ..self_no_start}),
            ])
        }
        let mut name_to_color_map = HashMap::<&str, ButtonStyle>::new();
        name_to_color_map.insert("Start", ButtonStyle::Success);

        for option_list in option_2d_list {
            let mut action_row = Vec::<CreateButton>::new();
            for (label, enabled, option) in option_list {
                println!("CustomID: {} Label: {}", option.make_nord_custom_id(&message_id, update), label);
                action_row.push(
                    CreateButton::new(option.make_nord_custom_id(&message_id, update))
                        .label(&format!("{}", label))
                        .style({
                            *name_to_color_map.get(label).unwrap_or(
                                if enabled {  &ButtonStyle::Primary } 
                                else { &ButtonStyle::Secondary }
                            )
                        })
                );
            }
            action_rows.push(action_row);
        }
        for action_row in action_rows {
            components.push(CreateActionRow::Buttons(action_row));
        }
        components.push(
            CreateActionRow::Buttons(
                vec![
                    CreateButton::new(format!("delete-{}", message_id))
                        .style(ButtonStyle::Secondary)
                        .label("Dispose of the old!")
                        .emoji("🗑️".parse::<ReactionType>().unwrap()),
                    // stop button
                    CreateButton::new(format!("stop-{}", message_id))
                        .style(ButtonStyle::Secondary)
                        .label("Dispose of this"),
                    CreateButton::new(format!("clear-{}", message_id))
                        .style(ButtonStyle::Secondary)
                        .label("Keep both")
                ]
            )
        );
        components
    }
}

#[derive(Clone, Debug)]
pub struct RgbColor {
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


pub fn apply_nord(mut _image: DynamicImage, options: NordOptions, info: &ImageInformation) -> DynamicImage {
    let mut image = _image.clone();
    println!("{:?}", image.dimensions());
    //image = image.grayscale();
    let image_information = calculate_average_brightness(&image.to_rgba8());
    println!("Brightness of image is: {:.3}", image_information.brightness.average);
    let model_path = options.model.to_struct().path;
    let environment = Environment::builder()
    .with_name("background_removal")
    .with_log_level(onnxruntime::LoggingLevel::Warning)
    .build().unwrap();

    let session = 
        environment
        .new_session_builder().unwrap()
        .with_optimization_level(GraphOptimizationLevel::Basic).unwrap()
        .with_model_from_file(model_path).unwrap()
    ;
    

    if options.erase_most_present_color {

        if options.model != Models::Algorithm {
            // Remove background
            let segmented_image = remove_background(session, image, &options);
            image = segmented_image.clone();
        } else {
            //Remove most present color if above threshold
            let mut mod_image = image.to_rgba8();
            let (most_present_color, percentage) = get_most_present_colors(&mut mod_image);
            println!("Most present color: {:?} with percentage {:.3}", most_present_color, percentage);
            if percentage >= options.erase_when_percentage {
                // there is actually a color to remove -> remove it
                remove_most_present_colors(&mut mod_image, most_present_color, 40.);
                image = DynamicImage::from(mod_image);
            }
        }


    }

    if options.invert {
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
    if options.sepia {
        apply_sepia(&mut mod_image);
    }
    image = DynamicImage::from(mod_image);
    image = image.huerotate(options.hue_rotate as i32);

    if options.nord {
        mod_image = image.to_rgba8();
        apply_nord_filter(&mut mod_image, &options);
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



pub fn calculate_average_brightness(image: &RgbaImage) -> ImageInformation {
    let image_information = get_image_information(&image);
    println!("IMAGE INFORMATION -------------\n{:?}", image_information);
    image_information
    // let (width, height) = image.dimensions();
    // let sample_distance = (width / 50).max(10) as usize;
    // let resized_image = DynamicImage::ImageRgba8(image.clone());
    // // Proceed with brightness calculation
    // let mut total_brightness = 0.0;
    // let num_pixels = resized_image.width() * resized_image.height() / sample_distance.max(1 as usize) as u32;

    // for (i, Rgba([r, g, b, _])) in resized_image.to_rgba8().pixels().enumerate() {
    //     if i % sample_distance != 0 {
    //         continue;
    //     }
    //     let brightness = calculate_avg_pixel_brightness(*r, *g, *b);
    //     total_brightness += brightness;
    // }
    // total_brightness / num_pixels as f32
}



pub fn calculate_avg_pixel_brightness(r: u8, g: u8, b: u8) -> f32 {
    (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0
}



pub fn apply_nord_filter(image: &mut RgbaImage, options: &NordOptions) {
    let mut smallest_grey = f32::MAX;
    let mut biggest_grey = f32::MIN;
    let max_brightness = if options.erase_most_present_color {1.} else {0.85};

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
        let current_pixel_br = color.brightness();
        let grayscale_similarity = color.calculate_grayscale_similarity();

        if grayscale_similarity < smallest_grey {
            smallest_grey = grayscale_similarity;
        }
        if grayscale_similarity > biggest_grey {
            biggest_grey = grayscale_similarity;
        }

        let darken_by = (current_pixel_br - max_brightness).max(0.0);
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

        let strength = (1.0 - (current_pixel_br - nearest_color.brightness()).abs()) * 0.8;

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
    println!("amount of clolors: {}", color_map.len());
    (RgbColor { r: most_present_color.0, g: most_present_color.1, b: most_present_color.2 }, max_count / total_count)

}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn map_distance_to_transparency(distance: f32, max_distance: f32) -> u8 {
    if distance >= max_distance {
        0
    } else {
        let alpha = smoothstep(0.0, max_distance, distance);
        (alpha * 255.0) as u8
    }
}

pub fn remove_most_present_colors(image: &mut RgbaImage, most_present_color: RgbColor, max_distance: f32) {
    for pixel in image.pixels_mut() {
        let Rgba([r, g, b, a]) = *pixel;
        let rgb = (r, g, b);
        let distance = most_present_color.color_distance(rgb);

        if distance < max_distance {
            let new_alpha = map_distance_to_transparency(distance, max_distance);
            *pixel = Rgba([r, g, b, new_alpha]);
        }
    }
    // Apply a Gaussian blur to the alpha channel for smoothing
    // apply_gaussian_blur_to_alpha(image, 2.0);
}

fn apply_gaussian_blur_to_alpha(image: &mut RgbaImage, sigma: f32) {
    let (width, height) = image.dimensions();
    let mut alpha_image = RgbaImage::new(width, height);

    // Extract the alpha channel
    for (x, y, pixel) in image.enumerate_pixels() {
        alpha_image.put_pixel(x, y, Rgba([0, 0, 0, pixel[3]]));
    }

    // Apply Gaussian blur to the alpha channel
    let blurred_alpha_image = gaussian_blur_f32(&alpha_image, sigma);

    // Update the image with the blurred alpha channel
    for (x, y, blurred_pixel) in blurred_alpha_image.enumerate_pixels() {
        let pixel = image.get_pixel_mut(x, y);
        pixel[3] = blurred_pixel[3];
    }
}

#[derive(Clone, Debug)]
pub struct ImageInformation {
    pub brightness: Brightness,
    pub grayscale_similarity: GrayScaleSimilarity,
    pub color_map: ColorMap,
    pub image_type: Option<ImageType>,
}

impl ImageInformation {
    pub fn new() -> Self {
        ImageInformation {
            brightness: Brightness { average: 0.0, min: 0.0, max: 0.0 },
            grayscale_similarity: GrayScaleSimilarity { average: 0.0, min: 0.0, max: 0.0 },
            color_map: ColorMap { most_present_color: (0, 0, 0), most_present_color_percentage: 0.0, amount: 0 },
            image_type: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct GrayScaleSimilarity {
    pub average: f32,
    pub min: f32,
    pub max: f32,
}
#[derive(Clone, Debug)]
struct ColorMap {
    pub most_present_color: (u8, u8, u8),
    pub most_present_color_percentage: f64,
    pub amount: u64,
}
#[derive(Clone, Debug)]
pub struct Brightness {
    pub average: f32,
    pub min: f32,
    pub max: f32,
}

fn get_image_information(image: &RgbaImage) -> ImageInformation {
    let mut total_brightness = 0.0;
    let mut total_grayscale = 0.0;
    let mut color_map: HashMap<(u8, u8, u8), u64> = HashMap::new();
    let mut image_information = ImageInformation::new();
    let mut min_brightness = f32::MAX;
    let mut max_brightness = f32::MIN;
    let mut min_grayscale = f32::MAX;
    let mut max_grayscale = f32::MIN;
    
    let num_pixels = image.width() * image.height();
    const SAMPLE_DISTANCE: usize = 50;
    let pixel_amount = num_pixels / SAMPLE_DISTANCE.max(1) as u32;

    for (i, Rgba([r, g, b, a])) in image.pixels().enumerate() {
        if i % SAMPLE_DISTANCE != 0 || *a <= 128 {
            continue;
        }
        let pixel = RgbColor { r: *r, g: *g, b: *b };
        let brightness = pixel.brightness();
        let grayscale_similarity = pixel.calculate_grayscale_similarity();

        total_brightness += brightness;
        total_grayscale += grayscale_similarity;

        if brightness < min_brightness {
            min_brightness = brightness;
        }
        if brightness > max_brightness {
            max_brightness = brightness;
        }
        if grayscale_similarity < min_grayscale {
            min_grayscale = grayscale_similarity;
        }
        if grayscale_similarity > max_grayscale {
            max_grayscale = grayscale_similarity;
        }

        *color_map.entry((pixel.r, pixel.g, pixel.b)).or_insert(0) += 1;
    }

    let average_brightness = total_brightness / pixel_amount as f32;
    let average_grayscale_similarity = total_grayscale / pixel_amount as f32;

    let (most_present_color, &most_present_color_count) = color_map.iter().max_by_key(|&(_, count)| count).unwrap_or((&(0, 0, 0), &0));
    let most_present_color_percentage = most_present_color_count as f64 / pixel_amount as f64;
    let color_amount = color_map.len() as u64;

    image_information.brightness = Brightness {
        average: average_brightness,
        min: min_brightness,
        max: max_brightness,
    };

    image_information.grayscale_similarity = GrayScaleSimilarity {
        average: average_grayscale_similarity,
        min: min_grayscale,
        max: max_grayscale,
    };

    image_information.color_map = ColorMap {
        most_present_color: *most_present_color,
        most_present_color_percentage,
        amount: color_amount,
    };
    // predict image type
    if 
        image_information.grayscale_similarity.average < 0.001 
        && image_information.color_map.most_present_color_percentage > 0.1
    {
        image_information.image_type = Some(ImageType::Cartoon);
    } else {
        image_information.image_type = Some(ImageType::Picture);
    }
    image_information
}



fn preprocess_image(image: &DynamicImage, options: &NordOptions) -> Array4<f32> {
    let model = options.model.to_struct();
    let nwidth: u32 = model.width;
    let nheight: u32 = model.height;
    let resized = image.resize_exact(nwidth, nheight, image::imageops::FilterType::Nearest);
    let rgb_image = resized.to_rgb8();

    let mut input_tensor = Array4::<f32>::zeros((1, 3, nheight as usize, nwidth as usize));
    for (y, x, pixel) in rgb_image.enumerate_pixels() {
        input_tensor[[0, 0, x as usize, y as usize]] = pixel[0] as f32 / 255.0;
        input_tensor[[0, 1, x as usize, y as usize]] = pixel[1] as f32 / 255.0;
        input_tensor[[0, 2, x as usize, y as usize]] = pixel[2] as f32 / 255.0;
    }

    input_tensor
}

fn segment_image<'a>(
    session: &'a mut Session<'_>, 
    image: &DynamicImage,
    options: &NordOptions
) -> Result<
    onnxruntime::tensor::OrtOwnedTensor<'a, 'a, f32, ndarray::Dim<ndarray::IxDynImpl>>,
    Box<dyn std::error::Error>
> 
{
    let input_tensor = preprocess_image(image, &options);
    println!("Input tensor shape: {:?}", input_tensor.shape());
    let input_array = vec![input_tensor];
    let output: Vec<OrtOwnedTensor<f32, ndarray::Dim<ndarray::IxDynImpl>>> = session.run(input_array).unwrap();
    println!("Output tensor shape: {:?}", output[0].shape());
    let tensor = output.into_iter().next().unwrap();
    Ok(tensor)
}
fn apply_mask(image: &DynamicImage, mask: &onnxruntime::tensor::OrtOwnedTensor<f32, ndarray::Dim<ndarray::IxDynImpl>>) -> DynamicImage {
    let (orig_width, orig_height) = image.dimensions();
    let mask_width = mask.shape()[2] as u32;
    let mask_height = mask.shape()[3] as u32;

    // Convert the mask to a Vec<u8> by scaling f32 values to u8
    let mask_data: Vec<u8> = mask
    .to_slice()
    .unwrap()
    .iter()
    .map(|&v| (v * 255.0).min(255.0).max(0.0) as u8)
    .collect();

    // Ensure mask dimensions match image dimensions
    let mask_image = DynamicImage::ImageLuma8(
        image::GrayImage::from_raw(mask_width, mask_height, mask_data).unwrap()
    );


    let resized_mask = mask_image.resize_exact(orig_width, orig_height, image::imageops::FilterType::Nearest).to_luma8();

    let mut masked_image = RgbaImage::new(orig_width, orig_height);
    for (x, y, pixel) in masked_image.enumerate_pixels_mut() {
        let pixel_value = image.get_pixel(x, y);
        let mask_value = resized_mask.get_pixel(x, y)[0];
        // // Extract the original pixel's RGBA values
        let [r, g, b, a] = pixel_value.0;

        // Modify alpha channel based on mask value
        // let alpha = if mask_value > 170 { a } else { mask_value / 170 * 255 }; // Set transparency if mask_value <= 127

        *pixel = image::Rgba([r, g, b, mask_value]);
    }

    DynamicImage::ImageRgba8(masked_image)
}


pub fn remove_background<'a>(mut session: Session<'_>, image: DynamicImage, options: &NordOptions) -> DynamicImage {
    let mask = segment_image(&mut session, &image, &options).unwrap();
    let segmented_image = apply_mask(&image, &mask);
    segmented_image
}
