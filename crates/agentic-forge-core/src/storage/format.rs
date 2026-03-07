//! Binary format for .forge files.

use crate::types::{ForgeError, ForgeResult, FOOTER_MAGIC, FOOTER_SIZE, FORGE_MAGIC, FORMAT_VERSION, HEADER_SIZE};
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct ForgeHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub flags: u32,
    pub blueprint_count: u64,
    pub entity_count: u64,
    pub file_count: u64,
    pub dependency_count: u64,
    pub test_count: u64,
    pub data_offset: u64,
    pub index_offset: u64,
    pub checksum: [u8; 32],
    pub created_at: u64,
    pub updated_at: u64,
    pub reserved: [u8; 140],
}

impl ForgeHeader {
    pub fn new() -> Self {
        let now = crate::types::now_micros();
        Self {
            magic: FORGE_MAGIC,
            version: FORMAT_VERSION,
            flags: 0,
            blueprint_count: 0,
            entity_count: 0,
            file_count: 0,
            dependency_count: 0,
            test_count: 0,
            data_offset: HEADER_SIZE as u64,
            index_offset: 0,
            checksum: [0u8; 32],
            created_at: now,
            updated_at: now,
            reserved: [0u8; 140],
        }
    }

    pub fn write_to(&self, writer: &mut impl Write) -> ForgeResult<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.version.to_le_bytes())?;
        writer.write_all(&self.flags.to_le_bytes())?;
        writer.write_all(&self.blueprint_count.to_le_bytes())?;
        writer.write_all(&self.entity_count.to_le_bytes())?;
        writer.write_all(&self.file_count.to_le_bytes())?;
        writer.write_all(&self.dependency_count.to_le_bytes())?;
        writer.write_all(&self.test_count.to_le_bytes())?;
        writer.write_all(&self.data_offset.to_le_bytes())?;
        writer.write_all(&self.index_offset.to_le_bytes())?;
        writer.write_all(&self.checksum)?;
        writer.write_all(&self.created_at.to_le_bytes())?;
        writer.write_all(&self.updated_at.to_le_bytes())?;
        writer.write_all(&self.reserved)?;
        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> ForgeResult<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if magic != FORGE_MAGIC {
            return Err(ForgeError::InvalidMagic);
        }

        let mut buf4 = [0u8; 4];
        let mut buf8 = [0u8; 8];
        let mut checksum = [0u8; 32];
        let mut reserved = [0u8; 140];

        reader.read_exact(&mut buf4)?;
        let version = u32::from_le_bytes(buf4);
        if version != FORMAT_VERSION {
            return Err(ForgeError::UnsupportedVersion(version));
        }

        reader.read_exact(&mut buf4)?;
        let flags = u32::from_le_bytes(buf4);

        reader.read_exact(&mut buf8)?;
        let blueprint_count = u64::from_le_bytes(buf8);

        reader.read_exact(&mut buf8)?;
        let entity_count = u64::from_le_bytes(buf8);

        reader.read_exact(&mut buf8)?;
        let file_count = u64::from_le_bytes(buf8);

        reader.read_exact(&mut buf8)?;
        let dependency_count = u64::from_le_bytes(buf8);

        reader.read_exact(&mut buf8)?;
        let test_count = u64::from_le_bytes(buf8);

        reader.read_exact(&mut buf8)?;
        let data_offset = u64::from_le_bytes(buf8);

        reader.read_exact(&mut buf8)?;
        let index_offset = u64::from_le_bytes(buf8);

        reader.read_exact(&mut checksum)?;

        reader.read_exact(&mut buf8)?;
        let created_at = u64::from_le_bytes(buf8);

        reader.read_exact(&mut buf8)?;
        let updated_at = u64::from_le_bytes(buf8);

        reader.read_exact(&mut reserved)?;

        Ok(Self {
            magic,
            version,
            flags,
            blueprint_count,
            entity_count,
            file_count,
            dependency_count,
            test_count,
            data_offset,
            index_offset,
            checksum,
            created_at,
            updated_at,
            reserved,
        })
    }

    pub fn compute_size() -> usize {
        HEADER_SIZE
    }
}

impl Default for ForgeHeader {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ForgeFooter {
    pub magic: [u8; 8],
    pub total_size: u64,
    pub section_count: u32,
    pub checksum: [u8; 32],
    pub reserved: [u8; 12],
}

impl ForgeFooter {
    pub fn new(total_size: u64, section_count: u32) -> Self {
        Self {
            magic: FOOTER_MAGIC,
            total_size,
            section_count,
            checksum: [0u8; 32],
            reserved: [0u8; 12],
        }
    }

