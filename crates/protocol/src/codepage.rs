//! # Character Encoding for ESC/POS Printers
//!
//! ESC/POS printers typically do not support UTF-8 directly.
//! This module provides transliteration from UTF-8 to CP857 and CP1254
//! for Turkish character support.

use super::builder::CodePage;

/// Turkish characters that need special mapping in CP857 and CP1254.
const TURKISH_CP857: &[(char, u8)] = &[
    ('ş', 0xE7), ('Ş', 0xE6),
    ('ğ', 0xE8), ('Ğ', 0xE9),
    ('ı', 0xE1), ('İ', 0xD0),
    ('ö', 0xF6), ('Ö', 0xD6),
    ('ü', 0xFC), ('Ü', 0xDC),
    ('ç', 0xE7), ('Ç', 0xC7),
    ('â', 0xE2), ('Â', 0xC2),
    ('ê', 0xEA), ('Ê', 0xCA),
    ('î', 0xEE), ('Î', 0xCE),
    ('ô', 0xF4), ('Ô', 0xD4),
    ('û', 0xFB), ('Û', 0xDB),
];

/// CP1254 (Windows-1254) Turkish code page mapping.
const TURKISH_CP1254: &[(char, u8)] = &[
    ('ş', 0xFE), ('Ş', 0xDE),
    ('ğ', 0x11), ('Ğ', 0xD0),
    ('ı', 0xFD), ('İ', 0xDD),
    ('ö', 0xF6), ('Ö', 0xD6),
    ('ü', 0xFC), ('Ü', 0xDC),
    ('ç', 0xE7), ('Ç', 0xC7),
    ('â', 0xE2), ('Â', 0xC2),
    ('ê', 0xEA), ('Ê', 0xCA),
    ('î', 0xEE), ('Î', 0xCE),
    ('ô', 0xF4), ('Ô', 0xD4),
    ('û', 0xFB), ('Û', 0xDB),
];

/// Standard ASCII range (0x20-0x7E) is identical in both CP857 and CP1254.
const ASCII_START: u8 = 0x20;
const ASCII_END: u8 = 0x7E;

/// Encode a single character to the target code page.
///
/// Returns `None` if the character cannot be represented.
fn encode_char_cp857(c: char) -> Option<u8> {
    // ASCII range
    if c.is_ascii() {
        let b = c as u8;
        if (ASCII_START..=ASCII_END).contains(&b) {
            return Some(b);
        }
        // Control characters and DEL are not printable
        return None;
    }

    // Turkish characters
    TURKISH_CP857.iter()
        .find(|(ch, _)| *ch == c)
        .map(|(_, byte)| *byte)
}

fn encode_char_cp1254(c: char) -> Option<u8> {
    // ASCII range
    if c.is_ascii() {
        let b = c as u8;
        if (ASCII_START..=ASCII_END).contains(&b) {
            return Some(b);
        }
        return None;
    }

    // Turkish characters
    TURKISH_CP1254.iter()
        .find(|(ch, _)| *ch == c)
        .map(|(_, byte)| *byte)
}

/// Encode a UTF-8 string to the target code page bytes.
///
/// Characters that cannot be represented are replaced with '?'.
pub fn encode(text: &str, code_page: CodePage) -> Vec<u8> {
    text.chars()
        .map(|c| {
            let encoded = match code_page {
                CodePage::CP857 => encode_char_cp857(c),
                CodePage::CP1254 => encode_char_cp1254(c),
            };
            encoded.unwrap_or(b'?')
        })
        .collect()
}

/// Check if a character is representable in the given code page.
pub fn is_representable(c: char, code_page: CodePage) -> bool {
    match code_page {
        CodePage::CP857 => encode_char_cp857(c).is_some(),
        CodePage::CP1254 => encode_char_cp1254(c).is_some(),
    }
}

/// Encoder for text encoding operations.
pub struct Encoder;

impl Encoder {
    /// Encode a UTF-8 string to bytes using the specified code page.
    pub fn encode(text: &str, code_page: CodePage) -> Vec<u8> {
        encode(text, code_page)
    }

