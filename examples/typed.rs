use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_ehttp::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IpInfo {
    pub ip: String,
}
fn main() {
    App::new()
        .add_plugins((MinimalPlugins, HttpPlugin))
        .add_systems(Update, handle_response)
        .add_systems(
            Update,
            send_request.run_if(on_timer(std::time::Duration::from_secs(1))),
        )
        .register_request_type::<IpInfo>()
        .run();
}

fn send_request(mut commands: Commands) {
    let req = ehttp::Request::get("https://api.ipify.org?format=json");
    commands.spawn(RequestBundle::<IpInfo>::new(req));
}

fn handle_response(mut responses: EventReader<TypedResponseEvent<IpInfo>>) {
    for response in &mut responses.read() {
        match response.parse() {
            Some(v) => {
                println!("response: {:#?}", v);
            }
            None => {
                println!("Failed to parse: {:#?}", response.result);
            }
        }
    }
}
