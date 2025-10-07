use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use std::path::PathBuf;

use comically::device::Device;
use comically::{ComicConfig, ComicFile, ImageFormat, OutputFormat, PngCompression, SplitStrategy};

#[derive(Parser)]
#[command(name = "comically-cli")]
#[command(about = "Convert comic archives (CBZ/CBR) to e-reader formats", long_about = None)]
#[command(version)]
struct Args {
    /// Input comic file (CBZ or CBR)
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output directory
    #[arg(short, long, value_name = "DIR", default_value = ".")]
    output_dir: PathBuf,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormatArg::Cbz)]
    format: OutputFormatArg,

    /// Device preset
    #[arg(
        short,
        long,
        value_name = "DEVICE",
        default_value = "kindle-paperwhite"
    )]
    device: String,

    /// Custom device width (requires --device custom)
    #[arg(long, value_name = "PIXELS")]
    width: Option<u32>,

    /// Custom device height (requires --device custom)
    #[arg(long, value_name = "PIXELS")]
    height: Option<u32>,

    /// Image format
    #[arg(long, value_enum, default_value_t = ImageFormatArg::Jpeg)]
    image_format: ImageFormatArg,

    /// JPEG/WebP quality (0-100)
    #[arg(long, value_name = "QUALITY", default_value_t = 85)]
    quality: u8,

    /// PNG compression level
    #[arg(long, value_enum, default_value_t = PngCompressionArg::Default)]
    png_compression: PngCompressionArg,

    /// Brightness adjustment (-100 to +100)
    #[arg(
        long,
        value_name = "VALUE",
        default_value_t = -10,
        allow_hyphen_values = true
    )]
    brightness: i32,

    /// Gamma correction (0.1 to 3.0)
    #[arg(long, value_name = "VALUE", default_value_t = 1.8)]
    gamma: f32,

    /// Margin color
    #[arg(long, value_enum, default_value_t = MarginColorArg::None)]
    margin_color: MarginColorArg,

    /// Page split strategy
    #[arg(long, value_enum, default_value_t = SplitStrategyArg::RotateSplit)]
    split: SplitStrategyArg,

    /// Right-to-left reading direction (manga mode)
    #[arg(long, default_value_t = true)]
    rtl: bool,

    /// Disable automatic cropping
    #[arg(long, default_value_t)]
    no_auto_crop: bool,

    /// Verbose output
    #[arg(short, long, default_value_t)]
    verbose: bool,

    /// Quiet mode (minimal output)
    #[arg(short, long, default_value_t)]
    quiet: bool,
}

