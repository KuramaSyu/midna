use env_logger::{Builder, Env};
use log::{info, warn, debug, Level::Debug, set_max_level};
mod colors;

fn main() {
    env_logger::init();
    let mut image = colors::get_image().unwrap();
    colors::apply_nord(image);
}