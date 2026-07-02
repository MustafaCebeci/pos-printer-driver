//! # POS Printer NAPI Binding
//!
//! Node.js native addon for ESC/POS printer control.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::fs;
use pos_printer_protocol::{BarcodeType, QRErrorCorrection, ImageFormat};

/// Convert PrinterError to napi::Error with error code for JS-side discrimination.
fn to_napi_err(e: pos_printer_transport::PrinterError) -> napi::Error {
    use pos_printer_transport::PrinterError::*;
    let (code, msg) = match &e {
        ConnectionFailed(s) => ("CONNECTION_FAILED", s.clone()),
        ConnectionRefused => ("CONNECTION_REFUSED", e.to_string()),
        ConnectionLost => ("CONNECTION_LOST", e.to_string()),
        WriteError(s) => ("WRITE_ERROR", s.clone()),
        ReadError(s) => ("READ_ERROR", s.clone()),
        NotConnected => ("NOT_CONNECTED", e.to_string()),
        UsbError(s) => ("USB_ERROR", s.clone()),
        SerialError(s) => ("SERIAL_ERROR", s.clone()),
        Timeout => ("TIMEOUT", e.to_string()),
        DeviceNotFound(s) => ("DEVICE_NOT_FOUND", s.clone()),
    };
    // napi 3.x Error doesn't have with_code(), embed code in message
    napi::Error::new(Status::GenericFailure, format!("[{}] {}", code, msg))
}

/// Printer transport wrapper for Node.js binding.
#[napi]
pub struct Printer {
    transport: Box<dyn pos_printer_transport::PrinterTransport>,
}

#[napi]
impl Printer {
    /// Create a new TCP/IP printer connection.
    #[napi(factory)]
    pub fn connect_tcp(
        ip: String,
        port: u16,
        timeout_ms: Option<u32>,
        max_retries: Option<u32>,
    ) -> Result<Self> {
        let retry_config = max_retries.map(|attempts| pos_printer_transport::tcp::RetryConfig {
            max_attempts: attempts,
            ..Default::default()
        });

        let transport = pos_printer_transport::TcpTransport::new(&ip, port, timeout_ms, retry_config)
            .map_err(to_napi_err)?;

        Ok(Printer {
            transport: Box::new(transport),
        })
    }

    /// Create a new serial port printer connection.
    #[napi(factory)]
    pub fn connect_serial(path: String, baud_rate: Option<u32>) -> Result<Self> {
        let transport = pos_printer_transport::SerialTransport::new(&path, baud_rate)
            .map_err(to_napi_err)?;

        Ok(Printer {
            transport: Box::new(transport),
        })
    }

    /// Create a new USB printer connection.
    #[napi(factory)]
    pub fn connect_usb(vendor_id: Option<u16>, product_id: Option<u16>) -> Result<Self> {
        let transport = pos_printer_transport::UsbTransport::new(vendor_id, product_id)
            .map_err(to_napi_err)?;

        Ok(Printer {
            transport: Box::new(transport),
        })
    }

    /// Print raw text to the printer.
    #[napi]
    pub fn print_text(&mut self, text: String) -> Result<()> {
        let data = pos_printer_protocol::EscPosBuilder::new()
            .init()
            .text_line(&text)
            .build();

        self.transport
            .write(&data)
            .map_err(to_napi_err)
    }

    /// Cut the paper.
    #[napi]
    pub fn cut(&mut self, partial: Option<bool>) -> Result<()> {
        let cut_type = if partial.unwrap_or(false) {
            pos_printer_protocol::CutType::Partial
        } else {
            pos_printer_protocol::CutType::Full
        };

        let data = pos_printer_protocol::EscPosBuilder::new()
            .cut(cut_type)
            .build();

        self.transport
            .write(&data)
            .map_err(to_napi_err)
    }

    /// Open the cash drawer.
    #[napi]
    pub fn open_drawer(&mut self) -> Result<()> {
        let data = pos_printer_protocol::EscPosBuilder::new()
            .open_drawer()
            .build();

        self.transport
            .write(&data)
            .map_err(to_napi_err)
    }

