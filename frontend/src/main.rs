use gloo::net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew::{prelude::*, Renderer};
use sha256::digest as digest;

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
    };
    
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
    };

    let delete_user = {
        let message = message.clone();
        let get_users = get_users.clone();

        Callback::from(move |id: i32| {
            let message = message.clone();
            let get_users = get_users.clone();

            spawn_local(async move {
                let response = Request::delete(&format!("http://127.0.0.1:8000/api/users/{id}"))
                    .send()
                    .await;

                match response {
                    Ok(resp) if resp.ok() => {
                        message.set("User deleted successfully".into());

                        get_users.emit(());
                    }

                    _ => {
                        message.set("Failed to delete user".into())
                    }
                }
            })
        })
    };

    let edit_user = {
        let user_state = user_state.clone();
        let users = users.clone();

        Callback::from(move |id: i32| {
            if let Some(user) = users.iter().find(|u| u.id == id) {
                user_state.set((user.name.clone(), user.hashed_password.clone(), Some(id)));
                
            }
        })
    };

    // html

    html! {
        <div class="container mx-auto p-4"></div>
        <h1 class="text-4xl font-bold text-blue-500 mb-4"> {"User Management"}</h1>
        <div class="mb-4">
            <input type="text" placeholder="Name" class="border border-gray-300 rounded-md p-2 w-full"

            oninput={Callback::from({
                let user_state = user_state.clone();
                move |e| {
                    let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    user_state.set((user_state.0.clone(), input.value(), user_state.2));
                }
                })}

                />
            <input type="password" placeholder="Password" class="border border-gray-300 rounded-md p-2 w-full" 
            oninput={Callback::from({
                let user_state = user_state.clone();
                move |e| {
                    let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                    user_state.set((user_state.0.clone(), user_state.1.clone(), Some(digest(input.value().as_bytes())));
                }
                })}

                />
            <button class="bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded" onclick={ if user_state.2.is_some() { update_user.clone() } else { create_users.clone()}>
                    </button>

                    if !message.is_empty() {
                        <p class="text-red-500 mt-2">{&*message}</p>
                    }
        </div>
        <button 
            class="bg-gray-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded"
            onclick={get_users.reform(|_| ())}
        >
            {"Fetch Users"}
        </button>

        <h2 class="text-2xl font-bold text-blue-500 mb-4">{"Users"}</h2>

        <table class="min-w-full divide-y divide-gray-200">
            <thead class="bg-gray-50">
                <tr>
                    <th class="px-6 py-3 bg-gray-50 text-left text-xs leading-4 font-medium text-gray-500 uppercase tracking-wider">
                        {"Name"}
                    </th>
                    <th class="px-6 py-3 bg-gray-50 text-left text-xs leading-4 font-medium text-gray-500 uppercase tracking-wider">
                        {"Hashed Password"}
                    </th>
                    <th class="px-6 py-3 bg-gray-50 text-left text-xs leading-4 font-medium text-gray-500 uppercase tracking-wider">
                        {"Actions"}
                    </th>
                </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
                {for users.iter().map(|user| html! {
                    <tr>
                        <td class="px-6 py-4 whitespace-nowrap">
                            <div class="text-sm text-gray-900">{&user.name}</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                            <div class="text-sm text-gray-900">{&user.hashed_password}</div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                            <button class="bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded" onclick={edit_user.reform(|_| user.id)}>
                                {"Edit"}
                            </button>
                            <button class="bg-red-500 hover:bg-red-600 text-white font-bold py-2 px-4 rounded" onclick={delete_user.reform(|_| user.id)}>
                                {"Delete"}
                            </button>
                        </td>
                    </tr>
                })}
            </tbody>
        </table>
    }
}

fn main() {
    Renderer::<App>::new().render();
}
