use clap::Parser;
use rasterize_text::Color;
use crate::Verbosity;

/// The command-line interface (CLI).
/// ---
/// The CLI is intended for parsing user input from the command-line in the main function.
/// This is achieved with the `parse` function, which parses the command line arguments from [`std::env::args`](https://doc.rust-lang.org/stable/std/env/fn.args.html).
/// ```no_run
/// use clap::Parser;
/// let args = rasterize_text_cli::Cli::parse();
/// ```
#[derive(Debug, Parser)]
#[clap(name = "rasterize-text", author, version)]
#[clap(about = "This is the about message.")]
#[clap(after_help = "This is long message after help.")]
#[clap(trailing_var_arg = true)]
#[clap(arg_required_else_help = true)]
pub struct Cli {

    /// Single-line of text to render.
    #[clap(help = "Single-line of text to render.")]
    #[clap(short = 't', long)]
    #[clap(required = true)]
    pub text: String,

    /// Output png file.
    #[clap(help = "Output PNG file path.")]
    #[clap(short = 'o', long)]
    #[clap(required = true)]
    pub output: std::path::PathBuf,

    /// Text color as a space delimited rgba value.
    #[clap(help = "Text color as a space delimited RGBA value.")]
    #[clap(short = 'c', long)]
    #[clap(default_value_t = Color::default())]
    pub color: Color,

    /// Text size in pixels.
    #[clap(help = "Text size in pixels.")]
    #[clap(short = 's', long)]
    #[clap(default_value_t = 50.0)]
    pub size: f32,

    /// Path to a font file in ttf format. If no file is provided, DejaVu Sans is used.
    #[clap(help = "Path to a ttf font file. If no file is provided, DejaVu Sans is used.")]
    #[clap(short = 'f', long)]
    #[clap(required = false)]
    pub font: Option<std::path::PathBuf>,

    /// Set the logging [`Verbosity`] level.
    #[clap(help = "Set the logging verbosity level.")]
    #[clap(short = 'v', long)]
    #[clap(hide_possible_values = false)]
    #[clap(value_enum)]
    #[clap(default_value_t = Verbosity::default())]
    pub verbosity: Verbosity,
}