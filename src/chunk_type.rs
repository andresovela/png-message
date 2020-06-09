use std::fmt;
use std::convert::TryFrom;
use std::str::{self, FromStr};

/// A 4-byte chunk type code.
#[derive(Debug, PartialEq)]
struct ChunkType {
    bytes: [u8; 4],
}

impl ChunkType {
    fn bytes(&self) -> &[u8; 4] {
        &self.bytes
    }

    /// Returns `true` if all the bytes are ASCII letters
    /// (A-Z and a-z).
    fn is_valid(&self) -> bool {
        for byte in self.bytes.iter() {
            // Before A or after z
            if *byte < 65 || *byte > 122 {
                return false;
            }

            // Between Z and a
            if *byte > 90u8 && *byte < 97u8 {
                return false;
            }
        }

        self.is_reserved_bit_valid()
    }

    /// Returns `true` if this a critical chunk.
    /// A chunk is considered critical if bit 5 of the first byte
    /// is zero. Otherwise, the chunk is considered ancillary.
    fn is_critical(&self) -> bool {
        (self.bytes[0] & 0b0010_0000) == 0
    }

    /// Returns `true` if this is a public chunk.
    /// A chunk is considered public if bit 5 of the second byte
    /// is zero. Otherwise, the chunk is considered private.
    fn is_public(&self) -> bool {
        (self.bytes[1] & 0b0010_0000) == 0
    }

    /// Returns `true` if the reserved bit is valid.
    /// The reserved bit must be zero to conform to version 1.2
    /// of the PNG specification.
    /// The reserved bit is bit 5 of the third byte.
    fn is_reserved_bit_valid(&self) -> bool {
        (self.bytes[2] & 0b0010_0000) == 0
    }

    /// Returns `true` if it is safe to copy this chunk.
    /// A chunk is considered safe to copy if bit 5 of the fourth byte
    /// is 1. Otherwise, the chunk is unsafe to copy.
    fn is_safe_to_copy(&self) -> bool {
        (self.bytes[3] & 0b0010_0000) != 0
    }
}


impl TryFrom<[u8; 4]> for ChunkType {
    type Error = &'static str;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        Ok(ChunkType { bytes: value })
    }
}


impl FromStr for ChunkType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.bytes().count() != 4 {
            return Err("Invalid string length");
        }

        if !s.is_ascii() {
            return Err("Invalid string")
        }

        for (_, c) in s.bytes().enumerate() {
            if !c.is_ascii_alphabetic() {
                return Err("Invalid string");
            }
        }

        let mut bytes: [u8; 4] = [0; 4];
        bytes.clone_from_slice(&s.as_bytes()[0..4]);

        Ok( ChunkType { bytes })
    }
}


impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(s) = str::from_utf8(&self.bytes) {
            write!(f, "{}", s)
        }
        else {
            Err(fmt::Error)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = &[82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }
}