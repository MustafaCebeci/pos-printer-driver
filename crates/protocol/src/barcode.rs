//! # Barcode Support for ESC/POS Printers
//!
//! Native barcode command support for various barcode types.

/// Supported barcode types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarcodeType {
    /// Code 128
    Code128,
    /// Code 39
    Code39,
    /// EAN-13
    EAN13,
    /// EAN-8
    EAN8,
    /// UPC-A
    UpcA,
    /// UPC-E
    UpcE,
    /// ITF
    ITF,
    /// CODABAR
    CODABAR,
}

impl BarcodeType {
    /// Get the ESC/POS barcode system code.
    pub fn system_code(&self) -> u8 {
        match self {
            BarcodeType::Code128 => 0x49, // 'I'
            BarcodeType::Code39 => 0x41,  // 'A'
            BarcodeType::EAN13 => 0x43,  // 'C'
            BarcodeType::EAN8 => 0x44,    // 'D'
            BarcodeType::UpcA => 0x41,   // 'A' (UPC-A uses same system as Code 39)
            BarcodeType::UpcE => 0x45,   // 'E'
            BarcodeType::ITF => 0x49,    // 'I'
            BarcodeType::CODABAR => 0x4B, // 'K'
        }
    }
}

/// Encode barcode data for ESC/POS printer.
///
/// Returns the barcode command bytes.
pub fn encode_barcode(barcode_type: BarcodeType, data: &str) -> Vec<u8> {
    let mut cmd = Vec::new();
    let system = barcode_type.system_code();

    // GS k - Barcode print command
    cmd.push(0x1D); // GS
    cmd.push(0x6B); // k
    cmd.push(system);
    cmd.push(data.len() as u8);
    cmd.extend_from_slice(data.as_bytes());

    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barcode_system_codes() {
        assert_eq!(BarcodeType::Code128.system_code(), 0x49);
        assert_eq!(BarcodeType::Code39.system_code(), 0x41);
        assert_eq!(BarcodeType::EAN13.system_code(), 0x43);
        assert_eq!(BarcodeType::EAN8.system_code(), 0x44);
        assert_eq!(BarcodeType::UpcA.system_code(), 0x41);
        assert_eq!(BarcodeType::UpcE.system_code(), 0x45);
        assert_eq!(BarcodeType::ITF.system_code(), 0x49);
        assert_eq!(BarcodeType::CODABAR.system_code(), 0x4B);
    }

    #[test]
    fn test_encode_barcode() {
        let result = encode_barcode(BarcodeType::Code128, "1234567890");
        // Expected: 1D 6B 49 0A 31 32 33 34 35 36 37 38 39 30
        assert_eq!(result[0], 0x1D);
        assert_eq!(result[1], 0x6B);
        assert_eq!(result[2], 0x49);
        assert_eq!(result[3], 10); // length
        assert_eq!(&result[4..], "1234567890".as_bytes());
    }
}
