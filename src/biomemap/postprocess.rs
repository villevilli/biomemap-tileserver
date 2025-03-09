use std::{
    ops::{Deref, DerefMut},
    sync::LazyLock,
};

use cubiomes::{
    colors::BiomeColorMap,
    generator::{Cache, Range, Scale},
    noise::{BiomeNoise, SurfaceNoiseRelease},
};
use image::{GrayImage, ImageBuffer, Rgb, RgbImage, imageops::resize};

use super::CachePool;

static COLOR_MAP: LazyLock<BiomeColorMap> = std::sync::LazyLock::new(BiomeColorMap::new);

pub fn get_image(x: i32, y: i32, cache_pool: &CachePool, scale: Scale) -> RgbImage {
    cache_pool
        .get(x * 256, 320, y * 256, scale)
        .unwrap()
        .to_image(*COLOR_MAP)
}

pub fn concat_lower_zoom(x: i32, y: i32, cache_pool: &CachePool, scale: Scale) -> RgbImage {
    let mut img = RgbImage::new(256, 256);

    for img_x in 0..=1 {
        for img_y in 0..=1 {
            let cache = cache_pool
                .get(
                    ((x * 256) * (2)) + (img_x * 256),
                    320,
                    ((y * 256) * (2)) + (img_y * 256),
                    scale,
                )
                .unwrap();

            for sub_img_x in 0..(256 / (2)) {
                for sub_img_y in 0..(256 / (2)) {
                    *img.get_pixel_mut(
                        (sub_img_x + ((256 / (2)) * img_x)) as u32,
                        sub_img_y + (256 / (2) * img_y) as u32,
                    ) = Rgb::from(
                        COLOR_MAP[cache
                            .biome_at((sub_img_x * (2)) as u32, 0, sub_img_y * (2))
                            .unwrap()],
                    );
                }
            }
        }
    }
    img
}

pub fn upsacale_blockscale(x: i32, y: i32, zoom: i32, cache_pool: &CachePool) -> RgbImage {
    let tilecount = 2_u32.pow(zoom as u32);

    let size = 256 / tilecount;

    let img = resize(
        &Cache::new(
            cache_pool.as_generatr_ref(),
            Range {
                x: x * size as i32,
                y: 320,
                z: y * size as i32,
                size_x: size,
                size_y: 0,
                size_z: size,
                scale: Scale::Block,
            },
        )
        .unwrap()
        .to_image(*COLOR_MAP),
        256,
        256,
        image::imageops::FilterType::Nearest,
    );

    if zoom >= 0 {

        //img.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
        //    if x == 0 || y == 0 {
        //        pixel.0 = [0; 3]
        //    }
        //});
    }

    img
}

pub fn draw_contours<Levels, Pixel, Container>(
    heightmap: &GrayImage,
    levels: Levels,
    tile: &mut ImageBuffer<Pixel, Container>,
    brightness: u8,
    alpha: u8,
) where
    Levels: Iterator<Item = u8>,
    Pixel: image::Pixel,
    Container: Deref<Target = [Pixel::Subpixel]> + DerefMut,
    Pixel::Subpixel: From<u8>,
{
    let mut map: Vec<bool> = Vec::new();
    let w = heightmap.width() as usize;

    for i in levels {
        higher_lower(heightmap, i, &mut map);

        tile.enumerate_pixels_mut().for_each(|(x, y, pixel)| {
            let this_pixel = map[calc_2d_index(x as usize + 1, w, y as usize + 1)];
            let above_pixel = map[calc_2d_index(x as usize + 1, w, y as usize)];
            let left_pixel = map[calc_2d_index(x as usize, w, y as usize + 1)];

            if this_pixel != above_pixel || this_pixel != left_pixel {
                pixel.apply_with_alpha(|_| brightness.into(), |_| alpha.into());
            }
        });
    }
}

