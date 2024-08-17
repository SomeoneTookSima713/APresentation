//! Includes the source to all shader files in this same directory.

// #[include_wgsl_oil::include_wgsl_oil("rect.wgsl")]
// pub mod rect {}

// #[include_wgsl_oil::include_wgsl_oil("post_process.wgsl")]
// pub mod post_process {}

pub mod rect {
    pub const SOURCE: &str = include_str!("rect.wgsl");
}

pub mod post_process {
    pub const SOURCE: &str = include_str!("post_process.wgsl");
}