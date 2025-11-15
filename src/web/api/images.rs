use std::io::Cursor;

use axum::{
    extract::{Multipart, Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use image::{DynamicImage, ImageFormat, ImageReader};
use log::{error, warn};

use crate::{utils::uuid::generate_uuid_string, web::api::CombinedState};

pub const MAX_IMAGE_BYTES: usize = 30 * 1024 * 1024; // 30 MB
pub const THUMBNAIL_MAX_WIDTH: u32 = 128;
pub const THUMBNAIL_MAX_HEIGHT: u32 = 96;

#[derive(serde::Serialize)]
pub struct ImageUploadResponse {
    pub image_id: String,
    pub width: u32,
    pub height: u32,
    pub thumbnail_width: u32,
    pub thumbnail_height: u32,
}

fn build_thumbnail(image: &DynamicImage) -> Result<(Vec<u8>, u32, u32), StatusCode> {
    let thumbnail = image.thumbnail(THUMBNAIL_MAX_WIDTH, THUMBNAIL_MAX_HEIGHT);
    let width = thumbnail.width();
    let height = thumbnail.height();
    let mut cursor = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| {
            error!("Failed to encode thumbnail PNG: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((cursor.into_inner(), width, height))
}

fn decode_image_from_bytes(bytes: &[u8]) -> Result<DynamicImage, StatusCode> {
    let mut reader = ImageReader::new(Cursor::new(bytes));
    reader = reader.with_guessed_format().map_err(|err| {
        warn!(
            "Failed to guess image format while regenerating thumbnail: {}",
            err
        );
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    })?;
    reader.decode().map_err(|err| {
        warn!(
            "Failed to decode image while regenerating thumbnail: {}",
            err
        );
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    })
}

pub async fn upload_image(
    State(combined_state): State<CombinedState>,
    mut multipart: Multipart,
) -> Result<Json<ImageUploadResponse>, StatusCode> {
    let ((_display, storage), _events) = combined_state;
    let mut image_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if let Some(name) = field.name() {
            if name != "file" {
                continue;
            }
        }

        let mut data = Vec::new();
        let mut field_reader = field;
        while let Some(chunk) = field_reader
            .chunk()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?
        {
            if data.len() + chunk.len() > MAX_IMAGE_BYTES {
                return Err(StatusCode::PAYLOAD_TOO_LARGE);
            }
            data.extend_from_slice(&chunk);
        }

        if data.is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }

        image_bytes = Some(data);
        break;
    }

    let uploaded = image_bytes.ok_or(StatusCode::BAD_REQUEST)?;

    let mut reader = ImageReader::new(Cursor::new(&uploaded));
    reader = reader.with_guessed_format().map_err(|err| {
        warn!("Failed to guess image format: {}", err);
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    })?;

    let decoded = reader.decode().map_err(|err| {
        warn!("Failed to decode image: {}", err);
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    })?;
    let width = decoded.width();
    let height = decoded.height();

    let mut cursor = Cursor::new(Vec::new());
    decoded
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| {
            error!("Failed to encode PNG: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let png_bytes = cursor.into_inner();

    let (thumbnail_bytes, thumbnail_width, thumbnail_height) = build_thumbnail(&decoded)?;

    let image_id = generate_uuid_string();
    {
        let storage_guard = storage.lock().unwrap();
        if !storage_guard.save_image(&image_id, &png_bytes) {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        if !storage_guard.save_thumbnail(&image_id, &thumbnail_bytes) {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    Ok(Json(ImageUploadResponse {
        image_id,
        width,
        height,
        thumbnail_width,
        thumbnail_height,
    }))
}

pub async fn fetch_image(
    State(combined_state): State<CombinedState>,
    Path(image_id): Path<String>,
) -> Result<Response, StatusCode> {
    let ((_display, storage), _events) = combined_state;
    let storage_guard = storage.lock().unwrap();
    if let Some(bytes) = storage_guard.load_image(&image_id) {
        let headers = [(header::CONTENT_TYPE, HeaderValue::from_static("image/png"))];
        Ok((headers, Bytes::from(bytes)).into_response())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn fetch_image_thumbnail(
    State(combined_state): State<CombinedState>,
    Path(image_id): Path<String>,
) -> Result<Response, StatusCode> {
    let ((_display, storage), _events) = combined_state;

    let storage_guard = storage.lock().unwrap();

    if let Some(bytes) = storage_guard.load_thumbnail(&image_id) {
        let headers = [(header::CONTENT_TYPE, HeaderValue::from_static("image/png"))];
        return Ok((headers, Bytes::from(bytes)).into_response());
    }

    let image_bytes = match storage_guard.load_image(&image_id) {
        Some(bytes) => bytes,
        None => return Err(StatusCode::NOT_FOUND),
    };

    drop(storage_guard);

    let decoded = decode_image_from_bytes(&image_bytes)?;
    let (thumbnail_bytes, _, _) = build_thumbnail(&decoded)?;

    {
        let storage_guard = storage.lock().unwrap();
        if !storage_guard.save_thumbnail(&image_id, &thumbnail_bytes) {
            warn!("Failed to persist regenerated thumbnail for {}", image_id);
        }
    }

    let headers = [(header::CONTENT_TYPE, HeaderValue::from_static("image/png"))];
    Ok((headers, Bytes::from(thumbnail_bytes)).into_response())
}
