use std::{
    collections::BTreeMap,
    error::Error,
    ops::{Deref, DerefMut},
    sync::{LazyLock, Mutex},
};

use cubiomes::{
    colors::BiomeColorMap,
    generator::{Cache, Generator, Range, Scale},
};
use image::{GrayImage, Rgb, RgbImage, imageops::resize};

use crate::tileprovider::TileProvider;

static COLOR_MAP: LazyLock<BiomeColorMap> = std::sync::LazyLock::new(BiomeColorMap::new);

pub struct CachePool<'pool> {
    generator: &'pool Generator,
    caches: Mutex<BTreeMap<Scale, Vec<Cache<'pool>>>>,
}

impl<'pool> CachePool<'pool> {
    pub fn new(generator: &'pool Generator) -> Self {
        Self {
            generator,
            caches: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn as_generatr_ref(&self) -> &Generator {
        self.generator
    }

    pub fn get<'lock>(
        &'lock self,
        x: i32,
        y: i32,
        z: i32,
        scale: Scale,
    ) -> Result<CacheLock<'lock, 'pool>, Box<dyn Error>>
    where
        'pool: 'lock,
    {
        let mut caches = self.caches.lock().unwrap();

        caches.entry(scale).or_default();

        let cache: Option<Cache<'pool>> = caches.get_mut(&scale).unwrap().pop();

        if let Some(mut cache) = cache {
            cache.move_cache(x, y, z)?;
            return Ok(CacheLock::new(cache, self));
        }

        Ok(CacheLock::new(
            Cache::new(
                self.generator,
                Range {
                    scale,
                    x,
                    z,
                    size_x: 256,
                    size_z: 256,
                    y,
                    size_y: 0,
                },
            )?,
            self,
        ))
    }

    fn give_back(&self, cache: Cache<'pool>, scale: Scale) {
        self.caches
            .lock()
            .unwrap()
            .get_mut(&scale)
            .unwrap()
            .push(cache);
    }
}

pub struct CacheLock<'lock, 'pool> {
    inner: Option<Cache<'pool>>,
    pool: &'lock CachePool<'pool>,
}

impl<'pool> Deref for CacheLock<'_, 'pool> {
    type Target = Cache<'pool>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<'lock, 'pool> DerefMut for CacheLock<'lock, 'pool>
where
    'pool: 'lock,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl<'lock, 'pool> CacheLock<'lock, 'pool> {
    fn new(cache: Cache<'pool>, pool: &'lock CachePool<'pool>) -> Self
    where
        'pool: 'lock,
    {
        Self {
            inner: (Some(cache)),
            pool,
        }
    }
}

impl<'lock, 'pool> Drop for CacheLock<'lock, 'pool> {
    fn drop(&mut self)
    where
        'pool: 'lock,
    {
        let scale = (self.inner.as_ref()).unwrap().range().scale;

        self.pool.give_back(self.inner.take().unwrap(), scale);
    }
}

impl TileProvider for CachePool<'_> {
    fn get_tile(&self, zoom: i32, x: i32, y: i32) -> Option<image::DynamicImage> {
        let mut tile = match zoom {
            -8 => get_image(x, y, self, Scale::HalfRegion),
            -7 => concat_lower_zoom(x, y, self, Scale::QuadChunk),
            -6 => get_image(x, y, self, Scale::QuadChunk),
            -5 => concat_lower_zoom(x, y, self, Scale::Chunk),
            -4 => get_image(x, y, self, Scale::Chunk),
            -3 => concat_lower_zoom(x, y, self, Scale::Quad),
            -2 => get_image(x, y, self, Scale::Quad),
            -1 => concat_lower_zoom(x, y, self, Scale::Block),
            0 => get_image(x, y, self, Scale::Block),
            1..=8 => upsacale_blockscale(x, y, zoom, self),
            _ => return None,
        };

        if zoom >= -3 {
            draw_shading(&mut tile, x, y, zoom, self);
        }

        Some(tile.into())
    }
}

fn upsacale_blockscale(x: i32, y: i32, zoom: i32, cache_pool: &CachePool) -> RgbImage {
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

fn draw_shading(tile: &mut RgbImage, x: i32, y: i32, zoom: i32, cache_pool: &CachePool) {
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

    raw_draw_shading(&heightmap, tile, zoom, 16);
}

/// Draws contour lines onto the given image at the zoom level.
///
/// Heightmap must be big enough and should begin one left and end one right of the are in the image
fn raw_draw_shading(heightmap: &GrayImage, tile: &mut RgbImage, zoom: i32, strenght: i8) {
    let (w, h) = heightmap.dimensions();
    let rel_zoom = zoom + 2;

    let heightmap_to_img_scaler = 2_u32.pow(rel_zoom.unsigned_abs());

    for hmy in 1..(h - 2) {
        for hmx in 1..(w - 2) {
            let start_x;
            let start_y;

            let stroke = strenght;

            //let stroke = if zoom > 0 {
            //    strenght
            //} else {
            //    (strenght as i32 / (2_u32.pow((2 * zoom).unsigned_abs()) as i32)) as i8
            //};

            if rel_zoom.is_positive() {
                start_x = (hmx - 1) * heightmap_to_img_scaler;
                start_y = (hmy - 1) * heightmap_to_img_scaler;
            } else {
                start_x = ((hmx - 1) / heightmap_to_img_scaler).min(tile.width() - 1);
                start_y = ((hmy - 1) / heightmap_to_img_scaler).min(tile.height() - 1);
            }

            if height_change_x(hmx, hmy, heightmap) == Direction::Lower {
                shift_lightness(
                    tile,
                    start_x,
                    start_y,
                    heightmap_to_img_scaler,
                    heightmap_to_img_scaler,
                    stroke,
                );
            } else if height_change_x(hmx, hmy, heightmap) == Direction::Higher {
                shift_lightness(
                    tile,
                    start_x,
                    start_y,
                    heightmap_to_img_scaler,
                    heightmap_to_img_scaler,
                    -stroke,
                );
            }

            if height_change_y(hmx, hmy, heightmap) == Direction::Lower {
                shift_lightness(
                    tile,
                    start_x,
                    start_y,
                    heightmap_to_img_scaler,
                    heightmap_to_img_scaler,
                    stroke,
                );
            } else if height_change_y(hmx, hmy, heightmap) == Direction::Higher {
                shift_lightness(
                    tile,
                    start_x,
                    start_y,
                    heightmap_to_img_scaler,
                    heightmap_to_img_scaler,
                    -stroke,
                );
            }
        }
    }
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
    Higher,
    Flat,
    Lower,
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
            true => Direction::Lower,
            false => Direction::Higher,
        }
    }
}

fn get_image(x: i32, y: i32, cache_pool: &CachePool, scale: Scale) -> RgbImage {
    cache_pool
        .get(x * 256, 320, y * 256, scale)
        .unwrap()
        .to_image(*COLOR_MAP)
}

fn concat_lower_zoom(x: i32, y: i32, cache_pool: &CachePool, scale: Scale) -> RgbImage {
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
