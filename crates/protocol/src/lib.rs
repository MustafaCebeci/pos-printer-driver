//! # POS Printer Protocol Layer
//!
//! ESC/POS command builder and encoder for thermal receipt printers.
//! Handles text encoding, barcode, QR code, and image formatting.

pub mod builder;
pub mod codepage;
pub mod barcode;
pub mod qr;
pub mod image;

pub use builder::{EscPosBuilder, Align, FontSize, CutType, CodePage};
pub use barcode::BarcodeType;
pub use qr::QRErrorCorrection;
pub use image::ImageFormat;

pub use barcode::encode_barcode;
pub use qr::encode_qr;
pub use image::{image_to_raster, encode_raster_image};
