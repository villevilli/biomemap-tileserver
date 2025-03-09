use std::net::SocketAddrV4;

use actix_web::{
    App, HttpResponse, HttpServer, Responder, get,
    http::header::ContentType,
    web::{self, Data},
};
use biomemap_tileserver::{
    biomemap::{CachePool, ContourLines, ShadedBiomeTile, UnshadedBiomeTile},
    tileprovider::{TilePos, tilecache::TileCache},
};
use cubiomes::{
    enums::MCVersion,
    generator::{Generator, GeneratorFlags},
};
use image::ImageFormat;

const SEED: i64 = 3846517875239123423;
const NOTILEPNG: &[u8] = include_bytes!("notile.png").as_slice();

// Note change urls if you change this
const TILE_IMAGE_FORMAT: ImageFormat = image::ImageFormat::Png;

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

    let cache_pool = CachePool::new(g);

    let shade_tile_cache = web::Data::new(TileCache::new(
        ShadedBiomeTile::from(cache_pool.clone()),
        TILE_IMAGE_FORMAT,
    ));

    let unsahded_tile_cahce = web::Data::new(TileCache::new(
        UnshadedBiomeTile::from(cache_pool.clone()),
        TILE_IMAGE_FORMAT,
    ));

    let contour_line_cache = web::Data::new(TileCache::new(
        ContourLines::from(cache_pool),
        TILE_IMAGE_FORMAT,
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(shade_tile_cache.clone())
            .app_data(unsahded_tile_cahce.clone())
            .app_data(contour_line_cache.clone())
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
        .content_type(cache_pool.format().to_mime_type())
        .body(tile)
}

#[get("/biomemap/{zoom}/{x}/{y}.png")]
async fn get_biome_tile_shaded(
    path: web::Path<(i32, i32, i32)>,
    cache_pool: Data<TileCache<UnshadedBiomeTile<'_>>>,
) -> impl Responder {
    let (zoom, x, y) = path.into_inner();
    let Some(tile) = cache_pool.get_cached_tile(TilePos::new(zoom, x, y)) else {
        return HttpResponse::NotFound()
            .content_type(cache_pool.format().to_mime_type())
            .body(NOTILEPNG);
    };

    HttpResponse::Ok()
        .content_type(cache_pool.format().to_mime_type())
        .body(tile)
}

#[get("/contours/{zoom}/{x}/{y}.png")]
async fn get_contour_tile(
    path: web::Path<(i32, i32, i32)>,
    cache_pool: Data<TileCache<ContourLines<'_>>>,
) -> impl Responder {
    let (zoom, x, y) = path.into_inner();

    let Some(tile) = cache_pool.get_cached_tile(TilePos::new(zoom, x, y)) else {
        return HttpResponse::NotFound().finish();
    };

    HttpResponse::Ok()
        .content_type(cache_pool.format().to_mime_type())
        .body(tile)
}
