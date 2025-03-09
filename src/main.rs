use std::net::SocketAddrV4;

use actix_web::{
    App, HttpResponse, HttpServer, Responder, get,
    http::header::ContentType,
    web::{self, Data},
};
use biomemap_tileserver::{
    biomemap::{CachePool, ContourLines, ShadedBiomeTile},
    tileprovider::{TilePos, TileProvider, tilecache::TileCache},
};
use cubiomes::{
    enums::MCVersion,
    generator::{Generator, GeneratorFlags},
};
use image::codecs::png::PngEncoder;

const SEED: i64 = 3846517875239123423;
const NOTILEPNG: &[u8] = include_bytes!("notile.png").as_slice();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // SAFETY: probs??? i dont think anything elsee is touching the env vars yet ... lol
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    let address = SocketAddrV4::new("0.0.0.0".parse().unwrap(), 3000);
    let g = Box::leak(Box::new(Generator::new(
        MCVersion::MC_1_21_WD,
        SEED,
        cubiomes::enums::Dimension::DIM_OVERWORLD,
        GeneratorFlags::empty(),
    )));

    let cache_pool = web::Data::new(CachePool::new(g));

    let tile_cache = web::Data::new(TileCache::new(
        ShadedBiomeTile::new(CachePool::new(g)),
        image::ImageFormat::Png,
    ));
    HttpServer::new(move || {
        App::new()
            .app_data(cache_pool.clone())
            .app_data(tile_cache.clone())
            .service((
                get_biome_tile,
                get_biome_tile_shaded,
                get_contour_tile,
                actix_files::Files::new("/", concat!(env!("OUT_DIR"), "/pages"))
                    .index_file("index.html"),
            ))
    })
    .bind(address)?
    .run()
    .await?;

    Ok(())
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("pages/index.html"))
}

#[get("/biomemap_shaded/{zoom}/{x}/{y}.png")]
async fn get_biome_tile(
    path: web::Path<(i32, i32, i32)>,
    cache_pool: Data<TileCache<ShadedBiomeTile<'_>>>,
) -> impl Responder {
    let (zoom, x, y) = path.into_inner();

    let Some(tile) = cache_pool.get_cached_tile(TilePos { zoom, x, y }) else {
        return HttpResponse::NotFound()
            .content_type(ContentType::png())
            .body(NOTILEPNG);
    };

    HttpResponse::Ok()
        .content_type(ContentType::png())
        .body(tile)
}

#[get("/biomemap/{zoom}/{x}/{y}.png")]
async fn get_biome_tile_shaded(
    path: web::Path<(i32, i32, i32)>,
    cache_pool: Data<CachePool<'_>>,
) -> impl Responder {
    let (zoom, x, y) = path.into_inner();
    let Some(tile) = cache_pool.get_tile(zoom, x, y, false) else {
        return HttpResponse::NotFound()
            .content_type(ContentType::png())
            .body(NOTILEPNG);
    };

    let mut buf = Vec::new();

    tile.write_with_encoder(PngEncoder::new(&mut buf)).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::png())
        .body(buf)
}

#[get("/contours/{zoom}/{x}/{y}.png")]
async fn get_contour_tile(
    path: web::Path<(i32, i32, i32)>,
    cache_pool: Data<CachePool<'_>>,
) -> impl Responder {
    let (zoom, x, y) = path.into_inner();

    let Some(tile) = ContourLines(&cache_pool).get_tile(TilePos::new(zoom, x, y)) else {
        return HttpResponse::NotFound().finish();
    };

    let mut buf = Vec::new();

    tile.write_with_encoder(PngEncoder::new(&mut buf)).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::png())
        .body(buf)
}