    pub fn write_to(&self, writer: &mut impl Write) -> ForgeResult<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.total_size.to_le_bytes())?;
        writer.write_all(&self.section_count.to_le_bytes())?;
        writer.write_all(&self.checksum)?;
        writer.write_all(&self.reserved)?;
        Ok(())
    }

    pub fn read_from(reader: &mut impl Read) -> ForgeResult<Self> {
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;
        if magic != FOOTER_MAGIC {
            return Err(ForgeError::Corrupt(0));
        }

        let mut buf8 = [0u8; 8];
        let mut buf4 = [0u8; 4];
        let mut checksum = [0u8; 32];
        let mut reserved = [0u8; 12];

        reader.read_exact(&mut buf8)?;
        let total_size = u64::from_le_bytes(buf8);

        reader.read_exact(&mut buf4)?;
        let section_count = u32::from_le_bytes(buf4);

        reader.read_exact(&mut checksum)?;
        reader.read_exact(&mut reserved)?;

        Ok(Self {
            magic,
            total_size,
            section_count,
            checksum,
            reserved,
        })
    }

    pub fn compute_size() -> usize {
        FOOTER_SIZE
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    pub section_type: SectionType,
    pub offset: u64,
    pub size: u64,
    pub checksum: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SectionType {
    Blueprints = 0,
    Entities = 1,
    Files = 2,
    Dependencies = 3,
    Tests = 4,
    Types = 5,
    Functions = 6,
    Wiring = 7,
    DataFlows = 8,
    ImportGraph = 9,
    Metadata = 10,
    Index = 11,
}

impl SectionType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Blueprints),
            1 => Some(Self::Entities),
            2 => Some(Self::Files),
            3 => Some(Self::Dependencies),
            4 => Some(Self::Tests),
            5 => Some(Self::Types),
            6 => Some(Self::Functions),
            7 => Some(Self::Wiring),
            8 => Some(Self::DataFlows),
            9 => Some(Self::ImportGraph),
            10 => Some(Self::Metadata),
            11 => Some(Self::Index),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_header_roundtrip() {
        let header = ForgeHeader::new();
        let mut buf = Vec::new();
        header.write_to(&mut buf).unwrap();
        let parsed = ForgeHeader::read_from(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(parsed.magic, FORGE_MAGIC);
        assert_eq!(parsed.version, FORMAT_VERSION);
    }

    #[test]
    fn test_header_invalid_magic() {
        let mut buf = vec![0xFF, 0xFF, 0xFF, 0xFF];
        buf.extend_from_slice(&[0u8; 252]);
        let result = ForgeHeader::read_from(&mut Cursor::new(&buf));
        assert!(result.is_err());
    }

    #[test]
    fn test_footer_roundtrip() {
        let footer = ForgeFooter::new(1024, 5);
        let mut buf = Vec::new();
        footer.write_to(&mut buf).unwrap();
        let parsed = ForgeFooter::read_from(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(parsed.total_size, 1024);
        assert_eq!(parsed.section_count, 5);
    }

    #[test]
    fn test_footer_invalid_magic() {
        let buf = vec![0xFF; 64];
        let result = ForgeFooter::read_from(&mut Cursor::new(&buf));
        assert!(result.is_err());
    }

    #[test]
    fn test_section_type_roundtrip() {
        for v in 0..=11u8 {
            let st = SectionType::from_u8(v).unwrap();
            assert_eq!(st as u8, v);
        }
    }

    #[test]
    fn test_section_type_invalid() {
        assert!(SectionType::from_u8(200).is_none());
    }

    #[test]
    fn test_header_default() {
        let header = ForgeHeader::default();
        assert_eq!(header.magic, FORGE_MAGIC);
    }

    #[test]
    fn test_header_size() {
        assert_eq!(ForgeHeader::compute_size(), HEADER_SIZE);
    }

    #[test]
    fn test_footer_size() {
        assert_eq!(ForgeFooter::compute_size(), FOOTER_SIZE);
    }

    #[test]
    fn test_header_counts() {
        let mut header = ForgeHeader::new();
        header.blueprint_count = 5;
        header.entity_count = 20;
        let mut buf = Vec::new();
        header.write_to(&mut buf).unwrap();
        let parsed = ForgeHeader::read_from(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(parsed.blueprint_count, 5);
        assert_eq!(parsed.entity_count, 20);
    }
}
