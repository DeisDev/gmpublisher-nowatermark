use image::{imageops, imageops::FilterType, Rgba, RgbaImage};
use steamworks::SteamId;

/// The preview size Steam recommends for Workshop items
const ICON_SIZE: u32 = 512;
/// The icon is rendered at a multiple of its final size and downscaled, which keeps the curves smooth
const SUPERSAMPLE: u32 = 2;

/// Radius of the avatar itself, in final pixels, leaving room for the outline
const AVATAR_RADIUS: f32 = 208.0;
/// Thickness of the white outline drawn around the avatar
const OUTLINE_THICKNESS: f32 = 24.0;
/// How far the drop shadow fades out past the outline
const SHADOW_SPREAD: f32 = 14.0;
const SHADOW_ALPHA: f32 = 0.3;

/// Used when Steam can't give us the user's avatar
const FALLBACK_AVATAR: &[u8] = include_bytes!("../../../public/img/steam_anonymous.jpg");

/// Draws the default Workshop preview image: the user's Steam avatar, outlined in white.
pub fn compose() -> RgbaImage {
	compose_with(avatar())
}

fn compose_with(avatar: Option<RgbaImage>) -> RgbaImage {
	let avatar = avatar.unwrap_or_else(fallback_avatar);

	let scale = SUPERSAMPLE as f32;
	let size = ICON_SIZE * SUPERSAMPLE;
	let center = size as f32 / 2.0;

	let avatar_radius = AVATAR_RADIUS * scale;
	let outline_radius = avatar_radius + OUTLINE_THICKNESS * scale;
	let shadow_radius = outline_radius + SHADOW_SPREAD * scale;

	let mut canvas = RgbaImage::new(size, size);

	// The shadow and the outline go down first, so that the avatar covers their inner edges
	let from = (center - shadow_radius).floor().max(0.0) as u32;
	let to = (center + shadow_radius).ceil().min(size as f32) as u32;

	for y in from..to {
		let dy = y as f32 + 0.5 - center;
		for x in from..to {
			let dx = x as f32 + 0.5 - center;
			let distance = (dx * dx + dy * dy).sqrt();

			if distance > shadow_radius {
				continue;
			}

			let pixel = canvas.get_pixel_mut(x, y);

			let fade = ((shadow_radius - distance) / (SHADOW_SPREAD * scale)).clamp(0.0, 1.0);
			blend(pixel, [0, 0, 0], SHADOW_ALPHA * fade * fade);

			let outline = coverage(outline_radius, distance) - coverage(avatar_radius, distance);
			blend(pixel, [255, 255, 255], outline);
		}
	}

	let diameter = (avatar_radius * 2.0).round() as u32;
	let avatar = imageops::resize(&avatar, diameter, diameter, FilterType::CatmullRom);
	let offset = (size - diameter) / 2;

	for (x, y, source) in avatar.enumerate_pixels() {
		let dx = (offset + x) as f32 + 0.5 - center;
		let dy = (offset + y) as f32 + 0.5 - center;
		let mask = coverage(avatar_radius, (dx * dx + dy * dy).sqrt());

		if mask <= 0.0 {
			continue;
		}

		let alpha = mask * (source[3] as f32 / 255.0);
		blend(canvas.get_pixel_mut(offset + x, offset + y), [source[0], source[1], source[2]], alpha);
	}

	imageops::resize(&canvas, ICON_SIZE, ICON_SIZE, FilterType::Lanczos3)
}

/// How much of a pixel covers a disc of the given radius, giving the edges a pixel of anti-aliasing
#[inline]
fn coverage(radius: f32, distance: f32) -> f32 {
	(radius + 0.5 - distance).clamp(0.0, 1.0)
}

/// Source-over compositing of a straight-alpha colour onto a pixel
#[inline]
fn blend(pixel: &mut Rgba<u8>, color: [u8; 3], alpha: f32) {
	let alpha = alpha.clamp(0.0, 1.0);
	if alpha <= 0.0 {
		return;
	}

	let dst_alpha = pixel[3] as f32 / 255.0;
	let out_alpha = alpha + dst_alpha * (1.0 - alpha);

	if out_alpha <= 0.0 {
		*pixel = Rgba([0, 0, 0, 0]);
		return;
	}

	for channel in 0..3 {
		let mixed = color[channel] as f32 * alpha + pixel[channel] as f32 * dst_alpha * (1.0 - alpha);
		pixel[channel] = (mixed / out_alpha).round().clamp(0.0, 255.0) as u8;
	}

	pixel[3] = (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8;
}

/// The current user's Steam avatar, preferring the largest one Steam offers
fn avatar() -> Option<RgbaImage> {
	if !steam!().connected() {
		return None;
	}

	let steam_id = steam!().client().steam_id;

	if let Some(avatar) = avatar_of(steam_id) {
		return Some(avatar);
	}

	// Steam may still be downloading the avatar, so request it and give it a moment
	if steam!().client().friends().request_user_information(steam_id, false) {
		for _ in 0..10 {
			sleep_ms!(100);
			if let Some(avatar) = avatar_of(steam_id) {
				return Some(avatar);
			}
		}
	}

	None
}

fn avatar_of(steam_id: SteamId) -> Option<RgbaImage> {
	let friend = steam!().client().friends().get_friend(steam_id);

	// These come back as raw RGBA buffers, at the sizes the Steam API documents
	friend
		.large_avatar()
		.map(|buffer| (buffer, 184))
		.or_else(|| friend.medium_avatar().map(|buffer| (buffer, 64)))
		.or_else(|| friend.small_avatar().map(|buffer| (buffer, 32)))
		.and_then(|(buffer, size)| RgbaImage::from_raw(size, size, buffer))
}

fn fallback_avatar() -> RgbaImage {
	image::load_from_memory_with_format(FALLBACK_AVATAR, image::ImageFormat::Jpeg)
		.map(|avatar| avatar.to_rgba8())
		.unwrap_or_else(|_| RgbaImage::from_pixel(64, 64, Rgba([45, 45, 45, 255])))
}

#[tauri::command]
pub fn default_workshop_icon() -> Option<crate::Base64Image> {
	Some(crate::Base64Image::new(compose().into_raw(), ICON_SIZE, ICON_SIZE))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn outlines_the_avatar_in_white() {
		let icon = compose_with(None);

		assert_eq!(icon.dimensions(), (ICON_SIZE, ICON_SIZE));

		// Outside of the drop shadow
		assert_eq!(icon.get_pixel(0, 0)[3], 0);
		assert_eq!(icon.get_pixel(ICON_SIZE / 2, 0)[3], 0);

		// Halfway through the outline
		let outline = icon.get_pixel(ICON_SIZE / 2, (ICON_SIZE as f32 / 2.0 - (AVATAR_RADIUS + OUTLINE_THICKNESS / 2.0)) as u32);
		assert_eq!(outline.0, [255, 255, 255, 255]);

		// The avatar itself
		assert_eq!(icon.get_pixel(ICON_SIZE / 2, ICON_SIZE / 2)[3], 255);
	}
}
