use std::{collections::HashMap, io::Cursor};

use image::ImageFormat;

use super::{TilePos, TileProvider};

pub struct TileCache<Source>
where
    Source: TileProvider,
{
    source: Source,
    format: ImageFormat,
    memcache: HashMap<TilePos, Vec<u8>>,
}

impl<S> TileCache<S>
where
    S: TileProvider,
{
    pub fn new(source: S, format: ImageFormat) -> Self {
        Self {
            source,
            format,
            memcache: HashMap::new(),
        }
    }

    pub fn get_cached_tile(&mut self, pos: TilePos) -> Option<Vec<u8>> {
        // I couldn't get the lifetimes to co-operate without this. Yes it does
        // hash twice ):
        #[allow(clippy::map_entry)]
        if self.memcache.contains_key(&pos) {
            Some(self.memcache.get(&pos).unwrap().clone())
        } else {
            self.memcache.insert(pos, self.get_tile(pos)?);

            self.get_cached_tile(pos)
        }
    }

    fn get_tile(&self, pos: TilePos) -> Option<Vec<u8>> {
        let mut buf = Cursor::new(Vec::new());
        self.source
            .get_tile(pos)?
            .write_to(&mut buf, self.format)
            .unwrap_or_else(|_| panic!("Writing tile {pos:?} failed"));
        Some(buf.into_inner())
    }
}
