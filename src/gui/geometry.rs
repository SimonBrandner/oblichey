use crate::geometry::{Rectangle, Vec2D, Vec2DNumber};
use eframe::egui::{Pos2, Rect};
use num::NumCast;
use std::cmp::{max, min};

pub trait ToPos2 {
	fn to_pos2(&self) -> Pos2;
}

impl<T: Vec2DNumber> ToPos2 for Vec2D<T> {
	fn to_pos2(&self) -> Pos2 {
		Pos2 {
			x: <f32 as NumCast>::from(self.x).unwrap_or(0.0),
			y: <f32 as NumCast>::from(self.y).unwrap_or(0.0),
		}
	}
}

pub trait ToRect {
	fn to_rect(&self) -> Rect;
}

impl<T: Vec2DNumber> ToRect for Rectangle<T> {
	fn to_rect(&self) -> Rect {
		// EGUI requires min to be the top left corner and max to be the bottom right corner
		let min_position = Vec2D::new(min(self.min.x, self.max.x), min(self.min.y, self.max.y));
		let max_position = Vec2D::new(max(self.min.x, self.max.x), max(self.min.y, self.max.y));

		Rect {
			min: min_position.to_pos2(),
			max: max_position.to_pos2(),
		}
	}
}
