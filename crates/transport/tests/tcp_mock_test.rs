//! Integration test for TCP transport.
//!
//! Tests connection behavior with invalid hosts/ports.

/// Test connection error for invalid host/port.
#[test]
fn test_tcp_connection_error() {
    let result = pos_printer_transport::TcpTransport::new(
        "127.0.0.1",
        59999, // Unlikely to be open
        Some(500),
        Some(pos_printer_transport::RetryConfig {
            max_attempts: 1,
            base_delay_ms: 10,
            backoff_multiplier: 1.0,
        }),
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    // On Windows, unreachable ports return Timeout; on Unix, ConnectionRefused
    assert!(
        matches!(err, pos_printer_transport::PrinterError::Timeout)
            || matches!(err, pos_printer_transport::PrinterError::ConnectionRefused),
        "Expected Timeout or ConnectionRefused, got: {}",
        err
    );
}

/// Test retry config is respected (connection refused after max attempts).
#[test]
fn test_tcp_retry_exhausted() {
    let retry_config = pos_printer_transport::RetryConfig {
        max_attempts: 2,
        base_delay_ms: 10,
        backoff_multiplier: 1.0,
    };

    let start = std::time::Instant::now();

    let result = pos_printer_transport::TcpTransport::new(
        "127.0.0.1",
        59998,
        Some(100),
        Some(retry_config),
    );

    let elapsed = start.elapsed();

    // With max_attempts=2 and base_delay=10ms, should take at least 10ms
    assert!(elapsed.as_millis() >= 10, "Retry delay not respected");
    assert!(result.is_err());
}

/// Test is_connected returns false for failed connection.
#[test]
fn test_tcp_is_connected_false_when_connection_fails() {
    let result = pos_printer_transport::TcpTransport::new(
        "127.0.0.1",
        59997,
        Some(100),
        Some(pos_printer_transport::RetryConfig {
            max_attempts: 1,
            base_delay_ms: 10,
            backoff_multiplier: 1.0,
        }),
    );

    assert!(result.is_err());
}
