use chrono::Local;                      // Display log time in logging message.
use clap::Parser;                       // Parse command-line arguments rfom the user.
use color_eyre::eyre::{Report, Result}; // Handle errors with backtracking.
use env_logger::Builder;                // Build a custom log message based on a Verbosity level.
use rasterize_text_cli::Cli;            // The command-line interface for the rasterize-text crate.
use std::io::Write;                     // Use the writeln macro for the loggin messages.

fn main() -> Result<(), Report> {

    // Parse arguments from the CLI
    let args = Cli::parse();
    // initialize color_eyre crate for colorized logs
    color_eyre::install()?;

    // Customize logging message format
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf, 
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, args.verbosity.to_levelfilter())
        .init();

    // Convert input text to str to allow for unicode normalization
    let text = args.text.as_str();
    // Read font
    let font = match &args.font {
        Some(path) => rasterize_text::read_font_file(path)?,
        None       => rasterize_text::read_font_bytes(rasterize_text::REGULAR_FONT)?,
    };
    let image = rasterize_text::rasterize(&text, &font, args.size, &args.color);
    image.save(args.output)?;

    Ok(())
}
