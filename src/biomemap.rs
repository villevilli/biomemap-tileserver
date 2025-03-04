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
use image::{Rgb, RgbImage};

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
    fn get_tile(&self, zoom: i32, x: i32, y: i32) -> image::DynamicImage {
        match zoom {
            0 => get_image(x, y, self, Scale::Block),
            -1 => concat_lower_zoom(x, y, self, Scale::Block),
            -2 => get_image(x, y, self, Scale::Quad),
            -3 => concat_lower_zoom(x, y, self, Scale::Quad),
            -4 => get_image(x, y, self, Scale::Chunk),
            -5 => concat_lower_zoom(x, y, self, Scale::Chunk),
            -6 => get_image(x, y, self, Scale::QuadChunk),
            -7 => concat_lower_zoom(x, y, self, Scale::QuadChunk),
            -8 => get_image(x, y, self, Scale::HalfRegion),
            _ => unimplemented!(),
        }
        .into()
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
            dbg!(img_x);
            dbg!(img_y);
            let cache = cache_pool
                .get(
                    ((x * 255) * (2)) + (img_x * 255),
                    320,
                    ((y * 255) * (2)) + (img_y * 255),
                    scale,
                )
                .unwrap();

            for sub_img_x in 0..(256 / (2)) {
                for sub_img_y in 0..(256 / (2)) {
                    //dbg!(x);
                    //dbg!(y);

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
