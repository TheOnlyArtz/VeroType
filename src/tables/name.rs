use std::io::{Read, Seek};

use crate::{VeroTypeError, buffer::VeroBufReader};

use super::TableMetadata;

/// Represents the [name table](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html)
#[derive(Debug)]
pub struct Name {
    /// The format of the name table
    format: TableFormat,

    /// The amount of name records in the table
    pub(crate) count: u16,

    /// offset in bytes to the name variable
    pub(crate) string_offset: u16,

    /// A vector of size <count> of the name records
    name_records: Vec<NameRecord>,

    /// The name can't be represented as a String since
    /// there's no guarantee for it to be all valid ASCII chars
    name: Vec<u8>,
}

impl Name {
    pub(crate) fn from_reader<B: Read + Seek>(
        reader: &mut VeroBufReader<B>,
        metadata: &TableMetadata,
    ) -> Result<Self, VeroTypeError> {
        reader.seek_to(metadata.offset.into())?;
        let mut buf = vec![0u8; metadata.length as usize];

        reader.read_exact(&mut buf)?;

        let format = u16::from_be_bytes(buf[0..2].try_into()?);
        let count = u16::from_be_bytes(buf[2..4].try_into()?);
        let string_offset = u16::from_be_bytes(buf[4..6].try_into()?);

        // well, we know that a name record is 12 bytes, we also know where
        // the record array starts and where it ends by doing offset + (count * 12)
        let end_of_array: usize = usize::from(6 + count * 12);
        let array_buffer = &buf[6..end_of_array];
        // TODO: look into safety
        let records = array_buffer
            .chunks(12)
            .map(NameRecord::from_buffer)
            .map(Result::unwrap)
            .collect::<Vec<NameRecord>>();
        
        let string_buffer = &buf[end_of_array..];
        
        Ok(Self {
            format: TableFormat::from(format),
            count,
            string_offset,
            name_records: records,
            name: string_buffer.to_vec()
        })
    }
}

/// Represents a name record
#[derive(Debug)]
struct NameRecord {
    /// Platform identifier code.
    platform_id: PlatformId,

    /// Platform-specific encoding identifier
    platform_specific_id: PlatformSpecificId,

    /// Language identifier
    /// not enumed because there are literally DOZENS
    /// [find them here](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6name.html)
    language_id: u16,

    /// Name identifier, also find them where you can find language identifier
    name_id: u16,

    /// Name string length in bytes
    length: u16,

    /// Name string offset in bytes from stringOffset
    offset: u16,
}

impl NameRecord {
    fn from_buffer(buf: &[u8]) -> Result<Self, VeroTypeError> {
        Ok(Self {
            platform_id: PlatformId::from(u16::from_be_bytes(buf[0..2].try_into()?)),
            platform_specific_id: PlatformSpecificId::from(u16::from_be_bytes(
                buf[2..4].try_into()?,
            )),
            language_id: u16::from_be_bytes(buf[4..6].try_into()?),
            name_id: u16::from_be_bytes(buf[6..8].try_into()?),
            length: u16::from_be_bytes(buf[8..10].try_into()?),
            offset: u16::from_be_bytes(buf[10..12].try_into()?),
        })
    }
}

/// Represents the platform identifier
#[derive(Debug)]
pub enum PlatformId {
    Unicode,
    Macintosh,
    Reserved,
    Microsoft,
    Unknown,
}

impl From<u16> for PlatformId {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Unicode,
            1 => Self::Macintosh,
            2 => Self::Reserved,
            3 => Self::Microsoft,
            _ => Self::Unknown,
        }
    }
}

/// Represents the platform-specific identifier
#[derive(Debug)]
pub enum PlatformSpecificId {
    Version1,
    Version1_1,
    #[warn(deprecated)]
    Iso10646,
    Unicode2_0Bmp,
    Unicode2_0NonBmp,
    Unknown,
}

impl From<u16> for PlatformSpecificId {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::Version1,
            1 => Self::Version1_1,
            2 => Self::Iso10646,
            3 => Self::Unicode2_0Bmp,
            4 => Self::Unicode2_0NonBmp,
            _ => Self::Unknown,
        }
    }
}

/// Represents a table format
/// the name table can have 2 formats
/// 0 => TrueType
/// 1 => OpenType (which is not supported on Apple platforms)
/// Unknown is there for safety but it really shouldn't appear
#[derive(Debug)]
pub enum TableFormat {
    TrueType,
    OpenType,
    Unknown(u16),
}

impl From<u16> for TableFormat {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::TrueType,
            1 => Self::OpenType,
            _ => Self::Unknown(value),
        }
    }
}
