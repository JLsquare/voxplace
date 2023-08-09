mod app_state;
mod database;
mod place;
mod routes;
mod voxel;
mod websocket;

use crate::app_state::AppState;
use crate::database::db::Database;
use crate::routes::{check_admin, create_place, draw_voxel_http, edit_user, get_cooldown, get_grid, get_palette, get_places_info, get_top_users, get_user_profile, get_username, get_voxel, get_voxel_palette, login_user, register_user, save_voxel, ws_index};
use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use std::sync::RwLock;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Database::new();

    let app_state = Data::new(RwLock::new(AppState::new(db.unwrap())));

    println!("Starting server on port 8000");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .app_data(app_state.clone())
            .service(get_grid)
            .service(ws_index)
            .service(draw_voxel_http)
            .service(register_user)
            .service(login_user)
            .service(get_places_info)
            .service(get_username)
            .service(check_admin)
            .service(create_place)
            .service(get_palette)
            .service(get_cooldown)
            .service(get_voxel)
            .service(get_voxel_palette)
            .service(get_user_profile)
            .service(edit_user)
            .service(save_voxel)
            .service(get_top_users)
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
