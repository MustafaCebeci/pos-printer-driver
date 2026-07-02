//! # ESC/POS Builder
//!
//! Chainable API for building ESC/POS command sequences.

use super::codepage::Encoder;

/// Text alignment options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

/// Font size options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSize {
    Normal,
    Double,
}

/// Cut type for paper cutter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CutType {
    Full,
    Partial,
}

/// Code page for text encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodePage {
    CP857,
    CP1254,
}

/// ESC/POS command builder.
///
/// Provides a chainable API for building print command sequences.
///
/// # Example
/// ```
/// use pos_printer_protocol::{EscPosBuilder, Align, CutType};
///
/// let data = EscPosBuilder::new()
///     .init()
///     .align(Align::Center)
///     .bold(true)
///     .text_line("MERHABA")
///     .bold(false)
///     .feed_lines(3)
///     .cut(CutType::Full)
///     .build();
/// ```
pub struct EscPosBuilder {
    commands: Vec<u8>,
    encoding: CodePage,
}

// ESC/POS command constants
const ESC: u8 = 0x1B;
const GS: u8 = 0x1D;
const CMD_INIT: &[u8] = &[ESC, 0x40];
const CMD_ALIGN_LEFT: &[u8] = &[ESC, 0x61, 0x00];
const CMD_ALIGN_CENTER: &[u8] = &[ESC, 0x61, 0x01];
const CMD_ALIGN_RIGHT: &[u8] = &[ESC, 0x61, 0x02];
const CMD_BOLD_ON: &[u8] = &[ESC, 0x45, 0x01];
const CMD_BOLD_OFF: &[u8] = &[ESC, 0x45, 0x00];
const CMD_FONT_NORMAL: &[u8] = &[GS, 0x21, 0x00];
const CMD_FONT_2X2: &[u8] = &[GS, 0x21, 0x11];
const CMD_CUT_FULL: &[u8] = &[GS, 0x56, 0x00];
const CMD_CUT_PARTIAL: &[u8] = &[GS, 0x56, 0x01];
const CMD_DRAWER: &[u8] = &[ESC, 0x70, 0x00, 0x19, 0xFA];
const LF: u8 = 0x0A;

impl EscPosBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(256),
            encoding: CodePage::CP857,
        }
    }

    /// Set the code page for text encoding.
    pub fn code_page(mut self, cp: CodePage) -> Self {
        self.encoding = cp;
        self
    }

    /// Initialize the printer (ESC @).
    pub fn init(mut self) -> Self {
        self.commands.extend_from_slice(CMD_INIT);
        self
    }

    /// Set text alignment.
    pub fn align(mut self, align: Align) -> Self {
        let cmd = match align {
            Align::Left => CMD_ALIGN_LEFT,
            Align::Center => CMD_ALIGN_CENTER,
            Align::Right => CMD_ALIGN_RIGHT,
        };
        self.commands.extend_from_slice(cmd);
        self
    }

    /// Enable or disable bold text.
    pub fn bold(mut self, on: bool) -> Self {
        self.commands.extend_from_slice(if on { CMD_BOLD_ON } else { CMD_BOLD_OFF });
        self
    }

    /// Set font size.
    pub fn font_size(mut self, size: FontSize) -> Self {
        let cmd = match size {
            FontSize::Normal => CMD_FONT_NORMAL,
            FontSize::Double => CMD_FONT_2X2,
        };
        self.commands.extend_from_slice(cmd);
        self
    }

    /// Add a text line with the current encoding.
    ///
    /// Turkish characters are automatically transliterated to the configured code page.
    pub fn text_line(mut self, text: &str) -> Self {
        let encoded = Encoder::encode(text, self.encoding);
        self.commands.extend_from_slice(&encoded);
        self.commands.push(LF); // Line feed
        self
    }

    /// Feed n lines of paper.
    pub fn feed_lines(mut self, n: u8) -> Self {
        for _ in 0..n {
            self.commands.push(LF);
        }
        self
    }

    /// Cut the paper.
    pub fn cut(mut self, kind: CutType) -> Self {
        let cmd = match kind {
            CutType::Full => CMD_CUT_FULL,
            CutType::Partial => CMD_CUT_PARTIAL,
        };
        self.commands.extend_from_slice(cmd);
        self
    }

    /// Open the cash drawer.
    pub fn open_drawer(mut self) -> Self {
        self.commands.extend_from_slice(CMD_DRAWER);
        self
    }

    /// Build the final command byte sequence.
    pub fn build(self) -> Vec<u8> {
        self.commands
    }
}

impl Default for EscPosBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_command() {
        let result = EscPosBuilder::new().init().build();
        assert_eq!(result, CMD_INIT);
    }

    #[test]
    fn test_align_commands() {
        let left = EscPosBuilder::new().align(Align::Left).build();
        assert_eq!(left, CMD_ALIGN_LEFT);

        let center = EscPosBuilder::new().align(Align::Center).build();
        assert_eq!(center, CMD_ALIGN_CENTER);

        let right = EscPosBuilder::new().align(Align::Right).build();
        assert_eq!(right, CMD_ALIGN_RIGHT);
    }

    #[test]
    fn test_bold_commands() {
        let on = EscPosBuilder::new().bold(true).build();
        assert_eq!(on, CMD_BOLD_ON);

        let off = EscPosBuilder::new().bold(false).build();
        assert_eq!(off, CMD_BOLD_OFF);
    }

    #[test]
    fn test_font_size_commands() {
        let normal = EscPosBuilder::new().font_size(FontSize::Normal).build();
        assert_eq!(normal, CMD_FONT_NORMAL);

        let double = EscPosBuilder::new().font_size(FontSize::Double).build();
        assert_eq!(double, CMD_FONT_2X2);
    }

    #[test]
    fn test_feed_lines() {
        let result = EscPosBuilder::new().feed_lines(3).build();
        assert_eq!(result, vec![0x0A, 0x0A, 0x0A]);
    }

    #[test]
    fn test_cut_commands() {
        let full = EscPosBuilder::new().cut(CutType::Full).build();
        assert_eq!(full, CMD_CUT_FULL);

        let partial = EscPosBuilder::new().cut(CutType::Partial).build();
        assert_eq!(partial, CMD_CUT_PARTIAL);
    }

    #[test]
    fn test_open_drawer() {
        let result = EscPosBuilder::new().open_drawer().build();
        assert_eq!(result, CMD_DRAWER);
    }
}
