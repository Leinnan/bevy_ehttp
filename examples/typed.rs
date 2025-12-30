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
        .add_systems(
            Update,
            send_request.run_if(on_timer(std::time::Duration::from_secs(1))),
        )
        .register_request_type::<IpInfo>()
        .run();
}

fn send_request(mut commands: Commands) {
    commands
        .spawn(RequestBundle::<IpInfo>::get(
            "https://api.ipify.org?format=json",
        ))
        .observe(handle_response);
}

fn handle_response(response: On<ResponseTyped<IpInfo>>) {
    println!("Response: {:?}", **response);
}
