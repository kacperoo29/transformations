use serde::{Serialize, Deserialize};

use crate::vec::Vector2f;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShapeState {
    Drawing,
    Complete
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    state: ShapeState,
    points: Vec<Vector2f>,
}

impl Shape {
    pub fn new() -> Shape {
        Shape {
            state: ShapeState::Drawing,
            points: Vec::new(),
        }
    }

    pub fn add_point(&mut self, point: Vector2f) {
        if self.state == ShapeState::Complete {
            return;
        }
        
        self.points.push(point);
    }

    pub fn get_points(&self) -> &Vec<Vector2f> {
        &self.points
    }

    pub fn get_state(&self) -> &ShapeState {
        &self.state
    }

    pub fn shift(&mut self, shift: Vector2f) {
        for point in self.points.iter_mut() {
            point.set_x(point.x() + shift.x());
            point.set_y(point.y() + shift.y());
        }
    }

    pub fn rotate_rel_to_point(&mut self, angle: f32, point: Vector2f) {
        for pt in self.points.iter_mut() {
            let x = point.x() + (pt.x() - point.x()) * angle.cos() - (pt.y() - point.y()) * angle.sin();
            let y = point.y() + (pt.x() - point.x()) * angle.sin() + (pt.y() - point.y()) * angle.cos();

            pt.set_x(x);
            pt.set_y(y);
        }
    }

    pub fn scale_rel_to_point(&mut self, scale: Vector2f, point: Vector2f) {
        for pt in self.points.iter_mut() {
            let x = point.x() + (pt.x() - point.x()) * scale.x();
            let y = point.y() + (pt.y() - point.y()) * scale.y();

            pt.set_x(x);
            pt.set_y(y);
        }
    }
}