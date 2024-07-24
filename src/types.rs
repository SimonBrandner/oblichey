use std::ops::Add;

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

#[derive(Debug, Clone, Default)]
pub struct Face;

#[derive(Debug, Clone)]
pub struct DetectedFace {
	pub rectangle: Rectangle,
	pub face: Face,
}
