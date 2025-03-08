use std::net::SocketAddrV4;

use actix_web::{
    App, HttpResponse, HttpServer, Responder, get,
    http::header::ContentType,
    web::{self, Data},
};
use biomemap::{CachePool, ContourLines};
use cubiomes::{
    enums::MCVersion,
    generator::{Generator, GeneratorFlags},
};
use image::codecs::png::PngEncoder;
use tileprovider::TileProvider;

mod biomemap;
mod tileprovider;

const SEED: i64 = 3846517875239123423;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address = SocketAddrV4::new("0.0.0.0".parse().unwrap(), 3000);

    let server = HttpServer::new(|| {
        let g = Box::leak(Box::new(Generator::new(
            MCVersion::MC_1_21_WD,
            SEED,
            cubiomes::enums::Dimension::DIM_OVERWORLD,
            GeneratorFlags::empty(),
        )));

        let cache_pool = web::Data::new(CachePool::new(g));

        App::new().app_data(cache_pool).service((
            get_biome_tile,
            get_biome_tile_shaded,
            get_contour_tile,
            actix_files::Files::new("/", concat!(env!("OUT_DIR"), "/pages"))
                .index_file("index.html"),
        ))
    })
    .bind(address)?
    .run();

    println!("server running at http://{}", address);

    server.await
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("pages/index.html"))
}

#[get("/biomemap/{zoom}/{x}/{y}.png")]
async fn get_biome_tile(
    path: web::Path<(i32, i32, i32)>,
    cache_pool: Data<CachePool<'_>>,
) -> impl Responder {
    let (zoom, x, y) = path.into_inner();
    let Some(tile) = cache_pool.get_tile(zoom, x, y, false) else {
        return HttpResponse::NotFound()
            .content_type(ContentType::png())
            .body(include_bytes!("notile.png").as_slice());
    };

    let mut buf = Vec::new();

    tile.write_with_encoder(PngEncoder::new(&mut buf)).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::png())
        .body(buf)
}

#[get("/biomemap_shaded/{zoom}/{x}/{y}.png")]
async fn get_biome_tile_shaded(
    path: web::Path<(i32, i32, i32)>,
    cache_pool: Data<CachePool<'_>>,
) -> impl Responder {
    let (zoom, x, y) = path.into_inner();
    let Some(tile) = cache_pool.get_tile(zoom, x, y, true) else {
        return HttpResponse::NotFound()
            .content_type(ContentType::png())
            .body(include_bytes!("notile.png").as_slice());
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

    let Some(tile) = ContourLines(&cache_pool).get_tile(zoom, x, y) else {
        return HttpResponse::NotFound().finish();
    };

    let mut buf = Vec::new();

    tile.write_with_encoder(PngEncoder::new(&mut buf)).unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::png())
        .body(buf)
}
