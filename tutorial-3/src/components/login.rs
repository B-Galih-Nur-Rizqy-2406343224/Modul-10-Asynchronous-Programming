use web_sys::HtmlInputElement;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::User;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let user = use_context::<User>().expect("No context found.");

    let oninput = {
        let current_username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = user.clone();
        Callback::from(move |_| *user.username.borrow_mut() = (*username).clone())
    };

    html! {
        <div class="bg-indigo-900 flex w-screen h-screen">
            <div class="container mx-auto flex flex-col justify-center items-center">
                <h1 class="text-white text-5xl font-bold mb-3">{"💬 GalihChat"}</h1>
                <p class="text-indigo-300 text-base mb-8">{"Real-time chat, powered by Rust & WebAssembly"}</p>
                <form class="flex">
                    <input {oninput} class="rounded-l-lg p-4 border-t mr-0 border-b border-l text-gray-800 border-gray-200 bg-white" placeholder="Enter your username" />
                    <Link<Route> to={Route::Chat}>
                        <button {onclick} disabled={username.len()<1} class="px-8 rounded-r-lg bg-violet-500 text-white font-bold p-4 uppercase border-violet-500 border-t border-b border-r hover:bg-violet-400">
                            {"Go Chatting!"}
                        </button>
                    </Link<Route>>
                </form>
            </div>
        </div>
    }
}
