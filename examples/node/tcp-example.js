/**
 * TCP Connection Example
 *
 * Connects to a SUNLUX RP8020 (or any ESC/POS printer) via TCP on port 9100.
 * Prints "MERHABA DÜNYA" and cuts the paper.
 *
 * Usage:
 *   node tcp-example.js [ip] [port]
 *
 * Defaults: ip=192.168.1.100, port=9100
 *
 * To test without real hardware, run a TCP echo server first:
 *   node -e "require('net').createServer(s => s.pipe(s)).listen(9100)"
 */

const { Printer } = require('../..');

const ip = process.argv[2] || '192.168.1.100';
const port = parseInt(process.argv[3], 10) || 9100;

async function main() {
  console.log(`Connecting to ${ip}:${port}...`);

  // Connect with TCP transport
  const printer = await Printer.connect_tcp(ip, port, 5000, 3);

  console.log('Connected:', printer.is_connected());

  // Print a text line
  await printer.print_text('MERHABA DÜNYA');

  // Feed and cut
  await printer.feed_lines(3);
  await printer.cut();

  // Disconnect
  await printer.disconnect();
  console.log('Done!');
}

main().catch((err) => {
  console.error('Error:', err.message);
  // Error codes are embedded in message as [CODE] format
  if (err.message.includes('[CONNECTION_REFUSED]')) {
    console.error('Hint: Is the printer powered on and connected to the network?');
  } else if (err.message.includes('[TIMEOUT]')) {
    console.error('Hint: Check the IP address and firewall settings.');
  }
  process.exit(1);
});