impl Args {
    fn parse_device(&self) -> Result<Device> {
        if self.device == "custom" {
            let w = self
                .width
                .context("--width is required when using --device custom")?;
            let h = self
                .height
                .context("--height is required when using --device custom")?;
            return Ok(Device::Custom {
                width: w,
                height: h,
            });
        }

        comically::device::Preset::try_from(self.device.as_str())
            .map(Into::into)
            .map_err(|e| anyhow::anyhow!(e))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormatArg {
    Cbz,
    Epub,
    Mobi,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Cbz => OutputFormat::Cbz,
            OutputFormatArg::Epub => OutputFormat::Epub,
            OutputFormatArg::Mobi => OutputFormat::Mobi,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum ImageFormatArg {
    Jpeg,
    Png,
    Webp,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum PngCompressionArg {
    Fast,
    Default,
    Best,
}

impl From<PngCompressionArg> for PngCompression {
    fn from(arg: PngCompressionArg) -> Self {
        match arg {
            PngCompressionArg::Fast => PngCompression::Fast,
            PngCompressionArg::Default => PngCompression::Default,
            PngCompressionArg::Best => PngCompression::Best,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum MarginColorArg {
    None,
    Black,
    White,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum SplitStrategyArg {
    None,
    Split,
    Rotate,
    RotateSplit,
}

impl From<SplitStrategyArg> for SplitStrategy {
    fn from(arg: SplitStrategyArg) -> Self {
        match arg {
            SplitStrategyArg::None => SplitStrategy::None,
            SplitStrategyArg::Split => SplitStrategy::Split,
            SplitStrategyArg::Rotate => SplitStrategy::Rotate,
            SplitStrategyArg::RotateSplit => SplitStrategy::RotateAndSplit,
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
    setup_logging(args.verbose, args.quiet);

    // Validate input file
    if !args.input.exists() {
        anyhow::bail!("Input file does not exist: {}", args.input.display());
    }

    // Create output directory if it doesn't exist
    if !args.output_dir.exists() {
        std::fs::create_dir_all(&args.output_dir).context("Failed to create output directory")?;
    }

    // Build config
    let config = build_config(&args)?;
    let output_format = config.output_format;

    // Create comic
    let comic = ComicFile::new(args.input.clone());

    if !args.quiet {
        log::info!(
            "Converting: `{}` to {output_format:?}",
            args.input.display()
        );
    }

    // Open archive
    let archive: Vec<_> = comically::archive::unarchive_comic_iter(&comic)
        .context("Failed to open comic archive")?
        .filter_map(|result| {
            result
                .map_err(|e| log::warn!("Failed to load archive file: {}", e))
                .ok()
        })
        .collect();
    let num_images = archive.len();

    if !args.quiet {
        log::info!("Found {num_images} images");
    }

    // Process images
    if !args.quiet {
        log::info!("Processing images...");
    }
    let images =
        comically::image::process_batch(archive, &config).context("Failed to process images")?;

    if !args.quiet {
        log::info!("Processed {} images", images.len());
    }

    // Build output
    if !args.quiet {
        log::info!("Building {output_format:?}...");
    }

    let bytes = match output_format {
        OutputFormat::Cbz => comically::cbz::build(&images),
        OutputFormat::Epub => comically::epub::build(comic.title(), &config, &images),
        OutputFormat::Mobi => {
            if !comically::is_kindlegen_available() {
                anyhow::bail!(
                    "KindleGen is not available. Please install it to create MOBI files."
                );
            }
            let bytes = comically::epub::build(comic.title(), &config, &images);
            let epub_path = args
                .output_dir
                .join(comic.with_extension(OutputFormat::Epub));
            std::fs::write(&epub_path, bytes).context("Failed to write EPUB file")?;

            let output_mobi = args.output_dir.join(comic.with_extension(output_format));
            let spawned = comically::mobi::create(epub_path, output_mobi.clone())
                .context("Failed to start MOBI conversion")?;
            spawned.wait().context("MOBI conversion failed")?;
            return Ok(());
        }
    };

    let output_path = args.output_dir.join(comic.with_extension(output_format));
    std::fs::write(&output_path, bytes).context("Failed to write output file")?;

    if !args.quiet {
        log::info!("Done: {}", output_path.display());
    }

    Ok(())
}

fn setup_logging(verbose: bool, quiet: bool) {
    if quiet {
        return;
    }

    let level = if verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    env_logger::Builder::from_default_env()
        .filter_level(level)
        .format_timestamp(None)
        .format_target(false)
        .init();
}

fn build_config(args: &Args) -> Result<ComicConfig> {
    // Validate quality
    if args.quality > 100 {
        anyhow::bail!("Quality must be between 0 and 100");
    }

    // Validate brightness
    if args.brightness < -100 || args.brightness > 100 {
        anyhow::bail!("Brightness must be between -100 and 100");
    }

    // Validate gamma
    if args.gamma < 0.1 || args.gamma > 3.0 {
        anyhow::bail!("Gamma must be between 0.1 and 3.0");
    }

    // Build device preset
    let device = args.parse_device()?;

    // Build image format
    let image_format = match args.image_format {
        ImageFormatArg::Jpeg => ImageFormat::Jpeg {
            quality: args.quality,
        },
        ImageFormatArg::Png => ImageFormat::Png {
            compression: args.png_compression.into(),
        },
        ImageFormatArg::Webp => ImageFormat::WebP {
            quality: args.quality,
        },
    };

    // MOBI requires JPEG
    let image_format = if args.format == OutputFormatArg::Mobi {
        if args.image_format != ImageFormatArg::Jpeg {
            log::warn!("MOBI format requires JPEG images, overriding image format");
        }
        ImageFormat::Jpeg {
            quality: args.quality,
        }
    } else {
        image_format
    };

    // Build margin color
    let margin_color = match args.margin_color {
        MarginColorArg::None => None,
        MarginColorArg::Black => Some(0),
        MarginColorArg::White => Some(255),
    };

    Ok(ComicConfig {
        output_format: args.format.into(),
        device,
        image_format,
        brightness: args.brightness,
        gamma: args.gamma,
        split: args.split.into(),
        right_to_left: args.rtl,
        auto_crop: !args.no_auto_crop,
        margin_color,
    })
}
