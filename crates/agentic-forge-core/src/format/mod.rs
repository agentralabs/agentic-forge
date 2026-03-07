//! Binary .forge file format.

use crate::storage::format::{ForgeFooter, ForgeHeader};
use crate::types::blueprint::Blueprint;
use crate::types::{ForgeError, ForgeResult, FORGE_MAGIC};
use std::io::{Read, Write};
use std::path::Path;

pub struct ForgeWriter;

impl ForgeWriter {
    pub fn write_to_file(blueprints: &[Blueprint], path: &Path) -> ForgeResult<usize> {
        let data = serde_json::to_vec(blueprints)?;
        let checksum = blake3::hash(&data);

        let mut header = ForgeHeader::new();
        header.blueprint_count = blueprints.len() as u64;
        header.checksum = *checksum.as_bytes();

        let mut entity_count = 0u64;
        let mut file_count = 0u64;
        let mut dep_count = 0u64;
        let mut test_count = 0u64;
        for bp in blueprints {
            entity_count += bp.entities.len() as u64;
            file_count += bp.files.len() as u64;
            dep_count += bp.dependencies.len() as u64;
            test_count += bp.test_cases.len() as u64;
        }
        header.entity_count = entity_count;
        header.file_count = file_count;
        header.dependency_count = dep_count;
        header.test_count = test_count;

        let mut file = std::fs::File::create(path)?;
        header.write_to(&mut file)?;
        file.write_all(&data)?;

        let total = crate::types::HEADER_SIZE as u64 + data.len() as u64 + crate::types::FOOTER_SIZE as u64;
        let footer = ForgeFooter::new(total, 1);
        footer.write_to(&mut file)?;

        Ok(data.len())
    }

    pub fn write_to_bytes(blueprints: &[Blueprint]) -> ForgeResult<Vec<u8>> {
        let data = serde_json::to_vec(blueprints)?;
        let checksum = blake3::hash(&data);
        let mut header = ForgeHeader::new();
        header.blueprint_count = blueprints.len() as u64;
        header.checksum = *checksum.as_bytes();

        let mut buf = Vec::new();
        header.write_to(&mut buf)?;
        buf.extend_from_slice(&data);
        let total = crate::types::HEADER_SIZE as u64 + data.len() as u64 + crate::types::FOOTER_SIZE as u64;
        let footer = ForgeFooter::new(total, 1);
        footer.write_to(&mut buf)?;
        Ok(buf)
    }
}

pub struct ForgeReader;

impl ForgeReader {
    pub fn read_from_file(path: &Path) -> ForgeResult<Vec<Blueprint>> {
        let data = std::fs::read(path)?;
        Self::read_from_bytes(&data)
    }

    pub fn read_from_bytes(data: &[u8]) -> ForgeResult<Vec<Blueprint>> {
        if data.len() < crate::types::HEADER_SIZE + crate::types::FOOTER_SIZE {
            return Err(ForgeError::Truncated);
        }
        let mut cursor = std::io::Cursor::new(data);
        let header = ForgeHeader::read_from(&mut cursor)?;

        let payload_start = crate::types::HEADER_SIZE;
        let payload_end = data.len() - crate::types::FOOTER_SIZE;
        let payload = &data[payload_start..payload_end];

        let blueprints: Vec<Blueprint> = serde_json::from_slice(payload)?;

        if blueprints.len() as u64 != header.blueprint_count {
            return Err(ForgeError::Corrupt(0));
        }

        Ok(blueprints)
    }

    pub fn read_header(data: &[u8]) -> ForgeResult<ForgeHeader> {
        if data.len() < 4 {
            return Err(ForgeError::Truncated);
        }
        let mut cursor = std::io::Cursor::new(data);
        ForgeHeader::read_from(&mut cursor)
    }

    pub fn is_forge_file(data: &[u8]) -> bool {
        data.len() >= 4 && data[..4] == FORGE_MAGIC
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::intent::Domain;

    #[test]
    fn test_write_and_read_bytes() {
        let bp = Blueprint::new("Test", "A test", Domain::Api);
        let bytes = ForgeWriter::write_to_bytes(&[bp.clone()]).unwrap();
        let loaded = ForgeReader::read_from_bytes(&bytes).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Test");
    }

    #[test]
    fn test_write_and_read_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.forge");
        let bp = Blueprint::new("FileTest", "Test", Domain::Cli);
        ForgeWriter::write_to_file(&[bp], &path).unwrap();
        let loaded = ForgeReader::read_from_file(&path).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "FileTest");
    }

    #[test]
    fn test_read_header() {
        let bp = Blueprint::new("Test", "A test", Domain::Web);
        let bytes = ForgeWriter::write_to_bytes(&[bp]).unwrap();
        let header = ForgeReader::read_header(&bytes).unwrap();
        assert_eq!(header.blueprint_count, 1);
    }

    #[test]
    fn test_is_forge_file() {
        let bp = Blueprint::new("Test", "A test", Domain::Web);
        let bytes = ForgeWriter::write_to_bytes(&[bp]).unwrap();
        assert!(ForgeReader::is_forge_file(&bytes));
        assert!(!ForgeReader::is_forge_file(&[0, 0, 0, 0]));
    }

    #[test]
    fn test_truncated_data() {
        assert!(ForgeReader::read_from_bytes(&[]).is_err());
        assert!(ForgeReader::read_from_bytes(&[0u8; 10]).is_err());
    }

    #[test]
    fn test_multiple_blueprints() {
        let bp1 = Blueprint::new("A", "First", Domain::Api);
        let bp2 = Blueprint::new("B", "Second", Domain::Cli);
        let bytes = ForgeWriter::write_to_bytes(&[bp1, bp2]).unwrap();
        let loaded = ForgeReader::read_from_bytes(&bytes).unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_empty_blueprints() {
        let bytes = ForgeWriter::write_to_bytes(&[]).unwrap();
        let loaded = ForgeReader::read_from_bytes(&bytes).unwrap();
        assert!(loaded.is_empty());
    }
}
