//! USB transport implementation using nusb (pure Rust, no libusb runtime dependency).
//!
//! # Cross-Platform Behavior
//! - **Linux/macOS**: Primary recommended transport. Works with direct USB access.
//! - **Windows**: Best-effort only. If `usbprint.sys` has claimed the interface,
//!   nusb will fail with `UsbInterfaceClaimed`. Use Zadig to install WinUSB driver
//!   or fall back to TCP/Serial.
//!
//! # Windows Driver Conflict Resolution
//! If `connect_usb()` returns `UsbInterfaceClaimed`:
//! 1. Download [Zadig](https://zadig.akeo.ie/)
//! 2. Find your printer in the device list
//! 3. Select `WinUSB` instead of `usbprint.sys`
//! 4. Click "Install WCID Driver"
//!
//! Alternatively, use TCP/IP connection (port 9100) which works reliably on all platforms.

use super::{PrinterError, PrinterTransport};
use nusb::transfer::{Bulk, Direction, Out};
use nusb::MaybeFuture;
use std::sync::Mutex;
use std::time::Duration;

/// USB printer information.
#[derive(Debug, Clone)]
pub struct UsbPrinterInfo {
    /// Vendor ID (e.g., 0x0483 for SUNLUX)
    pub vendor_id: u16,
    /// Product ID (e.g., 0x5720 for RP8020)
    pub product_id: u16,
    /// Manufacturer name
    pub manufacturer: String,
    /// Product name
    pub product: String,
    /// Serial number, if available
    pub serial_number: Option<String>,
}

/// USB transport for USB-connected ESC/POS printers.
pub struct UsbTransport {
    interface: Mutex<Option<nusb::Interface>>,
}

