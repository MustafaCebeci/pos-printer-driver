//! Serial (RS232) transport implementation for COM port printers.
//!
//! Default settings: 9600 baud, 8 data bits, 1 stop bit, No parity, No flow control (8N1).

use super::{PrinterError, PrinterTransport};
use std::sync::Mutex;
use std::io::Write;

/// Serial transport for RS232 printers.
pub struct SerialTransport {
    port: Mutex<Option<Box<dyn serialport::SerialPort>>>,
    #[allow(dead_code)]
    port_name: String,
}

/// List available serial ports on the system.
///
/// Returns a list of port names:
/// - Windows: `COM1`, `COM2`, etc.
/// - Linux: `/dev/ttyS0`, `/dev/ttyUSB0`, etc.
/// - macOS: `/dev/cu.usbserial-*`, `/dev/cu.usbmodem*`, etc.
pub fn list_serial_ports() -> Vec<String> {
    serialport::available_ports()
        .map(|ports| ports.into_iter().map(|p| p.port_name).collect())
        .unwrap_or_default()
}

/// Default serial settings for ESC/POS printers.
const DEFAULT_BAUD_RATE: u32 = 9600;
const DEFAULT_DATA_BITS: serialport::DataBits = serialport::DataBits::Eight;
const DEFAULT_STOP_BITS: serialport::StopBits = serialport::StopBits::One;
const DEFAULT_FLOW_CONTROL: serialport::FlowControl = serialport::FlowControl::None;
const DEFAULT_PARITY: serialport::Parity = serialport::Parity::None;

impl SerialTransport {
    /// Create a new serial transport on the specified port.
    pub fn new(path: &str, baud_rate: Option<u32>) -> Result<Self, PrinterError> {
        let baud = baud_rate.unwrap_or(DEFAULT_BAUD_RATE);

        serialport::new(path, baud)
            .data_bits(DEFAULT_DATA_BITS)
            .stop_bits(DEFAULT_STOP_BITS)
            .flow_control(DEFAULT_FLOW_CONTROL)
            .parity(DEFAULT_PARITY)
            .open()
            .map(|port| Self {
                port: Mutex::new(Some(port)),
                port_name: path.to_string(),
            })
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("No such file")
                    || msg.contains("not found")
                    || msg.contains("不存在")
                    || msg.contains("The system cannot find")
                {
                    PrinterError::DeviceNotFound(format!("Serial port not found: {}", path))
                } else if msg.contains("Permission denied")
                    || msg.contains("Access is denied")
                    || msg.contains("permission")
                {
                    PrinterError::SerialError(format!(
                        "Permission denied for {}: {}. \
                        On Linux, add user to 'dialout' group: sudo usermod -a -G dialout $USER",
                        path, e
                    ))
                } else {
                    PrinterError::SerialError(e.to_string())
                }
            })
    }
}

impl PrinterTransport for SerialTransport {
    fn write(&mut self, data: &[u8]) -> Result<(), PrinterError> {
        let mut guard = self.port.lock().unwrap();
        let port = guard.as_mut()
            .ok_or(PrinterError::NotConnected)?;

        port.write_all(data)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::BrokenPipe
                    || e.kind() == std::io::ErrorKind::ConnectionReset
                {
                    *guard = None;
                    PrinterError::ConnectionLost
                } else {
                    PrinterError::WriteError(e.to_string())
                }
            })
    }

    fn is_connected(&self) -> bool {
        self.port.lock().unwrap().is_some()
    }

    fn flush(&mut self) -> Result<(), PrinterError> {
        let mut guard = self.port.lock().unwrap();
        let port = guard.as_mut()
            .ok_or(PrinterError::NotConnected)?;

        port.flush()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::BrokenPipe
                    || e.kind() == std::io::ErrorKind::ConnectionReset
                {
                    *guard = None;
                    PrinterError::ConnectionLost
                } else {
                    PrinterError::WriteError(e.to_string())
                }
            })
    }

    fn transport_type(&self) -> &'static str {
        "Serial"
    }
}
