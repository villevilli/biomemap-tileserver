use std::sync::LazyLock;

use cubiomes::{
    colors::BiomeColorMap,
    generator::{Cache, Range, Scale},
};
use image::{GrayImage, Rgb, RgbImage, imageops::resize};

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

pub fn draw_shading(tile: &mut RgbImage, x: i32, y: i32, zoom: i32, cache_pool: &CachePool) {
    let tilecount = 2_u32.pow(zoom.unsigned_abs());

    let size = if zoom.is_negative() {
        256 * tilecount
    } else {
        256 / tilecount
    };

    let heightmap = cache_pool
        .as_generatr_ref()
        .generate_heightmap_image(
            x * (size / 4) as i32 - 1,
            y * (size / 4) as i32 - 1,
            ((size / 4) + 3).max(3),
            ((size / 4) + 3).max(3),
            0.0,
            320.0,
        )
        .unwrap();

    raw_draw_shading(&heightmap, tile, zoom, 24);
}

/// Draws contour lines onto the given image at the zoom level.
///
/// Heightmap must be big enough and should begin one left and end one right of the are in the image
fn raw_draw_shading(heightmap: &GrayImage, tile: &mut RgbImage, zoom: i32, strenght: i8) {
    let w;
    let h;
    let tile_scale;
    let rel_zoom = zoom + 2;

    let heightmap_to_img_scaler = 2_u32.pow(rel_zoom.unsigned_abs());

    if rel_zoom.is_positive() {
        // Heightmap is smaller than tile
        tile_scale = heightmap_to_img_scaler;
        w = heightmap.width() - 2;
        h = heightmap.width() - 2;
    } else {
        // Heightmap is bigger than tile
        tile_scale = 1;
        (w, h) = tile.dimensions();
    }

    for x in 0..h {
        for y in 0..w {
            let tile_x: u32;
            let tile_y: u32;
            let hmx;
            let hmy;
            let stroke;

            if rel_zoom.is_positive() {
                // Heightmap is smaller than tile
                tile_x = (x) * heightmap_to_img_scaler;
                tile_y = (y) * heightmap_to_img_scaler;

                hmx = x + 1;
                hmy = y + 1;

                stroke = strenght;
            } else {
                // Heightmap is bigger than tile
                hmx = ((x) * heightmap_to_img_scaler) + 1;
                hmy = ((y) * heightmap_to_img_scaler) + 1;

                tile_x = x;
                tile_y = y;

                stroke = strenght / (heightmap_to_img_scaler as i8);
            }
            match height_change_x(hmx, hmy, heightmap) {
                Direction::Higher(n) => {
                    shift_lightness(
                        tile,
                        tile_x,
                        tile_y,
                        tile_scale,
                        tile_scale,
                        -height_diff_shade_calculator(stroke, n),
                    );
                }
                Direction::Lower(n) => {
                    shift_lightness(
                        tile,
                        tile_x,
                        tile_y,
                        tile_scale,
                        tile_scale,
                        height_diff_shade_calculator(stroke, n),
                    );
                }
                Direction::Flat => (),
            }

            match height_change_y(hmx, hmy, heightmap) {
                Direction::Higher(n) => {
                    shift_lightness(
                        tile,
                        tile_x,
                        tile_y,
                        tile_scale,
                        tile_scale,
                        -height_diff_shade_calculator(stroke, n),
                    );
                }
                Direction::Lower(n) => {
                    shift_lightness(
                        tile,
                        tile_x,
                        tile_y,
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
    let cmp_point = heightmap.get_pixel(hmx, hmy).0[0];
    let left_point = heightmap.get_pixel(hmx - 1, hmy).0[0];
    dir(left_point, cmp_point)
}

/// Gets heightmap change from this and above (eg is abover higher flat or lower than self)
fn height_change_y(hmx: u32, hmy: u32, heightmap: &GrayImage) -> Direction {
    let cmp_point = heightmap.get_pixel(hmx, hmy).0[0];
    let up_point = heightmap.get_pixel(hmx, hmy - 1).0[0];
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
