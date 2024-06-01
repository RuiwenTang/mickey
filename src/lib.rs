//! mickey is a 2D Vector Graphic rendering engine use wgpu as backend.
//!
//! To use mickey, you need create a `PictureRecorder`, then use `PictureRecorder` to record draw call and generate a `Picture`.
//! Example:
//!
//! ```rust
//!
//! use mickey::core::{Color, Paint, PictureRecorder, Rect};
//!
//! let mut recorder = PictureRecorder::new();
//! let mut paint = Paint::new();
//! paint.color = Color::from_rgba_u8(0x42, 0x85, 0xF4, 0xFF).into();
//! let rect = Rect::from_xywh(10.0, 10.0, 100.0, 160.0);
//! recorder.draw_rect(&rect, &paint);
//!
//! let picture = recorder.finish_record();
//!
//! ```
//!
//! After that, you can use `mickey::gpu::Surface` to replay the draw call in this `Picture` to a wgpu::Texture.
//!

pub mod core;
pub mod gpu;
pub mod text;

pub(crate) mod render;
