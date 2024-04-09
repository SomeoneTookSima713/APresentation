pub mod consts {
    /// The initial window size.
    pub const WINDOW_SIZE: (i32, i32) = (1280,720);
    /// The window's title.
    pub const WINDOW_TITLE: &str = "APresentation";

    pub const ICON_WIDTH: usize = 64;
    pub const ICON_HEIGHT: usize = 64;
    /// The raw icon data in RGBA-format, contained in a reference to an array.
    /// 
    /// Neat (but partially unintended) side-effect to doing it this way: The
    /// program won't compile if you change the dimensions of the image file,
    /// but not the ones from the constants in this file!
    /// 
    /// That being said: If you get an error of this constant being assigned a
    /// value of a wrong type, the icon's dimensions defined above don't match
    /// the actual icon's dimensions.
    pub const ICON_DATA: &[u8;ICON_WIDTH*ICON_HEIGHT*4] = include_bytes!(concat!(env!("OUT_DIR"), "/icon.bin"));
}