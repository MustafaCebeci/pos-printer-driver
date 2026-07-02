//! # POS Printer Transport Layer
//!
//! Trait-based abstraction for printer communication channels.
//! Supports TCP, Serial, and USB transports.

use thiserror::Error;

/// Errors that can occur during printer communication.
#[derive(Debug, Error)]
pub enum PrinterError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Connection refused — is the printer powered on and connected?")]
    ConnectionRefused,

    #[error("Connection lost — printer disconnected")]
    ConnectionLost,

    #[error("Write error: {0}")]
    WriteError(String),

    #[error("Read error: {0}")]
    ReadError(String),

    #[error("Not connected")]
    NotConnected,

    #[error("USB error: {0} (Windows'ta driver çakışması olabilir, TCP önerilir)")]
    UsbError(String),

    #[error("Serial error: {0}")]
    SerialError(String),

    #[error("Timeout")]
    Timeout,

    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}

/// Common trait for all printer transport implementations.
///
/// Implementors must be Send-safe since printer operations
/// may be dispatched from different threads.
pub trait PrinterTransport: Send {
    /// Write data to the printer.
    fn write(&mut self, data: &[u8]) -> Result<(), PrinterError>;

    /// Check if the transport is currently connected.
    fn is_connected(&self) -> bool;

    /// Flush any pending writes.
    fn flush(&mut self) -> Result<(), PrinterError>;

    /// Get the transport type name for debugging.
    fn transport_type(&self) -> &'static str;
}

// Re-export transport implementations
pub mod tcp;
mod serial;
mod usb;

pub use tcp::{TcpTransport, RetryConfig};
pub use serial::{SerialTransport, list_serial_ports};
pub use usb::{UsbTransport, list_usb_printers, UsbPrinterInfo};
