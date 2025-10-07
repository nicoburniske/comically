//! Image processing pipeline for manga/comic optimization

pub mod decode;
pub mod encode;
pub mod transform;

// Re-export public API
use anyhow::Result;
use arrayvec::ArrayVec;
pub use encode::{compress_to_jpeg, compress_to_png, compress_to_webp, PngCompression};
use imageproc::image::DynamicImage;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::archive::ArchiveFile;
use crate::comic::{ComicConfig, ProcessedImage};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ImageFormat {
    Jpeg { quality: u8 },
    Png { compression: PngCompression },
    WebP { quality: u8 },
}

impl ImageFormat {
    pub fn cycle(&self) -> Self {
        match self {
            ImageFormat::Jpeg { .. } => ImageFormat::Png {
                compression: PngCompression::Default,
            },
            ImageFormat::Png { .. } => ImageFormat::WebP { quality: 85 },
            ImageFormat::WebP { .. } => ImageFormat::Jpeg { quality: 85 },
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg { .. } => "jpg",
            ImageFormat::Png { .. } => "png",
            ImageFormat::WebP { .. } => "webp",
        }
    }

    pub fn adjust_quality(&mut self, increase: bool, fine: bool) {
        let step = if fine { 1 } else { 5 };
        match self {
            ImageFormat::Jpeg { quality } | ImageFormat::WebP { quality } => {
                if increase {
                    *quality = (*quality + step).min(100);
                } else {
                    *quality = quality.saturating_sub(step);
                }
            }
            ImageFormat::Png { compression } => {
                *compression = if increase {
                    match compression {
                        PngCompression::Fast => PngCompression::Default,
                        PngCompression::Default => PngCompression::Best,
                        PngCompression::Best => PngCompression::Best,
                    }
                } else {
                    match compression {
                        PngCompression::Fast => PngCompression::Fast,
                        PngCompression::Default => PngCompression::Fast,
                        PngCompression::Best => PngCompression::Default,
                    }
                };
            }
        }
    }
}

/// Stack-allocated container for 1-3 images (no heap allocation)
pub struct Split<T>(ArrayVec<T, 3>);

impl<T> Split<T> {
    #[inline(always)]
    pub fn one(t: T) -> Self {
        let mut vec = ArrayVec::new();
        vec.push(t);
        Split(vec)
    }

    #[inline(always)]
    pub fn two(t1: T, t2: T) -> Self {
        let mut vec = ArrayVec::new();
        vec.push(t1);
        vec.push(t2);
        Split(vec)
    }

    #[inline(always)]
    pub fn three(t1: T, t2: T, t3: T) -> Self {
        Split(ArrayVec::from([t1, t2, t3]))
    }

    #[inline(always)]
    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> Split<U> {
        Split(self.0.into_iter().map(f).collect())
    }
}

impl<T> IntoIterator for Split<T> {
    type Item = T;
    type IntoIter = <ArrayVec<T, 3> as IntoIterator>::IntoIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[inline(always)]
pub fn process_batch(files: Vec<ArchiveFile>, config: &ComicConfig) -> Result<Vec<ProcessedImage>> {
    process_batch_with_progress(files, config, || {})
}

pub fn process_batch_with_progress<F>(
    files: Vec<ArchiveFile>,
    config: &ComicConfig,
    on_progress: F,
) -> Result<Vec<ProcessedImage>>
where
    F: Fn() + Send + Sync,
{
    log::info!("Processing {} archive images", files.len());

    // Parallel stage: decode + process + encode
    // This eliminates intermediate Vec allocation and keeps data hot in cache
    let mut images: Vec<ProcessedImage> = files
        .par_iter()
        .map(|archive_file| {
            // Decode image
            let img = decode::decode(&archive_file.data)?;

            // Process image (transform, crop, resize, split)
            let processed_images = process(img, config);

            let mut encoded_images = ArrayVec::<ProcessedImage, 3>::new();

            // Encode immediately while data is hot in cache
            for (i, img) in processed_images.into_iter().enumerate() {
                let processed =
                    encode::encode_image_part(archive_file, &img, i, config.image_format);
                encoded_images.push(processed);
            }

            // Report progress after processing this file
            on_progress();

            Ok(encoded_images)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();

    // Serial sort + dedup (fast, no benefit from parallelism)
    images.sort_unstable_by(|a, b| a.file_name.cmp(&b.file_name));
    images.dedup_by(|a, b| a.file_name == b.file_name);

    Ok(images)
}

/// Process a single image file with Kindle-optimized transformations
pub fn process(img: DynamicImage, config: &ComicConfig) -> Split<DynamicImage> {
    let img = transform::Image::from(img.into_luma8())
        .gamma(config.gamma)
        .autocontrast()
        .brightness(config.brightness);

    if config.auto_crop {
        transform::split_rotate(img.auto_crop(), config)
    } else {
        transform::split_rotate(img, config)
    }
    .map(|img| DynamicImage::ImageLuma8(img.into()))
}
