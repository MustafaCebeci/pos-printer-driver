//! # Image Processing for ESC/POS Printers
//!
//! Converts images to 1-bit BMP format for thermal printer printing.
//! Supports Floyd-Steinberg dithering for better quality.

use image::{DynamicImage, GenericImageView};

/// Maximum width for 80mm thermal paper at 203 DPI (576 pixels).
pub const MAX_WIDTH: u32 = 576;

/// Convert an image to 1-bit ESC/POS raster format.
///
/// 1. Load and resize image to max 576px width
/// 2. Convert to grayscale
/// 3. Apply Floyd-Steinberg dithering
/// 4. Pack into bytes (8 pixels per byte, MSB first)
pub fn image_to_raster(image_data: &[u8], format: ImageFormat) -> Result<Vec<u8>, ImageError> {
    // Decode image
    let img = load_image(image_data, format)?;

    // Resize to max width while maintaining aspect ratio
    let img = resize_to_width(img, MAX_WIDTH)?;

    // Convert to grayscale
    let grayscale = to_grayscale(&img);

    // Apply Floyd-Steinberg dithering
    let dithered = floyd_steinberg_dither(grayscale);

    // Pack into ESC/POS raster format
    let raster = pack_raster_bytes(&dithered);

    Ok(raster)
}

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    PNG,
    JPEG,
}

#[derive(Debug, thiserror::Error)]
pub enum ImageError {
    #[error("Failed to decode image: {0}")]
    DecodeError(String),
    #[error("Unsupported format")]
    UnsupportedFormat,
}

/// Load image from bytes.
fn load_image(data: &[u8], format: ImageFormat) -> Result<DynamicImage, ImageError> {
    match format {
        ImageFormat::PNG => {
            image::load_from_memory_with_format(data, image::ImageFormat::Png)
                .map_err(|e| ImageError::DecodeError(e.to_string()))
        }
        ImageFormat::JPEG => {
            image::load_from_memory_with_format(data, image::ImageFormat::Jpeg)
                .map_err(|e| ImageError::DecodeError(e.to_string()))
        }
    }
}

/// Resize image to specified width, maintaining aspect ratio.
fn resize_to_width(img: DynamicImage, max_width: u32) -> Result<DynamicImage, ImageError> {
    let (width, height) = img.dimensions();

    if width <= max_width {
        return Ok(img);
    }

    let ratio = max_width as f32 / width as f32;
    let new_height = (height as f32 * ratio) as u32;

    Ok(img.resize(max_width, new_height, image::imageops::FilterType::Lanczos3))
}

/// Convert image to grayscale luminance values.
fn to_grayscale(img: &DynamicImage) -> Vec<Vec<u8>> {
    let gray = img.to_luma8();
    let (width, height) = gray.dimensions();

    let mut pixels = Vec::with_capacity(height as usize);
    for y in 0..height {
        let mut row = Vec::with_capacity(width as usize);
        for x in 0..width {
            row.push(gray.get_pixel(x, y)[0]);
        }
        pixels.push(row);
    }
    pixels
}

/// Apply Floyd-Steinberg dithering to convert grayscale to 1-bit.
///
/// Uses the standard Floyd-Steinberg error diffusion algorithm.
fn floyd_steinberg_dither(grayscale: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let height = grayscale.len();
    if height == 0 {
        return vec![];
    }
    let width = grayscale[0].len();

    // Work with a copy to diffuse errors
    let mut pixels: Vec<Vec<i32>> = grayscale
        .iter()
        .map(|row| row.iter().map(|&p| p as i32).collect())
        .collect();

    for y in 0..height {
        for x in 0..width {
            let old_pixel = pixels[y][x];
            let new_pixel = if old_pixel > 127 { 255 } else { 0 };
            pixels[y][x] = new_pixel;

            let error = old_pixel - new_pixel;

            // Distribute error to neighbors
            if x + 1 < width {
                pixels[y][x + 1] += error * 7 / 16;
            }
            if y + 1 < height {
                if x > 0 {
                    pixels[y + 1][x - 1] += error * 3 / 16;
                }
                pixels[y + 1][x] += error * 5 / 16;
                if x + 1 < width {
                    pixels[y + 1][x + 1] += error / 16;
                }
            }
        }
    }

    // Convert back to u8
    pixels
        .iter()
        .map(|row| row.iter().map(|&p| p.clamp(0, 255) as u8).collect())
        .collect()
}

