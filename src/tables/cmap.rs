/// A representation of the [cmap table](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html)
/// including methods to extract it's values safely and efficiently 
/// supporting only formats 4 and 12 as these are the most used formats while other
/// are either for specialized uses or just never got materialized as the reference manual suggests.
#[derive(Debug)]
pub struct Cmap {
    /// The version of the cmap table
    /// it's almost guarenteed to be set to zero
    version: u16,
    
    /// The number of encoding subtables
    subtables: u16,
}

/// A representation of the cmap [sub table](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html)
#[derive(Debug)]
pub struct CmapSub {
    /// The platform identifier
    platform_id: u16,
    
    /// The platform specific encoding identifier
    platform_specific_id: u16,
    
    /// The offset of the mapping table
    offset: u32
}