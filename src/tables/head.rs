use std::io::{Read, Seek};

use crate::{VeroTypeError, buffer::VeroBufReader};

use super::TableMetadata;

/// Represents the flags field of the 'head' table in a TrueType font file.
/// Each field corresponds to a specific bit in the 16-bit flags value.
#[repr(C)]
#[derive(Debug)]
pub struct HeadFlags {
    /// Bit 0: The Y value of 0 specifies the baseline. If set, a Y coordinate
    /// of 0 corresponds to the font's baseline.
    pub y_value_zero_is_baseline: u8,
    /// Bit 1: x position of left most black bit is LSB. If set, the x coordinate
    /// of the leftmost black bit of a glyph corresponds to the least significant bit
    /// in coordinate values.
    pub x_pos_leftmost_black_bit_lsb: u8,
    /// Bit 2: Scaled point size and actual point size will differ. If set, indicates
    /// that the glyphs in the font are designed such that their scaled point size
    /// might differ from the actual point size (e.g., for hinting purposes).
    pub scaled_point_size_differs: u8,
    /// Bit 3: Use integer scaling instead of fractional. If set, the font scaler
    /// should use integer scaling rather than fractional scaling for glyph rendering.
    pub use_integer_scaling: u8,
    /// Bit 4: (Used by the Microsoft implementation of the TrueType scaler). This
    /// bit is specific to Microsoft's TrueType scaler implementation.
    pub microsoft_scaler_flag: u8,
    /// Bit 5: This bit should be set in fonts that are intended to be laid out
    /// vertically, and in which the glyphs have been drawn such that an x-coordinate
    /// of 0 corresponds to the desired vertical baseline.
    pub vertical_layout: u8,
    /// Bit 6: This bit must be set to zero. This bit is reserved and should always be 0.
    pub must_be_zero: u8,
    /// Bit 7: This bit should be set if the font requires layout for correct
    /// linguistic rendering (e.g., Arabic fonts). Indicates the need for complex
    /// text layout operations.
    pub requires_linguistic_layout: u8,
    /// Bit 8: This bit should be set for an AAT font which has one or more
    /// metamorphosis effects designated as happening by default. Applicable to
    /// Apple Advanced Typography (AAT) fonts.
    pub aat_default_metamorphosis: u8,
    /// Bit 9: This bit should be set if the font contains any strong right-to-left
    /// glyphs. Indicates the presence of glyphs that are inherently right-to-left.
    pub strong_rtl_glyphs: u8,
    /// Bit 10: This bit should be set if the font contains Indic-style rearrangement
    /// effects. Indicates the need for specific glyph rearrangement rules for Indic scripts.
    pub indic_rearrangement: u8,
    /// Bits 11-13: Defined by Adobe. These bits are reserved for Adobe's use and definition.
    pub adobe_defined: u8,
    /// Bit 14: This bit should be set if the glyphs in the font are simply generic
    /// symbols for code point ranges, such as for a last resort font. Indicates
    /// that the glyphs are placeholders or generic representations.
    pub generic_symbol_font: u8,
    /// Bit 15: Reserved (implied by the 16-bit nature of the flags) and should be 0.
    pub reserved: u8,
}

impl HeadFlags {
    pub fn from_bits(bits: u16) -> Self {
        HeadFlags {
            y_value_zero_is_baseline: (bits & 0b0000_0000_0000_0001) as u8,
            x_pos_leftmost_black_bit_lsb: ((bits & 0b0000_0000_0000_0010) >> 1) as u8,
            scaled_point_size_differs: ((bits & 0b0000_0000_0000_0100) >> 2) as u8,
            use_integer_scaling: ((bits & 0b0000_0000_0000_1000) >> 3) as u8,
            microsoft_scaler_flag: ((bits & 0b0000_0000_0001_0000) >> 4) as u8,
            vertical_layout: ((bits & 0b0000_0000_0010_0000) >> 5) as u8,
            must_be_zero: ((bits & 0b0000_0000_0100_0000) >> 6) as u8,
            requires_linguistic_layout: ((bits & 0b0000_0000_1000_0000) >> 7) as u8,
            aat_default_metamorphosis: ((bits & 0b0000_0001_0000_0000) >> 8) as u8,
            strong_rtl_glyphs: ((bits & 0b0000_0010_0000_0000) >> 9) as u8,
            indic_rearrangement: ((bits & 0b0000_0100_0000_0000) >> 10) as u8,
            adobe_defined: ((bits & 0b0011_1000_0000_0000) >> 11) as u8,
            generic_symbol_font: ((bits & 0b0100_0000_0000_0000) >> 14) as u8,
            reserved: 0, // Bit 15 is reserved and should be 0
        }
    }
}

/// A representation of the [head table](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6head.html)
/// including methods to extract it's values safely and efficiently
#[derive(Debug)]
pub struct Head {
    /// The version of the head table
    /// it's almost guarenteed to be set to version 0x00010000
    version: u32,

    /// Font revision set by the font author/manufacturer
    font_revision: u32,