pub fn generate_heightmap(x: i32, y: i32, zoom: i32, cache_pool: &CachePool) -> GrayImage {
    let rel_zoom = zoom + 2;

    let scale = 2_u32.pow((rel_zoom).unsigned_abs());

    let noise: BiomeNoise = SurfaceNoiseRelease::new(
        cache_pool.as_generatr_ref().dimension(),
        cache_pool.as_generatr_ref().seed(),
    )
    .into();

    let scaled_x;
    let scaled_y;

    if rel_zoom.is_negative() {
        scaled_x = x * scale as i32;
        scaled_y = y * scale as i32;
    } else {
        scaled_x = x / scale as i32;
        scaled_y = y / scale as i32;
    }

    GrayImage::from_fn(256 + 2, 256 + 2, |img_x, img_y| {
        let offset_x;
        let offset_y;

        if rel_zoom.is_negative() {
            offset_x = img_x * scale;
            offset_y = img_y * scale;
        } else {
            offset_x = img_x / scale;
            offset_y = img_y / scale;
        }
        [((cache_pool
            .as_generatr_ref()
            .approx_surface_noise(
                offset_x as i32 + (scaled_x),
                offset_y as i32 + (scaled_y),
                1,
                1,
                &noise,
            )
            .unwrap()[0]
            * (320.0 / 255.0))
            .clamp(0.0, 255.0) as u8)]
        .into()
    })
}

fn calc_2d_index(x: usize, width: usize, y: usize) -> usize {
    y * width + x
}

fn higher_lower(map: &[u8], targe_height: u8, buf: &mut Vec<bool>) {
    buf.reserve(map.len());
    buf.clear();

    map.iter().for_each(|h| buf.push(*h < targe_height));
}

/// Draws contour lines onto the given image at the zoom level.
///
/// Heightmap must be big enough and should begin one left and end one right of the are in the image
pub fn draw_shading(heightmap: &GrayImage, tile: &mut RgbImage, strenght: i8) {
    let tile_scale = 1;
    let (w, h) = tile.dimensions();

    for x in 0..h {
        for y in 0..w {
            let stroke = strenght;

            match height_change_x(x, y, heightmap) {
                Direction::Higher(n) => {
                    shift_lightness(
                        tile,
                        x,
                        y,
                        tile_scale,
                        tile_scale,
                        -height_diff_shade_calculator(stroke, n),
                    );
                }
                Direction::Lower(n) => {
                    shift_lightness(
                        tile,
                        x,
                        y,
                        tile_scale,
                        tile_scale,
                        height_diff_shade_calculator(stroke, n),
                    );
                }
                Direction::Flat => (),
            }

            match height_change_y(x, y, heightmap) {
                Direction::Higher(n) => {
                    shift_lightness(
                        tile,
                        x,
                        y,
                        tile_scale,
                        tile_scale,
                        -height_diff_shade_calculator(stroke, n),
                    );
                }
                Direction::Lower(n) => {
                    shift_lightness(
                        tile,
                        x,
                        y,
                        tile_scale,
                        tile_scale,
                        height_diff_shade_calculator(stroke, n),
                    );
                }
                Direction::Flat => (),
            }
        }
    }
}

fn height_diff_shade_calculator(stroke: i8, h_diff: i8) -> i8 {
    stroke.saturating_add(h_diff / stroke)
}

fn shift_lightness(
    img: &mut RgbImage,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    shift_amount: i8,
) {
    let (imgh, imgw) = img.dimensions();
    for y in start_y..((start_y + width).min(imgh)) {
        for x in start_x..((start_x + height).min(imgw)) {
            img.get_pixel_mut(x, y)
                .0
                .iter_mut()
                .for_each(|pixel| *pixel = pixel.saturating_add_signed(shift_amount));
        }
    }
}

#[derive(PartialEq, Eq)]
enum Direction {
    Higher(i8),
    Flat,
    Lower(i8),
}

/// Gets heightmap change from this and left (eg is left higher flat or lower than self)
fn height_change_x(hmx: u32, hmy: u32, heightmap: &GrayImage) -> Direction {
    let cmp_point = heightmap.get_pixel(hmx + 1, hmy).0[0];
    let left_point = heightmap.get_pixel(hmx, hmy).0[0];
    dir(left_point, cmp_point)
}

/// Gets heightmap change from this and above (eg is abover higher flat or lower than self)
fn height_change_y(hmx: u32, hmy: u32, heightmap: &GrayImage) -> Direction {
    let cmp_point = heightmap.get_pixel(hmx, hmy + 1).0[0];
    let up_point = heightmap.get_pixel(hmx, hmy).0[0];
    dir(up_point, cmp_point)
}

fn dir(x1: u8, x2: u8) -> Direction {
    if x1 == x2 {
        Direction::Flat
    } else {
        match x1 < x2 {
            true => Direction::Lower((x2 - x1) as i8),
            false => Direction::Higher((x1 - x2) as i8),
        }
    }
}
