use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_ehttp::prelude::*;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, HttpPlugin))
        .add_systems(Update, handle_response)
        .add_systems(
            Update,
            send_request.run_if(on_timer(std::time::Duration::from_secs(1))),
        )
        .run()
}

fn send_request(mut commands: Commands) {
    let req = ehttp::Request::get("https://api.ipify.org?format=json");
    commands.spawn(HttpRequest(req));
}

fn handle_response(mut requests: EventReader<RequestCompleted>) {
    for request in &mut requests.read() {
        match &**request {
            Ok(response) => println!("response: {:?}", response.text()),
            Err(e) => println!("response error: {:?}", e),
        }
    }
}
