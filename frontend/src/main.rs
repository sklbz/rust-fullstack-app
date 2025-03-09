use gloo::net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::{prelude::*, Renderer};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    id: i32,
    name: String,
    hashed_password: String,
}

#[function_component(App)]
fn app() -> Html {
    let user_state = use_state(|| ("".to_string(), None as Option<i32>));
    let message = use_state(|| "".to_string());
    let users = use_state(|| Vec::new());

    let get_users = {
        let users = users.clone();
        let message = message.clone();

        Callback::from(move |_| {
            let users = users.clone();
            let message = message.clone();

            spawn_local(async move {{
                match Request::get("127.0.0.1:8000/api/users").send().await {
                    Ok(response) if response.ok() => {
                            let fetched_users: Vec<User> = response.json().await.unwrap_or_default();
                            users.set(fetched_users);
                    }

                    _ => {
                        message.set("Failed to fetch users".to_string());
                    }
                }
            }})
        })
    };

    let create_users = {
        let users_state = user_state.clone();
        let message = message.clone();
        let get_users = get_users.clone();

        Callback::from(move |_| {
            let (name, hashed_password) = users_state.as_ref().clone();
            let message = message.clone();
            let get_users = get_users.clone();

            spawn_local(async move {
                let users_data = serde_json::json!({ "name": name, "hashed_password": hashed_password });

                let response = Request::post("127.0.0.1:8000/api/users")
                    .header("Content-Type", "application/json")
                    .body(users_data.to_string())
                    .send()
                    .await;

                match response {
                    Ok(response) if response.ok() => {
                        get_users.emit(());
                    }
                    _ => {
                        message.set("Failed to create user".to_string());
                    }
                }

                user_state.set(("".to_string(), "".to_string()), None);
            })
        })
    }
    
    let update_user = {
        let users_state = user_state.clone();
        let message = message.clone();
        let get_users = get_users.clone();

        Callback::from(move |_| {
            let (name, hashed_password, editing_user_id) = users_state.as_ref().clone();
            let user_state = users_state.clone();
            let message = message.clone();
            let get_users = get_users.clone();

            if let Some(id) = editing_user_id {
                spawn_local(async move {
                    let response = Request::put(format!("127.0.0.1:8000/api/users/{}", id))
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&(id, name.as_str(), hashed_password.as_str())).unwrap())
                        .send()
                        .await;

                    match response {
                        Ok(response) if response.ok() => {
                            message.send("User updated successfully".into());
                            get_users.emit(());
                        }

                        _ => {
                            message.send("Failed to update user".into());
                        }
                    }

                    user_state.set(("".to_string(), "".to_string()), None);
                })
            }
        })
    }
}

fn main() {
    Renderer::<App>::new().render();
}
