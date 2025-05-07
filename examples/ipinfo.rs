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
    let req = ehttp::Request::get("https://api.ipify.org/eee?format=json");
    commands.spawn(HttpRequest(req));
}

fn on_response(t: Trigger<OnResponseString>) {
    match &**t {
        Ok(response) => println!("[{:?}]: {:?}", t.url(), response.text()),
        Err(e) => println!("response error: {:?}", e),
    }
}
