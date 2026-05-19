use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
    time: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}

fn format_time(timestamp_ms: u64) -> String {
    let total_secs = timestamp_ms / 1000;
    let hours = (total_secs / 3600) % 24;
    let minutes = (total_secs / 60) % 60;
    format!("{:02}:{:02} UTC", hours, minutes)
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
            <div class="flex w-screen">
                <div class="flex-none w-56 h-screen bg-indigo-50 border-r border-indigo-100">
                    <div class="text-lg font-semibold p-3 text-indigo-800 border-b border-indigo-200">{"Online Users"}</div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-white rounded-lg p-2 shadow-sm">
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="text-xs font-semibold text-gray-700">{u.name.clone()}</div>
                                        <div class="text-xs text-green-500 mt-1">{"● Online"}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col">
                    <div class="w-full h-14 border-b-2 border-indigo-200 bg-indigo-50 flex items-center px-4">
                        <span class="text-xl font-bold text-indigo-800">{"💬 GalihChat"}</span>
                        <span class="ml-3 text-sm text-indigo-400">{"— Real-time chat, powered by Rust & WebAssembly"}</span>
                    </div>
                    <div class="w-full grow overflow-auto border-b-2 border-gray-200 p-2">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class="flex items-end w-3/6 bg-white border border-gray-100 shadow-sm m-4 rounded-tl-lg rounded-tr-lg rounded-br-lg">
                                        <img class="w-8 h-8 rounded-full m-3" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="p-3 flex-grow">
                                            <div class="text-sm font-semibold text-indigo-700">
                                                {m.from.clone()}
                                            </div>
                                            <div class="text-xs text-gray-600 mt-1">
                                                if m.message.ends_with(".gif") {
                                                    <img class="mt-3" src={m.message.clone()}/>
                                                } else {
                                                    {m.message.clone()}
                                                }
                                            </div>
                                            <div class="text-xs text-gray-400 mt-1">
                                                { m.time.map(format_time).unwrap_or_default() }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="w-full h-14 flex px-3 items-center bg-white">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Type a message..." class="block w-full py-2 pl-4 mx-3 bg-gray-100 rounded-full outline-none focus:text-gray-700" name="message" required=true />
                        <button onclick={submit} class="p-3 shadow-sm bg-indigo-600 w-10 h-10 rounded-full flex justify-center items-center hover:bg-indigo-500">
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path><path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}