    /// Check if printer is connected.
    #[napi]
    pub fn is_connected(&self) -> bool {
        self.transport.is_connected()
    }

    /// Disconnect from the printer.
    #[napi]
    pub fn disconnect(&mut self) -> Result<()> {
        self.transport
            .flush()
            .map_err(to_napi_err)
    }

    /// Print a barcode.
    #[napi]
    pub fn print_barcode(&mut self, barcode_type: String, data: String) -> Result<()> {
        let bt = match barcode_type.to_uppercase().as_str() {
            "CODE128" => BarcodeType::Code128,
            "CODE39" => BarcodeType::Code39,
            "EAN13" => BarcodeType::EAN13,
            "EAN8" => BarcodeType::EAN8,
            "UPC_A" | "UPCA" => BarcodeType::UpcA,
            "UPC_E" | "UPCE" => BarcodeType::UpcE,
            "ITF" => BarcodeType::ITF,
            "CODABAR" => BarcodeType::CODABAR,
            _ => return Err(napi::Error::new(
                Status::GenericFailure,
                format!("Unknown barcode type: {}", barcode_type),
            )),
        };
        let data = pos_printer_protocol::encode_barcode(bt, &data);
        self.transport.write(&data).map_err(to_napi_err)
    }

    /// Print a QR code.
    #[napi]
    pub fn print_qr(&mut self, data: String) -> Result<()> {
        let qr_data = pos_printer_protocol::encode_qr(&data, QRErrorCorrection::LevelM);
        self.transport.write(&qr_data).map_err(to_napi_err)
    }

    /// Print an image from a file path.
    #[napi]
    pub fn print_image(&mut self, path: String) -> Result<()> {
        let image_data = fs::read(&path).map_err(|e| {
            napi::Error::new(Status::GenericFailure, format!("Failed to read image file: {}", e))
        })?;
        let format = if path.to_lowercase().ends_with(".png") {
            ImageFormat::PNG
        } else if path.to_lowercase().ends_with(".jpg") || path.to_lowercase().ends_with(".jpeg") {
            ImageFormat::JPEG
        } else {
            return Err(napi::Error::new(
                Status::GenericFailure,
                "Unsupported image format: only PNG and JPEG are supported".to_string(),
            ));
        };
        let (width, height) = (576u16, 200u16); // Fixed height for simplicity
        let raster = pos_printer_protocol::image_to_raster(&image_data, format)
            .map_err(|e| napi::Error::new(Status::GenericFailure, e.to_string()))?;
        let cmd = pos_printer_protocol::encode_raster_image(&raster, width, height);
        self.transport.write(&cmd).map_err(to_napi_err)
    }

    /// Feed paper by n lines.
    #[napi]
    pub fn feed_lines(&mut self, lines: u8) -> Result<()> {
        let data = pos_printer_protocol::EscPosBuilder::new()
            .feed_lines(lines)
            .build();
        self.transport.write(&data).map_err(to_napi_err)
    }
}

/// List available serial ports.
#[napi]
pub fn list_serial_ports() -> Vec<String> {
    pos_printer_transport::list_serial_ports()
}

/// USB printer information.
#[napi]
pub struct UsbPrinterInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: String,
    pub product: String,
    pub serial_number: Option<String>,
}

impl From<pos_printer_transport::UsbPrinterInfo> for UsbPrinterInfo {
    fn from(info: pos_printer_transport::UsbPrinterInfo) -> Self {
        UsbPrinterInfo {
            vendor_id: info.vendor_id,
            product_id: info.product_id,
            manufacturer: info.manufacturer,
            product: info.product,
            serial_number: info.serial_number,
        }
    }
}

/// List available USB printers.
#[napi]
pub fn list_usb_printers() -> Vec<UsbPrinterInfo> {
    pos_printer_transport::list_usb_printers()
        .map(|v| v.into_iter().map(UsbPrinterInfo::from).collect())
        .unwrap_or_default()
}
