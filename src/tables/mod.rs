use thiserror::Error;

/// An enum for the required tables
/// tables where every TrueType formatted font must include in it's
/// file's table directory.
/// For more information, see the [Apple Documentation Table 2](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html)
pub enum RequiredTables {
    Cmap,
    Glyf,
    Head,
    Hhea,
    Hmtx,
    Loca,
    Maxp,
    Name,
    Post,
}

impl TryFrom<&str> for RequiredTables {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        Ok(match value {
            "cmap" => Self::Cmap,
            "glyf" => Self::Glyf,
            "head" => Self::Head,
            "hhea" => Self::Hhea,
            "hmtx" => Self::Hmtx,
            "loca" => Self::Loca,
            "maxp" => Self::Maxp,
            "name" => Self::Name,
            "post" => Self::Post,
            _ => Err(())?,
        })
    }
}

/// Represents the error messages which may occur when trying
/// to parse tables from raw binary buffers
#[derive(Error, Debug)]
pub enum TableEncodingError {
    #[error("The required buffer length for this table is {0} bytes, got {0} bytes")]
    InvalidBufferLength(usize, usize),
}

/// Represents the offset subtable directory and it's metadata
/// providing us with a important info such as the number of tables
#[derive(Debug)]
pub struct OffsetSub {
    scalar_type: u32,
    num_tables: u16,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
}

impl OffsetSub {
    /// Constructs the offset sub table from a raw buffer
    /// the offset sub table buffer size must be 12 per the reference manual.
    pub fn from_buffer(buf: &[u8]) -> Result<Self, TableEncodingError> {
        if buf.len() != 12 {
            return Err(TableEncodingError::InvalidBufferLength(12, buf.len()));
        }
        // safety should arise from the assertion above
        let buf: [u8; 12] = buf.try_into().unwrap();

        let raw_scalar_type = &buf[0..4];
        let raw_num_tables = &buf[4..6];
        let search_range = &buf[6..8];
        let entry_selector = &buf[8..10];
        let range_shift = &buf[10..12];

        Ok(Self {
            scalar_type: u32::from_be_bytes(raw_scalar_type.try_into().unwrap()),
            num_tables: u16::from_be_bytes(raw_num_tables.try_into().unwrap()),
            search_range: u16::from_be_bytes(search_range.try_into().unwrap()),
            entry_selector: u16::from_be_bytes(entry_selector.try_into().unwrap()),
            range_shift: u16::from_be_bytes(range_shift.try_into().unwrap()),
        })
    }

    /// Returns the number of tables exists in the font file
    pub fn num_tables(&self) -> u16 {
        self.num_tables
    }
}
