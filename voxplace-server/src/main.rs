mod app_state;
mod database;
mod place;
mod routes;
mod voxel;
mod websocket;

use crate::app_state::AppState;
use crate::database::db::Database;
use crate::routes::{
    check_admin, create_place, draw_voxel_http, get_grid, get_places_info, get_username,
    login_user, register_user, ws_index,
};
use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use std::sync::RwLock;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = Database::new();

    let app_state = Data::new(RwLock::new(AppState::new(db.unwrap())));
    app_state.write().unwrap().start_write_place_loop();

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
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
