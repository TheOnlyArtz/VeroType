use std::io::{Read, Seek};

use crate::{VeroTypeError, buffer::VeroBufReader};

use super::TableMetadata;

/// Represents the flags field of the 'head' table in a TrueType font file.
/// Each field corresponds to a specific bit in the 16-bit flags value.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HeadFlags {
    bits: u16,
}

impl HeadFlags {
    /// Constructs a `HeadFlags` instance from a raw `u16` value.
    pub fn from_bits(bits: u16) -> Self {
        HeadFlags { bits }
    }

    /// Checks if the Y value of 0 specifies the baseline (bit 0).
    pub fn y_value_zero_is_baseline(&self) -> bool {
        (self.bits & 0b0000_0000_0000_0001) != 0
    }

    /// Checks if the x position of the leftmost black bit is the LSB (bit 1).
    pub fn x_pos_leftmost_black_bit_lsb(&self) -> bool {
        (self.bits & 0b0000_0000_0000_0010) != 0
    }

    /// Checks if the scaled point size and actual point size will differ (bit 2).
    pub fn scaled_point_size_differs(&self) -> bool {
        (self.bits & 0b0000_0000_0000_0100) != 0
    }

    /// Checks if integer scaling should be used instead of fractional (bit 3).
    pub fn use_integer_scaling(&self) -> bool {
        (self.bits & 0b0000_0000_0000_1000) != 0
    }

    /// Checks the Microsoft implementation of the TrueType scaler flag (bit 4).
    pub fn microsoft_scaler_flag(&self) -> bool {
        (self.bits & 0b0000_0000_0001_0000) != 0
    }

    /// Checks if the font is intended for vertical layout (bit 5).
    pub fn vertical_layout(&self) -> bool {
        (self.bits & 0b0000_0000_0010_0000) != 0
    }

    /// Checks if bit 6 (must be zero) is set.
    pub fn must_be_zero(&self) -> bool {
        (self.bits & 0b0000_0000_0100_0000) != 0
    }

    /// Checks if the font requires layout for correct linguistic rendering (bit 7).
    pub fn requires_linguistic_layout(&self) -> bool {
        (self.bits & 0b0000_0000_1000_0000) != 0
    }

    /// Checks if it's an AAT font with default metamorphosis effects (bit 8).
    pub fn aat_default_metamorphosis(&self) -> bool {
        (self.bits & 0b0000_0001_0000_0000) != 0
    }

    /// Checks if the font contains any strong right-to-left glyphs (bit 9).
    pub fn strong_rtl_glyphs(&self) -> bool {
        (self.bits & 0b0000_0010_0000_0000) != 0
    }

    /// Checks if the font contains Indic-style rearrangement effects (bit 10).
    pub fn indic_rearrangement(&self) -> bool {
        (self.bits & 0b0000_0100_0000_0000) != 0
    }

    /// Returns the raw value of the Adobe-defined bits (bits 11-13).
    pub fn adobe_defined(&self) -> u8 {
        ((self.bits & 0b0011_1000_0000_0000) >> 11) as u8
    }

    /// Checks if the glyphs are generic symbols for code point ranges (bit 14).
    pub fn generic_symbol_font(&self) -> bool {
        (self.bits & 0b0100_0000_0000_0000) != 0
    }

    /// Returns the raw bits of the flags.
    pub fn bits(&self) -> u16 {
        self.bits
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
    ///   That is OK; do not reset it.)
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
    ///    (wrapped as `VeroTypeError::IoError`).
    /// * **Reading Error:** If an error occurs while reading the 'head' table data from the `reader`
    ///    (wrapped as `VeroTypeError::IoError`). This could happen if the end of the file is reached
    ///    before the expected number of bytes are read.
    /// * **Data Conversion Error:** If an error occurs during the conversion of the byte slices
    ///    to the expected data types (e.g., `u32`, `u16`, `i64`, `i16`). Note that the `unwrap()`
    ///    calls on `try_into()` will panic if the slice lengths are incorrect, which should be
    ///    prevented by the `metadata.length` check. However, underlying `from_be_bytes` errors
    ///    could potentially occur.
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
            glyph_data_format: i16::from_be_bytes(buf[52..54].try_into()?),
        })
    }

    /// Returns the version of the head table.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// Returns the font revision set by the font author/manufacturer.
    pub fn font_revision(&self) -> u32 {
        self.font_revision
    }

    /// Returns the check sum adjustment value.
    pub fn checksum_adjustment(&self) -> u32 {
        self.checksum_adjustment
    }

    /// Returns the magic number (always 0x5F0F3CF5).
    pub fn magic_number(&self) -> u32 {
        self.magic_number
    }

    /// Returns the flags which guide the font rendering and processing.
    pub fn flags(&self) -> HeadFlags {
        self.flags
    }

    /// Returns the units per em value.
    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    /// Returns the date the font was created.
    pub fn created(&self) -> i64 {
        self.created
    }

    /// Returns the date the font was last modified.
    pub fn modified(&self) -> i64 {
        self.modified
    }

    /// Returns the minimum x value for all glyph bounding boxes.
    pub fn x_min(&self) -> i16 {
        self.x_min
    }

    /// Returns the minimum y value for all glyph bounding boxes.
    pub fn y_min(&self) -> i16 {
        self.y_min
    }

    /// Returns the maximum x value for all glyph bounding boxes.
    pub fn x_max(&self) -> i16 {
        self.x_max
    }

    /// Returns the maximum y value for all glyph bounding boxes.
    pub fn y_max(&self) -> i16 {
        self.y_max
    }

    /// Returns the mac style flags.
    pub fn mac_style(&self) -> u16 {
        self.mac_style
    }

    /// Returns the smallest readable size in pixel.
    pub fn lowest_rec_ppem(&self) -> u16 {
        self.lowest_rec_ppem
    }

    /// Returns the font direction hint.
    pub fn font_direction_hint(&self) -> i16 {
        self.font_direction_hint
    }

    /// Returns the index to loc format (0 for short offsets, 1 for long).
    pub fn index_to_loc_format(&self) -> i16 {
        self.index_to_loc_format
    }

    /// Returns the glyph data format (0 is for the current format).
    pub fn glyph_data_format(&self) -> i16 {
        self.glyph_data_format
    }
}