use std::{collections::HashMap, io::Cursor, sync::Mutex};

use actix_web::web::Bytes;
use image::ImageFormat;
use log::debug;

use super::{TilePos, TileProvider};

pub struct TileCache<Source>
where
    Source: TileProvider,
{
    source: Source,
    format: ImageFormat,
    memcache: Mutex<HashMap<TilePos, Bytes>>,
}

impl<S> TileCache<S>
where
    S: TileProvider,
{
    pub fn new(source: S, format: ImageFormat) -> Self {
        Self {
            source,
            format,
            memcache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_cached_tile(&self, pos: TilePos) -> Option<Bytes> {
        {
            let memcache = self.memcache.lock().unwrap();

            if let Some(tile) = memcache.get(&pos) {
                debug!("Cache Hit");
                return Some(tile.clone());
            }
        }

        let val: Bytes = self.generate_tile(pos)?.into();

        {
            let mut memcache = self.memcache.lock().unwrap();
            memcache.insert(pos, val.clone());
            debug!("Cache Miss");
            Some(val)
        }
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
