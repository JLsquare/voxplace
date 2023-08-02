use crate::place::Place;
use crate::voxel::UpdateMessage;
use actix::{Actor, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws;
use serde_json::json;
use std::sync::Arc;

pub struct PlaceWebSocketConnection {
    pub place: Arc<Place>,
}

impl Actor for PlaceWebSocketConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.place.voxel.add_session(addr);
    }
}

impl Handler<UpdateMessage> for PlaceWebSocketConnection {
    type Result = ();

    fn handle(&mut self, msg: UpdateMessage, ctx: &mut Self::Context) {
        ctx.text(
            serde_json::to_string(&json!({
                "type": "update",
                "x": msg.0,
                "y": msg.1,
                "z": msg.2,
                "color": msg.3,
            }))
            .unwrap(),
        );
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PlaceWebSocketConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}
