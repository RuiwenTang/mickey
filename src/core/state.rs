use nalgebra::{Matrix4, Vector3};

pub(crate) struct State {
    matrix_stack: Vec<Matrix4<f32>>,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            matrix_stack: vec![Matrix4::identity()],
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

    pub(crate) fn restore(&mut self) {
        self.matrix_stack.pop();

        if self.matrix_stack.is_empty() {
            self.matrix_stack.push(Matrix4::identity());
        }
    }

    pub(crate) fn translate(&mut self, dx: f32, dy: f32) {
        let current_matrix = self.matrix_stack.pop();

        self.matrix_stack
            .push(current_matrix.unwrap() * Matrix4::new_translation(&Vector3::new(dx, dy, 0.0)));
    }
}
