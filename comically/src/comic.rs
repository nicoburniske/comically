use std::path::Path;
use std::{fs, path::PathBuf};

use crate::device::Device;
use crate::image::ImageFormat;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SplitStrategy {
    None,
    Split,
    Rotate,
    RotateAndSplit,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum OutputFormat {
    Mobi,
    Epub,
    Cbz,
}

impl OutputFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            OutputFormat::Mobi => "mobi",
            OutputFormat::Epub => "epub",
            OutputFormat::Cbz => "cbz",
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComicConfig {
    pub device: Device,
    pub right_to_left: bool,
    pub split: SplitStrategy,
    pub auto_crop: bool,
    pub brightness: i32,
    // Gamma correction: 0.0-3.0
    pub gamma: f32,
    pub output_format: OutputFormat,
    pub margin_color: Option<u8>,
    pub image_format: ImageFormat,
}

impl Default for ComicConfig {
    fn default() -> Self {
        Self {
            device: crate::device::Preset::KindlePw11.into(),
            right_to_left: true,
            split: SplitStrategy::RotateAndSplit,
            auto_crop: true,
            brightness: -10,
            gamma: 1.8,
            output_format: OutputFormat::Mobi,
            margin_color: None,
            image_format: ImageFormat::Jpeg { quality: 85 },
        }
    }
}

impl ComicConfig {
    pub fn load() -> Option<Self> {
        let config_path = Self::config_path()?;

        fs::read_to_string(&config_path)
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
    }

    pub fn save(&self) -> Option<()> {
        let config_path = Self::config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).ok()?;
        }

        serde_json::to_string_pretty(self)
            .ok()
            .and_then(|json| fs::write(&config_path, json).ok())
    }

    fn config_path() -> Option<PathBuf> {
        let home = std::env::home_dir()?;
        Some(home.join(".config").join("comically").join("config.json"))
    }

    pub fn device_dimensions(&self) -> (u32, u32) {
        self.device.dimensions()
    }
}

#[derive(Debug, Clone)]
pub struct ProcessedImage {
    pub file_name: String,
    pub data: Vec<u8>,
    pub dimensions: (u32, u32),
    pub format: ImageFormat,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ArchiveExt {
    Cbz,
    Cbr,
    Zip,
    Rar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComicFile {
    title: String,
    input: PathBuf,
    ext: ArchiveExt,
}

impl ComicFile {
    pub fn new(file: PathBuf) -> Self {
        let title = file
            .file_stem()
            .expect("Comic file should be a file, not a directory")
            .to_string_lossy()
            .to_string();

        assert!(file.extension().is_some());
        let ext = file.extension().and_then(|ext| ext.to_str()).unwrap();
        let ext = match ext {
            "cbz" => ArchiveExt::Cbz,
            "cbr" => ArchiveExt::Cbr,
            "zip" => ArchiveExt::Zip,
            "rar" => ArchiveExt::Rar,
            _ => panic!("Unsupported archive format: {}", ext),
        };

        ComicFile {
            title,
            input: file,
            ext,
        }
    }

    pub fn as_path(&self) -> &Path {
        &self.input
    }

    pub fn extension(&self) -> ArchiveExt {
        self.ext
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn with_extension(&self, extension: OutputFormat) -> PathBuf {
        // don't use .with_extension() bc it replaces everything after the first dot
        let mut filename = self.title.clone();
        filename.push('.');
        filename.push_str(extension.as_str());
        filename.into()
    }
}

#[test]
fn output_path_with_dots() {
    let comic = ComicFile::new(PathBuf::from("Dr. STONE v01 (2018) (Digital) (1r0n).cbz"));

    assert_eq!(
        comic.with_extension(OutputFormat::Epub),
        PathBuf::from("Dr. STONE v01 (2018) (Digital) (1r0n).epub")
    );
}
