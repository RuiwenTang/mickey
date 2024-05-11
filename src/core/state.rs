use nalgebra::{Matrix4, Vector3};

use super::geometry::degree_to_radian;

pub(crate) struct ClipState {
    pub(crate) clip_op: Vec<usize>,
}

impl ClipState {
    fn save_clip(&mut self, index: usize) {
        self.clip_op.push(index);
    }
}

pub(crate) struct State {
    matrix_stack: Vec<Matrix4<f32>>,
    clip_stack: Vec<ClipState>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            matrix_stack: vec![Matrix4::identity()],
            clip_stack: vec![ClipState { clip_op: vec![] }],
        }
    }

    pub(crate) fn current_transform(&self) -> Matrix4<f32> {
        return self
            .matrix_stack
            .last()
            .expect("State stack is error")
            .clone();
    }

    pub(crate) fn save(&mut self) {
        let last_matrix = self.matrix_stack.last().unwrap();

        self.matrix_stack.push(last_matrix.clone());
    }

    pub(crate) fn restore(&mut self) -> Option<ClipState> {
        self.matrix_stack.pop();

        if self.matrix_stack.is_empty() {
            self.matrix_stack.push(Matrix4::identity());
        }

        let clip_state = self.clip_stack.pop();
        if self.clip_stack.is_empty() {
            self.clip_stack.push(ClipState { clip_op: vec![] });
        }

        return clip_state;
    }

    pub(crate) fn save_clip(&mut self, index: usize) {
        self.clip_stack.last_mut().unwrap().save_clip(index);
    }

    pub(crate) fn pop_clip_stack(&mut self) -> Option<ClipState> {
        return self.clip_stack.pop();
    }

    pub(crate) fn translate(&mut self, dx: f32, dy: f32) {
        let current_matrix = self.matrix_stack.pop();

        self.matrix_stack
            .push(current_matrix.unwrap() * Matrix4::new_translation(&Vector3::new(dx, dy, 0.0)));
    }

    pub(crate) fn rotate_at(&mut self, degree: f32, px: f32, py: f32) {
        let current_matrix = self.matrix_stack.pop();
        let rotate = Matrix4::new_rotation(Vector3::new(0.0, 0.0, degree_to_radian(degree)));
        let pre = Matrix4::new_translation(&Vector3::new(-px, -py, 0.0));
        let post = Matrix4::new_translation(&Vector3::new(px, py, 0.0));

        self.matrix_stack
            .push(current_matrix.unwrap() * post * rotate * pre);
    }

    pub(crate) fn rotate(&mut self, degree: f32) {
        let current_matrix = self.matrix_stack.pop();
        let rotate = Matrix4::new_rotation(Vector3::new(0.0, 0.0, degree_to_radian(degree)));

        self.matrix_stack.push(current_matrix.unwrap() * rotate);
    }

    pub(crate) fn scale(&mut self, sx: f32, sy: f32) {
        let current_matrix = self.matrix_stack.pop();
        let s: Matrix4<f32> = Matrix4::new(
            sx, 0.0, 0.0, 0.0, 0.0, sy, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );

        self.matrix_stack.push(current_matrix.unwrap() * s);
    }
}
