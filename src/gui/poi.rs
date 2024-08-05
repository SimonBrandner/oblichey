// As a Person of Interest fan, I decided it would be fun to draw Person of Interest-like
// squares around the detected faces, so that is what the code in this file

// This was adapted from https://gist.github.com/BarboraGrycova/76405f726249d44d167d46478202a965
// with author's permission

use crate::geometry::{Rectangle, Vec2D};
use num::abs;

const CORNER_RECTANGLE_SIZE: Vec2D<i32> = Vec2D { x: 20, y: 10 };
const CENTER_RECTANGLE_SIZE: Vec2D<i32> = Vec2D { x: 14, y: 4 };
const SIDE_RECTANGLE_SIZE: Vec2D<i32> = Vec2D { x: 12, y: 4 };
const MIN_SPACE_BETWEEN_SIDE_RECTANGLES: i32 = 6;

#[derive(PartialEq, Eq)]
enum Orientation {
	Horizontal,
	Vertical,
}

fn draw_corner_rectangles(
	top_left_position: &Vec2D<i32>,
	top_right_position: &Vec2D<i32>,
	bottom_left_position: &Vec2D<i32>,
	bottom_right_position: &Vec2D<i32>,
) -> Vec<Rectangle<i32>> {
	let mut rectangles = Vec::new();

	rectangles.push(Rectangle::new(
		top_left_position.clone(),
		*top_left_position + CORNER_RECTANGLE_SIZE,
	));
	rectangles.push(Rectangle::new(
		top_left_position.clone(),
		*top_left_position + CORNER_RECTANGLE_SIZE.with_flipped_axes(),
	));

	rectangles.push(Rectangle::new(
		top_right_position.clone(),
		Vec2D::new(
			top_right_position.x - CORNER_RECTANGLE_SIZE.x,
			top_right_position.y + CORNER_RECTANGLE_SIZE.y,
		),
	));
	rectangles.push(Rectangle::new(
		top_right_position.clone(),
		Vec2D::new(
			top_right_position.x - CORNER_RECTANGLE_SIZE.y,
			top_right_position.y + CORNER_RECTANGLE_SIZE.x,
		),
	));

	rectangles.push(Rectangle::new(
		bottom_left_position.clone(),
		Vec2D::new(
			bottom_left_position.x + CORNER_RECTANGLE_SIZE.x,
			bottom_left_position.y - CORNER_RECTANGLE_SIZE.y,
		),
	));
	rectangles.push(Rectangle::new(
		bottom_left_position.clone(),
		Vec2D::new(
			bottom_left_position.x + CORNER_RECTANGLE_SIZE.y,
			bottom_left_position.y - CORNER_RECTANGLE_SIZE.x,
		),
	));

	rectangles.push(Rectangle::new(
		bottom_right_position.clone(),
		*bottom_right_position - CORNER_RECTANGLE_SIZE,
	));
	rectangles.push(Rectangle::new(
		bottom_right_position.clone(),
		*bottom_right_position - CORNER_RECTANGLE_SIZE.with_flipped_axes(),
	));

	rectangles
}

fn draw_center_rectangles(
	square_size: &i32,
	small_rectangles_offset: &i32,
	top_left_position: &Vec2D<i32>,
	bottom_right_position: &Vec2D<i32>,
) -> Vec<Rectangle<i32>> {
	let center_rectangle_offset = square_size / 2 - (CENTER_RECTANGLE_SIZE.y / 2);
	let mut rectangles = Vec::new();

	let center_rectangle_left_position = Vec2D::new(
		top_left_position.x + small_rectangles_offset,
		top_left_position.y + center_rectangle_offset,
	);
	rectangles.push(Rectangle::new(
		center_rectangle_left_position,
		center_rectangle_left_position + CENTER_RECTANGLE_SIZE,
	));

	let center_rectangle_top_position = Vec2D::new(
		top_left_position.x + center_rectangle_offset,
		top_left_position.y + small_rectangles_offset,
	);
	rectangles.push(Rectangle::new(
		center_rectangle_top_position,
		center_rectangle_top_position + CENTER_RECTANGLE_SIZE.with_flipped_axes(),
	));

	let center_rectangle_right_position = Vec2D::new(
		bottom_right_position.x - small_rectangles_offset,
		bottom_right_position.y - center_rectangle_offset,
	);
	rectangles.push(Rectangle::new(
		center_rectangle_right_position,
		center_rectangle_right_position - CENTER_RECTANGLE_SIZE,
	));

	let center_rectangle_bottom_position = Vec2D::new(
		bottom_right_position.x - center_rectangle_offset,
		bottom_right_position.y - small_rectangles_offset,
	);
	rectangles.push(Rectangle::new(
		center_rectangle_bottom_position,
		center_rectangle_bottom_position - CENTER_RECTANGLE_SIZE.with_flipped_axes(),
	));

	rectangles
}