/// Pack 1-bit pixels into bytes for ESC/POS GS v 0 command.
///
/// Each byte contains 8 pixels, MSB first.
fn pack_raster_bytes(dithered: &[Vec<u8>]) -> Vec<u8> {
    let height = dithered.len();
    if height == 0 {
        return vec![];
    }
    let width = dithered[0].len();

    // Width in bytes (rounded up to byte boundary)
    let bytes_per_line = width.div_ceil(8);

    let mut result = Vec::with_capacity(bytes_per_line * height);

    for row in dithered {
        for byte_idx in 0..bytes_per_line {
            let mut byte = 0u8;
            for bit in 0..8 {
                let pixel_idx = byte_idx * 8 + bit;
                if pixel_idx < width {
                    // White (255) = 0, Black (0) = 1 in ESC/POS
                    if row[pixel_idx] < 128 {
                        byte |= 1 << (7 - bit);
                    }
                }
            }
            result.push(byte);
        }
    }

    result
}

/// Generate ESC/POS GS v 0 command for raster image.
///
/// Returns the full command including the image data.
pub fn encode_raster_image(raster_data: &[u8], width_pixels: u16, height_pixels: u16) -> Vec<u8> {
    let mut cmd = Vec::new();

    let bytes_per_line = (width_pixels as usize).div_ceil(8) as u8;

    // GS v 0 - Print raster bit image
    cmd.push(0x1D); // GS
    cmd.push(0x76); // v
    cmd.push(0x30); // 0
    cmd.push(0x00); // m = 0 (normal mode)

    // pL pH (bytes per line, low then high)
    cmd.push(bytes_per_line);
    cmd.push(0x00); // Assuming height < 256, simple case

    // xL xH yL yH (width and height, each as low/high bytes)
    let width_bytes = width_pixels.div_ceil(8);
    cmd.push((width_bytes & 0xFF) as u8);
    cmd.push(((width_bytes >> 8) & 0xFF) as u8);
    cmd.push((height_pixels & 0xFF) as u8);
    cmd.push(((height_pixels >> 8) & 0xFF) as u8);

    // Image data
    cmd.extend_from_slice(raster_data);

    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_raster_bytes_simple() {
        // 8 pixels = 1 byte
        // In thermal printer bitmaps: 0 = white, 1 = black
        // Luminance: 255 = white, 0 = black
        // After threshold at 128: 255 -> white (0), 0 -> black (1)
        let row = vec![255, 255, 255, 255, 255, 255, 255, 255]; // All white
        let packed = pack_raster_bytes(&[row]);
        assert_eq!(packed, vec![0x00]); // All white = all 0 bits

        let row = vec![0, 0, 0, 0, 0, 0, 0, 0]; // All black
        let packed = pack_raster_bytes(&[row]);
        assert_eq!(packed, vec![0xFF]); // All black = all 1 bits
    }

    #[test]
    fn test_pack_raster_bytes_mixed() {
        // Black, white, black, white...
        let row = vec![0, 255, 0, 255, 0, 255, 0, 255];
        let packed = pack_raster_bytes(&[row]);
        // MSB first: 1 0 1 0 1 0 1 0 = 0xAA
        assert_eq!(packed, vec![0xAA]);
    }

    #[test]
    fn test_pack_raster_bytes_partial_byte() {
        // Only 3 pixels in a 16-pixel wide image
        let packed = pack_raster_bytes(&[vec![0u8, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]]);
        // With 16 pixel width, we need 2 bytes
        assert_eq!(packed.len(), 2);
    }

    #[test]
    fn test_floyd_steinberg_basic() {
        // All white should remain white
        let input = vec![vec![255u8, 255, 255], vec![255u8, 255, 255]];
        let result = floyd_steinberg_dither(input);
        // All should be white (255)
        for row in &result {
            for &pixel in row {
                assert_eq!(pixel, 255);
            }
        }
    }

    #[test]
    fn test_encode_raster_image_structure() {
        let raster = vec![0xFF, 0x00, 0xAA, 0x55];
        let cmd = encode_raster_image(&raster, 16, 4);

        // Check header
        assert_eq!(cmd[0], 0x1D);
        assert_eq!(cmd[1], 0x76);
        assert_eq!(cmd[2], 0x30);
        assert_eq!(cmd[3], 0x00);
    }
}
