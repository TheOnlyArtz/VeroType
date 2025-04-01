use std::io::{self, BufReader, Read, Seek};

use thiserror::Error;

macro_rules! impl_read {
    ($fn_name:ident, $typ:ty) => {
        pub fn $fn_name(&mut self) -> Result<$typ, VeroBufReaderError> {
            let size = size_of::<$typ>();
            let mut buf = vec![0; size];

            self.inner.read_exact(&mut buf)?;

            Ok(<$typ>::from_be_bytes(buf.try_into().unwrap()))
        }
    };
}

/// Represents the possible errors that can occur when using `VeroBufReader`.
#[derive(Error, Debug)]
pub enum VeroBufReaderError {
    /// An error occurred during a read operation on the underlying buffer.
    /// This variant transparently wraps `std::io::Error`.
    #[error(transparent)]
    ReadError(#[from] io::Error),

    /// An error occurred during a seek operation on the underlying buffer.
    /// This variant contains the `std::io::Error` that caused the seek failure.
    #[error("Failed to seek, error context: {0}")]
    FailedToSeek(io::Error),
}

/// A Struct which encapsulates and provides a robust API
/// for interacting with a buffer
pub struct VeroBufReader<B: Read + Seek> {
    inner: BufReader<B>,
}

impl<B> VeroBufReader<B>
where
    B: Read + Seek,
{
    /// Returns a new buf reader from anything which implements read
    /// the most obvious use case would be a File
    /// but it's also useful for loading fonts off a network buffer
    /// and such
    pub fn from_buffer(buffer: B) -> Self {
        Self {
            inner: BufReader::new(buffer),
        }
    }

    /// Seeks to a specifc place in the buffer
    /// from the start of the file
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use vero_buf_reader::VeroBufReader;
    ///
    /// let data = vec![0, 0, 0, 10, 0, 0, 0, 20]; // Represents two u32 values: 10 and 20 in big-endian
    /// let cursor = Cursor::new(data);
    /// let mut reader = VeroBufReader::from_buffer(cursor);
    ///
    /// // Seek to the beginning of the second u32 (at index 4)
    /// reader.seek_to(4).unwrap();
    /// let second_value = reader.read_u32().unwrap();
    /// assert_eq!(second_value, 20);
    /// ```
    pub fn seek_to(&mut self, pos: u64) -> Result<(), VeroBufReaderError> {
        self.inner
            .seek(std::io::SeekFrom::Start(pos))
            .map_err(|ctx| VeroBufReaderError::FailedToSeek(ctx))?;

        Ok(())
    }

    /// Skips n bytes from the CURRENT cursor positon
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::Cursor;
    /// use vero_buf_reader::VeroBufReader;
    ///
    /// let data = vec![0, 0, 0, 10, 0, 0, 0, 20]; // Represents two u32 values: 10 and 20 in big-endian
    /// let cursor = Cursor::new(data);
    /// let mut reader = VeroBufReader::from_buffer(cursor);
    ///
    /// // Read the first u32
    /// let first_value = reader.read_u32().unwrap();
    /// assert_eq!(first_value, 10);
    ///
    /// // Skip the next 4 bytes (the second u32)
    /// reader.skip(4).unwrap();
    ///
    /// // Attempting to read should now result in an EOF error
    /// assert!(reader.read_u32().is_err());
    /// ```
    pub fn skip(&mut self, n: i64) -> Result<(), VeroBufReaderError> {
        self.inner
            .seek(std::io::SeekFrom::Current(n))
            .map_err(|ctx| VeroBufReaderError::FailedToSeek(ctx))?;

        Ok(())
    }

    impl_read!(read_i32, i32);
    impl_read!(read_u32, u32);
    impl_read!(read_i16, i16);
    impl_read!(read_u16, u16);
    impl_read!(read_i8, i8);
    impl_read!(read_u8, u8);
}
