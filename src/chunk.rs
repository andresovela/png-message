use std::fmt;
use std::string::FromUtf8Error;
use std::convert::TryFrom;
use crc::crc32;
use crate::chunk_type::ChunkType;

#[derive(Debug)]
pub struct Chunk {
    length: usize,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let crc_data: Vec<u8> = chunk_type.bytes.iter().chain(data.iter()).copied().collect();

        Chunk {
            length: data.len(),
            chunk_type: chunk_type,
            crc: crc32::checksum_ieee(&crc_data[..]),
            data: data,
        }
    }

    pub fn length(&self) -> u32 {
        self.length as u32
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data[..]
    }

    pub fn crc(&self) -> u32 {
        self.crc
    }

    pub fn data_as_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let length_bytes = (self.length as u32).to_be_bytes();
        let crc_bytes = self.crc.to_be_bytes();

        length_bytes.iter()
            .chain(self.chunk_type.bytes.iter())
            .chain(self.data.iter())
            .chain(crc_bytes.iter())
            .copied()
            .collect()
    }
}


impl TryFrom<&[u8]> for Chunk {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 12 {
            return Err("Invalid length");
        }

        let mut length = [0u8; 4];
        length.copy_from_slice(&value[0..4]);
        let length = u32::from_be_bytes(length) as usize;

        if value.len() - length as usize != 12 {
            return Err("Invalid input");
        }

        let mut chunk_type_bytes = [0u8; 4];
        chunk_type_bytes.copy_from_slice(&value[4..8]);

        let chunk_type = ChunkType::try_from(chunk_type_bytes)?;

        let data_len = value.len() - 12;
        let data_and_crc = &value[8..];
        
        let mut data = Vec::with_capacity(data_len);
        data.extend_from_slice(&data_and_crc[..data_len]);

        let mut crc = [0u8; 4];
        crc.copy_from_slice(&value[8 + data_len..]);
        let crc = u32::from_be_bytes(crc);

        let checksum = crc32::checksum_ieee(&value[4..8+data_len]);
        if crc != checksum {
            return Err("Invalid CRC");
        }

        Ok(
            Chunk {
                length,
                chunk_type,
                data: data.to_vec(),
                crc,
            }
        )
    }
}


impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data: Vec<u8> = "This is where your secret message will be!"
            .bytes()
            .collect();
        Chunk::new(chunk_type, data)
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }
}
