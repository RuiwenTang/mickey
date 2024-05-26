use std::rc::Rc;

use crate::{core::Color, text::TextBlob};

pub(crate) struct SolidTextFragment {
    color: Color,
    blob: Rc<TextBlob>,
}
