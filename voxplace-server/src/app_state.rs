use crate::database::db::Database;
use crate::database::place::PlaceUserUpdate;
use crate::place::Place;
use crate::voxel::Voxel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub database: Arc<Mutex<Database>>,
    pub places: HashMap<i64, Arc<Place>>,
    last_update: i64,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        let mut places = HashMap::new();
        database.init();

        let places_infos = match database.get_places_infos() {
            Ok(places_infos) => {
                places_infos
            },
            Err(e) => {
                eprintln!("Failed to get places info: {}", e);
                vec![]
            }
        };

        for place_info in places_infos {
            println!("Place info: {:?}", place_info);
            if place_info.online {
                let voxel_id = place_info.voxel_id.parse::<i64>().unwrap();
                let place_id = place_info.place_id.parse::<i64>().unwrap();

                let voxel_info = match database.get_voxel_info(voxel_id) {
                    Ok(voxel_info) => voxel_info,
                    Err(e) => {
                        eprintln!("Failed to get voxel info: {}", e);
                        continue;
                    }
                };

                let voxel = match Voxel::read(&voxel_info.path, voxel_info.voxel_id) {
                    Ok(voxel) => voxel,
                    Err(e) => {
                        eprintln!("Failed to read voxel: {}", e);
                        continue;
                    }
                };
                let place = Place::new(place_id, true, place_info.cooldown, voxel);
                let place_id = place.id;
                let place_arc = Arc::new(place);
                place_arc.voxel.start_save_loop();
                places.insert(place_id, place_arc);
            }
        }

        Self {
            database: Arc::new(Mutex::new(database)),
            places,
            last_update: 0,
        }
    }

    pub fn add_place(&mut self, place: Place) {
        match self.database.lock() {
            Ok(database) => match database.save_new_voxel(&place.voxel) {
                Ok(_) => {
                    println!("Added new voxel with id {}", place.voxel.id)
                }
                Err(e) => {
                    eprintln!("Failed to save new voxel: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Failed to lock the database: {}", e);
            }
        }

        match self.database.lock() {
            Ok(database) => match database.save_new_place(&place) {
                Ok(_) => {
                    let place_id = place.id;
                    let place_arc = Arc::new(place);
                    place_arc.voxel.start_save_loop();
                    self.places.insert(place_id, place_arc);
                    println!("Added new place with id {}", place_id)
                }
                Err(e) => {
                    eprintln!("Failed to save new place: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Failed to lock the database: {}", e);
            }
        }
    }

    pub fn get_places_updates(&mut self) -> Vec<PlaceUserUpdate> {
        let mut updates = Vec::new();

        for (_, place) in self.places.iter_mut() {
            let place_updates = place.get_place_updates();
            for update in place_updates {
                updates.push(update);
            }
        }

        updates
    }

    pub fn places_updates(&mut self) {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        if time - self.last_update > 5 {
            self.last_update = time;
            let updates = self.get_places_updates();
            match self.database.lock().unwrap().save_places_users(updates) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to save updates: {}", e);
                }
            }
        }
    }
}