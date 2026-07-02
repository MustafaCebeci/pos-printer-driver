//! # QR Code Support for ESC/POS Printers
//!
//! QR code generation using ESC/POS native commands.

/// QR code error correction levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QRErrorCorrection {
    /// Level L - 7% recovery
    LevelL,
    /// Level M - 15% recovery
    LevelM,
    /// Level Q - 25% recovery
    LevelQ,
    /// Level H - 30% recovery
    LevelH,
}

impl QRErrorCorrection {
    fn to_byte(self) -> u8 {
        match self {
            QRErrorCorrection::LevelL => 0x31, // '1'
            QRErrorCorrection::LevelM => 0x32, // '2'
            QRErrorCorrection::LevelQ => 0x33, // '3'
            QRErrorCorrection::LevelH => 0x34, // '4'
        }
    }
}

/// QR code model types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QRModel {
    /// Model 1
    Model1,
    /// Model 2 (recommended)
    Model2,
}

impl QRModel {
    fn to_byte(self) -> u8 {
        match self {
            QRModel::Model1 => 0x31, // '1'
            QRModel::Model2 => 0x32, // '2'
        }
    }
}

/// QR code module size (1-16).
const DEFAULT_MODULE_SIZE: u8 = 6;

/// Encode QR code data for ESC/POS printer.
///
/// Uses ESC/POS native QR commands for the printer to generate the QR code.
pub fn encode_qr(data: &str, correction: QRErrorCorrection) -> Vec<u8> {
    let data_len = data.len() as u16;

    let mut cmd = vec![
        // QR Code: Select model (GS ( k pL pH cn 49)
        0x1D, 0x28, 0x6B, 0x04, 0x00, 0x31, 0x50, 0x30,
        QRModel::Model2.to_byte(),
        // Set module size (GS ( k pL pH cn 49 n)
        0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x51, 0x30,
        DEFAULT_MODULE_SIZE,
        // Set error correction (GS ( k pL pH cn 50 n)
        0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x52, 0x30,
        correction.to_byte(),
        // Store data in symbol area (GS ( k pL pH cn 51 n [data])
        0x1D, 0x28, 0x6B,
        (data_len & 0xFF) as u8,
        ((data_len >> 8) & 0xFF) as u8,
        0x31, 0x53, 0x30,
    ];
    cmd.extend_from_slice(data.as_bytes());

    // Print (GS ( k pL pH cn 60)
    cmd.extend_from_slice(&[
        0x1D, 0x28, 0x6B, 0x03, 0x00, 0x31, 0x54, 0x30,
    ]);

    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_error_correction_bytes() {
        assert_eq!(QRErrorCorrection::LevelL.to_byte(), 0x31);
        assert_eq!(QRErrorCorrection::LevelM.to_byte(), 0x32);
        assert_eq!(QRErrorCorrection::LevelQ.to_byte(), 0x33);
        assert_eq!(QRErrorCorrection::LevelH.to_byte(), 0x34);
    }

    #[test]
    fn test_qr_model_bytes() {
        assert_eq!(QRModel::Model1.to_byte(), 0x31);
        assert_eq!(QRModel::Model2.to_byte(), 0x32);
    }

    #[test]
    fn test_encode_qr_basic() {
        let result = encode_qr("TEST", QRErrorCorrection::LevelM);
        // Should start with model selection command: 1D 28 6B 04 00 31 50 30
        assert_eq!(result[0], 0x1D);
        assert_eq!(result[1], 0x28);
        assert_eq!(result[2], 0x6B);
        assert_eq!(result[3], 0x04);
        assert_eq!(result[4], 0x00);
        assert_eq!(result[5], 0x31);
        assert_eq!(result[6], 0x50);
        assert_eq!(result[7], 0x30);
    }
}
