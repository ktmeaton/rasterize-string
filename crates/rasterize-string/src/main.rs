use color_eyre::eyre::{Result, Report};
use rasterize_string::*;

fn main() -> Result<(), Report> {
    println!("Hello, world!");

    env_logger::init();
    let font = load_font(&"assets/fonts/dejavu/DejaVuSans.ttf")?;
    let color = &[255, 0, 0, 255];
    let image_buffer = text_to_image_buffer(&"Testingggg éь", &font, 30.0, &color)?;
    image_buffer.save("image_buffer.png")?;

    Ok(())
}