/// List USB printers available on the system.
///
/// Filters for devices with Printer Class (0x07) interface or known POS vendor/product IDs.
pub fn list_usb_printers() -> Result<Vec<UsbPrinterInfo>, PrinterError> {
    let devices = nusb::list_devices()
        .wait()
        .map_err(|e| PrinterError::UsbError(format!("Failed to enumerate USB devices: {}", e)))?;

    let printers: Vec<UsbPrinterInfo> = devices
        .filter_map(|dev| {
            let vid = dev.vendor_id();
            let pid = dev.product_id();

            // Filter: either Printer class (0x07) interface or known POS vendor/product combos
            let has_printer_interface = dev.interfaces().any(|i| i.class() == 0x07);
            let is_known_pos_device =
                KNOWN_POS_DEVICES
                    .iter()
                    .any(|(v, p)| *v == vid && *p == pid);

            if has_printer_interface || is_known_pos_device {
                Some(UsbPrinterInfo {
                    vendor_id: vid,
                    product_id: pid,
                    manufacturer: dev.manufacturer_string().map(String::from).unwrap_or_default(),
                    product: dev.product_string().map(String::from).unwrap_or_default(),
                    serial_number: dev.serial_number().map(String::from),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(printers)
}

/// Known POS printer vendor/product ID pairs.
///
/// Add your printer's VID/PID here when confirmed working.
const KNOWN_POS_DEVICES: &[(u16, u16)] = &[
    // SUNLUX
    (0x0483, 0x5720), // SUNLUX RP8020
    // Generic thermal printer class
    (0x0416, 0x5011), // Winbond-based printers
];

/// Find bulk OUT endpoint address by opening device temporarily.
fn find_bulk_out_addr(
    device_info: &nusb::DeviceInfo,
) -> Result<u8, PrinterError> {
    let device = device_info
        .open()
        .wait()
        .map_err(|e| PrinterError::UsbError(format!("Failed to open USB device: {}", e)))?;

    for config in device.configurations() {
        for intf in config.interfaces() {
            for alt in intf.alt_settings() {
                for ep in alt.endpoints() {
                    if ep.direction() == Direction::Out
                        && ep.transfer_type() == nusb::descriptors::TransferType::Bulk
                    {
                        return Ok(ep.address());
                    }
                }
            }
        }
    }

    Err(PrinterError::UsbError(
        "No bulk OUT endpoint found on printer".into(),
    ))
}

impl UsbTransport {
    /// Connect to a USB printer by vendor/product ID.
    ///
    /// If both `vendor_id` and `product_id` are `None`, connects to the first
    /// available USB printer device found.
    ///
    /// # Windows Note
    /// On Windows, if another driver (e.g., usbprint.sys) has claimed the
    /// printer interface, this returns `PrinterError::UsbError` with message
    /// containing "interface already claimed" or similar. Use Zadig to
    /// install WinUSB driver as described in the module doc.
    pub fn new(
        vendor_id: Option<u16>,
        product_id: Option<u16>,
    ) -> Result<Self, PrinterError> {
        let devices = nusb::list_devices()
            .wait()
            .map_err(|e| {
                PrinterError::UsbError(format!("Failed to enumerate USB devices: {}", e))
            })?;

        let matching: Vec<_> = devices
            .filter(|d| {
                let vid_match = vendor_id.is_none_or(|v| d.vendor_id() == v);
                let pid_match = product_id.is_none_or(|p| d.product_id() == p);
                // Filter by printer interface class (0x07) or explicit VID/PID match
                let has_printer_interface = d.interfaces().any(|i| i.class() == 0x07);
                vid_match
                    && pid_match
                    && (has_printer_interface || vendor_id.is_some() || product_id.is_some())
            })
            .collect();

        // Prefer Printer class devices if no specific VID/PID given
        let device_info = if vendor_id.is_none() && product_id.is_none() {
            let printer_class = matching.iter().find(|d| d.interfaces().any(|i| i.class() == 0x07));
            printer_class.cloned().or_else(|| matching.into_iter().next())
        } else {
            matching.into_iter().next()
        };

        let device_info = device_info.ok_or_else(|| {
            PrinterError::DeviceNotFound("No matching USB printer found".into())
        })?;

        // Find bulk OUT endpoint address before claiming interface
        let bulk_out_addr = find_bulk_out_addr(&device_info)?;

        // Open the device and claim interface 0
        let device = device_info
            .open()
            .wait()
            .map_err(|e| PrinterError::UsbError(format!("Failed to open USB device: {}", e)))?;

        let interface = match device.claim_interface(0).wait() {
            Ok(iface) => iface,
            Err(e) => {
                let msg = e.to_string();
                // Detect Windows usbprint.sys conflict
                if msg.contains("already claimed")
                    || msg.contains("interface claim")
                    || msg.contains("ACCESS")
                {
                    return Err(PrinterError::UsbError(format!(
                        "USB interface already claimed by another driver (usbprint.sys on Windows). \
                         Use Zadig (https://zadig.akeo.ie/) to install WinUSB driver for your printer. \
                         Original error: {}",
                        e
                    )));
                }
                return Err(PrinterError::UsbError(format!(
                    "Failed to claim USB interface: {}",
                    e
                )));
            }
        };

        // Store the bulk endpoint address in a static for reuse
        // SAFETY: Written exactly once during init, read-only thereafter.
        unsafe {
            BULK_OUT_ADDR = Some(bulk_out_addr);
        }

        Ok(UsbTransport {
            interface: Mutex::new(Some(interface)),
        })
    }
}

// SAFETY: Written once during `new()`, read-only after. Single-threaded access
// via Mutex ensures no data races.
static mut BULK_OUT_ADDR: Option<u8> = None;

impl PrinterTransport for UsbTransport {
    fn write(&mut self, data: &[u8]) -> Result<(), PrinterError> {
        let mut guard = self.interface.lock().unwrap();
        let interface = guard.as_mut().ok_or(PrinterError::NotConnected)?;

        // SAFETY: BULK_OUT_ADDR is set during `new()` before this is called.
        let addr = unsafe { BULK_OUT_ADDR }.ok_or_else(|| {
            PrinterError::UsbError("USB endpoint not initialized".into())
        })?;

        let mut ep: nusb::Endpoint<Bulk, Out> = interface
            .endpoint(addr)
            .map_err(|e| PrinterError::UsbError(format!("Invalid endpoint: {}", e)))?;

        let completion = ep.transfer_blocking(data.into(), Duration::from_secs(5));

        match completion.status {
            Ok(()) => Ok(()),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("BROKEN_PIPE") || msg.contains("CONNECTION_RESET") {
                    *guard = None;
                    Err(PrinterError::ConnectionLost)
                } else {
                    Err(PrinterError::WriteError(e.to_string()))
                }
            }
        }
    }

    fn is_connected(&self) -> bool {
        self.interface.lock().unwrap().is_some()
    }

    fn flush(&mut self) -> Result<(), PrinterError> {
        let guard = self.interface.lock().unwrap();
        if guard.is_none() {
            return Err(PrinterError::NotConnected);
        }
        Ok(())
    }

    fn transport_type(&self) -> &'static str {
        "USB"
    }
}
