use std::{
    collections::{BTreeMap, btree_map::IntoIter},
    io::{Read, Seek},
};

use head::Head;
use name::Name;
use thiserror::Error;

use crate::{VeroTypeError, buffer::VeroBufReader};

pub mod head;
pub mod name;

/// An enum for the required tables
/// tables where every TrueType formatted font must include in it's
/// file's table directory.
/// For more information, see the [Apple Documentation Table 2](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html)
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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

impl TryFrom<&[u8]> for RequiredTables {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, ()> {
        // TODO: This unwrap
        Ok(match str::from_utf8(value).unwrap() {
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
pub struct OffsetTable {
    scalar_type: u32,
    num_tables: u16,
    search_range: u16,
    entry_selector: u16,
    range_shift: u16,
}

impl OffsetTable {
    /// Constructs the offset sub table from a raw buffer
    /// the offset sub table buffer size must be 12 per the reference manual.
    pub fn from_buffer(buf: &[u8]) -> Result<Self, TableEncodingError> {
        if buf.len() != 12 {
            return Err(TableEncodingError::InvalidBufferLength(12, buf.len()));
        }

        Ok(Self {
            scalar_type: u32::from_be_bytes(buf[0..4].try_into().unwrap()),
            num_tables: u16::from_be_bytes(buf[4..6].try_into().unwrap()),
            search_range: u16::from_be_bytes(buf[6..8].try_into().unwrap()),
            entry_selector: u16::from_be_bytes(buf[8..10].try_into().unwrap()),
            range_shift: u16::from_be_bytes(buf[10..12].try_into().unwrap()),
        })
    }

    /// Parses an offset table completely from a reader reference
    /// which reads the WHOLE file
    pub(crate) fn from_reader<B: Read + Seek>(
        reader: &mut VeroBufReader<B>,
    ) -> Result<Self, VeroTypeError> {
        // since we know it's a fixed size of 12 we can seek to byte 0 and read exactly
        // 12 bytes in order to get the buffer
        // then we can use from_buffer
        reader.seek_to(0)?;

        // Allocate the fixed-size buffer of 12 bytes
        let mut buffer = [0u8; 12];
        reader.read_exact(&mut buffer)?;

        Ok(Self::from_buffer(&buffer)?)
    }

    /// Returns the number of tables exists in the font file
    pub fn num_tables(&self) -> u16 {
        self.num_tables
    }
}

/// Represents all of the tables and their respective data types.
#[derive(Debug)]
pub struct Tables {
    /// The offset table, which provides the starting offsets of other tables.
    pub offset: OffsetTable,

    /// The headers of the tables
    pub headers: TablesHeaders,

    /// The head table
    pub head_table: Head,
}

impl Tables {
    /// Constructs a `Tables` instance by reading data from a `VeroBufReader`.
    ///
    /// This method reads the offset table from the provided reader, which
    /// is typically the first table in the data structure.
    ///
    /// # Errors
    ///
    /// This method can return a `VeroTypeError` if an error occurs while
    /// reading or parsing the offset table. This could include issues with
    /// the underlying reader or the format of the offset table data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Assuming you have a VeroBufReader instance named 'reader'
    /// // and an OffsetTable struct defined elsewhere.
    /// # use std::io::{Cursor, Read, Seek};
    /// # use vero_buf_reader::VeroBufReader;
    /// #
    /// # #[derive(Debug)]
    /// # struct OffsetTable {}
    /// #
    /// # impl OffsetTable {
    /// #     pub fn from_reader<B: Read + Seek>(reader: &mut VeroBufReader<B>) -> Result<Self, VeroTypeError> {
    /// #         // Implementation to read the offset table
    /// #         Ok(OffsetTable {})
    /// #     }
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # enum VeroTypeError {
    /// #     OffsetTableError,
    /// #     IoError(std::io::Error),
    /// # }
    /// #
    /// # fn main() -> Result<(), VeroTypeError> {
    /// #     let data = vec![0u8; 12]; // Example data for the offset table
    /// #     let cursor = Cursor::new(data);
    /// #     let mut reader = VeroBufReader::from_buffer(cursor);
    ///     let tables_result = Tables::from_reader(&mut reader);
    ///
    ///     match tables_result {
    ///         Ok(tables) => {
    ///             println!("Successfully parsed tables: {:?}", tables);
    ///         }
    ///         Err(e) => {
    ///             eprintln!("Error parsing tables: {:?}", e);
    ///         }
    ///     }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn from_reader<B: Read + Seek>(
        reader: &mut VeroBufReader<B>,
    ) -> Result<Self, VeroTypeError> {
        let offset_table = OffsetTable::from_reader(reader)?;
        let headers = TablesHeaders::from_reader(reader, offset_table.num_tables())?;
        let head_table = Head::from_reader(reader, headers.get(RequiredTables::Head).unwrap())?;
        let name_table = Name::from_reader(reader, headers.get(RequiredTables::Name).unwrap())?;
        
        println!("{:?}", name_table);
        Ok(Self {
            offset: offset_table,
            head_table,
            headers,
        })
    }
}

/// Represents the table headers and maps a table tag to it's offset
/// in the file / buffer
#[derive(Debug)]
pub struct TablesHeaders {
    /// A Binary Tree Map which maps a table type represented by the
    /// RequiredTables enum and it's Metadata, the metadata disgards the tag field
    /// as it's represented as the key of the entry.
    inner: BTreeMap<RequiredTables, TableMetadata>,
}

impl TablesHeaders {
    /// Constructs a `TablesHeaders` instance by reading table headers from the provided `VeroBufReader`.
    ///
    /// This method reads the specified number of table headers from the reader. It assumes that the
    /// reader's current position is at the beginning of the table header entries, immediately following
    /// the Offset Table in a TrueType font file. Each table header is expected to be 16 bytes long.
    ///
    /// The method parses each 16-byte chunk into `TableMetadata` and stores it in a `BTreeMap`,
    /// keyed by the corresponding `RequiredTables` enum variant. Tables with tags that do not match
    /// any variant of `RequiredTables` are currently ignored (see TODO in the code).
    ///
    /// # Arguments
    ///
    /// * `reader`: A mutable reference to a `VeroBufReader` that provides access to the font file data.
    /// * `num_tables`: The total number of tables present in the font file, as indicated in the Offset Table.
    ///
    /// # Errors
    ///
    /// This method can return a `VeroTypeError` in the following cases:
    ///
    /// * **Reading Error:** If an error occurs while reading the table header data from the `reader`
    ///   (wrapped as `VeroTypeError::IoError`). This could happen if the end of the file is reached
    ///   before the expected number of bytes are read.
    /// * **`TableMetadata::from_buffer` Error:** If an error occurs while parsing a 16-byte chunk
    ///   into a `TableMetadata` instance. This could indicate an issue with the format of the table
    ///   header data.
    /// * **`RequiredTables::try_from` Error:** Although the current implementation doesn't explicitly
    ///   propagate this as an error, the `try_from` conversion from the 4-byte tag to `RequiredTables`
    ///   might fail if the tag is not recognized. In the current code, such tables are skipped.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    ///
    /// * `Ok(Self)`: A new `TablesHeaders` instance containing a `BTreeMap` of `TableMetadata`,
    ///   keyed by `RequiredTables`.
    /// * `Err(VeroTypeError)`: An error that occurred during the process.
    pub fn from_reader<B: Read + Seek>(
        reader: &mut VeroBufReader<B>,
        num_tables: u16,
    ) -> Result<Self, VeroTypeError> {
        // We know the offset table takes 12 bytes from the start of the file
        // we also know that a table header size is 16 bytes
        // to get the buffer of all of the tables we would need
        // to multiply the num_tables by 16 bytes
        // then process the headers in chuncks of 16 bytes
        let mut buffer = vec![0u8; usize::from(num_tables) * 16];
        reader.read_exact(&mut buffer)?;

        // Initialize the headers binary tree map
        let mut headers: BTreeMap<RequiredTables, TableMetadata> = BTreeMap::new();

        // divide the buffer into chunks of 16 bytes where every entry is a different table
        let chunks = buffer.chunks(16).collect::<Vec<&[u8]>>();

        // Iterate over every raw table data and parse it to it's metadata
        // TODO: Handle tables which are not required
        for raw_table in chunks {
            let tag = &raw_table[0..4];

            if let Ok(table_type) = RequiredTables::try_from(tag) {
                let metadata = TableMetadata::from_buffer(raw_table)?;

                // Add the entry to the headers BTreeMap
                headers.insert(table_type, metadata);
            }
        }

        Ok(Self { inner: headers })
    }

    /// Retrieves the `TableMetadata` for a specific required table.
    ///
    /// This method takes a `RequiredTables` enum variant as input and returns an `Option`
    /// containing a reference to the `TableMetadata` associated with that table, if it exists
    /// in the parsed headers.
    ///
    /// # Arguments
    ///
    /// * `k`: The `RequiredTables` variant representing the table for which to retrieve metadata.
    ///
    /// # Returns
    ///
    /// An `Option<&TableMetadata>`:
    ///
    /// * `Some(&TableMetadata)`: If metadata for the specified table is found.
    /// * `None`: If no metadata is found for the specified table.
    pub fn get(&self, k: RequiredTables) -> Option<&TableMetadata> {
        self.inner.get(&k)
    }
}

impl IntoIterator for TablesHeaders {
    type Item = (RequiredTables, TableMetadata);

    type IntoIter = IntoIter<RequiredTables, TableMetadata>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

/// Represents metadata for a table within a larger data structure.
#[derive(Debug)]
pub struct TableMetadata {
    /// The checksum of the table. This value can be used to verify the
    /// integrity of the table data.
    checksum: u32,
    /// The offset of the table, in bytes, from the beginning of the file
    /// or buffer containing the data structure. This indicates where the
    /// actual table data starts.
    offset: u32,
    /// The length of this table in bytes. This represents the actual size
    /// of the table data and does not include any padding that might be
    /// present.
    length: u32,
}

impl TableMetadata {
    /// Constructs a `TableMetadata` instance from a raw byte buffer.
    ///
    /// This method expects a buffer of exactly 16 bytes. The bytes are
    /// interpreted as follows (all values are in big-endian order):
    ///
    /// * Bytes 0-3: Reserved (not used in this implementation)
    /// * Bytes 4-7: Checksum of the table
    /// * Bytes 8-11: Offset of the table from the beginning of the file
    /// * Bytes 12-15: Length of the table in bytes
    ///
    /// # Errors
    ///
    /// This method will return a `TableEncodingError::InvalidBufferLength`
    /// if the provided buffer does not have a length of exactly 16 bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use vero_buf_reader::TableEncodingError;
    ///
    /// let buffer: [u8; 16] = [
    ///     0x00, 0x00, 0x00, 0x00, // Reserved
    ///     0x12, 0x34, 0x56, 0x78, // Checksum: 0x12345678
    ///     0x00, 0x01, 0x00, 0x00, // Offset: 0x00010000 (4096)
    ///     0x00, 0x00, 0x0A, 0x00, // Length: 0x00000A00 (2560)
    /// ];
    ///
    /// match TableMetadata::from_buffer(&buffer) {
    ///     Ok(metadata) => {
    ///         assert_eq!(metadata.checksum, 0x12345678);
    ///         assert_eq!(metadata.offset, 0x00010000);
    ///         assert_eq!(metadata.length, 0x00000A00);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Error creating TableMetadata: {:?}", e);
    ///     }
    /// }
    ///
    /// let invalid_buffer: [u8; 10] = [0; 10];
    /// assert!(TableMetadata::from_buffer(&invalid_buffer).is_err());
    /// ```
    pub fn from_buffer(buf: &[u8]) -> Result<Self, TableEncodingError> {
        // Each table metadata should be EXACTLY 16 bytes
        if buf.len() != 16 {
            return Err(TableEncodingError::InvalidBufferLength(16, buf.len()));
        }

        Ok(Self {
            checksum: u32::from_be_bytes(buf[4..8].try_into().unwrap()),
            offset: u32::from_be_bytes(buf[8..12].try_into().unwrap()),
            length: u32::from_be_bytes(buf[12..16].try_into().unwrap()),
        })
    }
}
