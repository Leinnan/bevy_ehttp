# bevy_ehttp

[![Crates.io](https://img.shields.io/crates/v/bevy_ehttp)](https://crates.io/crates/bevy_ehttp)
[![Documentation](https://docs.rs/bevy_ehttp/badge.svg)](https://docs.rs/bevy_ehttp)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/bevy_ehttp/bevy_ehttp#license)

A ehttp Bevy Plugin that works both on native and on WASM.

Simple request will invoke `RequestCompleted(pub Result<Response, String>)` event once completed.

There is also option to call typed request that will allow to deserialize response to given type by using `RequestBundle<T>`. More details available in [typed.rs example](examples/typed.rs).

## Example

```rust
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

```

## Thanks

Big thanks to the creators of the Bevy Engine and to the [foxzool](https://github.com/foxzool) user for creating [bevy_http_client](https://github.com/foxzool/bevy_http_client) that this plugin is based on.

## License

`bevy_ehttp` is dual-licensed under MIT and Apache 2.0 at your option.

## Bevy compatibility table

Bevy version | Crate version
--- | ---
0.13 | 0.2
0.12 | 0.1