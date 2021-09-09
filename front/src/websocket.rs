use anyhow::Error;
use std::collections::HashSet;
use yew::worker::*;
use yew::format::Json;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

use schema::Message as WsMsg;

static URL: &str = "192.168.1.61";  // .1.5 at my place
static PORT: u16 = 8000;

pub struct WebSocket {
    link: AgentLink<Self>,
    ws: Option<WebSocketTask>,
    subscribers: HashSet<HandlerId>,
}

pub enum Msg {
    WsMsg(WsMsg),
    WsReceiveError(Error),
    WsOpen,
    WsClosed,
    WsError,
}

impl WebSocket {
    fn connect(&mut self) -> Result<(), ()> {
        if self.ws.is_some() {
            return Ok(())
        }

        let callback = self.link.callback(|Json(data)| {
            // Msg::WsMsg(data)
            match data {
                Ok(msg) => Msg::WsMsg(msg),
                Err(err) => Msg::WsReceiveError(err),
            }
        });
        let notification = self.link.callback(|status| match status {
            WebSocketStatus::Opened => Msg::WsOpen,
            WebSocketStatus::Closed => Msg::WsClosed,
            WebSocketStatus::Error => Msg::WsError,
        });

        match WebSocketService::connect(&format!("ws://{}:{}/", URL, PORT), callback, notification) {
            Ok(ws) => {
                self.ws = Some(ws);
                Ok(())
            },
            Err(_) => {
                Err(())
            }
        }
    }
}

impl Agent for WebSocket {
    type Reach = Context<Self>;
    type Message = Msg;
    type Input = WsMsg;
    type Output = WsMsg;

    fn create(link: AgentLink<Self>) -> Self {
        let mut this = Self {
            link,
            ws: None,
            subscribers: HashSet::new(),
        };
        this.connect();
        this
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::WsMsg(msg) => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, msg.clone());
                }
            },

            Msg::WsClosed => {
                self.ws = None;
            },

            Msg::WsError => {
                log!("web socket send error");
            }

            _ => (),
        }
    }

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        // Ensure that we're connected
        self.connect();

        match &mut self.ws {
            Some(ws) => ws.send(Json(&msg)),
            None => {
                log!("could not send message: socket not connected");
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
