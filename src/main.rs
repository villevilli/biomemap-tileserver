use std::net::SocketAddrV4;

use actix_web::{
    App, HttpResponse, HttpServer, Responder, get,
    http::header::ContentType,
    web::{self, Data},
};
use biomemap::CachePool;
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
    let mut buf = Vec::new();

    if cache_pool
        .get_tile(zoom, x, y)
        .write_with_encoder(PngEncoder::new(&mut buf))
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok()
        .content_type(ContentType::png())
        .body(buf)
}
