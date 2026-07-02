//! TCP/IP transport implementation for network printers.
//!
//! Most ESC/POS printers support TCP connections on port 9100 (raw socket).
//! This implementation provides thread-safe access with automatic retry.

use std::net::{TcpStream, Shutdown, SocketAddr};
use std::io::Write;
use std::time::Duration;
use std::sync::Mutex;

use super::{PrinterError, PrinterTransport};

/// Configuration for connection retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of connection attempts (default: 3)
    pub max_attempts: u32,
    /// Initial delay between retries in milliseconds (default: 100)
    pub base_delay_ms: u64,
    /// Multiplier for exponential backoff (default: 2.0)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            backoff_multiplier: 2.0,
        }
    }
}

/// TCP transport for ESC/POS printers.
#[derive(Debug)]
pub struct TcpTransport {
    stream: Mutex<Option<TcpStream>>,
    ip: String,
    port: u16,
    timeout: Duration,
    retry_config: RetryConfig,
}

impl TcpTransport {
    /// Create a new TCP transport and connect to the printer.
    pub fn new(
        ip: &str,
        port: u16,
        timeout_ms: Option<u32>,
        retry_config: Option<RetryConfig>,
    ) -> Result<Self, PrinterError> {
        let timeout = Duration::from_millis(timeout_ms.unwrap_or(5000) as u64);
        let retry_config = retry_config.unwrap_or_default();

        let transport = Self {
            stream: Mutex::new(None),
            ip: ip.to_string(),
            port,
            timeout,
            retry_config,
        };

        // Initial connection with retry
        transport.connect()?;

        Ok(transport)
    }

    /// Establish connection with retry logic.
    fn connect(&self) -> Result<(), PrinterError> {
        let addr: SocketAddr = format!("{}:{}", self.ip, self.port)
            .parse()
            .map_err(|e| PrinterError::ConnectionFailed(format!("Invalid address: {}", e)))?;

        let mut attempts = 0;
        let mut delay_ms = self.retry_config.base_delay_ms;

        loop {
            attempts += 1;

            match TcpStream::connect_timeout(&addr, self.timeout) {
                Ok(stream) => {
                    stream.set_write_timeout(Some(self.timeout)).ok();
                    stream.set_read_timeout(Some(self.timeout)).ok();

                    let mut guard = self.stream.lock().unwrap();
                    *guard = Some(stream);
                    return Ok(());
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    if attempts >= self.retry_config.max_attempts {
                        return Err(PrinterError::Timeout);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
                    if attempts >= self.retry_config.max_attempts {
                        return Err(PrinterError::ConnectionRefused);
                    }
                }
                Err(e) => {
                    if attempts >= self.retry_config.max_attempts {
                        return Err(PrinterError::ConnectionFailed(e.to_string()));
                    }
                }
            }

            // Exponential backoff
            if attempts < self.retry_config.max_attempts {
                std::thread::sleep(Duration::from_millis(delay_ms));
                delay_ms = (delay_ms as f64 * self.retry_config.backoff_multiplier) as u64;
            }
        }
    }

    /// Reconnect to the printer.
    pub fn reconnect(&mut self) -> Result<(), PrinterError> {
        // Clear existing connection
        {
            let mut guard = self.stream.lock().unwrap();
            if let Some(ref mut s) = *guard {
                let _ = s.shutdown(Shutdown::Both);
            }
            *guard = None;
        }

        self.connect()
    }

    /// Check if the underlying TCP connection is still alive.
    fn is_peer_connected(stream: &TcpStream) -> bool {
        stream.peer_addr().is_ok()
    }
}

impl PrinterTransport for TcpTransport {
    fn write(&mut self, data: &[u8]) -> Result<(), PrinterError> {
        let mut guard = self.stream.lock().unwrap();
        let stream = guard.as_mut().ok_or(PrinterError::NotConnected)?;

        stream.write_all(data).map_err(|e| {
            // Detect connection loss
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
        let guard = self.stream.lock().unwrap();
        guard.as_ref().is_some_and(Self::is_peer_connected)
    }

    fn flush(&mut self) -> Result<(), PrinterError> {
        let mut guard = self.stream.lock().unwrap();
        let stream = guard.as_mut().ok_or(PrinterError::NotConnected)?;

        stream.flush().map_err(|e| {
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
        "TCP"
    }
}

impl Drop for TcpTransport {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.stream.lock() {
            if let Some(ref mut s) = *guard {
                let _ = s.shutdown(Shutdown::Both);
            }
            *guard = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay_ms, 100);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_is_connected_false_when_none() {
        let transport = TcpTransport {
            stream: Mutex::new(None),
            ip: "127.0.0.1".to_string(),
            port: 9100,
            timeout: Duration::from_secs(5),
            retry_config: RetryConfig::default(),
        };

        assert!(!transport.is_connected());
    }

    #[test]
    fn test_transport_type() {
        let transport = TcpTransport {
            stream: Mutex::new(None),
            ip: "127.0.0.1".to_string(),
            port: 9100,
            timeout: Duration::from_secs(5),
            retry_config: RetryConfig::default(),
        };

        assert_eq!(transport.transport_type(), "TCP");
    }
}
