# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.0] - 2024

### Added

- **TCP Transport** — Connect to printers via TCP/IP on port 9100. Includes exponential backoff retry logic (100ms → 200ms → 400ms) and connection-drop detection.
- **Serial Transport** — Connect via RS-232 serial port (9600 baud 8N1 default). Supports baud rates 9600–115200. `listSerialPorts()` helper enumerates available ports.
- **USB Transport** — Connect via USB using the `nusb` Rust library. `listUsbPrinters()` scans for USB devices with Printer class (0x07) or known POS printer VID/PID pairs.
- **ESC/POS Builder** — Chained API for initializing the printer, setting alignment, bold, and font size.
- **Turkish Character Support** — Automatic CP857 and CP1254 transliteration for ş, ğ, ı, İ, ö, ü, ç characters.
- **Barcode Printing** — Code128, Code39, EAN-13, EAN-8, UPC-A, UPC-E, ITF, CODABAR barcode types.
- **QR Code Printing** — QR Code with Level M error correction.
- **Image Printing** — PNG and JPEG support with Floyd-Steinberg dithering, auto-resized to 576px width.
- **Cash Drawer Trigger** — Standard ESC/POS drawer kick-out pulse on pin 2/5.
- **Paper Cut** — Full and partial cut commands.
- **napi-rs Binding** — Full Node.js native addon with TypeScript type definitions (`index.d.ts`).
- **Cross-platform** — Windows (x64, arm64), Linux (x64, arm64), macOS (x64, arm64) via napi-rs prebuild.

### Error Codes

All transport errors expose a machine-readable code embedded in the error message:

| Code | Meaning |
|------|---------|
| `CONNECTION_FAILED` | Could not establish connection |
| `CONNECTION_REFUSED` | Printer rejected the connection |
| `CONNECTION_LOST` | Printer disconnected mid-operation |
| `WRITE_ERROR` | Failed to write to printer |
| `READ_ERROR` | Failed to read from printer |
| `NOT_CONNECTED` | Operation attempted without active connection |
| `USB_ERROR` | USB communication error |
| `SERIAL_ERROR` | Serial port error |
| `TIMEOUT` | Operation timed out |
| `DEVICE_NOT_FOUND` | Requested device does not exist |

### Platform Notes

- **TCP** is the recommended transport for production use.
- **Serial** requires `dialout` group membership on Linux.
- **USB on Windows** may require Zadig to replace `usbprint.sys` with WinUSB.
