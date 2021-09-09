extern crate web_sys;

use anyhow::Error;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::format::Json;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::agent::Bridge;
use std::rc::Rc;
use std::convert::TryFrom;
use std::collections::HashMap;

use schema::{Message as WsMsg, Role, Prompt, Answer};
use crate::answer_selector::AnswerSelector;
use crate::judgement::Judgement;
use crate::websocket::WebSocket;
use crate::cards::{view_prompt, view_answer};

struct RoundResults {
    prompt: Prompt,
    winner: String,
    winning_answers: Vec<Answer>,
    scores: HashMap<String, u64>,
}

pub struct Round {
    link: ComponentLink<Self>,
    props: Props,
    state: State,
    ws: Box<dyn Bridge<WebSocket>>,
    results: Option<RoundResults>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    // pub ws: Callback<WsMsg>,
    pub role: Role,
    pub prompt: Prompt,
    pub hand: Vec<Answer>,
    pub on_exit: Callback<()>,
}

#[derive(Debug)]
enum State {
    // Player states
    SelectingAnswers,
    WaitingForAnswersApproval(Vec<Answer>),
    WaitingForOtherPlayers(Vec<Answer>),
    AwaitingJudgement(HashMap<usize, Vec<Answer>>),
    // Czar states
    WaitingForAnswers,
    JudgingAnswers(HashMap<usize, Vec<Answer>>),
    // Common states
}

pub enum Msg {
    SubmitAnswer(Vec<Answer>),
    SubmitJudgement(usize),
    RoundExited,
    WsMsg(WsMsg),
}

// macro_rules! map(
//     { $($key:expr => $value:expr),+ } => {
//         {
//             let mut m = ::std::collections::HashMap::new();
//             $(
//                 m.insert($key, $value);
//             )+
//             m
//         }
//      };
// );

impl Component for Round {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let ws = WebSocket::bridge(link.callback(|msg| Msg::WsMsg(msg)));
        let state = match &props.role {
            Role::Player => State::SelectingAnswers,
            Role::Czar => State::WaitingForAnswers
        };
        // let winning_answers = props.hand[..props.prompt.n_answers as usize].to_vec();
        // let results = RoundResults {
        //     prompt: props.prompt.clone(),
        //     winner: "Jesus".to_owned(),
        //     winning_answers,
        //     scores: map!{ "Jesus".to_owned() => 5, "Yoda".to_owned() => 1, "Sufjan Stevens".to_owned() => 6 },
        // };
        Self {
            link,
            props,
            state,
            ws,
            results: None,
            // results: Some(results),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SubmitAnswer(answers) => {
                // Submit to server
                self.ws.send(WsMsg::SubmitAnswer(answers.clone()));
                // Switch state
                self.state = State::WaitingForAnswersApproval(answers);

                false
            },

            Msg::SubmitJudgement(id) => {
                // Submit to server
                self.ws.send(WsMsg::SubmitJudgement(id));
                // Switch state
                // self.state = State::WaitingForJudgementApproval(answers);

                false
            },

            Msg::RoundExited => {
                self.results = None;
                self.props.on_exit.emit(());

                true
            }

            Msg::WsMsg(msg) => match msg {
                WsMsg::AnswerRejected => todo!(),

                WsMsg::AnswerAccepted => {
                    if let State::WaitingForAnswersApproval(answers) = &self.state {
                        // Remove selected answers from hand
                        self.props.hand.retain(|answer| !answers.contains(answer));
                        // TODO find a way to do this without cloning
                        self.state = State::WaitingForOtherPlayers(answers.clone());
                        true
                    } else {
                        log!("error: WsMsg::AnswerAccepted: no answer was submitted");
                        false
                    }
                },

                WsMsg::ReadyToJudge(answers) => {
                    log!("Ready to judge");
                    match self.props.role {
                        Role::Czar => {
                            log!("I'm a Czar ready to judge");
                            self.state = State::JudgingAnswers(answers);
                        },
                        Role::Player => {
                            self.state = State::AwaitingJudgement(answers);
                        }
                    }
                    true
                },

                WsMsg::RoundEnded { winner, winning_answers, scores } => {
                    self.results = Some(RoundResults {
                        prompt: self.props.prompt.clone(),
                        winner,
                        winning_answers,
                        scores,
                    });
                    // self.state = State::DisplayingResults { winner, winning_answers, scores };
                    true
                },

                _ => false,
            },
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.state = match &props.role {
            Role::Player => State::SelectingAnswers,
            Role::Czar => State::WaitingForAnswers,
        };
        self.props = props;
        true
    }

    fn view(&self) -> Html {
        if let Some(RoundResults{
            prompt,
            winner,
            winning_answers,
            scores,
        }) = &self.results {
            let mut sorted_scores = scores
                .iter()
                .collect::<Vec<_>>();
            // Sort scores from bigger to smaller
            sorted_scores.sort_by(|(_, &a), (_, &b)| b.cmp(&a));
            html!{
                <>
                <h2>{ format!("{} has won!", winner) }</h2>
                <div style="display: flex; justify-content: center; flex-wrap: wrap;">
                    { view_prompt(&prompt) }
                    <div style="display: flex; justify-content: center; flex-wrap: wrap;">
                        {
                            for winning_answers.iter().map(|answer| {
                                view_answer(&answer, None)
                            })
                        }
                    </div>
                </div>
                <table class="scores">
                    <tr><th class="left">{"Player"}</th><th class="right">{"Score"}</th></tr>
                    {
                        for sorted_scores.iter().map(|(name, score)| html!{
                            <tr><td class="left">{name}</td><td class="right">{score}</td></tr>
                        })
                    }
                </table>
                <div class="next-round">
                    <button onclick=self.link.callback(|_| Msg::RoundExited)>{"Next round"}</button>
                </div>
                </>
            }
        } else {
            match &self.state {
                // Player states

                State::SelectingAnswers | State::WaitingForAnswersApproval(_) => html!{
                    <>
                    <h2>{"Select your answer"}</h2>
                    <AnswerSelector
                        hand=self.props.hand.clone(),
                        prompt=self.props.prompt.clone(),
                        submitted=self.link.callback(|answers| Msg::SubmitAnswer(answers))
                    />
                    </>
                },

                State::WaitingForOtherPlayers(answers) => html!{
                    // TODO
                    <>
                    <h2>{"Waiting for other players to finish..."}</h2>
                    <div style="display: flex; justify-content: center; flex-wrap: wrap;">
                        { view_prompt(&self.props.prompt) }
                        <div style="display: flex; justify-content: center; flex-wrap: wrap;">
                            {
                                for answers.iter().map(|answer| {
                                    view_answer(&answer, None)
                                })
                            }
                        </div>
                    </div>
                    </>
                },

                State::AwaitingJudgement(answers) => {
                    html!{
                        <Judgement
                            prompt=self.props.prompt.clone()
                            answers=answers.clone()
                        />
                    }
                },

                // Czar states

                State::WaitingForAnswers => html!{
                    <>
                    <h2>{"Waiting for players to select their answer..."}</h2>
                    <div style="display: flex; justify-content: center; flex-wrap: wrap;">
                        { view_prompt(&self.props.prompt) }
                    </div>
                    </>
                },

                State::JudgingAnswers(answers) => {
                    html!{
                        <Judgement
                            prompt=self.props.prompt.clone()
                            answers=answers.clone()
                            on_judge=self.link.callback(|i| Msg::SubmitJudgement(i))
                        />
                    }
                },

                // Common states

                state => {
                    log!("unimplemented state {:?}", state);
                    todo!()
                }
            }
        }
    }
}
