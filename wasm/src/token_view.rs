use std::rc::Rc;

use yew::{function_component, html, Html, Properties};

use crate::Token;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub tokens: Rc<Vec<Token>>,
}

#[function_component(TokenView)]
pub fn token_view(props: &Props) -> Html {
    let tokens = &props.tokens;

    html! {
        <table>
            <thead>
                <tr>
                    <th>{"Surface"}</th>
                    <th>{"Part-of-Speech"}</th>
                    <th>{"Pronunciation"}</th>
                </tr>
            </thead>
            <tbody>
                {
                    for tokens.iter().map(|Token { surface, pos, pron }| html! {
                        <tr>
                            <td>{surface}</td>
                            <td>{pos}</td>
                            <td>{pron}</td>
                        </tr>
                    })
                }
            </tbody>
        </table>
    }
}
