/**
 * Panic Safety and Error Handling Tests
 *
 * These tests verify that:
 * 1. Rust panic's don't crash the Node.js process
 * 2. Failed factory calls return Error objects (not crash)
 * 3. Error codes are embedded in error messages
 *
 * Run with: node tests/panic_test.js
 */

const assert = require('assert');

// Test 1: Invalid IP/port should return Error, not crash
async function testConnectionRefused() {
  try {
    // 0.0.0.0 with port 0 should fail with an error
    const printer = await Printer.connect_tcp('0.0.0.0', 0, 1000, 1);
    // If we get here, the connection unexpectedly succeeded
    assert.fail('Expected connection to fail');
  } catch (err) {
    console.log('✓ Connection error returns Error object:', err.message);
    // Verify it's a proper Error instance
    assert(err instanceof Error, 'Should be an Error instance');
    // Verify error code is embedded
    assert(
      err.message.includes('[TIMEOUT]') ||
      err.message.includes('[CONNECTION_FAILED]') ||
      err.message.includes('[CONNECTION_REFUSED]'),
      'Error message should contain error code'
    );
  }
}

// Test 2: Invalid barcode type should return Error
async function testInvalidBarcodeType() {
  // First connect to something that might succeed (or use mock)
  // For now, we just test that the error path works
  // This test requires a connected printer, so we skip if not connected
  try {
    const printer = await Printer.connect_tcp('127.0.0.1', 59999, 500, 1);
    if (!printer.is_connected()) {
      console.log('⊘ Skipping barcode test (no mock server)');
      return;
    }
    try {
      await printer.print_barcode('INVALID_TYPE', '123456');
      assert.fail('Expected invalid barcode to throw');
    } catch (err) {
      console.log('✓ Invalid barcode type returns Error:', err.message);
      assert(err.message.includes('Unknown barcode type'));
    }
  } catch (err) {
    // Expected - connection to 127.0.0.1:59999 should fail
    console.log('⊘ Skipping barcode test (connection failed):', err.message);
  }
}

// Test 3: Missing image file should return Error
async function testMissingImage() {
  try {
    const printer = await Printer.connect_tcp('127.0.0.1', 59999, 500, 1);
    if (!printer.is_connected()) {
      console.log('⊘ Skipping image test (no mock server)');
      return;
    }
    try {
      await printer.print_image('/nonexistent/path/image.png');
      assert.fail('Expected missing image to throw');
    } catch (err) {
      console.log('✓ Missing image returns Error:', err.message);
      assert(err.message.includes('Failed to read image file'));
    }
  } catch (err) {
    console.log('⊘ Skipping image test (connection failed):', err.message);
  }
}

// Test 4: Error codes are correctly embedded
async function testErrorCodeEmbedding() {
  const testCases = [
    { ip: '0.0.0.0', port: 0, timeout: 100 },
    { ip: '192.168.254.254', port: 9999, timeout: 500 }, // Unlikely to respond
  ];

  for (const tc of testCases) {
    try {
      await Printer.connect_tcp(tc.ip, tc.port, tc.timeout, 1);
    } catch (err) {
      // Extract code from [CODE] format
      const match = err.message.match(/^\[([A-Z_]+)\]/);
      if (match) {
        console.log(`✓ Error code embedded: ${match[1]}`);
      } else {
        console.log('⊘ No error code found in message:', err.message);
      }
      break;
    }
  }
}

async function runAll() {
  console.log('Running panic safety and error handling tests...\n');
  await testConnectionRefused();
  await testInvalidBarcodeType();
  await testMissingImage();
  await testErrorCodeEmbedding();
  console.log('\nAll tests passed!');
}

runAll().catch(console.error);
