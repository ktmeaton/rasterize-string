#![doc = include_str!("../../../README.md")]
//!
//! # Examples
//!
//! ## English
//!
//! This example demonstrates how to rasterize English text to a pixel image.
//! It uses an English font that is vendored within the crate binary (DejaVu Sans).
//!
//! ```rust
//! use rasterize_text::{Color, read_font_bytes, EN_FONT, rasterize};
//!
//! // Configure aesthetics
//! let text  = "This is a test, we love Unicode ÅΩ!"; // A single-line of text to rasterize.
//! let font  = read_font_bytes(&EN_FONT)?;            // Load the English font that is packaged with the crate (DejaVu Sans)
//! let size  = 50.0;                                  // A font-size in pixels.
//! let color = Color { r: 255, g: 0, b: 0, a: 255 };  // An opaque red color for the text.
//!
//! // Rasterize the text to pixels.
//! let image = rasterize(&text, &font, size, &color);
//!
//! // Get some stats
//! assert_eq!(image.height(), 45 );
//! assert_eq!(image.width(), 740 );
//!
//! // Save image to file
//! image.save("rasterize_en.png")?;
//! # Ok::<(), color_eyre::eyre::Report>(())
//! ```
//!
//! ![image](../../../assets/fonts/dejavu/DejaVuSans.png)
//!
//! This example demonstrates how to rasterize Korean text to a pixel image.
//! It uses a Korean font that is loaded from a local file path.
//!
//! ```rust
//! use rasterize_text::{Color, read_font_file, rasterize};
//!
//! // Configure aesthetics
//! let text      = "제 눈에 안경이다";                                          // A single-line of text to rasterize.
//! let font      = read_font_file(&"../../assets/fonts/noto/NotoSansKR.ttf")?; // Load a local Korean font
//! let size      = 50.0;                                                       // A font-size in pixels.
//! let color     = Color { r: 0, g: 0, b: 255, a: 212 };                       // A transparent blue color for the text.
//!
//! // Rasterize the text to pixels.
//! let image = rasterize(&text, &font, size, &color);
//!
//! // Get some stats
//! assert_eq!(image.height(), 43 );
//! assert_eq!(image.width(), 237 );
//!
//! // Save image to file
//! image.save("rasterize_kr.png")?;
//! # Ok::<(), color_eyre::eyre::Report>(())
//! ```
//!
//! ![image](../../../assets/fonts/noto/NotoSansKR.png)

use image::{ImageBuffer, Rgba};
use log;
use rusttype::{point, Font, Scale};
//use std::error::Error;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use unicode_normalization::UnicodeNormalization;

// Embed fonts at compile time, so that their is a universal fallback
// See license in assets/fonts/*/LICENSE

/// English font DejaVu Sans is provided within the application (vendored).
pub const EN_FONT: &[u8] = include_bytes!("../../../assets/fonts/dejavu/DejaVuSans.ttf");
/// English bold font DejaVu Sans Bold is provided within the application (vendored).
pub const EN_BOLD_FONT: &[u8] = include_bytes!("../../../assets/fonts/dejavu/DejaVuSans-Bold.ttf");
/// Korean font Noto Sans is provided within the application (vendored).
pub const KR_FONT: &[u8] = include_bytes!("../../../assets/fonts/noto/NotoSansKR.ttf");

#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("Failed to read the font file: {1:?}.\nNote: The current working directory is: {:?}", std::env::current_dir().unwrap_or(PathBuf::new()))]
    FileReadError(#[source] std::io::Error, PathBuf),
    #[error("Failed to read font bytes.")]
    BytesReadError,
}

