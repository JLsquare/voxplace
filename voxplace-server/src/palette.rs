use std::sync::RwLock;
use actix_web::{get, HttpResponse, Responder};
use actix_web::web::{Data, Path};
use crate::app_state::AppState;

pub struct Palette {
    palette_id: i64,
    colors: Vec<(u8, u8, u8)>,
}

impl Palette {
    pub fn new(palette_id: i64, colors: Option<Vec<(u8, u8, u8)>>) -> Self {
        if let Some(colors) = colors {
            Self {
                palette_id,
                colors
            }
        } else {
            let colors: Vec<(u8, u8, u8)> = vec![
                (0x6d, 0x00, 0x1a),
                (0xbe, 0x00, 0x39),
                (0xff, 0x45, 0x00),
                (0xff, 0xa8, 0x00),
                (0xff, 0xd6, 0x35),
                (0xff, 0xf8, 0xb8),
                (0x00, 0xa3, 0x68),
                (0x00, 0xcc, 0x78),
                (0x7e, 0xed, 0x56),
                (0x00, 0x75, 0x6f),
                (0x00, 0x9e, 0xaa),
                (0x00, 0xcc, 0xc0),
                (0x24, 0x50, 0xa4),
                (0x36, 0x90, 0xea),
                (0x51, 0xe9, 0xf4),
                (0x49, 0x3a, 0xc1),
                (0x6a, 0x5c, 0xff),
                (0x94, 0xb3, 0xff),
                (0x81, 0x1e, 0x9f),
                (0xb4, 0x4a, 0xc0),
                (0xe4, 0xab, 0xff),
                (0xde, 0x10, 0x7f),
                (0xff, 0x38, 0x81),
                (0xff, 0x99, 0xaa),
                (0x6d, 0x48, 0x2f),
                (0x9c, 0x69, 0x26),
                (0xff, 0xb4, 0x70),
                (0x00, 0x00, 0x00),
                (0x51, 0x52, 0x52),
                (0x89, 0x8d, 0x90),
                (0xd4, 0xd7, 0xd9),
                (0xff, 0xff, 0xff),
            ];
            Self {
                palette_id,
                colors,
            }
        }
    }

    pub fn palette_id(&self) -> i64 {
        self.palette_id
    }

    pub fn colors(&self) -> &Vec<(u8, u8, u8)> {
        &self.colors
    }
}

#[get("/api/palette/get/{id}")]
async fn get_palette(
    data: Data<RwLock<AppState>>,
    path: Path<String>
) -> impl Responder {
    let id = match path.into_inner().parse::<i64>() {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid voxel"),
    };

    let app_state = match data.read() {
        Ok(state) => state,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read app state"),
    };

    let db = match app_state.database.lock() {
        Ok(db) => db,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read database"),
    };

    let palette = match db.get_palette(id) {
        Ok(palette) => palette,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read palette"),
    };

    let palette_hex: Vec<String> = palette
        .iter()
        .map(|&(r, g, b)| format!("#{:02x}{:02x}{:02x}", r, g, b))
        .collect();

    HttpResponse::Ok().json(palette_hex)
}