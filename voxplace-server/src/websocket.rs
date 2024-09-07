use crate::place::Place;
use actix_ws::{Message, MessageStream, Session};
use std::sync::{Arc, RwLock};
use futures_util::StreamExt;

pub struct PlaceWebSocketConnection {
    pub place: Arc<RwLock<Place>>,
}

impl PlaceWebSocketConnection {
    pub fn new(place: Arc<RwLock<Place>>) -> Self {
        Self { place }
    }

    pub async fn run(self, mut session: Session, mut msg_stream: MessageStream) {
        {
            let place = self.place.write().unwrap();
            place.voxel.add_session(session.clone());
        }

        while let Some(msg) = msg_stream.next().await {
            match msg {
                Ok(Message::Ping(bytes)) => {
                    if session.pong(&bytes).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Text(text)) => {
                    if session.text(text).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Binary(bin)) => {
                    if session.binary(bin).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Close(reason)) => {
                    let _ = session.close(reason).await;
                    break;
                }
                _ => break,
            }
        }
    }
}