/// Read [TrueType](https://en.wikipedia.org/wiki/TrueType) [`Font`] data from a file [`Path`].
///
/// Returns a [`Result`] which contains either a [`Font`] if reading was successful, or a [`FontError`] on failure.
///
/// A [`Font`] can be passed to downstream functions such as [`rasterize`]. A [`FontError`] can be matched against it's enum variants to better understand why the file read failed.
///
/// # Arguments
///
/// - `path`: Path to a file containing font data in [TrueType](https://en.wikipedia.org/wiki/TrueType) (*.ttf) format.
///     - Accepts any type that can be converted to a [`Path`] reference.
///     - Example types include [`str`], [`String`], and [`PathBuf`].
///
/// # Examples
///
/// ```rust
/// let path = "../../assets/fonts/dejavu/DejaVuSans.ttf";
/// let font = rasterize_text::read_font_file(&path)?;
/// let font = rasterize_text::read_font_file(&String::from(path))?;
/// let font = rasterize_text::read_font_file(&std::path::PathBuf::from(path))?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn read_font_file<P>(path: &P) -> Result<Font, FontError>
where
    P: AsRef<Path>,
{
    let font_bytes = std::fs::read(path)
        .map_err(|e| FontError::FileReadError(e, path.as_ref().to_path_buf()))?;
    Font::try_from_vec(font_bytes).ok_or(FontError::BytesReadError)
}

/// Read [TrueType](https://en.wikipedia.org/wiki/TrueType) [`Font`] data from [`u8`] bytes.
///
/// Returns a [`Result`] which contains either a [`Font`] if reading was successful, or a [`FontError`] on failure.
///
/// A [`Font`] can be passed to downstream functions such as [`rasterize`]. A [`FontError`] can be matched against it's enum variants to better understand why the file read failed.
///
/// You might want to use this function if you're including (vendoring) a font directly within your application. For example, this crate provides the DejaVu Sans font [`EN_FONT`] in the binary data, to be used as a default for testing.
///
/// You might also want to use this function if you're trying to mirror the CSS functionality of loading base64 fonts in the `@font-face` rule.
///
/// # Arguments
///
/// - `bytes`: Represents [TrueType](https://en.wikipedia.org/wiki/TrueType) font data as an iterable of [`u8`] bytes.
///     - Example types include [`std::slice`], and [`Vec`].
///
/// # Examples
///
/// ```rust
/// // The example regular font (DejaVu Sans) as a slice.
/// let bytes: &[u8] = rasterize_text::EN_FONT;
/// let font = rasterize_text::read_font_bytes(&bytes)?;
///
/// // The example bold font (DejaVu Sans Bold) as a vector.
/// let bytes: Vec<u8> = Vec::from(rasterize_text::EN_BOLD_FONT);
/// let font = rasterize_text::read_font_bytes(&bytes)?;
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
pub fn read_font_bytes(bytes: &[u8]) -> Result<Font, FontError> {
    Font::try_from_vec(bytes.to_vec()).ok_or(FontError::BytesReadError)
}

