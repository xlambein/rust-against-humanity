use yew::prelude::*;
use std::collections::HashMap;

use schema::{Prompt, Answer};

use crate::cards::{view_prompt, view_answer};

pub struct Judgement {
    link: ComponentLink<Self>,
    props: Props,
    selection: Option<usize>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub prompt: Prompt,
    pub answers: HashMap<usize, Vec<Answer>>,
    #[prop_or_default]
    pub on_judge: Option<Callback<usize>>,
}

pub enum Msg {
    Select(usize),
    // Unselect(usize),
    Submit,
}

impl Component for Judgement {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            selection: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Select(i) => {
                if self.can_judge() {
                    self.selection = Some(i);
                    true
                } else {
                    false
                }
            },

            Msg::Submit => {
                if let Some(i) = self.selection {
                    if let Some(on_judge) = &self.props.on_judge {
                        on_judge.emit(i);
                    } else {
                        log!("error: trying to submit when player isn't Czar");
                    }
                } else {
                    log!("error: trying to submit when no judgement has been selected");
                }
                false
            },
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        self.selection = None;
        true
    }

    fn view(&self) -> Html {
        let view_submission = |(i, answers): (&usize, &Vec<Answer>)| {
            let i = *i;
            let is_selected = self.selection.map(|sel| sel == i).unwrap_or(false);
            html! {
                <div
                    class={if is_selected {"selected"} else {""}}
                    style="display: flex; justify-content: center; flex-wrap: wrap;"
                    onclick=self.link.callback(move |_| Msg::Select(i))
                >
                    { view_prompt(&self.props.prompt) }
                    <div style="display: flex; justify-content: center; flex-wrap: wrap;">
                        {
                            for answers.iter().map(|answer| {
                                view_answer(&answer, None)
                            })
                        }
                    </div>
                </div>
            }
        };
        html!{
            <>
            <h2>{
                if self.can_judge() {  // Are we Czar?
                    "T\u{00A0}I\u{00A0}M\u{00A0}E\u{00A0} \u{00A0}F\u{00A0}O\u{00A0}R\u{00A0} \u{00A0}J\u{00A0}U\u{00A0}D\u{00A0}G\u{00A0}E\u{00A0}M\u{00A0}E\u{00A0}N\u{00A0}T"
                } else {
                    "A\u{00A0}W\u{00A0}A\u{00A0}I\u{00A0}T\u{00A0}I\u{00A0}N\u{00A0}G\u{00A0} \u{00A0}J\u{00A0}U\u{00A0}D\u{00A0}G\u{00A0}E\u{00A0}M\u{00A0}E\u{00A0}N\u{00A0}T"
                }
            }</h2>
            { for self.props.answers.iter().map(view_submission) }
            {
                if self.can_judge() {
                    html!{
                        <div style="display: flex; justify-content: center;">
                            <button
                                class="submit-answer"
                                onclick=self.link.callback(|_| Msg::Submit)
                                disabled=self.selection.is_none()
                            >{"Submit"}</button>
                        </div>
                    }
                } else {
                    html!{}
                }
            }
            </>
        }
    }
}

impl Judgement {
    fn can_judge(&self) -> bool {
        self.props.on_judge.is_some()
    }
}
