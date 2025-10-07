use imageproc::image;
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/splash.jpg");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let input_path = Path::new(&manifest_dir).join("assets").join("splash.jpg");
    let output_path = Path::new(&out_dir).join("splash.jpg");

    println!("cargo:warning=Processing splash image...");

    // Process the image
    let img = image::open(&input_path).expect("Failed to open splash image");
    let img = img.to_luma8();

    let threshold_value = 155;

    // Apply threshold
    let img = imageproc::contrast::threshold(
        &img,
        threshold_value,
        imageproc::contrast::ThresholdType::Binary,
    );

    // Resize
    let (width, height) = img.dimensions();
    let factor = 0.2;
    let width = (width as f32 * factor) as u32;
    let height = (height as f32 * factor) as u32;

    let img = image::imageops::resize(&img, width, height, image::imageops::FilterType::Lanczos3);

    // Save as JPEG
    let output = File::create(&output_path).expect("Failed to create output file");
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(output, 90);
    encoder.encode_image(&img).unwrap();

    println!("cargo:warning=Splash image processed successfully");
}
