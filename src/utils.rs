use image::{ImageBuffer, Rgba};

// This has been generated using ChatGPT, it's slow but it'll do for now
pub fn convert_yuyv_to_rgba(
	yuyv: &[u8],
	width: u32,
	height: u32,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
	let mut rgba_data = vec![0; (width * height * 4) as usize];

	for (i, chunk) in yuyv.chunks_exact(4).enumerate() {
		let y0 = chunk[0] as i32;
		let u = chunk[1] as i32 - 128;
		let y1 = chunk[2] as i32;
		let v = chunk[3] as i32 - 128;

		let (r0, g0, b0) = convert_pixel_yuyv_to_rgba(y0, u, v);
		let (r1, g1, b1) = convert_pixel_yuyv_to_rgba(y1, u, v);

		let rgba_index = i * 8;
		rgba_data[rgba_index..rgba_index + 4].copy_from_slice(&[r0, g0, b0, 255]);
		rgba_data[rgba_index + 4..rgba_index + 8].copy_from_slice(&[r1, g1, b1, 255]);
	}

	ImageBuffer::from_vec(width, height, rgba_data).unwrap()
}

fn convert_pixel_yuyv_to_rgba(y: i32, u: i32, v: i32) -> (u8, u8, u8) {
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
