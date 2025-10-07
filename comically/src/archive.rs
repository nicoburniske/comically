use anyhow::Context;
use unrar::Archive;
use zip::{HasZipMetadata, ZipArchive};

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use crate::comic::ArchiveExt;
use crate::ComicFile;

#[derive(Debug, Clone)]
pub struct ArchiveFile {
    // fully qualified path in the archive
    pub file_name: PathBuf,
    pub data: Vec<u8>,
}

impl ArchiveFile {
    pub fn file_stem(&self) -> &std::ffi::OsStr {
        self.file_name.file_stem().unwrap()
    }
    pub fn parent(&self) -> &Path {
        self.file_name.parent().unwrap()
    }
}

pub enum ArchiveIter {
    Zip(ZipReader),
    Rar(RarReader),
}

impl ArchiveIter {
    pub fn num_images(&self) -> usize {
        match self {
            ArchiveIter::Zip(reader) => reader
                .archive
                .file_names()
                .filter(|name| validate_file(name).is_some())
                .count(),
            ArchiveIter::Rar(reader) => reader.files.len(),
        }
    }
}

impl Iterator for ArchiveIter {
    type Item = anyhow::Result<ArchiveFile>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ArchiveIter::Zip(reader) => reader.next(),
            ArchiveIter::Rar(reader) => reader.next(),
        }
    }
}

pub fn unarchive_comic_iter(comic_file: &ComicFile) -> anyhow::Result<ArchiveIter> {
    let reader = match comic_file.extension() {
        ArchiveExt::Cbz | ArchiveExt::Zip => {
            let file = File::open(comic_file.as_path()).context("Failed to open zip file")?;
            ArchiveIter::Zip(ZipReader::new(file)?)
        }
        ArchiveExt::Cbr | ArchiveExt::Rar => {
            ArchiveIter::Rar(RarReader::new(comic_file.as_path())?)
        }
    };

    Ok(reader)
}

pub struct ZipReader {
    index: usize,
    archive: ZipArchive<BufReader<File>>,
}

impl ZipReader {
    fn new(file: File) -> anyhow::Result<Self> {
        let reader = BufReader::new(file);
        let archive = ZipArchive::new(reader).context("Failed to parse file as zip archive")?;
        Ok(Self { index: 0, archive })
    }
}

impl Iterator for ZipReader {
    type Item = anyhow::Result<ArchiveFile>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.archive.len() {
            let current_index = self.index;
            self.index += 1;

            let mut file = match self.archive.by_index(current_index) {
                Ok(f) => f,
                Err(e) => return Some(Err(e.into())),
            };

            if file.is_dir() {
                continue;
            }

            let outpath = match file.enclosed_name() {
                Some(path) => path.to_owned(),
                None => continue,
            };

            let file_name = match validate_file(&outpath) {
                Some(name) => name,
                None => continue,
            };

            let mut data = Vec::with_capacity(file.get_metadata().uncompressed_size as usize);
            if let Err(e) = Read::read_to_end(&mut file, &mut data) {
                return Some(Err(e.into()));
            }

            return Some(Ok(ArchiveFile { file_name, data }));
        }

        None
    }
}

pub struct RarReader {
    archive: Option<unrar::OpenArchive<unrar::Process, unrar::CursorBeforeHeader>>,
    files: Vec<unrar::FileHeader>,
    finished: bool,
}

// whoops
unsafe impl Send for RarReader {}

impl RarReader {
    fn new(path: &Path) -> anyhow::Result<Self> {
        let files: Vec<unrar::FileHeader> = Archive::new(path)
            .open_for_listing()
            .context("Failed to open RAR file")?
            .filter_map(|header| header.ok())
            .filter(|header| !header.is_directory())
            .filter(|header| validate_file(&header.filename).is_some())
            .collect();

        let archive = Archive::new(path)
            .open_for_processing()
            .context("Failed to open RAR file")?;

        Ok(Self {
            archive: Some(archive),
            files,
            finished: false,
        })
    }
}

impl Iterator for RarReader {
    type Item = anyhow::Result<ArchiveFile>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let archive = match self.archive.take() {
            Some(archive) => archive,
            None => {
                self.finished = true;
                return None;
            }
        };

        match archive.read_header() {
            Ok(Some(header)) => {
                let file_path = Path::new(&header.entry().filename);

                if header.entry().is_directory() {
                    let Ok(archive) = header.skip() else {
                        return None;
                    };
                    self.archive = Some(archive);
                    return self.next();
                }

                let Some(file_name) = validate_file(file_path) else {
                    let Ok(archive) = header.skip() else {
                        return None;
                    };
                    self.archive = Some(archive);
                    return self.next();
                };

                let Ok((data, new_archive)) = header.read() else {
                    return None;
                };
                self.archive = Some(new_archive);

                Some(Ok(ArchiveFile { file_name, data }))
            }
            _ => {
                self.finished = true;
                None
            }
        }
    }
}

fn validate_file(path: impl AsRef<Path>) -> Option<PathBuf> {
    let path = path.as_ref();
    let file_name = path.file_name()?;
    let file_name = file_name.to_string_lossy();
    if should_skip_file(&file_name) || !has_image_extension(path) {
        return None;
    }
    Some(path.to_path_buf())
}

fn should_skip_file(file_name: &str) -> bool {
    file_name.starts_with(".")
        || file_name.contains("__MACOSX")
        || file_name.contains("thumbs.db")
        || file_name.contains(".DS_Store")
}

fn has_image_extension(path: &Path) -> bool {
    static VALID_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png"];
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        for valid_ext in VALID_EXTENSIONS {
            if valid_ext == &ext_str {
                return true;
            }
        }
    }
    false
}

#[ignore]
#[test]
fn test_unarchive_comic_iter() {
    let files = unarchive_comic_iter(&ComicFile::new(std::path::PathBuf::from("v12.cbz")))
        .unwrap()
        .collect::<Vec<_>>();
    println!("{:?}", files.len());
}
