use num::{NumCast, Zero};
use std::{
	fmt::Debug,
	ops::{Add, Mul, Neg, Sub},
};

pub trait Vec2DNumber
where
	Self: NumCast,
	Self: Add<Output = Self>,
	Self: Mul<Output = Self>,
	Self: Sub<Output = Self>,
	Self: Ord,
	Self: Zero,
	Self: Clone,
	Self: Debug,
	Self: Copy,
{
}

impl<T> Vec2DNumber for T
where
	T: NumCast,
	T: Add<Output = T>,
	T: Mul<Output = T>,
	T: Sub<Output = T>,
	T: Ord,
	T: Zero,
	T: Clone,
	T: Debug,
	T: Copy,
{
}

fn calculate_length<T: Vec2DNumber>(min: T, max: T) -> T {
	let min_f32 = <f32 as NumCast>::from(min).unwrap_or(0.0);
	let max_f32 = <f32 as NumCast>::from(max).unwrap_or(0.0);
	<T as NumCast>::from((max_f32 - min_f32).abs()).unwrap_or(T::zero())
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2D<T: Vec2DNumber> {
	pub x: T,
	pub y: T,
}

impl<T: Vec2DNumber> Vec2D<T> {
	pub fn new(x: T, y: T) -> Self {
		Self { x, y }
	}

	pub fn to_i32(&self) -> Vec2D<i32> {
		Vec2D::new(
			<i32 as NumCast>::from(self.x).unwrap_or(0),
			<i32 as NumCast>::from(self.y).unwrap_or(0),
		)
	}

	pub fn with_flipped_axes(&self) -> Self {
		Self::new(self.y, self.x)
	}
}

impl<T: Vec2DNumber> Add for Vec2D<T> {
	type Output = Vec2D<T>;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl<T: Vec2DNumber> Sub for Vec2D<T> {
	type Output = Vec2D<T>;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}

impl<T: Vec2DNumber + Neg<Output = T>> Neg for Vec2D<T> {
	type Output = Vec2D<T>;

	fn neg(self) -> Self::Output {
		Self {
			x: -self.x,
			y: -self.y,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle<T: Vec2DNumber> {
	pub min: Vec2D<T>,
	pub max: Vec2D<T>,
}

impl<T: Vec2DNumber> Rectangle<T> {
	pub fn new(min: Vec2D<T>, max: Vec2D<T>) -> Self {
		Self { min, max }
	}

	pub fn to_i32(&self) -> Rectangle<i32> {
		Rectangle::new(self.min.to_i32(), self.max.to_i32())
	}

	pub fn width(&self) -> T {
		calculate_length(self.min.x, self.max.x)
	}

	pub fn height(&self) -> T {
		calculate_length(self.min.y, self.max.y)
	}

	pub fn size(&self) -> (T, T) {
		(self.width(), self.height())
	}

	pub fn intersection_over_union(&self, other: &Rectangle<T>) -> f32 {
		if self.min.x > other.max.x {
			return 0.0;
		}
		if self.min.y > other.max.y {
			return 0.0;
		}
		if other.min.x > self.max.x {
			return 0.0;
		}
		if other.min.y > self.max.y {
			return 0.0;
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

		let union_area =
			<f32 as NumCast>::from(self_area + other_area - intersection_area).unwrap_or(0.0);

		if union_area == 0.0 {
			0.0
		} else {
			<f32 as NumCast>::from(intersection_area).unwrap_or(0.0) / union_area
		}
	}
}
