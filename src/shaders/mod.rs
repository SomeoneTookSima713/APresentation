//! Includes the source to all shader files in this same directory.
//! 
//! If any errors occur during shader compilation, these will be thrown from
//! this file during compile time.

#[include_wgsl_oil::include_wgsl_oil("rect.wgsl")]
pub mod rect {}

#[include_wgsl_oil::include_wgsl_oil("post_process.wgsl")]
pub mod post_process {}