use std::ops::Add;

// The use of `usize` here probably needs to be reconsidered
#[derive(Debug, Clone, Copy)]
pub struct Vec2D {
	pub x: usize,
	pub y: usize,
}

impl Add<Vec2D> for Vec2D {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

#[derive(Debug, Clone)]
pub struct Rectangle {
	pub min: Vec2D,
	pub max: Vec2D,
}

impl Rectangle {
	pub fn intersection_over_union(&self, other: &Rectangle) -> f32 {
		let intersection_min_x = self.min.x.max(other.min.x);
		let intersection_min_y = self.min.y.max(other.min.y);
		let intersection_max_x = self.max.x.min(other.max.x);
		let intersection_max_y = self.max.y.min(other.max.y);

		let intersection_width = (intersection_max_x - intersection_min_x).max(0);
		let intersection_height = (intersection_max_y - intersection_min_y).max(0);
		let intersection_area = intersection_width * intersection_height;

		let self_area = (self.max.x - self.min.x) * (self.max.y - self.min.y);
		let other_area = (other.max.x - other.min.x) * (other.max.y - other.min.y);

		let union_area = self_area + other_area - intersection_area;

		if union_area == 0 {
			0.0
		} else {
			intersection_area as f32 / union_area as f32
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct Face;

#[derive(Debug, Clone)]
pub struct DetectedFace {
	pub rectangle: Rectangle,
	pub face: Face,
}
