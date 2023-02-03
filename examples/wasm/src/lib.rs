mod text_input;
mod token_view;

use std::io::Read;
use std::rc::Rc;

use gloo_worker::{HandlerId, Spawnable, Worker, WorkerBridge, WorkerScope};
use serde::{Deserialize, Serialize};
use yew::{html, Component, Context, Html};

use crate::text_input::TextInput;
use crate::token_view::TokenView;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub surface: String,
    pub pos: String,
    pub pron: String,
}

pub struct WorkerMessage {
    pub id: HandlerId,
    pub output: Vec<Token>,
}

#[ouroboros::self_referencing]
pub struct VibratoWorker {
    tokenizer: vibrato::Tokenizer,
    #[borrows(tokenizer)]
    #[covariant]
    worker: vibrato::tokenizer::worker::Worker<'this>,
}

impl Worker for VibratoWorker {
    type Input = String;
    type Message = WorkerMessage;
    type Output = Vec<Token>;

    fn create(_scope: &WorkerScope<Self>) -> Self {
        let model_data = include_bytes!("system.dic.zst");
        let mut decoder = ruzstd::StreamingDecoder::new(model_data.as_slice()).unwrap();
        let mut buff = vec![];
        decoder.read_to_end(&mut buff).unwrap();
        let dict = vibrato::Dictionary::read(buff.as_slice()).unwrap();
        let tokenizer = vibrato::Tokenizer::new(dict);
        VibratoWorkerBuilder {
            tokenizer,
            worker_builder: |tokenizer| tokenizer.new_worker(),
        }
        .build()
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        let WorkerMessage { id, output } = msg;
        scope.respond(id, output);
    }

    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId) {
        self.with_worker_mut(|worker| {
            worker.reset_sentence(&msg);
            worker.tokenize();
        });
        let output = self
            .borrow_worker()
            .token_iter()
            .map(|token| {
                let mut feature_spl = token.feature().split(',');
                Token {
                    surface: token.surface().to_string(),
                    pos: feature_spl.next().unwrap_or("").to_string(),
                    pron: feature_spl.next().unwrap_or("").to_string(),
                }
            })
            .collect();
        scope.send_message(WorkerMessage { id, output })
    }
}

pub enum Msg {
    SetText(String),
    WorkerResult(Vec<Token>),
}

pub struct App {
    bridge: WorkerBridge<VibratoWorker>,
    text: Rc<String>,
    tokens: Option<Rc<Vec<Token>>>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        let bridge = VibratoWorker::spawner()
            .callback(move |m| {
                link.send_message(Msg::WorkerResult(m));
            })
            .spawn("./vibrato_worker.js");

        // Send a dummy message.
        // The first response indicates that the worker is ready.
        bridge.send(String::new());

        Self {
            bridge,
            text: String::new().into(),
            tokens: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::SetText(text) => {
                self.text = Rc::new(text);
                self.bridge.send(self.text.to_string());
            }
            Msg::WorkerResult(tokens) => {
                self.tokens.replace(Rc::new(tokens));
            }
        };
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <header>
                    <h1>{"🎤 Vibrato Wasm Demo"}</h1>
                    <p class="header-link"><a href="https://github.com/daac-tools/vibrato">{"[Project Page]"}</a></p>
                </header>
                <main>
                    <div>
                        {
                            if self.tokens.is_some() {
                                html! {
                                    <TextInput
                                        callback={ctx.link().callback(Msg::SetText)}
                                        value={Rc::clone(&self.text)}
                                    />
                                }
                            } else {
                                html!{
                                    <input type="text" disabled=true />
                                }
                            }
                        }
                    </div>
                    {
                        if let Some(tokens) = &self.tokens {
                            html! {
                                <TokenView tokens={Rc::clone(&tokens)} />
                            }
                        } else {
                            html! {
                                <div id="loading">{"Loading..."}</div>
                            }
                        }
                    }
                </main>
            </>
        }
    }
}
