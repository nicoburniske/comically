# comically-cli

Command-line tool for converting comic archives (CBZ/CBR) to e-reader formats.

## Installation

```bash
cargo install --path .
```

## Usage

```bash
comically-cli [OPTIONS] <INPUT>
```

### Basic Examples

```bash
# Convert to CBZ (default)
comically-cli comic.cbr

# Convert to EPUB
comically-cli comic.cbz --format epub

# Convert to MOBI (requires KindleGen)
comically-cli comic.cbz --format mobi

# Specify output directory
comically-cli comic.cbz -o ~/converted
```

### Advanced Examples

```bash
# Manga mode with Kindle Paperwhite preset
comically-cli manga.cbr --rtl --device kindle-paperwhite --format epub

# Custom device dimensions
comically-cli comic.cbz --device custom --width 1264 --height 1680

# High quality JPEG output
comically-cli comic.cbz --image-format jpeg --quality 95

# PNG with best compression
comically-cli comic.cbz --image-format png --png-compression best

# Adjust brightness and gamma
comically-cli dark-comic.cbz --brightness 20 --gamma 1.2

# Rotate double-page spreads
comically-cli comic.cbz --split rotate

# Full customization
comically-cli comic.cbz \
  --format epub \
  --device kindle-scribe \
  --image-format jpeg \
  --quality 90 \
  --brightness 10 \
  --gamma 1.1 \
  --split split \
  --margin-color black \
  --rtl \
  -o output/
```

## Options

### Required Arguments

- `<INPUT>` - Path to input comic file (CBZ or CBR)

### Output Options

- `-o, --output-dir <DIR>` - Output directory (default: current directory)
- `-f, --format <FORMAT>` - Output format: `cbz`, `epub`, `mobi` (default: `cbz`)

### Device Presets

- `-d, --device <DEVICE>` - Device preset (default: `kindle-paperwhite`)
  - `kindle-paperwhite` - 1236x1648
  - `kindle-scribe` - 1860x2480
  - `kobo-libra` - 1264x1680
  - `kobo-clara` - 1072x1448
  - `remarkable-paper` - 1404x1872
  - `custom` - Requires `--width` and `--height`

- `--width <PIXELS>` - Custom device width (requires `--device custom`)
- `--height <PIXELS>` - Custom device height (requires `--device custom`)

### Image Processing

- `--image-format <FORMAT>` - Image format: `jpeg`, `png`, `webp` (default: `jpeg`)
- `--quality <QUALITY>` - JPEG/WebP quality 0-100 (default: `85`)
- `--png-compression <LEVEL>` - PNG compression: `fast`, `default`, `best`
- `--brightness <VALUE>` - Brightness adjustment -100 to +100 (default: `0`)
- `--gamma <VALUE>` - Gamma correction 0.1 to 3.0 (default: `1.0`)
- `--margin-color <COLOR>` - Margin color: `none`, `black`, `white` (default: `black`)

### Page Handling

- `--split <STRATEGY>` - Split strategy (default: `split`)
  - `none` - Keep double-page spreads as-is
  - `split` - Split double-page spreads into separate pages
  - `rotate` - Rotate double-page spreads 90° for vertical viewing
  - `rotate-split` - Show both rotated and split versions

- `--rtl` - Right-to-left reading direction (manga mode)
- `--no-auto-crop` - Disable automatic cropping of margins

### Logging

- `-v, --verbose` - Verbose output (shows debug information)
- `-q, --quiet` - Quiet mode (minimal output, only shows result path)

## Output Formats

### CBZ (Comic Book Archive)
- Standard ZIP archive with images
- Compatible with most comic readers
- Fast to create

### EPUB
- Standard e-book format
- Works on most e-readers
- Supports metadata and navigation

### MOBI
- Amazon Kindle format
- Requires KindleGen to be installed
- Creates EPUB first, then converts to MOBI

## Notes

- **MOBI format**: Automatically uses JPEG images regardless of `--image-format` setting
- **KindleGen**: Required for MOBI output. Download from Amazon's website
- **Memory usage**: Processes all images in memory (~40MB for 200-page manga)
- **Performance**: Uses parallel processing for image operations

## Exit Codes

- `0` - Success
- `1` - Error (check error message for details)

## Examples by Use Case

### For Kindle Paperwhite
```bash
comically-cli comic.cbz --device kindle-paperwhite --format epub
```

### For Manga
```bash
comically-cli manga.cbr --rtl --split rotate --device kindle-paperwhite
```

### For High-Quality Archive
```bash
comically-cli comic.cbz --image-format png --png-compression best
```

### For Small File Size
```bash
comically-cli comic.cbz --image-format jpeg --quality 70
```

### Batch Processing (with shell)
```bash
for file in *.cbz; do
  comically-cli "$file" --format epub -o converted/
done
```