fn draw_side_rectangles_segment(
	number_of_small_rectangles: &i32,
	shift: &i32,
	small_rectangle_position: &Vec2D<i32>,
	orientation: Orientation,
) -> Vec<Rectangle<i32>> {
	let mut small_rectangle_position = small_rectangle_position.clone();
	let mut rectangles = Vec::new();
	for _ in 0..*number_of_small_rectangles {
		let mut small_rect = SIDE_RECTANGLE_SIZE;
		if orientation == Orientation::Vertical {
			small_rect = SIDE_RECTANGLE_SIZE.with_flipped_axes();
		}
		if *shift < 0 {
			small_rect = -small_rect;
		}

		rectangles.push(Rectangle::new(
			small_rectangle_position.clone(),
			small_rectangle_position.clone() + small_rect,
		));

		if orientation == Orientation::Horizontal {
			small_rectangle_position.x += shift;
		} else {
			small_rectangle_position.y += shift;
		}
	}
	rectangles
}

fn calculate_square_properties(
	rectangle: Rectangle<i32>,
) -> (i32, Vec2D<i32>, Vec2D<i32>, Vec2D<i32>, Vec2D<i32>) {
	let rectangle_size_x = abs(rectangle.max.x - rectangle.min.x);
	let rectangle_size_y = abs(rectangle.max.y - rectangle.min.y);

	let square_offset = (rectangle_size_x - rectangle_size_y) / 2;
	let (square_size, top_left_position) = if rectangle_size_x >= rectangle_size_y {
		(
			rectangle_size_x,
			Vec2D::new(rectangle.min.x, rectangle.min.y - square_offset),
		)
	} else {
		(
			rectangle_size_y,
			Vec2D::new(rectangle.min.x + square_offset, rectangle.min.y),
		)
	};

	let top_right_position = Vec2D::new(top_left_position.x + square_size, top_left_position.y);
	let bottom_left_position = Vec2D::new(top_left_position.x, top_left_position.y + square_size);
	let bottom_right_position =
		Vec2D::new(bottom_left_position.x + square_size, bottom_left_position.y);

	(
		square_size,
		top_left_position,
		top_right_position,
		bottom_right_position,
		bottom_left_position,
	)
}

