use crate::camera::Frame;
use image::ImageBuffer;
use rayon::prelude::*;

pub fn brightness(grey_data: &[u8], width: u32, height: u32) -> f32 {
	let mut sum = 0.0;
	for pixel in grey_data {
		sum += *pixel as f32;
	}

	sum / (width * height) as f32
}

pub fn convert_grey_to_rgb(grey_data: &[u8], width: u32, height: u32) -> Frame {
	let mut rgb_data = vec![0; (width * height * 3) as usize];

	rgb_data
		.par_chunks_mut(3)
		.enumerate()
		.for_each(|(i, chunk)| {
			let grey_value = grey_data[i];
			chunk[0] = grey_value;
			chunk[1] = grey_value;
			chunk[2] = grey_value;
		});

	ImageBuffer::from_vec(width, height, rgb_data)
		.expect("Something went awry during image conversion")
}

pub fn convert_yuyv_to_rgb(yuyv: &[u8], width: u32, height: u32) -> Frame {
	let mut rgb_data = vec![0; (width * height * 3) as usize];

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

	ImageBuffer::from_vec(width, height, rgb_data)
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
