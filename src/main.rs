use actix_multipart::form::{MultipartForm, MultipartFormConfig};
use actix_web::{
    error::ErrorNotFound, get, http::StatusCode, post, web, App, HttpResponse, HttpServer,
    Responder,
};
use image::ImageError;
use media_manager::{MediaId, MediaManager};
use storage::Memory;
use thiserror::Error;

mod media_manager;
mod storage;

#[derive(Default)]
struct AppState {
    media_manager: MediaManager<u64, Memory>,
}

#[get("/get/{media_id}")]
async fn get(data: web::Data<AppState>, path: web::Path<MediaId>) -> impl Responder {
    match data.media_manager.retrieve(&path) {
        Some(b) => Ok(HttpResponse::build(StatusCode::OK)
            .content_type("image/png")
            .body(b)),
        None => Err(ErrorNotFound("This file was not found")),
    }
}

#[get("/exists/{media_id}")]
async fn exists(data: web::Data<AppState>, path: web::Path<MediaId>) -> impl Responder {
    web::Json(data.media_manager.contains(&path))
}

#[derive(MultipartForm)]
struct ImageUpload {
    #[multipart(limit = "20 MiB")]
    image: actix_multipart::form::bytes::Bytes,
}

#[derive(Error, Debug)]
enum ImageDecodingError {
    #[error("type is not recognizable")]
    NotRecognizable(#[from] std::io::Error),
    #[error("image decoding error")]
    ImageError(#[from] ImageError),
}

impl actix_web::error::ResponseError for ImageDecodingError {}

#[post("/upload")]
async fn upload(
    data: web::Data<AppState>,
    image_upload: MultipartForm<ImageUpload>,
) -> impl Responder {
    let ImageUpload { image } = image_upload.into_inner();

    let decoded_image = image::io::Reader::new(std::io::Cursor::new(image.data))
        .with_guessed_format()?
        .decode()?;

    let mut bytes: Vec<u8> = Vec::new();
    decoded_image.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageOutputFormat::Png,
    )?;

    let media_id = data.media_manager.store(actix_web::web::Bytes::from(bytes));

    actix_web::rt::spawn(async move {
        actix_web::rt::time::sleep(std::time::Duration::from_secs(60 * 60)).await;
        data.media_manager.delete(&media_id);
    });

    Ok::<actix_web::web::Json<MediaId>, ImageDecodingError>(web::Json(media_id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let app_data = web::Data::new(AppState::default());

    HttpServer::new(move || {
        let app = App::new()
            .app_data(web::PayloadConfig::new(250_000_000))
            .app_data(MultipartFormConfig::default().memory_limit(16_000_000))
            .app_data(app_data.clone())
            .service(get)
            .service(exists)
            .service(upload);

        #[cfg(debug_assertions)]
        {
            let cors = actix_cors::Cors::permissive();
            app.wrap(cors)
        }
        #[cfg(not(debug_assertions))]
        {
            app
        }
    })
    .bind((
        if cfg!(debug_assertions) {
            "0.0.0.0"
        } else {
            "127.0.0.1"
        },
        5040,
    ))?
    .run()
    .await
}
