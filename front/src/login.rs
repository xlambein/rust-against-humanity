use yew::prelude::*;
use web_sys::HtmlInputElement;

pub struct Login {
    link: ComponentLink<Self>,
    props: Props,
    username: String,
    username_ref: NodeRef,
}

pub enum Msg {
    Update(String),
    Submit,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub on_submit: Callback<String>,
}

impl Component for Login {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            username: "".to_owned(),
            username_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Update(value) => {
                self.username = value;
            },

            Msg::Submit => {
                self.props.on_submit.emit(self.username.clone());
            }
        }
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html!{
            <>
            <h2>{"Join game"}</h2>
            <div style="display: flex; justify-content: center;">
                // <label for="username">{"Name:"}</label>
                <input
                    ref=self.username_ref.clone()
                    type="text"
                    size="15"
                    id="username"
                    placeholder="Name"
                    oninput=self.link.callback(|e: InputData| Msg::Update(e.value))
                    onkeypress=self.link.batch_callback(|e: KeyboardEvent| {
                        if e.key() == "Enter" { vec![Msg::Submit] } else { vec![] }
                    })
                />
                {"\u{00A0}"}
                <button onclick=self.link.callback(|_| { Msg::Submit })>{"Join"}</button>
            </div>
            </>
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            if let Some(input) = self.username_ref.cast::<HtmlInputElement>() {
                if let Err(_) = input.focus() { log!("Could not focus username field, for some reason??"); }
            }
        }
    }
}
