use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use rayon::prelude::*;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

#[derive(StructOpt)]
struct Cli {
    #[structopt(long = "src")]
    input_dir: String,

    #[structopt(long = "dest")]
    output_dir: String,

    #[structopt(short, long, default_value = "430")]
    width: u32,

    #[structopt(short, long, default_value = "600")]
    height: u32,

    #[structopt(
        long,
        help = "Maintain aspect ratio and scale image to contain within <width> and <height>"
    )]
    scale_contain: bool,

    #[structopt(long, help = "Retain alpha channel")]
    alpha: bool,
}

fn main() {
    let args = Cli::from_args();

    let mut input_path = Path::new(&args.input_dir);
    let output_path = Path::new(&args.output_dir);

    if !input_path.exists() {
        return println!("{:?} does not exist", &args.input_dir);
    }

    // If the input path is a file, select the parent directory
    if input_path.is_file() {
        input_path = input_path.parent().expect("No parent for file");
    }

    // Ensure base output directory exists
    fs::create_dir_all(output_path).expect("Failed to create output directory");

    // Collect all image files
    let image_paths: Vec<PathBuf> = WalkDir::new(&args.input_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| {
            matches!(
                p.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase()
                    .as_str(),
                "jpg" | "jpeg" | "png"
            )
        })
        .collect();

    image_paths.par_iter().for_each(|path| {
        if let Err(e) = process_image(&args, path, input_path, output_path) {
            println!("Error processing {:?}: {}", path, e);
        }
    });
}

fn replace_alpha_with_white(image: &DynamicImage) -> DynamicImage {
    let (width, height) = image.dimensions();

    let result: RgbaImage = ImageBuffer::from_fn(width, height, |x, y| {
        let pixel = image.get_pixel(x, y);
        let [r, g, b, a] = pixel.0;

        // Blend onto white background
        let alpha = a as f32 / 255.0;
        let new_r = (r as f32 * alpha + 255.0 * (1.0 - alpha)).round() as u8;
        let new_g = (g as f32 * alpha + 255.0 * (1.0 - alpha)).round() as u8;
        let new_b = (b as f32 * alpha + 255.0 * (1.0 - alpha)).round() as u8;

        Rgba([new_r, new_g, new_b, 255]) // Full alpha now, since there's no transparency anymore
    });

    DynamicImage::ImageRgba8(result)
}

fn process_image(
    args: &Cli,
    path: &Path,
    input_path: &Path,
    output_path: &Path,
) -> Result<(), Box<dyn Error>> {
    // Get relative path from base input dir
    let relative_path = path.strip_prefix(input_path)?;
    let new_path = output_path.join(relative_path).with_extension("webp");
    // Skip processing image if output file already exists
    if new_path.exists() {
        return Ok(());
    }

    let mut img = image::open(path)?;
    let (width, height) = img.dimensions();

    // Resize image to exact dimensions if it doesn't match already
    if !args.scale_contain && (width != args.width || height != args.height) {
        img = img.resize_exact(args.width, args.height, FilterType::Lanczos3);
    }

    if args.scale_contain && (width > args.width || height > args.height) {
        img = img.resize(args.width, args.height, FilterType::Lanczos3);
    }

    if !args.alpha {
        img = replace_alpha_with_white(&img);
    }

    // Create parent dirs if needed
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)?;
    }

    save_as_webp(&img, &new_path, args.alpha)?;
    println!(
        "{} => {}",
        path.to_string_lossy(),
        new_path.to_string_lossy()
    );

    Ok(())
}

fn save_as_webp(img: &DynamicImage, path: &Path, alpha: bool) -> Result<(), Box<dyn Error>> {
    use webp::Encoder;
    let webp_data = if alpha {
        let channels = img.to_rgba8();
        let encoder = Encoder::from_rgba(&channels, channels.width(), channels.height());
        encoder.encode(85.0)
    } else {
        let channels = img.to_rgb8();
        let encoder = Encoder::from_rgb(&channels, channels.width(), channels.height());
        encoder.encode(85.0)
    };
    fs::write(path, &*webp_data)?;
    Ok(())
}
