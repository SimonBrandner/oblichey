use crate::{camera::Frame, geometry::Vec2D, processors::frame_processor::DETECTOR_INPUT_SIZE};
use image::{
	imageops::{crop, resize, FilterType},
	ImageBuffer,
};
use rayon::prelude::*;

pub fn brightness(grey_data: &[u8], size: &Vec2D<u32>) -> f32 {
	let mut sum = 0.0;
	for pixel in grey_data {
		sum += *pixel as f32;
	}

	sum / (size.x * size.y) as f32
}

pub fn convert_grey_to_rgb(grey_data: &[u8], size: &Vec2D<u32>) -> Frame {
	let mut rgb_data = vec![0; (size.x * size.y * 3) as usize];

	rgb_data
		.par_chunks_mut(3)
		.enumerate()
		.for_each(|(i, chunk)| {
			let grey_value = grey_data[i];
			chunk[0] = grey_value;
			chunk[1] = grey_value;
			chunk[2] = grey_value;
		});

	ImageBuffer::from_vec(size.x, size.y, rgb_data)
		.expect("Something went awry during image conversion")
}

pub fn convert_yuyv_to_rgb(yuyv: &[u8], size: &Vec2D<u32>) -> Frame {
	let mut rgb_data = vec![0; (size.x * size.y * 3) as usize];

	rgb_data
		.par_chunks_exact_mut(6)
		.enumerate()
		.for_each(|(i, chunk)| {
			let yuyv_chunk = &yuyv[i * 4..i * 4 + 4];
			let y0 = yuyv_chunk[0] as i32;
			let u = yuyv_chunk[1] as i32 - 128;
			let y1 = yuyv_chunk[2] as i32;
			let v = yuyv_chunk[3] as i32 - 128;

			let (r0, g0, b0) = convert_pixel_yuyv_to_rgb(y0, u, v);
			let (r1, g1, b1) = convert_pixel_yuyv_to_rgb(y1, u, v);

			chunk[0..3].copy_from_slice(&[r0, g0, b0]);
			chunk[3..6].copy_from_slice(&[r1, g1, b1]);
		});

	ImageBuffer::from_vec(size.x, size.y, rgb_data)
		.expect("Something went awry during image conversion")
}

fn convert_pixel_yuyv_to_rgb(y: i32, u: i32, v: i32) -> (u8, u8, u8) {
	let c = y - 16;
	let d = u;
	let e = v;

	let r = (298 * c + 409 * e + 128) >> 8;
	let g = (298 * c - 100 * d - 208 * e + 128) >> 8;
	let b = (298 * c + 516 * d + 128) >> 8;

	(
		r.clamp(0, 255) as u8,
		g.clamp(0, 255) as u8,
		b.clamp(0, 255) as u8,
	)
}

pub fn reshape_frame(frame: Frame, frame_size: &Vec2D<u32>) -> Frame {
	let original_aspect_ratio = frame_size.x as f32 / frame_size.y as f32;
	let model_aspect_ratio = DETECTOR_INPUT_SIZE.x as f32 / DETECTOR_INPUT_SIZE.y as f32;
	let (new_size, new_offset) = if original_aspect_ratio > model_aspect_ratio {
		let size = Vec2D::new(
			((DETECTOR_INPUT_SIZE.y as f32 / frame_size.y as f32) * frame_size.x as f32) as u32,
			DETECTOR_INPUT_SIZE.y,
		);
		let offset = Vec2D::new((size.x - DETECTOR_INPUT_SIZE.x) / 2, 0);

		(size, offset)
	} else {
		let size = Vec2D::new(
			DETECTOR_INPUT_SIZE.x,
			((DETECTOR_INPUT_SIZE.x as f32 / frame_size.x as f32) * frame_size.y as f32) as u32,
		);
		let offset = Vec2D::new(0, (size.y - DETECTOR_INPUT_SIZE.y) / 2);

		(size, offset)
	};

	let mut resized = resize(
		&frame,
		new_size.x as u32,
		new_size.y as u32,
		FilterType::CatmullRom,
	);

	let cropped = crop(
		&mut resized,
		new_offset.x as u32,
		new_offset.y as u32,
		DETECTOR_INPUT_SIZE.x as u32,
		DETECTOR_INPUT_SIZE.y as u32,
	);

	cropped.to_image()
}
