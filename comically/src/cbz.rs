use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use std::io::Cursor;

use crate::comic::ProcessedImage;

/// Build CBZ and return the bytes
pub fn build(images: &[ProcessedImage]) -> Vec<u8> {
    let cap = images.len() * images.first().map(|i| i.data.len()).unwrap_or(1);
    let mut buffer = Vec::with_capacity(cap);
    build_into(images, &mut buffer);
    buffer
}

/// Build CBZ into the provided buffer, reusing existing allocation
pub fn build_into(images: &[ProcessedImage], buffer: &mut Vec<u8>) {
    buffer.clear();
    let cursor = Cursor::new(buffer);
    let mut zip = ZipWriter::new(cursor);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add images in order
    for image in images.iter() {
        zip.start_file(&image.file_name, options).unwrap();
        std::io::Write::write_all(&mut zip, &image.data).unwrap();
    }

    // TODO: Add ComicInfo.xml if we have metadata

    zip.finish().unwrap();
}
