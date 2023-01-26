use yew::{function_component, html, Html, Properties};

use crate::Token;

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub tokens: Vec<Token>,
}

#[function_component(TokenView)]
pub fn token_view(props: &Props) -> Html {
    let Props { tokens } = props.clone();

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
                    for tokens.into_iter().map(|Token { surface, pos, pron }| html! {
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