/// Rasterize a string of text string to an [`ImageBuffer`].
///
/// Returns an [`ImageBuffer`] which contains the pixels of the text laid out horizontally.
///
/// The [`ImageBuffer`] can be used in downstream applications as provided by the [`image`] crate.
/// This could include things such as [`save`](https://docs.rs/image/latest/image/struct.ImageBuffer.html#method.save) to a local file, or investigating the dimensions with the [`width`](https://docs.rs/image/latest/image/struct.ImageBuffer.html#method.width) and [`height`](https://docs.rs/image/latest/image/struct.ImageBuffer.html#method.height).
///
/// # Arguments
///
/// - `text`: A text [`str`] reference to rasterize as a pixel image.
///     - Because we need to apply Unicode Normalization to this text, the input type must be &[`str`].
///     - Please see the #Examples for a demonstration of converting text types (ex. [`String`], [`PathBuf`]).
/// - `font`: A [`Font`] reference that contains [TrueType](https://en.wikipedia.org/wiki/TrueType) data.
///     - Potentially created by [`read_font_file`] or [`read_font_bytes`].
/// - `size`: Font size in pixels (ex. `50.0`).
/// - `color`: A [`Color`] that stores RGBA values reflecting the Red, Green, Blue, and Alpha channels.
///
/// # Examples
///
/// This example shows how to rasterize a regular [`str`] to a pixel image.
///
/// ```rust
/// use rasterize_text::{Color, rasterize, EN_FONT, read_font_bytes};
/// let text = "This is a test, we like unicode ÅΩ!";
///
/// // Configure text aesthetics
/// let font  = read_font_bytes(EN_FONT)?;       // Use default font provided with library (DejaVu Sans)
/// let size  = 50.0;                                 // Set font size to 50 pixels
/// let color = Color { r: 255, g: 0, b: 0, a: 255 }; // Render as an opaque red color.
///
/// // Rasterize the text to pixels.
/// let image = rasterize_text::rasterize(&text, &font, size, &color);
///
/// // Save to a local file.
/// image.save("rasterize_str.png")?;
///
/// // Get some stats
/// assert_eq!(image.height(), 45 );
/// assert_eq!(image.width(), 720 );
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
///
/// Converting a [`String`] to [`str`] for [`rasterize`].
///
/// ```rust
/// # use rasterize_text::{Color, rasterize, EN_FONT, read_font_bytes};
/// # let font  = read_font_bytes(EN_FONT)?;
/// # let size  = 50.0;
/// # let color = Color { r: 255, g: 0, b: 0, a: 255 };
/// let text = String::from("This is a test, we like unicode ÅΩ!");
/// let text = text.as_str();
/// let image = rasterize_text::rasterize(&text, &font, size, &color);
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```
///
/// Converting a [`PathBuf`] to [`str`] for [`rasterize`].
///
/// ```rust
/// # use rasterize_text::{Color, rasterize, EN_FONT, read_font_bytes};
/// # let font  = read_font_bytes(EN_FONT)?;
/// # let size  = 50.0;
/// # let color = Color { r: 255, g: 0, b: 0, a: 255 };
/// let text = std::path::PathBuf::from("This is a test, we like unicode ÅΩ!");
/// let text = text.as_os_str().to_str().unwrap_or("");
/// let image = rasterize_text::rasterize(&text, &font, size, &color);
/// # Ok::<(), color_eyre::eyre::Report>(())
/// ```