fn draw_side_rectangles(
	square_size: &i32,
	small_rectangles_offset: &i32,
	top_left_position: &Vec2D<i32>,
	bottom_right_position: &Vec2D<i32>,
) -> Vec<Rectangle<i32>> {
	let small_rectangles_space =
		(square_size - (2 * CORNER_RECTANGLE_SIZE.x + CENTER_RECTANGLE_SIZE.y)) / 2;
	let number_of_small_rectangles = (small_rectangles_space as f32
		/ (SIDE_RECTANGLE_SIZE.x + MIN_SPACE_BETWEEN_SIDE_RECTANGLES) as f32)
		.floor() as i32;
	let number_of_spaces = number_of_small_rectangles + 1;
	let size_of_space = (small_rectangles_space
		- (number_of_small_rectangles * SIDE_RECTANGLE_SIZE.x))
		/ number_of_spaces;

	let mut rectangles = Vec::new();
	rectangles.extend(draw_side_rectangles_segment(
		&number_of_small_rectangles,
		&(size_of_space + SIDE_RECTANGLE_SIZE.x),
		&Vec2D::new(
			top_left_position.x + CORNER_RECTANGLE_SIZE.x + size_of_space,
			top_left_position.y + small_rectangles_offset,
		),
		Orientation::Horizontal,
	));
	rectangles.extend(draw_side_rectangles_segment(
		&number_of_small_rectangles,
		&(size_of_space + SIDE_RECTANGLE_SIZE.x),
		&Vec2D::new(
			top_left_position.x + ((square_size + CENTER_RECTANGLE_SIZE.y) / 2) + size_of_space,
			top_left_position.y + small_rectangles_offset,
		),
		Orientation::Horizontal,
	));
	rectangles.extend(draw_side_rectangles_segment(
		&number_of_small_rectangles,
		&(size_of_space + SIDE_RECTANGLE_SIZE.x),
		&Vec2D::new(
			top_left_position.x + small_rectangles_offset,
			top_left_position.y + CORNER_RECTANGLE_SIZE.x + size_of_space,
		),
		Orientation::Vertical,
	));
	rectangles.extend(draw_side_rectangles_segment(
		&number_of_small_rectangles,
		&(size_of_space + SIDE_RECTANGLE_SIZE.x),
		&Vec2D::new(
			top_left_position.x + small_rectangles_offset,
			top_left_position.y + ((square_size + CENTER_RECTANGLE_SIZE.y) / 2) + size_of_space,
		),
		Orientation::Vertical,
	));
	rectangles.extend(draw_side_rectangles_segment(
		&number_of_small_rectangles,
		&(-(size_of_space + SIDE_RECTANGLE_SIZE.x)),
		&Vec2D::new(
			bottom_right_position.x - CORNER_RECTANGLE_SIZE.x - size_of_space,
			bottom_right_position.y - small_rectangles_offset,
		),
		Orientation::Horizontal,
	));
	rectangles.extend(draw_side_rectangles_segment(
		&number_of_small_rectangles,
		&(-(size_of_space + SIDE_RECTANGLE_SIZE.x)),
		&Vec2D::new(
			bottom_right_position.x - ((square_size + CENTER_RECTANGLE_SIZE.y) / 2) - size_of_space,
			bottom_right_position.y - small_rectangles_offset,
		),
		Orientation::Horizontal,
	));
	rectangles.extend(draw_side_rectangles_segment(
		&number_of_small_rectangles,
		&(-(size_of_space + SIDE_RECTANGLE_SIZE.x)),
		&Vec2D::new(
			bottom_right_position.x - small_rectangles_offset,
			bottom_right_position.y - CORNER_RECTANGLE_SIZE.x - size_of_space,
		),
		Orientation::Vertical,
	));
	rectangles.extend(draw_side_rectangles_segment(
		&number_of_small_rectangles,
		&(-(size_of_space + SIDE_RECTANGLE_SIZE.x)),
		&Vec2D::new(
			bottom_right_position.x - small_rectangles_offset,
			bottom_right_position.y - ((square_size + CENTER_RECTANGLE_SIZE.y) / 2) - size_of_space,
		),
		Orientation::Vertical,
	));

	rectangles
}

/// As a Person of Interest fan, I decided it would be fun to draw Person of Interest-like
/// squares around the detected faces, so that is what this function does
pub fn draw_poi_square(rectangle: Rectangle<u32>) -> (Vec<Rectangle<i32>>, Vec2D<i32>) {
	let small_rectangles_offset = (CORNER_RECTANGLE_SIZE.y - SIDE_RECTANGLE_SIZE.y) / 2;
	let (
		square_size,
		top_left_position,
		top_right_position,
		bottom_right_position,
		bottom_left_position,
	) = calculate_square_properties(rectangle.to_i32());

	let mut rectangles = Vec::new();
	rectangles.extend(draw_corner_rectangles(
		&top_left_position,
		&top_right_position,
		&bottom_left_position,
		&bottom_right_position,
	));
	rectangles.extend(draw_center_rectangles(
		&square_size,
		&small_rectangles_offset,
		&top_left_position,
		&bottom_right_position,
	));
	rectangles.extend(draw_side_rectangles(
		&square_size,
		&small_rectangles_offset,
		&top_left_position,
		&bottom_right_position,
	));

	(rectangles, top_right_position)
}
