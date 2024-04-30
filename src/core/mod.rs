pub(crate) mod path;
pub(crate) mod surface;

use bytemuck::{Pod, Zeroable};
pub use path::Path;
pub use surface::Surface;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