pub fn rasterize<T, I>(
    text: &T,
    font: &Font,
    size: f32,
    color: &Color,
) -> ImageBuffer<Rgba<u8>, Vec<u8>>
where
    T: AsRef<str> + Clone + UnicodeNormalization<I>,
    I: Iterator<Item = char>,
{
    // Use uniform scaling of the text based on the size in pixels
    let scale = Scale::uniform(size);
    log::debug!("Font Size (pixels): {scale:?}");

    // Configure spatial metrics based on the uniform scaling
    let metrics = font.v_metrics(scale);
    log::debug!("Font Metrics: {metrics:?}");

    // Apply unicode normalization
    let normalized = text.clone().nfc().collect::<String>();

    // layout the glyphs in the text horizontally
    let glyphs: Vec<_> = font
        .layout(normalized.as_ref(), scale, point(0., 0. + metrics.ascent))
        .collect();

    // Display the individual glyph info in debug mode
    glyphs
        .iter()
        .for_each(|glyph| log::debug!("Glyph: {glyph:?}"));

    // Get the width and height of the final image raster, based on the pixels used.
    // Note: In certain cases, the min_x can actually be less than 0! For example,
    // when "T" is rendered in isolation, it wil start at -2 pixels, because of kerning.
    let mut min_x: i32 = 0;
    let mut max_x: i32 = 0;
    let mut min_y: i32 = 0;
    let mut max_y: i32 = 0;

    // Iterate through the glyphs, updating our x coordinate extremes
    glyphs.iter().for_each(|glyph| {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            if bounding_box.min.x < min_x {
                min_x = bounding_box.min.x
            }
            if bounding_box.max.x > max_x {
                max_x = bounding_box.max.x
            }
            if bounding_box.min.y < min_y {
                min_y = bounding_box.min.y
            }
            if bounding_box.max.y > max_y {
                max_y = bounding_box.max.y
            }
        }
    });
    log::debug!("Minimum x coordinate: {min_x:?}");
    log::debug!("Maximum x coordinate: {max_x:?}");
    log::debug!("Minimum y coordinate: {min_y:?}");
    log::debug!("Maximum y coordinate: {max_y:?}");

    // If the min_x is negative, such as a pixel range from x=-2 to x=4,
    // we need to account or that in the width, which would be width=6 in this case.
    let width = max_x - min_x;
    let height = max_y - min_y;

    // // get output image height according to the font metrics
    // // Get height based on font metrics? Maybe we shouldn't use this, and
    // // focus on actually where the pixels go?
    // let metrics_height = (metrics.ascent - metrics.descent).ceil();
    // let height = std::cmp::max(height, metrics_height as i32);

    log::debug!("Image Width: {width:?}");
    log::debug!("Image Height: {height:?}");

    // construct an image buffer to hold RGBA pixels representing each character
    let mut image_buffer = ImageBuffer::<Rgba<u8>, Vec<_>>::new(width as u32, height as u32);

    // Make a default pixel, which is fully transparent
    let default_pixel: Rgba<u8> = Rgba([0, 0, 0, 0]);

    // Iterate through each glyph ('letter'), and add it's pixels to the buffer
    for glyph in glyphs {
        // I don't remember in which cases a glyph might not have a pixel bounding box...
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            log::debug!("{0:?}, {bounding_box:?}", glyph.id());

            // Iterate through all the pixels in this letter (glyph)
            // 'x' and 'y' are relative to this letter's bounding box.
            // 'v' is 'coverage', which I think represents the intensity
            // of how the pixel should be colored in. If it's '0' then the
            // pixel is not colored in.
            glyph.draw(|x, y, v| {
                // Convert the pixel's relative position to an absolute position in the buffer
                // With special handling for if the absolute position of the letter is negative
                // Ex. the letter 'T' starting at absolute position x=-2;
                let y = match bounding_box.min.y >= 0 {
                    true => y as i32 + bounding_box.min.y,
                    false => y as i32,
                };
                let x = match bounding_box.min.x >= 0 {
                    true => x as i32 + bounding_box.min.x,
                    false => x as i32,
                };

                // construct a pixel
                let pixel = Rgba([
                    (color.r as f32 * v) as u8,
                    (color.g as f32 * v) as u8,
                    (color.b as f32 * v) as u8,
                    (color.a as f32 * v) as u8,
                ]);

                // add pixel to image buffer, if that pixel is still the default
                // I can't remember why I had this check...
                if image_buffer.get_pixel(x as u32, y as u32) == &default_pixel {
                    image_buffer.put_pixel(x as u32, y as u32, pixel);
                }
            });
        }
    }

    image_buffer
}

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum ColorError {
    #[error("Failed to parse value {2} to RGBA color in: {1}")]
    RgbaParseError(#[source] std::num::ParseIntError, String, String),
    #[error("Failed to parse RGBA because of an incorrect number of values (expected 4): {0:?}.")]
    RgbaLengthError(Vec<u8>),
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Convert to lowercase for RUST_LOG env var compatibility
        let color = format!("{} {} {} {}", self.r, self.g, self.b, self.a);
        write!(f, "{color}")
    }
}

impl FromStr for Color {
    type Err = ColorError;

    /// Returns a [`Color`] converted from a [`str`].
    ///
    /// ## Examples
    ///
    fn from_str(color: &str) -> Result<Self, Self::Err> {
        //let rgba: Vec<u8> = color.split(" ").map(|s| s.parse::<u8>().map_err(ColorError::UnknownColorError)).collect::<Vec<Result<u8>, ColorError>>()?;
        let rgba: Vec<u8> = color
            // Split the color string on a space delimiter
            .split(" ")
            // Try to convert the space-delimited text to rgba values (8-bit, 0-255)
            .map(|s| {
                s.parse::<u8>()
                    .map_err(|e| ColorError::RgbaParseError(e, color.to_string(), s.to_string()))
            })
            // Gather the rgba values into a vector, throw an error if an issue was encountered
            .collect::<Result<Vec<u8>, ColorError>>()?;

        // Convert the rgba value vector into a fixed array of length 4
        let rgba: [u8; 4] = rgba
            .clone()
            .try_into()
            .map_err(|_| ColorError::RgbaLengthError(rgba))?;

        let color = Color {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        };

        Ok(color)
    }
}