    /// Check sum adjustment
    /// To compute: set it to 0, calculate the checksum for the 'head'
    /// table and put it in the table directory,
    /// sum the entire font as a uint32_t,
    /// then store 0xB1B0AFBA - sum.
    /// (The checksum for the 'head' table will be wrong as a result.
    ///  That is OK; do not reset it.)
    checksum_adjustment: u32,

    /// Magic number, obselete, always set to 0x5F0F3CF5
    magic_number: u32,

    /// The flags which guides the font rendering and processing
    flags: HeadFlags,

    /// Units per em (ranges from 64 to 16384)
    units_per_em: u16,

    /// Date the font was created
    created: i64,

    /// Date the font was last modified
    modified: i64,

    /// The minimum x value for all glyph bounding boxes
    x_min: i16,

    /// The minimum y value for all glyph bounding boxes
    y_min: i16,

    /// The maximum x value for all glyph bounding boxes
    x_max: i16,

    /// The maximum y value for all glyph bounding boxes
    y_max: i16,

    /// Macstyle (TODO)
    mac_style: u16,

    /// Smallest readable size in pixel
    lowest_rec_ppem: u16,

    /// font direction hint (TODO)
    font_direction_hint: i16,

    /// Index to loc format, 0 for short offsets and 1 for long
    index_to_loc_format: i16,

    /// Glyph data format (0 is for the current format)
    glyph_data_format: i16,
}

impl Head {
    /// Constructs a `Head` instance by reading data from the provided `VeroBufReader`.
    ///
    /// This method takes a mutable reference to a `VeroBufReader` and a `TableMetadata`
    /// struct. The `TableMetadata` provides the offset and length of the 'head' table
    /// within the font file.
    ///
    /// The method seeks to the beginning of the 'head' table data using the offset
    /// from the metadata, reads the specified number of bytes into a buffer, and then
    /// parses this buffer to populate the fields of the `Head` struct.
    ///
    /// # Arguments
    ///
    /// * `reader`: A mutable reference to a `VeroBufReader` that provides access to the font file data.
    /// * `metadata`: A reference to a `TableMetadata` struct containing the offset and length of the 'head' table.
    ///
    /// # Errors
    ///
    /// This method can return a `VeroTypeError` in the following cases:
    ///
    /// * **Seeking Error:** If an error occurs while seeking to the specified offset in the `reader`
    ///   (wrapped as `VeroTypeError::IoError`).
    /// * **Reading Error:** If an error occurs while reading the 'head' table data from the `reader`
    ///   (wrapped as `VeroTypeError::IoError`). This could happen if the end of the file is reached
    ///   before the expected number of bytes are read.
    /// * **Data Conversion Error:** If an error occurs during the conversion of the byte slices
    ///   to the expected data types (e.g., `u32`, `u16`, `i64`, `i16`). Note that the `unwrap()`
    ///   calls on `try_into()` will panic if the slice lengths are incorrect, which should be
    ///   prevented by the `metadata.length` check. However, underlying `from_be_bytes` errors
    ///   could potentially occur.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    ///
    /// * `Ok(Self)`: A new `Head` instance populated with the data read from the `reader`.
    /// * `Err(VeroTypeError)`: An error that occurred during the process.
    pub(crate) fn from_reader<B: Read + Seek>(
        reader: &mut VeroBufReader<B>,
        metadata: &TableMetadata,
    ) -> Result<Self, VeroTypeError> {
        reader.seek_to(metadata.offset.into())?;
        let mut buf = vec![0u8; metadata.length as usize];

        reader.read_exact(&mut buf)?;

        Ok(Self {
            version: u32::from_be_bytes(buf[0..4].try_into()?),
            font_revision: u32::from_be_bytes(buf[4..8].try_into()?),
            checksum_adjustment: u32::from_be_bytes(buf[8..12].try_into()?),
            magic_number: u32::from_be_bytes(buf[12..16].try_into()?),
            flags: HeadFlags::from_bits(u16::from_be_bytes(buf[16..18].try_into()?)),
            units_per_em: u16::from_be_bytes(buf[18..20].try_into()?),
            created: i64::from_be_bytes(buf[20..28].try_into()?),
            modified: i64::from_be_bytes(buf[28..36].try_into()?),
            x_min: i16::from_be_bytes(buf[36..38].try_into()?),
            y_min: i16::from_be_bytes(buf[38..40].try_into()?),
            x_max: i16::from_be_bytes(buf[40..42].try_into()?),
            y_max: i16::from_be_bytes(buf[42..44].try_into()?),
            mac_style: u16::from_be_bytes(buf[44..46].try_into()?),
            lowest_rec_ppem: u16::from_be_bytes(buf[46..48].try_into()?),
            font_direction_hint: i16::from_be_bytes(buf[48..50].try_into()?),
            index_to_loc_format: i16::from_be_bytes(buf[50..52].try_into()?),
            glyph_data_format: i16::from_be_bytes(buf[50..52].try_into()?),
        })
    }
}
