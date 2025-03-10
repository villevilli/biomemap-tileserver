use std::{
    collections::HashMap,
    fmt::Display,
    fs::create_dir_all,
    io::{Cursor, ErrorKind},
    path::PathBuf,
};

use actix_web::{ResponseError, http::StatusCode, web::Bytes};
use image::ImageFormat;
use log::error;
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
            Error::WriteError(e) => {
                error!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Error::ReadError(e) => {
                error!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Error::CreateDirError(e) => {
                error!("{}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

pub struct TileCache<Source>
where
    Source: TileProvider,
{
    source: Source,
    format: ImageFormat,
    base_path: PathBuf,
    memcache: RwLock<HashMap<TilePos, Bytes>>,
}

impl<S> TileCache<S>
where
    S: TileProvider,
{
    pub fn new<T>(source: S, format: ImageFormat, base_path: T) -> Result<Self, Error>
    where
        T: Into<PathBuf>,
    {
        let base_path = base_path.into();
        create_dir_all(&base_path).map_err(Error::CreateDirError)?;

        Ok(Self {
            source,
            format,
            base_path,
            memcache: RwLock::new(HashMap::new()),
        })
    }

    pub async fn get_cached_tile(&self, pos: TilePos) -> Result<Bytes, Error> {
        {
            let memcache = self.memcache.read();

            if let Some(tile) = memcache.get(&pos) {
                return Ok(tile.clone());
            }
        }

        let val: Bytes = self.read_or_gen_tile_fs(pos).await?.into();

        {
            let mut memcache = self.memcache.write();
            memcache.insert(pos, val.clone());
            Ok(val)
        }
    }

    pub fn format(&self) -> &ImageFormat {
        &self.format
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
