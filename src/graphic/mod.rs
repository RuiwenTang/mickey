mod paint;
mod picture;

pub use paint::*;
pub use picture::*;

/// The clip operation describes how to apply a clip to a path.
/// The default value is [`ClipOp::Intersect`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipOp {
    /// The intersect clip operation will draw only the parts of the path that are inside the clip.
    #[default]
    Intersect,
    /// The difference clip operation will draw only the parts of the path that are outside the clip.
    Difference,
}

/// The stroke cap describes the start and end of a stroke.
/// The default value is StrokeCap::Butt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StrokeCap {
    Round,
    Square,
    #[default]
    Butt,
}

/// The stroke join describes the corners of a stroke.
/// The default value is StrokeJoin::Miter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrokeJoin {
    Round,
    Bevel,
    /// The miter join is the most common join.
    /// The miter join will be clipped if the miter limit is reached.
    /// The default value is 4.0.
    Miter(f32),
}

impl Default for StrokeJoin {
    fn default() -> Self {
        StrokeJoin::Miter(4.0)
    }
}

/// The style describes how to draw a Geometry.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Style {
    #[default]
    Fill,
    /// The stroke style is used to draw a stroke outline.
    /// The stroke style has 4 arguments:
    /// * The stroke cap.
    /// * The stroke join.
    /// * The stroke width.
    Stroke(StrokeCap, StrokeJoin, f32),
}

/// The stroke info describes the stroke style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub width: f32,
}

impl Stroke {
    pub fn new(cap: StrokeCap, join: StrokeJoin, width: f32) -> Stroke {
        Stroke { cap, join, width }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn cap(&self) -> StrokeCap {
        self.cap
    }

    pub fn join(&self) -> StrokeJoin {
        self.join
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn with_cap(mut self, cap: StrokeCap) -> Self {
        self.cap = cap;
        self
    }

    pub fn with_join(mut self, join: StrokeJoin) -> Self {
        self.join = join;
        self
    }
}

impl Default for Stroke {
    fn default() -> Self {
        Stroke {
            cap: StrokeCap::default(),
            join: StrokeJoin::default(),
            width: 1.0,
        }
    }
}

impl Into<Style> for Stroke {
    fn into(self) -> Style {
        Style::Stroke(self.cap, self.join, self.width)
    }
}
