use yew::prelude::*;

use schema::{Prompt, Answer};


pub fn view_prompt(prompt: &Prompt) -> Html {
    let class = format!("card {}", "card-prompt");
    let insides = html!{
        <>
        <div class="content">{ &prompt.content }</div>
        {
            if prompt.n_answers > 1 {
                html! { <div class="n_answers">{ format!("PICK {}", prompt.n_answers) }</div> }
            } else {
                html! {}
            }
        }
        </>
    };
    html! {
        <div class=class>{ insides }</div>
    }
}

pub fn view_answer(answer: &Answer, callback: Option<Callback<yew::MouseEvent>>) -> Html {
    let class = format!("card {}", "card-answer");
    let insides = html!{
        <div class="content">{ &answer.content }</div>
    };
    if let Some(callback) = callback {
        html! {
            <div class=class style="cursor: pointer;" onclick=callback>
                { insides }
            </div>
        }
    } else {
        html! {
            <div class=class>{ insides }</div>
        }
    }
}