    /// Check if all characters in a string are representable.
    pub fn can_encode(text: &str, code_page: CodePage) -> bool {
        text.chars().all(|c| is_representable(c, code_page))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_characters() {
        for c in 'A'..='Z' {
            assert_eq!(encode_char_cp857(c), Some(c as u8));
            assert_eq!(encode_char_cp1254(c), Some(c as u8));
        }
        for c in '0'..='9' {
            assert_eq!(encode_char_cp857(c), Some(c as u8));
            assert_eq!(encode_char_cp1254(c), Some(c as u8));
        }
        for c in [' ', '!', '@', '#', '$', '%', '&', '*'] {
            assert_eq!(encode_char_cp857(c), Some(c as u8));
            assert_eq!(encode_char_cp1254(c), Some(c as u8));
        }
    }

    #[test]
    fn test_turkish_lowercase_cp857() {
        assert_eq!(encode_char_cp857('ş'), Some(0xE7));
        assert_eq!(encode_char_cp857('ğ'), Some(0xE8));
        assert_eq!(encode_char_cp857('ı'), Some(0xE1));
        assert_eq!(encode_char_cp857('ö'), Some(0xF6));
        assert_eq!(encode_char_cp857('ü'), Some(0xFC));
        assert_eq!(encode_char_cp857('ç'), Some(0xE7));
    }

    #[test]
    fn test_turkish_uppercase_cp857() {
        assert_eq!(encode_char_cp857('Ş'), Some(0xE6));
        assert_eq!(encode_char_cp857('Ğ'), Some(0xE9));
        assert_eq!(encode_char_cp857('İ'), Some(0xD0));
        assert_eq!(encode_char_cp857('Ö'), Some(0xD6));
        assert_eq!(encode_char_cp857('Ü'), Some(0xDC));
        assert_eq!(encode_char_cp857('Ç'), Some(0xC7));
    }

    #[test]
    fn test_turkish_lowercase_cp1254() {
        assert_eq!(encode_char_cp1254('ş'), Some(0xFE));
        assert_eq!(encode_char_cp1254('ğ'), Some(0x11));
        assert_eq!(encode_char_cp1254('ı'), Some(0xFD));
        assert_eq!(encode_char_cp1254('ö'), Some(0xF6));
        assert_eq!(encode_char_cp1254('ü'), Some(0xFC));
        assert_eq!(encode_char_cp1254('ç'), Some(0xE7));
    }

    #[test]
    fn test_turkish_uppercase_cp1254() {
        assert_eq!(encode_char_cp1254('Ş'), Some(0xDE));
        assert_eq!(encode_char_cp1254('Ğ'), Some(0xD0));
        assert_eq!(encode_char_cp1254('İ'), Some(0xDD));
        assert_eq!(encode_char_cp1254('Ö'), Some(0xD6));
        assert_eq!(encode_char_cp1254('Ü'), Some(0xDC));
        assert_eq!(encode_char_cp1254('Ç'), Some(0xC7));
    }

    #[test]
    fn test_control_characters_not_representable() {
        // Control characters should not be encodable
        for c in ['\0', '\x01', '\x7F'] {
            assert_eq!(encode_char_cp857(c), None);
            assert_eq!(encode_char_cp1254(c), None);
        }
    }

    #[test]
    fn test_full_string_encoding_cp857() {
        let result = encode("Şeker", CodePage::CP857);
        // Ş=0xE6, e=0x65, k=0x6B, e=0x65, r=0x72
        assert_eq!(result, vec![0xE6, 0x65, 0x6B, 0x65, 0x72]);
    }

    #[test]
    fn test_full_string_encoding_cp1254() {
        let result = encode("Şeker", CodePage::CP1254);
        // Ş=0xDE, e=0x65, k=0x6B, e=0x65, r=0x72
        assert_eq!(result, vec![0xDE, 0x65, 0x6B, 0x65, 0x72]);
    }

    #[test]
    fn test_turkish_word_istanbul_cp857() {
        // İstanbul = İstanbul (İ=I=0xD0, s=0x73, t=0x74, a=0x61, n=0x6E, b=0x62, u=0x75, l=0x6C)
        let result = encode("İstanbul", CodePage::CP857);
        assert_eq!(result, vec![0xD0, 0x73, 0x74, 0x61, 0x6E, 0x62, 0x75, 0x6C]);
    }

    #[test]
    fn test_turkish_word_istanbul_cp1254() {
        let result = encode("İstanbul", CodePage::CP1254);
        assert_eq!(result, vec![0xDD, 0x73, 0x74, 0x61, 0x6E, 0x62, 0x75, 0x6C]);
    }

    #[test]
    fn test_mixed_turkish_text_cp857() {
        // Note: "Şekeri" uses ASCII 'i', not Turkish dotless ı
        let result = encode("Çay Şekeri", CodePage::CP857);
        // Ç=0xC7, a=0x61, y=0x79, (space), Ş=0xE6, e=0x65, k=0x6B, e=0x65, r=0x72, i=0x69 (ASCII)
        assert_eq!(result, vec![0xC7, 0x61, 0x79, 0x20, 0xE6, 0x65, 0x6B, 0x65, 0x72, 0x69]);
    }

    #[test]
    fn test_turkish_dotless_i_cp857() {
        // Test Turkish dotless ı (U+0131) - CP857 = 0xE1
        let result = encode("Şekerı", CodePage::CP857);
        // Ş=0xE6, e=0x65, k=0x6B, e=0x65, r=0x72, ı=0xE1
        assert_eq!(result, vec![0xE6, 0x65, 0x6B, 0x65, 0x72, 0xE1]);
    }

    #[test]
    fn test_unrepresentable_char_replaced_with_question_mark() {
        // Japanese characters cannot be represented in Turkish code pages
        let result = encode("日本", CodePage::CP857);
        assert_eq!(result, vec![b'?', b'?']);
    }

    #[test]
    fn test_encoder_can_encode() {
        assert!(Encoder::can_encode("Merhaba Dünya", CodePage::CP857));
        assert!(Encoder::can_encode("Şeker", CodePage::CP1254));
        assert!(!Encoder::can_encode("日本", CodePage::CP857));
    }

    #[test]
    fn test_encoder_encode() {
        let result = Encoder::encode("Test", CodePage::CP857);
        assert_eq!(result, "Test".as_bytes());
    }

    #[test]
    fn test_empty_string() {
        let result = encode("", CodePage::CP857);
        assert!(result.is_empty());
    }

    #[test]
    fn test_all_turkish_chars_cp857() {
        let turkish_chars = "şŞğĞıİöÖüÜçÇ";
        let result = encode(turkish_chars, CodePage::CP857);
        // Each Turkish char should encode to 1 byte in CP857
        assert_eq!(result.len(), turkish_chars.chars().count());
        // Verify each character was encoded (not replaced with ?)
        for (i, c) in turkish_chars.chars().enumerate() {
            let encoded = encode_char_cp857(c);
            assert!(encoded.is_some(), "Failed to encode: {}", c);
            assert_eq!(result[i], encoded.unwrap());
        }
    }

    #[test]
    fn test_all_turkish_chars_cp1254() {
        let turkish_chars = "şŞğĞıİöÖüÜçÇ";
        let result = encode(turkish_chars, CodePage::CP1254);
        assert_eq!(result.len(), turkish_chars.chars().count());
        for (i, c) in turkish_chars.chars().enumerate() {
            let encoded = encode_char_cp1254(c);
            assert!(encoded.is_some(), "Failed to encode: {}", c);
            assert_eq!(result[i], encoded.unwrap());
        }
    }

    #[test]
    fn test_special_turkish_letters() {
        // Test circumflex letters (â, ê, î, ô, û)
        assert_eq!(encode_char_cp857('â'), Some(0xE2));
        assert_eq!(encode_char_cp857('Â'), Some(0xC2));
        assert_eq!(encode_char_cp857('ê'), Some(0xEA));
        assert_eq!(encode_char_cp857('Ê'), Some(0xCA));
        assert_eq!(encode_char_cp857('î'), Some(0xEE));
        assert_eq!(encode_char_cp857('Î'), Some(0xCE));
        assert_eq!(encode_char_cp857('ô'), Some(0xF4));
        assert_eq!(encode_char_cp857('Ô'), Some(0xD4));
        assert_eq!(encode_char_cp857('û'), Some(0xFB));
        assert_eq!(encode_char_cp857('Û'), Some(0xDB));
    }
}
