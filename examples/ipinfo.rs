use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_ehttp::prelude::*;

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, HttpPlugin))
        .add_systems(
            Update,
            send_request.run_if(on_timer(std::time::Duration::from_secs(1))),
        )
        .add_observer(on_response)
        .run();
}

fn send_request(mut commands: Commands) {
    commands.spawn(HttpRequest::get("https://api.ipify.org?format=json"));
}

fn on_response(t: On<ResponseString>) {
    match &**t {
        Ok(response) => println!("[{:?}]: {:?}", t.url(), response.text()),
        Err(e) => println!("response error: {:?}", e),
    }
}
