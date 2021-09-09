use yew::prelude::*;

use schema::{Prompt, Answer};

use crate::cards::{view_prompt, view_answer};

pub struct AnswerSelector {
    link: ComponentLink<Self>,
    props: Props,
    selected_answers: Vec<Option<Answer>>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub hand: Vec<Answer>,
    pub prompt: Prompt,
    pub submitted: Callback<Vec<Answer>>,
}

pub enum Msg {
    SelectAnswer(usize),
    UnselectAnswer(usize),
    SubmitAnswer,
}

impl Component for AnswerSelector {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let selected_answers = vec![None; props.prompt.n_answers as usize];
        Self {
            link,
            props,
            selected_answers,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SelectAnswer(i) => {
                log!("#{}: \"{}\"", i, self.props.hand[i].content);
                let next_answer = self.selected_answers
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x.is_none())
                    .map(|(i, _)| i)
                    .next()
                    .unwrap_or(self.props.prompt.n_answers as usize - 1);
                
                if let Some(old_answer) = self.selected_answers[next_answer].replace(self.props.hand.remove(i)) {
                    self.props.hand.push(old_answer);
                }
            }

            Msg::UnselectAnswer(i) => {
                if let Some(old_answer) = self.selected_answers[i].take() {
                    self.props.hand.push(old_answer);
                }
            }

            Msg::SubmitAnswer => {
                let answers = self.selected_answers.clone()
                    .into_iter()
                    .flatten()
                    .collect();
                self.props.submitted.emit(answers);
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        self.selected_answers = vec![None; self.props.prompt.n_answers as usize];
        true
    }

    fn view(&self) -> Html {
        let can_submit = self.selected_answers.iter().all(Option::is_some);
        html! {
            <div>
                <div style="display: flex; justify-content: center; flex-wrap: wrap;">
                    { view_prompt(&self.props.prompt) }
                    <div style="display: flex; justify-content: center; flex-wrap: wrap;">
                        {
                            for self.selected_answers.iter().enumerate().map(|(i, answer)| {
                                if let Some(ref answer) = answer {
                                    view_answer(answer, Some(self.link.callback(move |_| Msg::UnselectAnswer(i))))
                                } else {
                                    html!{ <div class="card card-placeholder"></div> }
                                }
                            })
                        }
                    </div>
                </div>
                <div style="display: flex; justify-content: center;">
                    {
                        if can_submit {
                            html! {
                                <button class="submit-answer" onclick=self.link.callback(|_| Msg::SubmitAnswer)>{"Submit Answer"}</button>
                            }
                        } else {
                            html! {
                                <button class="submit-answer" disabled=true>{"Submit Answer"}</button>
                            }
                        }
                    }
                </div>
                <div style="display: flex; justify-content: center; flex-wrap: wrap;">
                    {
                        for self.props.hand.iter().enumerate().map(|(i, answer)| {
                            view_answer(answer, Some(self.link.callback(move |_| Msg::SelectAnswer(i))))
                        })
                    }
                </div>
            </div>
        }
    }
}
