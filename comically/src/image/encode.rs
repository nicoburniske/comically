//! Image encoding: JPEG, PNG, WebP

use anyhow::{Context, Result};
use imageproc::image::{ColorType, DynamicImage, GenericImageView};
use webp::WebPMemory;

use crate::archive::ArchiveFile;
use crate::comic::ProcessedImage;

use super::ImageFormat;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum PngCompression {
    Fast,
    Default,
    Best,
}

impl PngCompression {
    pub fn cycle(&self) -> Self {
        match self {
            PngCompression::Fast => PngCompression::Default,
            PngCompression::Default => PngCompression::Best,
            PngCompression::Best => PngCompression::Fast,
        }
    }
}

/// Compress an image to JPEG format with the specified quality
pub fn compress_to_jpeg<W>(img: &DynamicImage, writer: &mut W, quality: u8) -> Result<()>
where
    W: std::io::Write,
{
    let mut encoder =
        imageproc::image::codecs::jpeg::JpegEncoder::new_with_quality(writer, quality);

    encoder
        .encode_image(img)
        .with_context(|| "Failed to compress image to JPEG")?;

    Ok(())
}

/// Compress an image to PNG format with the specified compression level
pub fn compress_to_png<W>(
    img: &DynamicImage,
    writer: &mut W,
    compression: PngCompression,
) -> Result<()>
where
    W: std::io::Write,
{
    use imageproc::image::codecs::png::{CompressionType, FilterType, PngEncoder};
    use imageproc::image::ImageEncoder;

    let compression_type = match compression {
        PngCompression::Fast => CompressionType::Fast,
        PngCompression::Default => CompressionType::Default,
        PngCompression::Best => CompressionType::Best,
    };

    let is_grayscale = img.color() == ColorType::L8 || img.color() == ColorType::La8;

    let encoder = PngEncoder::new_with_quality(
        writer,
        compression_type,
        if is_grayscale {
            FilterType::NoFilter
        } else {
            FilterType::Adaptive
        },
    );

    encoder
        .write_image(
            img.as_bytes(),
            img.width(),
            img.height(),
            img.color().into(),
        )
        .with_context(|| "Failed to compress image to PNG")?;

    Ok(())
}

/// Compress an image to WebP format with the specified quality
pub fn compress_to_webp(img: &DynamicImage, quality: u8) -> Result<WebPMemory> {
    let img = DynamicImage::from(img.to_rgb8());
    let encoder = webp::Encoder::from_image(&img)
        .map_err(|e| anyhow::anyhow!("Failed to create WebP encoder: {}", e))?;
    let webp_data = encoder.encode(quality as f32);
    Ok(webp_data)
}

pub fn encode_image_part(
    original: &ArchiveFile,
    img: &DynamicImage,
    part_num: usize,
    format: ImageFormat,
) -> ProcessedImage {
    let file_name = {
        let file = original.parent().display();
        let stem = original.file_stem().to_string_lossy();
        let extension = format.extension();
        format!("{file}_{stem}_{part_num:03}.{extension}")
    };

    let dimensions = img.dimensions();

    let img = ProcessedImage {
        file_name,
        data: encode_image(img, &format),
        dimensions,
        format,
    };

    log::trace!("Encoded image: {}", img.file_name);
    img
}

fn encode_image(img: &DynamicImage, format: &ImageFormat) -> Vec<u8> {
    let (width, height) = img.dimensions();
    let mut buffer = Vec::with_capacity((width * height) as usize);

    match format {
        ImageFormat::Jpeg { quality } => {
            compress_to_jpeg(img, &mut buffer, *quality).expect("Writing to vec should never fail");
        }
        ImageFormat::Png { compression } => {
            compress_to_png(img, &mut buffer, *compression)
                .expect("Writing to vec should never fail");
        }
        ImageFormat::WebP { quality } => {
            let webp_data =
                compress_to_webp(img, *quality).expect("Writing to vec should never fail");
            buffer.extend_from_slice(&webp_data);
        }
    }

    buffer
}
