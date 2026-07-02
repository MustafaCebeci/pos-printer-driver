/**
 * USB Connection Example
 *
 * Connects to a SUNLUX RP8020 (or any ESC/POS printer) via USB.
 * Prints "MERHABA DÜNYA" and cuts the paper.
 *
 * Usage:
 *   node usb-example.js [vendorId] [productId]
 *
 * Examples:
 *   node usb-example.js                                    # Auto-detect first USB printer
 *   node usb-example.js 0x0483 0x5720                     # SUNLUX RP8020
 *
 * ⚠️  Windows Note:
 *   If you get an "interface already claimed" error, another driver
 *   (usbprint.sys) has claimed the USB interface. Use Zadig to install
 *   WinUSB driver:
 *   1. Download Zadig from https://zadig.akeo.ie/
 *   2. Find your printer in the list
 *   3. Select WinUSB and click "Install WCID Driver"
 *
 *   Alternatively, use TCP (tcp-example.js) which works reliably on all platforms.
 */

const { Printer, listUsbPrinters } = require('../..');

async function main() {
  let vendorId = undefined;
  let productId = undefined;

  if (process.argv.length >= 4) {
    vendorId = parseInt(process.argv[2], 16);
    productId = parseInt(process.argv[3], 16);
  }

  // List available USB printers
  const printers = listUsbPrinters();
  if (printers.length === 0) {
    console.log('No USB printers found.');
    console.log('Make sure the printer is connected via USB and powered on.');
    return;
  }

  console.log('Available USB printers:');
  for (const p of printers) {
    console.log(`  - ${p.manufacturer} ${p.product} (VID: 0x${p.vendorId.toString(16).padStart(4, '0')}, PID: 0x${p.productId.toString(16).padStart(4, '0')})`);
  }

  console.log(`\nConnecting${vendorId ? ` to VID:0x${vendorId.toString(16)} PID:0x${productId.toString(16)}` : ''}...`);

  try {
    const printer = await Printer.connect_usb(vendorId, productId);
    console.log('Connected:', printer.is_connected());

    await printer.print_text('MERHABA DÜNYA');
    await printer.feed_lines(3);
    await printer.cut();

    await printer.disconnect();
    console.log('Done!');
  } catch (err) {
    console.error('Error:', err.message);
    if (err.message.includes('already claimed') || err.message.includes('usbprint.sys')) {
      console.error('\nHint: Another driver has claimed the USB interface.');
      console.error('Use Zadig (https://zadig.akeo.ie/) to install WinUSB driver.');
    }
    process.exit(1);
  }
}

main();
