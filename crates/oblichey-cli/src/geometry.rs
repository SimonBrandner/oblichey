use num::{NumCast, Unsigned, Zero};
use std::{
	fmt::Debug,
	ops::{Add, Mul, Neg, Sub},
};

const INTERSECTION_OVER_UNION_THRESHOLD: f32 = 0.5;

pub trait Vec2DNumber
where
	Self: NumCast
		+ Add<Output = Self>
		+ Mul<Output = Self>
		+ Sub<Output = Self>
		+ Ord
		+ Zero
		+ Clone
		+ Debug
		+ Copy,
{
}

impl<T> Vec2DNumber for T where
	Self: NumCast
		+ Add<Output = Self>
		+ Mul<Output = Self>
		+ Sub<Output = Self>
		+ Ord
		+ Zero
		+ Clone
		+ Debug
		+ Copy
{
}

/// Calculates distance between two points (min and max)
fn calculate_distance<T: Vec2DNumber>(min: T, max: T) -> Option<T> {
	let min_f32 = <f32 as NumCast>::from(min)?;
	let max_f32 = <f32 as NumCast>::from(max)?;
	<T as NumCast>::from((max_f32 - min_f32).abs())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2D<T: Vec2DNumber> {
	pub x: T,
	pub y: T,
}

impl<T: Vec2DNumber> Vec2D<T> {
	pub const fn new(x: T, y: T) -> Self {
		Self { x, y }
	}

	pub fn to_i32(self) -> Option<Vec2D<i32>> {
		Some(Vec2D::new(
			<i32 as NumCast>::from(self.x)?,
			<i32 as NumCast>::from(self.y)?,
		))
	}

	/// Returns a new `Vec2D` where the x and y axes are switched (x becomes y and y becomes x)
	pub const fn with_flipped_axes(&self) -> Self {
		Self::new(self.y, self.x)
	}
}

impl<T: Vec2DNumber> Add for Vec2D<T> {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl<T: Vec2DNumber> Sub for Vec2D<T> {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}

impl<T: Vec2DNumber + Neg<Output = T>> Neg for Vec2D<T> {
	type Output = Self;

	fn neg(self) -> Self::Output {
		Self {
			x: -self.x,
			y: -self.y,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle<T: Vec2DNumber> {
	pub min: Vec2D<T>,
	pub max: Vec2D<T>,
}

impl<T: Vec2DNumber> Rectangle<T> {
	pub const fn new(min: Vec2D<T>, max: Vec2D<T>) -> Self {
		Self { min, max }
	}

	pub fn to_i32(self) -> Option<Rectangle<i32>> {
		Some(Rectangle::new(self.min.to_i32()?, self.max.to_i32()?))
	}

	pub fn width(&self) -> Option<T> {
		calculate_distance(self.min.x, self.max.x)
	}

	pub fn height(&self) -> Option<T> {
		calculate_distance(self.min.y, self.max.y)
	}

	pub fn size(&self) -> Option<Vec2D<T>> {
		Some(Vec2D::new(self.width()?, self.height()?))
	}
}

impl<T: Vec2DNumber + Unsigned> Rectangle<T> {
	/// Takes two `Rectangle`s and calculates the ratio between the area of their intersection and
	/// the area of their union
	pub fn intersection_over_union(&self, other: &Self) -> Option<f32> {
		if self.min.x > other.max.x {
			return Some(0.0);
		}
		if self.min.y > other.max.y {
			return Some(0.0);
		}
		if other.min.x > self.max.x {
			return Some(0.0);
		}
		if other.min.y > self.max.y {
			return Some(0.0);
		}

		let intersection_min_x = std::cmp::max(self.min.x, other.min.x);
		let intersection_min_y = std::cmp::max(self.min.y, other.min.y);
		let intersection_max_x = std::cmp::min(self.max.x, other.max.x);
		let intersection_max_y = std::cmp::min(self.max.y, other.max.y);

		let intersection_width = std::cmp::max(intersection_max_x - intersection_min_x, T::zero());
		let intersection_height = std::cmp::max(intersection_max_y - intersection_min_y, T::zero());
		let intersection_area = intersection_width * intersection_height;

		let self_area = (self.max.x - self.min.x) * (self.max.y - self.min.y);
		let other_area = (other.max.x - other.min.x) * (other.max.y - other.min.y);

		let union_area = <f32 as NumCast>::from(self_area + other_area - intersection_area)?;
		if union_area == 0.0 {
			return None;
		}

		Some(<f32 as NumCast>::from(intersection_area)? / union_area)
	}

	/// When multiple `Rectangle`s are stacked on top of each other, this function reduces them to
	/// just one
	pub fn filter_out_colliding(rectangles: &mut Vec<Self>) {
		let mut i = 0;
		while i < rectangles.len() {
			let mut j = i + 1;
			while j < rectangles.len() {
				if rectangles[i]
					.intersection_over_union(&rectangles[j])
					.map_or(false, |i| i > INTERSECTION_OVER_UNION_THRESHOLD)
				{
					rectangles.remove(j);
					j -= 1;
				}
				j += 1;
			}
			i += 1;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::{Rectangle, Vec2D};
	use core::f32;

	#[test]
	fn calculates_rectangle_size() {
		let test_cases = vec![
			(Vec2D::new(0, 0), Vec2D::new(1, 1), Vec2D::new(1, 1)),
			(Vec2D::new(2, 3), Vec2D::new(3, 2), Vec2D::new(1, 1)),
			(Vec2D::new(-5, -4), Vec2D::new(5, 4), Vec2D::new(10, 8)),
			(Vec2D::new(-4, 3), Vec2D::new(4, 3), Vec2D::new(8, 0)),
			(Vec2D::new(4, -3), Vec2D::new(-4, 3), Vec2D::new(8, 6)),
		];

		for (min, max, expected_size) in test_cases {
			let rectangle = Rectangle::new(min, max);
			let size = rectangle.size().expect("Failed to calculate size");

			assert_eq!(size, expected_size);
		}
	}

	#[test]
	fn calculates_intersection_over_union() {
		let test_cases: Vec<(Rectangle<u32>, Rectangle<u32>, f32)> = vec![
			(
				Rectangle::new(Vec2D::new(0, 0), Vec2D::new(1, 1)),
				Rectangle::new(Vec2D::new(0, 0), Vec2D::new(1, 1)),
				1.0,
			),
			(
				Rectangle::new(Vec2D::new(0, 0), Vec2D::new(3, 1)),
				Rectangle::new(Vec2D::new(1, 0), Vec2D::new(4, 1)),
				0.5,
			),
			(
				Rectangle::new(Vec2D::new(0, 0), Vec2D::new(1, 1)),
				Rectangle::new(Vec2D::new(1, 1), Vec2D::new(2, 2)),
				0.0,
			),
		];

		for (rectangle_a, rectangle_b, expected_result) in test_cases {
			let result = rectangle_a
				.intersection_over_union(&rectangle_b)
				.expect("Failed to calculate intersection over union");
			assert!((result - expected_result).abs() <= f32::EPSILON);
		}
	}

	#[test]
	fn filters_out_colliding_rectangles() {
		let test_cases: Vec<(Vec<Rectangle<u32>>, usize)> = vec![
			(
				vec![
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(1, 1)),
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(1, 1)),
				],
				1,
			),
			(
				vec![
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(1, 1)),
					Rectangle::new(Vec2D::new(1, 1), Vec2D::new(2, 2)),
				],
				2,
			),
			(
				vec![
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(1, 1)),
					Rectangle::new(Vec2D::new(0, 0), Vec2D::new(1, 1)),
					Rectangle::new(Vec2D::new(2, 2), Vec2D::new(3, 3)),
					Rectangle::new(Vec2D::new(2, 2), Vec2D::new(3, 3)),
				],
				2,
			),
		];

		for (rectangles, expected_len) in test_cases {
			let mut rectangles = rectangles.clone();
			Rectangle::filter_out_colliding(&mut rectangles);
			assert_eq!(rectangles.len(), expected_len);
		}
	}
}
