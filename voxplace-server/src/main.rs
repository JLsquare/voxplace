mod app_state;
mod database;
mod place;
mod voxel;
mod websocket;
mod palette;
mod post;
mod comment;
mod user;

use crate::app_state::AppState;
use crate::database::db::Database;
use actix_cors::Cors;
use actix_web::web::Data;
use actix_web::{App, HttpServer};
use std::sync::RwLock;
use crate::comment::{create_comment, get_place_comments, get_post_comments};
use crate::palette::get_palette;
use crate::place::{create_place, draw_voxel_http, get_cooldown, get_grid, get_places_info, get_username, ws_index};
use crate::post::{create_post, get_post, get_top_posts};
use crate::user::{check_admin, edit_user, get_top_users, get_user_profile, login_user, register_user};
use crate::voxel::{create_voxel, get_user_voxels, get_voxel, save_voxel};

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
            .service(get_user_profile)
            .service(edit_user)
            .service(save_voxel)
            .service(get_top_users)
            .service(get_user_voxels)
            .service(create_voxel)
            .service(create_post)
            .service(get_top_posts)
            .service(get_post_comments)
            .service(get_place_comments)
            .service(create_comment)
            .service(get_post)
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
