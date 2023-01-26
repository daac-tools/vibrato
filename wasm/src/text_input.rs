use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Event, HtmlInputElement, InputEvent};
use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub value: String,
    pub callback: Callback<String>,
}

pub struct TextInput {
    node_ref: NodeRef,
}

impl Component for TextInput {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Props { value, callback } = ctx.props().clone();

        let oninput = Callback::from(move |e: InputEvent| {
            let event: Event = e.dyn_into().unwrap_throw();
            let event_target = event.target().unwrap_throw();
            let target: HtmlInputElement = event_target.dyn_into().unwrap_throw();
            callback.emit(target.value());
        });

        html! {
            <input
                ref={self.node_ref.clone()}
                type="text"
                placeholder="Enter Japanese here"
                {value}
                {oninput}
            />
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(input) = self.node_ref.cast::<HtmlInputElement>() {
                input.focus().unwrap();
            }
        }
    }
}
