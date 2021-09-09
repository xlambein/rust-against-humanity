#![recursion_limit = "1024"]

extern crate web_sys;

#[macro_use]
mod util;
mod answer_selector;
mod cards;
mod judgement;
mod login;
mod notification;
mod round;
mod websocket;

use anyhow::Error;
use std::convert::TryFrom;
use wasm_bindgen::prelude::*;
use yew::agent::Bridge;
use yew::format::Json;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

use login::Login;
use notification::Notification;
use round::Round;
use schema::{Answer, LoginRejectedReason, Message as WsMsg, Prompt, Role};
use websocket::WebSocket;

struct Model {
    link: ComponentLink<Self>,
    ws: Box<dyn Bridge<WebSocket>>,
    // hand: Vec<Answer>,
    state: State,
}

enum State {
    LoggingIn {
        error: Option<LoginRejectedReason>,
    },
    WaitingForNextRound,
    OngoingRound {
        role: Role,
        prompt: Prompt,
        hand: Vec<Answer>,
    },
}

enum Msg {
    Login(String),
    RoundExited,
    WsSend(WsMsg),
    WsOpen,
    // WsReady(Result<WsMsg, Error>),
    WsMsg(WsMsg),
    WsError(Error),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let ws = WebSocket::bridge(link.callback(|msg| Msg::WsMsg(msg)));
        Self {
            link,
            ws,
            state: State::LoggingIn { error: None },
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::WsSend(msg) => {
                self.ws.send(msg);
                false
            }

            Msg::WsOpen => true,

            Msg::WsError(err) => {
                log!("error: WsError: {:?}", err);
                false
            }

            Msg::WsMsg(msg) => match msg {
                WsMsg::LoginAccepted => {
                    log!("Login accepted!");
                    self.state = State::WaitingForNextRound;
                    true
                }

                WsMsg::LoginRejected(reason) => {
                    log!("Login rejected :(");
                    self.state = State::LoggingIn {
                        error: Some(reason),
                    };
                    true
                }

                WsMsg::JoinedLobby => todo!(),

                WsMsg::NewRound { role, prompt, hand } => {
                    log!("New round");
                    self.state = State::OngoingRound { role, prompt, hand };
                    true
                }

                WsMsg::GameEnded => {
                    self.state = State::WaitingForNextRound;
                    true
                }

                // WsMsg::PlayerJoined => {

                // }

                // WsMsg::RoundTimeout,
                // WsMsg::ReadyToJudge(answers_list) => todo!(),
                // WsMsg::JudgementRejected => todo!(),
                // WsMsg::JudgementTimeout,
                // WsMsg::RoundEnded { winner, scores } => todo!(),
                // _ => unreachable!(),
                _ => false,
            },

            Msg::RoundExited => false,

            Msg::Login(username) => {
                log!("Logging in as {}...", username);
                self.ws.send(WsMsg::Login(username));
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let state_view = match &self.state {
            State::LoggingIn { error } => html! {
                <>
                <Login on_submit=self.link.callback(|username| Msg::Login(username)) />
                {
                    if let Some(error) = error {
                        let error = match error {
                            LoginRejectedReason::UsernameIsTaken => "Username already taken".to_owned(),
                            LoginRejectedReason::GameIsFull => "Ongoing game is full".to_owned(),
                        };
                        html!{
                            <span style="color: red">{error}</span>
                        }
                    } else { html!{} }
                }
                </>
            },

            State::WaitingForNextRound => html! {
                <h2>{"Waiting for the next round to begin..."}</h2>
            },

            State::OngoingRound { role, prompt, hand } => html! {
                <Round
                    role=role,
                    prompt=prompt,
                    hand=hand.clone(),
                    on_exit=self.link.callback(|_| Msg::RoundExited)
                />
            },
        };
        html! {
            <>
            <Notification />
            { state_view }
            </>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    let app = App::<Model>::new();
    app.mount_to_body();
}
