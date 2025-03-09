use std::{collections::HashMap, io::Cursor, time::Instant};

use actix_web::web::Bytes;
use image::ImageFormat;
use log::debug;
use parking_lot::RwLock;

use super::{TilePos, TileProvider};

pub struct TileCache<Source>
where
    Source: TileProvider,
{
    source: Source,
    format: ImageFormat,
    memcache: RwLock<HashMap<TilePos, Bytes>>,
}

impl<S> TileCache<S>
where
    S: TileProvider,
{
    pub fn new(source: S, format: ImageFormat) -> Self {
        Self {
            source,
            format,
            memcache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_cached_tile(&self, pos: TilePos) -> Option<Bytes> {
        {
            let before_mutex = Instant::now();
            let memcache = self.memcache.read();
            debug!("Time to lock mutex (reading): {:?}", before_mutex.elapsed());

            if let Some(tile) = memcache.get(&pos) {
                return Some(tile.clone());
            }

            debug!("Time to read tile: {:?}", before_mutex.elapsed());
        }
        let before_write = Instant::now();

        let val: Bytes = self.generate_tile(pos)?.into();

        debug!("Time to generate tile: {:?}", before_write.elapsed());

        {
            let before_mutex = Instant::now();
            let mut memcache = self.memcache.write();
            memcache.insert(pos, val.clone());
            debug!("Time to write tile to cache: {:?}", before_mutex.elapsed());
            Some(val)
        }
    }

    pub fn format(&self) -> &ImageFormat {
        &self.format
    }

    fn generate_tile(&self, pos: TilePos) -> Option<Vec<u8>> {
        let mut buf = Cursor::new(Vec::new());
        self.source
            .get_tile(pos)?
            .write_to(&mut buf, self.format)
            .unwrap_or_else(|_| panic!("Writing tile {pos:?} failed"));
        Some(buf.into_inner())
    }
}
