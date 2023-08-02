use crate::database::db::Database;
use crate::database::place::PlaceUserUpdate;
use crate::place::Place;
use crate::voxel::Voxel;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub database: Arc<Mutex<Database>>,
    pub places: HashMap<i64, Arc<Place>>,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        let mut places = HashMap::new();
        database.init();

        let places_infos = match database.get_places_infos() {
            Ok(places_infos) => places_infos,
            Err(e) => {
                eprintln!("Failed to get places info: {}", e);
                vec![]
            }
        };

        for place_info in places_infos {
            if place_info.1 {
                let voxel = match Voxel::read(place_info.2) {
                    Ok(voxel) => voxel,
                    Err(e) => {
                        eprintln!("Failed to read voxel: {}", e);
                        continue;
                    }
                };
                let place = Place::new(place_info.0, true, voxel);
                let place_id = place.id;
                let place_arc = Arc::new(place);
                place_arc.voxel.start_save_loop();
                places.insert(place_id, place_arc);
            }
        }

        Self {
            database: Arc::new(Mutex::new(database)),
            places,
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

    pub fn get_places_updates(&self) -> Vec<PlaceUserUpdate> {
        let mut updates = Vec::new();

        for (_, place) in self.places.iter() {
            updates.extend(place.voxel.get_place_updates());
        }

        updates
    }

    pub fn start_write_place_loop(&self) {
        let updates = self.get_places_updates();
        let database = Arc::clone(&self.database);

        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let db = database.lock().unwrap();
            db.save_places_users(updates.clone()).unwrap();
        });
    }
}
