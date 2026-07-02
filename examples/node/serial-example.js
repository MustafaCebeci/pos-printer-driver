/**
 * Serial (COM Port) Connection Example
 *
 * Connects to a SUNLUX RP8020 (or any ESC/POS printer) via serial port.
 * Prints "MERHABA DÜNYA" and cuts the paper.
 *
 * Usage:
 *   node serial-example.js [port] [baudRate]
 *
 * Examples:
 *   node serial-example.js                      # Auto-detect ports, show list
 *   node serial-example.js COM3                # COM3 at default 9600 baud
 *   node serial-example.js /dev/ttyUSB0 115200 # Linux with higher baud rate
 *
 * Common baud rates: 9600, 19200, 38400, 57600, 115200
 *
 * Default settings: 9600 baud, 8N1 (8 data bits, no parity, 1 stop bit)
 */

const { Printer, listSerialPorts } = require('../..');

async function main() {
  let port = process.argv[2];
  let baudRate = process.argv[3] ? parseInt(process.argv[3], 10) : undefined;

  // List available serial ports if no port specified
  if (!port) {
    const ports = listSerialPorts();
    if (ports.length === 0) {
      console.log('No serial ports found.');
      console.log('Make sure the printer is connected and powered on.');
      return;
    }
    console.log('Available serial ports:');
    for (const p of ports) {
      console.log(`  - ${p}`);
    }
    console.log('\nUsage: node serial-example.js <port> [baudRate]');
    console.log('Example: node serial-example.js COM3 9600');
    return;
  }

  console.log(`Connecting to ${port}${baudRate ? ` at ${baudRate} baud` : ' at 9600 baud'}...`);

  try {
    const printer = await Printer.connect_serial(port, baudRate);
    console.log('Connected:', printer.is_connected());

    await printer.print_text('MERHABA DÜNYA');
    await printer.feed_lines(3);
    await printer.cut();

    await printer.disconnect();
    console.log('Done!');
  } catch (err) {
    console.error('Error:', err.message);
    if (err.message.includes('Permission denied')) {
      console.error('\nHint: On Linux, add yourself to the dialout group:');
      console.error('  sudo usermod -a -G dialout $USER');
      console.error('Then log out and log back in.');
    }
    process.exit(1);
  }
}

main();
