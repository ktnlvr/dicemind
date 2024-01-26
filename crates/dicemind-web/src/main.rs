use dicemind::prelude::*;
use log::{info, Level};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let output = use_state(String::default);
    let output_value = (*output).clone();
    
    let on_change = {
        let output = output.clone();
        
        Callback::from(move |e: Event| {
            let mut roller = FastRoller::default();
            let target = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input {
                let input = input.value();
                info!("{}", &input);
                match parse(&input) {
                    Ok(result) => match roller.roll(result) {
                        Ok(res) => output.set(res.to_string()),
                        Err(err) => output.set(format!("err. {}", err)),
                    },
                    Err(err) => output.set(format!("err. {}", err)),
                }
            }
        })
    };

    html! {
        <div>
            <input type={"text"} onchange={on_change}/>
            <p> {output_value} </p>
        </div>
    }
}

fn main() {
    let _ = console_log::init_with_level(Level::Debug);
    yew::Renderer::<App>::new().render();
}
