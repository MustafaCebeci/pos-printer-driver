# @berbervakti/pos-printer-driver

Cross-platform native driver for SUNLUX RP8020 (POS-80C) and other ESC/POS thermal receipt printers. Provides TCP, Serial (RS-232), and USB transport layers connected to Node.js via [napi-rs](https://napi.rs/).

**Tested on:** Windows 11 (x64)  
**Prebuilt binary — no Rust or build tools required**  
**Requires:** Node.js ≥ 10

---

## Features

- **Three transport layers**: TCP (recommended), Serial RS-232, USB
- **ESC/POS command builder**: chained API for text, alignment, bold, font size
- **Turkish character support**: automatic CP857/CP1254 transliteration for Turkish letters (ş, ğ, ı, İ, ö, ü, ç)
- **Barcodes**: Code128, Code39, EAN-13, EAN-8, UPC-A, UPC-E, ITF, CODABAR
- **QR codes**: QR Code with adjustable error correction
- **Image printing**: PNG/JPEG → raster bitmaps with Floyd-Steinberg dithering (576px width)
- **Cash drawer trigger**: standard ESC/POS drawer kick-out pulse
- **Auto cut**: full and partial paper cut

---

## Installation

```bash
npm install @berbervakti/pos-printer-driver
```

---

## Quick Start

### TCP (recommended for production)

```javascript
const { Printer } = require('pos-printer-driver');

async function main() {
  // TCP transport — connect to printer on port 9100
  const printer = await Printer.connect_tcp('192.168.1.100', 9100, 5000, 3);

  console.log('Connected:', printer.is_connected());

  await printer.print_text('MERHABA DÜNYA');
  await printer.feed_lines(3);
  await printer.cut();

  await printer.disconnect();
  console.log('Done!');
}

main().catch(console.error);
```

### Serial (RS-232)

```javascript
const { Printer, listSerialPorts } = require('pos-printer-driver');

// List available COM ports
const ports = listSerialPorts();
console.log('Available ports:', ports);

// Connect to first available port at 9600 baud (default)
const printer = await Printer.connect_serial('COM3');
await printer.print_text('MERHABA DÜNYA');
await printer.cut();
await printer.disconnect();
```

### USB

```javascript
const { Printer, listUsbPrinters } = require('pos-printer-driver');

// List USB printers
const printers = listUsbPrinters();
console.log('USB printers:', printers);

// Connect to first found USB printer
const printer = await Printer.connect_usb();
await printer.print_text('MERHABA DÜNYA');
await printer.cut();
await printer.disconnect();
```

---

## API Reference

### Factory Methods

| Method | Parameters | Returns | Description |
|--------|-----------|---------|-------------|
| `Printer.connect_tcp(ip, port, timeoutMs?, maxRetries?)` | `ip: string`, `port: number`, `timeoutMs?: number`, `maxRetries?: number` | `Printer` | Connect via TCP/IP. Default timeout: 5000ms, retries: 3 |
| `Printer.connect_serial(path, baudRate?)` | `path: string`, `baudRate?: number` | `Printer` | Connect via serial port. Default: 9600 baud, 8N1 |
| `Printer.connect_usb(vendorId?, productId?)` | `vendorId?: number`, `productId?: number` | `Printer` | Connect via USB. Scans all USB printers if no IDs given |

### Instance Methods

| Method | Parameters | Returns | Description |
|--------|-----------|---------|-------------|
| `print_text(text)` | `text: string` | `void` | Print a text line. Turkish characters are transliterated automatically |
| `cut(partial?)` | `partial?: boolean` | `void` | Cut the paper. `partial=true` for partial cut |
| `open_drawer()` | — | `void` | Trigger the cash drawer |
| `is_connected()` | — | `boolean` | Check if printer is connected |
| `disconnect()` | — | `void` | Flush and disconnect |
| `print_barcode(type, data)` | `type: string`, `data: string` | `void` | Print barcode. See supported types below |
| `print_qr(data)` | `data: string` | `void` | Print QR code |
| `print_image(path)` | `path: string` | `void` | Print PNG/JPEG image (max 576px wide) |
| `feed_lines(lines)` | `lines: number` | `void` | Feed paper by `n` lines |

### Standalone Functions

| Function | Returns | Description |
|----------|---------|-------------|
| `listSerialPorts()` | `string[]` | List available serial port paths |
| `listUsbPrinters()` | `UsbPrinterInfo[]` | List connected USB printers |

### UsbPrinterInfo

```typescript
interface UsbPrinterInfo {
  vendorId: number;    // USB VID
  productId: number;   // USB PID
  manufacturer: string;
  product: string;
  serialNumber?: string;
}
```

---

## Platform Notes

### TCP (recommended)

- Default port: **9100** (standard for most thermal printers)
- Retry logic: exponential backoff (100ms → 200ms → 400ms)
- Recommended for production — most reliable across all platforms

### Serial

- Default settings: **9600 baud, 8N1** (8 data bits, no parity, 1 stop bit)
- Supported baud rates: 9600, 19200, 38400, 57600, 115200
- **Linux**: your user account must be in the `dialout` group to access serial ports:
  ```bash
  sudo usermod -a -G dialout $USER
  # Then log out and log back in
  ```

### USB

> ⚠️ **Windows users**: If you get an "interface already claimed" error, the `usbprint.sys` Windows driver has claimed the USB interface. Use **Zadig** to install the WinUSB driver:

1. Download Zadig from [https://zadig.akeo.ie/](https://zadig.akeo.ie/)
2. In the device list, find your printer (check **Options → List All Devices**)
3. Select **WinUSB** as the target driver
4. Click **Install WCID Driver**
5. Retry your JavaScript code

Linux and macOS typically work without additional driver installation.

---

## Turkish Character Support

Characters are automatically transliterated when printed:

| Turkish | CP857 | CP1254 |
|---------|-------|--------|
| ş / Ş | 0xE7 / 0xE6 | 0xFE / 0xDE |
| ğ / Ğ | 0xE8 / 0xE9 | 0xF1 / 0xD1 |
| ı / İ | 0xE1 / 0xD0 | 0xFD / 0xD0 |
| ö / Ö | 0xE6 / 0xD6 | 0xF6 / 0xD6 |
| ü / Ü | 0xE5 / 0xD5 | 0xFC / 0xDC |
| ç / Ç | 0xE8 / 0xC7 | 0xE7 / 0xC7 |

Input is assumed to be UTF-8. Characters not representable in the target codepage are approximated (e.g., ş → s, ğ → g).

---

## Troubleshooting

| Error code in message | Cause | Solution |
|-----------------------|-------|---------|
| `[CONNECTION_REFUSED]` | Printer unreachable or not powered on | Verify printer IP, power cable, and network connection |
| `[CONNECTION_LOST]` | Printer disconnected mid-operation | Check physical connection, cables, printer status |
| `[TIMEOUT]` | Printer did not respond within timeout | Check network latency, printer buffer, IP address |
| `[DEVICE_NOT_FOUND]` | Serial port does not exist | Check COM port name in Device Manager |
| `[PERMISSION_DENIED]` | No read/write access to serial port (Linux) | Add user to `dialout` group |
| `[USB_ERROR]` | USB interface claim failed (Windows) | Use Zadig to install WinUSB driver |
| `[NOT_CONNECTED]` | Called method on disconnected printer | Reconnect before printing |

---

## Supported Printer Commands

### Barcode Types

| Type | System Code | Valid characters |
|------|------------|-----------------|
| `CODE128` | 0x49 | Any ASCII |
| `CODE39` | 0x41 | A-Z, 0-9, space, -, ., $, /, +, % |
| `EAN13` | 0x43 | 12-13 digits |
| `EAN8` | 0x44 | 7-8 digits |
| `UPC_A` | 0x41 | 11-12 digits |
| `UPC_E` | 0x45 | 6-8 digits |
| `ITF` | 0x49 | 1-254 digits (even) |
| `CODABAR` | 0x4B | A-D + digits + $, -, ., /, :, + |

### QR Code

- Error correction level: Level M (default)
- QR version: auto-selected based on data length

### Image Printing

- Supported formats: PNG, JPEG
- Maximum width: **576 pixels** (auto-resized if larger)
- Color: converted to 1-bit (binary) via Floyd-Steinberg dithering
- Alignment: left

---

## Contributing

Issues and pull requests are welcome. When reporting bugs, please include:

- Your operating system and platform (Windows/Linux/macOS + architecture)
- Node.js version
- Printer model
- The full error message and stack trace

---

## License

MIT License — see [LICENSE](LICENSE) for details.
