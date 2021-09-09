use yew::prelude::*;
use web_sys::HtmlElement;
use yew::services::{RenderService, render::RenderTask};

use schema::Message as WsMsg;
use crate::websocket::WebSocket;


pub struct Notification {
    link: ComponentLink<Self>,
    ws: Box<dyn Bridge<WebSocket>>,
    text: Option<String>,
    dialog_ref: NodeRef,
    transition_task: Option<RenderTask>,
}

pub enum Msg {
    WsMsg(WsMsg),
    Dismiss,
    Transition,
}

impl Component for Notification {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let ws = WebSocket::bridge(link.callback(|msg| Msg::WsMsg(msg)));
        Self {
            link,
            ws,
            text: None,
            dialog_ref: NodeRef::default(),
            transition_task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Dismiss => {
                self.text = None;
                true
            },

            Msg::Transition => {
                if let Some(_) = self.transition_task.take() {
                    if let Some(el) = self.dialog_ref.cast::<HtmlElement>() {
                        el.set_class_name("dialog dialog-visible")
                    }
                }
                false
            }

            Msg::WsMsg(msg) => match msg {
                WsMsg::LoginRejected(_) => {
                    self.text = Some("Login rejected".to_owned());
                    true
                },

                WsMsg::PlayerJoined { name } => {
                    self.text = Some(format!("{} joined the game", name));
                    true
                },

                WsMsg::PlayerLeft { name } => {
                    self.text = Some(format!("{} left the game", name));
                    true
                },

                _ => false,
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        if let Some(text) = &self.text {
            html!{
                <div ref=self.dialog_ref.clone() class="dialog" onclick=self.link.callback(|_| Msg::Dismiss)>
                    { text }
                </div>
            }
        } else {
            html!{}
        }
    }

    fn rendered(&mut self, _: bool) {
        self.transition_task = Some(RenderService::request_animation_frame(self.link.callback(|_| Msg::Transition)));
    }
}
