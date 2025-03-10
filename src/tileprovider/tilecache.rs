use std::{
    collections::{BTreeMap, HashMap},
    fmt::Display,
    fs::create_dir_all,
    io::{Cursor, ErrorKind},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use actix_web::{ResponseError, http::StatusCode, web::Bytes};
use image::ImageFormat;
use parking_lot::RwLock;
use tokio::{
    fs::{read, write},
    io,
};

use super::{TilePos, TileProvider};

#[derive(Debug)]
pub enum Error {
    NoTileInProvider,
    WriteError(io::Error),
    ReadError(io::Error),
    CreateDirError(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoTileInProvider => {
                writeln!(
                    f,
                    "the source could not provide a tile for the requested position"
                )
            }
            Error::WriteError(_) => writeln!(
                f,
                "Error occured while trying to write to the underyling fs"
            ),
            Error::ReadError(_) => writeln!(
                f,
                "Error occured while trying to read from the underyling fs"
            ),
            Error::CreateDirError(_) => {
                writeln!(f, "failed to create the directory to use with the cache")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::NoTileInProvider => None,
            Error::WriteError(e) => Some(e),
            Error::ReadError(e) => Some(e),
            Error::CreateDirError(e) => Some(e),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::NoTileInProvider => StatusCode::NOT_FOUND,
            Error::WriteError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ReadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::CreateDirError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub(crate) struct CachedTile {
    last_access: AtomicU64,
    data: Bytes,
}

impl From<Bytes> for CachedTile {
    fn from(data: Bytes) -> Self {
        Self::new(data)
    }
}

impl CachedTile {
    pub(crate) fn new(data: Bytes) -> Self {
        Self {
            last_access: AtomicU64::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("System time is set before EPOCH")
                    .as_secs(),
            ),
            data,
        }
    }

    pub(crate) fn get(&self) -> &Bytes {
        self.update_access_time();
        self.get_without_updating_time()
    }

    fn update_access_time(&self) {
        self.last_access.store(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Systtem time is set before EPOCH")
                .as_secs(),
            Ordering::Relaxed,
        );
    }

    pub(crate) fn get_without_updating_time(&self) -> &Bytes {
        &self.data
    }
}

pub struct TileCache<Source>
where
    Source: TileProvider,
{
    source: Source,
    format: ImageFormat,
    base_path: PathBuf,
    // The amount of tiles we will hold in the cache.
    max_capacity: usize,
    memcache: RwLock<HashMap<TilePos, CachedTile>>,
}

impl<S> TileCache<S>
where
    S: TileProvider,
{
    pub fn new<T>(
        source: S,
        max_capacity: usize,
        format: ImageFormat,
        base_path: T,
    ) -> Result<Self, Error>
    where
        T: Into<PathBuf>,
    {
        let base_path = base_path.into();
        create_dir_all(&base_path).map_err(Error::CreateDirError)?;

        Ok(Self {
            source,
            format,
            base_path,
            max_capacity,
            memcache: RwLock::new(HashMap::new()),
        })
    }

    pub async fn get_cached_tile(&self, pos: TilePos) -> Result<Bytes, Error> {
        let cur_cap;
        {
            let memcache = self.memcache.read();

            if let Some(tile) = memcache.get(&pos) {
                return Ok(tile.get().clone());
            }

            cur_cap = memcache.len()
        }

        if cur_cap >= self.max_capacity {
            self.cleanup().await;
        }

        let val: Bytes = self.read_or_gen_tile_fs(pos).await?.into();

        {
            let mut memcache = self.memcache.write();
            memcache.insert(pos, val.clone().into());

            Ok(val)
        }
    }

    pub fn format(&self) -> &ImageFormat {
        &self.format
    }

    /// Cleans up self (deallocates old data to gain back memory)
    ///
    /// Removes 1/3 of the oldest (by last access) memory cache entries.
    ///
    /// TODO:
    /// Check if skiping atomic access makes this faster (benchmark??)
    /// (needs unsafe so didnt do yet)
    async fn cleanup(&self) {
        let mut map = self.memcache.write();

        let data: BTreeMap<u64, &TilePos> = map
            .iter()
            .map(|(pos, tile)| (tile.last_access.load(Ordering::Relaxed), pos))
            .collect();

        let len = data.len();

        // Since we take less tan len elements, operation will never panic
        let Some(yield_until) = data.into_keys().nth(len - len / 3) else {
            // If for some reason all tiles are returned just clear the whole cahe.
            //
            // This is such a edge case I'm not gonna deal with it and probably means
            // incorrect usage anyways, since the cache size must be stupidly low and usage
            // stupidly hight to generate enough tiles at once. (and if you manage to create
            // say 5000 tiles in under one second you probably dont care about memory
            // caching anyways)
            map.clear();
            map.shrink_to_fit();
            return;
        };

        map.retain(|_, v| yield_until < v.last_access.load(Ordering::Relaxed));
        map.shrink_to_fit();
    }

    fn generate_tile(&self, pos: TilePos) -> Result<Vec<u8>, Error> {
        let mut buf = Cursor::new(Vec::new());
        self.source
            .get_tile(pos)
            .ok_or(Error::NoTileInProvider)?
            .write_to(&mut buf, self.format)
            .unwrap_or_else(|_| panic!("Writing tile {pos:?} failed"));
        Ok(buf.into_inner())
    }

    async fn read_or_gen_tile_fs(&self, pos: TilePos) -> Result<Vec<u8>, Error> {
        let dir = self.base_path.join(format!("{}/{}/", pos.zoom, pos.x));

        // We ignore this error, since the directory might already exist
        let _ = create_dir_all(&dir);

        let path = format!("{}{}.{}", dir.to_str().unwrap(), pos.y, {
            self.format.extensions_str()[0]
        });

        match read(&path).await {
            Ok(buf) => Ok(buf),
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    let img = self.generate_tile(pos)?;
                    write(path, &img).await.map_err(Error::WriteError)?;
                    Ok(img)
                } else {
                    Err(Error::ReadError(e))
                }
            }
        }
    }
}
