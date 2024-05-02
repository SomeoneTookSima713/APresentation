use once_cell::sync::Lazy;

/// The initial window size.
pub const WINDOW_SIZE: (u32, u32) = (1280,720);
pub const WINDOW_RESIZE_INCREMENTS: (u32, u32) = (1,1);
/// The window's title.
pub const WINDOW_TITLE: &str = "APresentation";

pub const ICON_FILE_DATA: &[u8] = include_bytes!("../icon.png");

pub static ICON_DATA: Lazy<image::RgbaImage> = Lazy::new(|| {
    let reader = image::io::Reader::with_format(std::io::Cursor::new(ICON_FILE_DATA), image::ImageFormat::Png);

    reader.decode().unwrap().to_rgba8()
});