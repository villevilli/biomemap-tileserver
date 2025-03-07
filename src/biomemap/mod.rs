mod postprocess;

use std::{
    collections::BTreeMap,
    error::Error,
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use cubiomes::generator::{Cache, Generator, Range, Scale};
use postprocess::{concat_lower_zoom, draw_contours, draw_shading, get_image, upsacale_blockscale};

use crate::tileprovider::TileProvider;

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
        } else {
        }

        Some(tile.into())
    }
}
