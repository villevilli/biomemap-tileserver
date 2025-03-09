mod postprocess;

use std::{
    collections::BTreeMap,
    error::Error,
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use cubiomes::generator::{Cache, Generator, Range, Scale};
use image::GrayAlphaImage;
use postprocess::{
    concat_lower_zoom, draw_contours, draw_shading, generate_heightmap, get_image,
    upsacale_blockscale,
};

use crate::tileprovider::{TilePos, TileProvider};

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

impl CachePool<'_> {
    pub fn get_tile(
        &self,
        zoom: i32,
        x: i32,
        y: i32,
        is_shaded: bool,
    ) -> Option<image::DynamicImage> {
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

        if is_shaded {
            let heightmap = generate_heightmap(x * 256, y * 256, zoom, self);

            draw_shading(&heightmap, &mut tile, 24);
        }

        Some(tile.into())
    }
}

pub struct ShadedBiomeTile<'a>(CachePool<'a>);

impl<'a> ShadedBiomeTile<'a> {
    pub fn new(inner: CachePool<'a>) -> ShadedBiomeTile<'a> {
        Self(inner)
    }
}

impl TileProvider for ShadedBiomeTile<'_> {
    fn get_tile(&self, pos: TilePos) -> Option<image::DynamicImage> {
        self.0.get_tile(pos.zoom, pos.x, pos.y, true)
    }
}

/*
impl<'a> From<CachePool<'a>> for ShadedBiomeTile<'a> {
    fn from(value: CachePool<'a>) -> Self {
        Self(value)
    }
}
*/

pub struct UnshadedBiomeTile<'a, 'b>(&'a CachePool<'b>)
where
    'b: 'a;

impl TileProvider for UnshadedBiomeTile<'_, '_> {
    fn get_tile(&self, pos: TilePos) -> Option<image::DynamicImage> {
        self.0.get_tile(pos.zoom, pos.x, pos.y, false)
    }
}

impl<'b, 'a> From<&'a CachePool<'b>> for UnshadedBiomeTile<'a, 'b>
where
    'b: 'a,
{
    fn from(value: &'a CachePool<'b>) -> Self {
        Self(value)
    }
}
pub struct ContourLines<'a, 'b>(pub &'a CachePool<'b>)
where
    'b: 'a;

impl TileProvider for ContourLines<'_, '_> {
    fn get_tile(&self, pos: TilePos) -> Option<image::DynamicImage> {
        let TilePos { x, y, zoom } = pos;

        let heightmap = generate_heightmap(x * 256, y * 256, zoom, self.0);

        let start_level: u8 = 62;
        let frequency = zoom_calc(zoom, |_| 30, |scale| ((15 * (scale)) as u8));

        let mut tile = GrayAlphaImage::from_pixel(256, 256, [0, 0].into());

        draw_contours(
            &heightmap,
            contour_levels(start_level, frequency / 3),
            &mut tile,
            30,
            255,
        );

        draw_contours(
            &heightmap,
            contour_levels(start_level, frequency),
            &mut tile,
            80,
            255,
        );

        Some(tile.into())
    }
}

impl<'b, 'a> From<&'a CachePool<'b>> for ContourLines<'a, 'b>
where
    'b: 'a,
{
    fn from(value: &'a CachePool<'b>) -> Self {
        Self(value)
    }
}

/// Generates heights every frequency levels.
///
/// Starts from start level and goes both ways until [u8::min] and [u8::max]
pub fn contour_levels(start_levels: u8, frequency: u8) -> impl Iterator<Item = u8> {
    (u8::MIN..u8::MAX)
        .step_by(frequency as usize)
        .map(move |x| x + (start_levels % frequency))
}

pub fn zoom_calc<F1, F2, T>(zoom: i32, zoomed_in: F1, zoomed_out: F2) -> T
where
    F1: Fn(u32) -> T,
    F2: Fn(u32) -> T,
{
    let scale = 2_u32.pow(zoom.unsigned_abs());

    if zoom.is_negative() {
        zoomed_out(scale)
    } else {
        zoomed_in(scale)
    }
}
