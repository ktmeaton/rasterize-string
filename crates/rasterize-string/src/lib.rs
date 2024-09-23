#![doc = include_str!("../README.md")]
//! 
//! # Examples
//! 
//! ```rust
//! use rasterize_string::{load_font, text_to_image_buffer};
//! 
//! // Configure aesthetics (font, size, color, etc.)
//! let text      = "This is a test!";
//! let font      = load_font(&"../../assets/fonts/dejavu/DejaVuSans.ttf")?;
//! let font_size = 50.0;
//! let color     = &[255, 0, 0, 255];
//! 
//! // Rasterize the text to pixels.
//! let image = text_to_image_buffer(&text, &font, font_size, color)?;
//! 
//! // Get some stats
//! assert_eq!(image.height(), 51 );
//! assert_eq!(image.width(), 284 );
//! 
//! // Save image to file
//! image.save("test.png")?;
//! # Ok::<(), color_eyre::eyre::Report>(())
//! ```
//! 
//! Something, something.

use color_eyre::eyre::{eyre, Result, Report, WrapErr};
use image::{ImageBuffer, Rgba};
use log;
use std::path::Path;
use std::fmt::Debug;
use rusttype::{Font, Scale, point};

/// Load font from a file path.
/// 
/// # Examples
/// 
/// ```rust
/// use rasterize_string::load_font;
/// let font = load_font(&"../../assets/fonts/dejavu/DejaVuSans.ttf")?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn load_font<P>(path: &P) -> Result<Font, Report>
where
    P: AsRef<Path> + Debug
{
    let font_bytes = std::fs::read(path).wrap_err_with(|| format!("Could not load font from file path: {path:?}"))?;
    let font = Font::try_from_vec(font_bytes).ok_or_else(|| eyre!("Could not convert file to Font: {path:?}"))?;
    Ok(font)
}

/// Load font from bytes.
/// 
/// Maybe of use if you're going to load a font from base64?
pub fn load_font_from_bytes(bytes: &[u8]) -> Result<Font, Report> {
    let font = Font::try_from_vec(bytes.to_vec()).ok_or_else(|| eyre!("Could not convert bytes to font."))?;
    Ok(font)
}

/// Convert text string to an [`ImageBuffer`](https://docs.rs/image/latest/image/struct.ImageBuffer.html).
/// 
/// # Parameters
/// 
/// # Examples
/// 
/// ```rust
/// use rasterize_string::{load_font, text_to_image_buffer};
/// let text = "This is a test!";
/// 
/// // Configure aesthetics (font, size, color, etc.)
/// let font      = load_font(&"../../assets/fonts/dejavu/DejaVuSans.ttf")?;
/// let font_size = 50.0;
/// let color     = &[255, 0, 0, 255];
/// 
/// // Rasterize the text to pixels.
/// let image      = text_to_image_buffer(&text, &font, font_size, color)?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn text_to_image_buffer<T>(text: &T, font: &Font, font_size: f32, color: &[u8; 4]) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Report> 
where 
    T: AsRef<str>,
{
    // Separate the color into rgba channels.
    let [r, g, b, a] = [color[0], color[1], color[2], color[3]];

    // Set font size and get metrics
    let scale = Scale::uniform(font_size);
    log::debug!("Font Size (pixels): {scale:?}");

    let metrics = font.v_metrics(scale);
    log::debug!("Font Metrics: {metrics:?}");

    // layout the glyphs in the text horizontally
    let glyphs: Vec<_> = font.layout(text.as_ref(), scale, point(0., 0. + metrics.ascent)).collect();
    glyphs.iter().for_each(|glyph| log::debug!("Glyph: {glyph:?}"));

    // get output image height from the font metrics, since height is only dependent on font
    let height = (metrics.ascent - metrics.descent).ceil() as u32;
    log::debug!("Text Height: {height:?}");

    // Get output image widths from the pixel bounding boxes, since width is dependent
    // on font + the text to write (for horizontal layout)
    let mut min_x: i32 = 0;
    let mut max_x: i32 = 0;

    glyphs.iter().for_each(|glyph| {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            if bounding_box.min.x < min_x { min_x = bounding_box.min.x }
            if bounding_box.max.x > max_x { max_x = bounding_box.max.x }
        }
    });
    log::debug!("Minimum x coordinate: {min_x:?}");
    log::debug!("Maximum x coordinate: {max_x:?}");

    let width = if min_x >= 0 { max_x } else { max_x - min_x };

    log::debug!("Image Width: {width:?}");
    log::debug!("Image Height: {height:?}");

    // construct an image buffer to hold text pixels
    let mut image_buffer = ImageBuffer::<Rgba<u8>, Vec<_>>::new(width as u32, height as u32);

    // the default pixel is fully transparent
    let default_pixel: Rgba<u8> = Rgba([0, 0, 0, 0]);

    // iterate through each glyph ('letter')
    for glyph in glyphs {

        if let Some(bounding_box) = glyph.pixel_bounding_box() {

            log::debug!("{0:?}, {bounding_box:?}", glyph.id());

            // rasterize each glyph, by iterating through the pixels
            // x, y are relative to bounding box, v is 'coverage'
            glyph.draw(|x, y, v| {
                //debug!("\t\tx: {x}, y: {y}, v: {v}");
                let y = y as i32 + bounding_box.min.y;

                // sometimes x bounding box is negative, because kerning is applied
                // ex. the letter 'T' in isolation
                // in this case, force 0 to be the start point
                let x = if bounding_box.min.x >= 0 {
                    x as i32 + bounding_box.min.x
                } else {
                    x as i32
                };

                // construct a pixel
                let pixel = Rgba([
                    (r as f32 * v) as u8,
                    (g as f32 * v) as u8,
                    (b as f32 * v) as u8,
                    (a as f32 * v) as u8,
                ]);

                // add pixel to image buffer, if that pixel is still the default
                // I can't remember why I had this check...
                if image_buffer.get_pixel(x as u32, y as u32) == &default_pixel {
                    image_buffer.put_pixel(x as u32, y as u32, pixel);
                }
            });
        }
    }

    Ok(image_buffer)
}    
