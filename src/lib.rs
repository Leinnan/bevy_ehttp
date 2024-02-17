#![doc = include_str!("../README.md")]

mod typed;

use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use crossbeam_channel::{bounded, Receiver};

use ehttp::{Request, Response};

pub mod prelude {
    pub use super::typed::{RegisterRequestTypeTrait, RequestBundle, TypedResponseEvent};
    pub use super::{
        HttpClientSetting, HttpPlugin, HttpRequest, RequestCompleted, RequestResponse, RequestTask,
    };
    pub use ehttp::{Error, Request, Response};
}

/// Plugin that provides support for send http request and handle response.
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use bevy_ehttp::prelude::*;
///
/// App::new()
///     .add_plugins(DefaultPlugins)
///     .add_plugins(HttpPlugin).run();
/// ```
#[derive(Default)]
pub struct HttpPlugin;

impl Plugin for HttpPlugin {
    fn build(&self, app: &mut App) {
        if !app.world.contains_resource::<HttpClientSetting>() {
            app.init_resource::<HttpClientSetting>();
        }
        app.add_systems(Update, (handle_request, handle_response));
        app.add_event::<RequestCompleted>();
    }
}

/// The setting of http client.
/// can set the max concurrent request.
#[derive(Resource)]
pub struct HttpClientSetting {
    /// max concurrent request
    pub max_concurrent: usize,
    current_clients: usize,
}

impl Default for HttpClientSetting {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            current_clients: 0,
        }
    }
}

impl HttpClientSetting {
    /// create a new http client setting
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            current_clients: 0,
        }
    }

    /// check if the client is available
    #[inline]
    pub fn is_available(&self) -> bool {
        self.current_clients < self.max_concurrent
    }
}

/// wrap for ehttp request
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct HttpRequest(pub Request);

impl HttpRequest {
    /// create a new http request
    pub fn new(request: Request) -> Self {
        Self(request)
    }

    /// create a new http get request
    pub fn get(url: impl ToString) -> Self {
        Self(Request::get(url))
    }

    /// create a new http post request
    pub fn post(url: &str, body: Vec<u8>) -> Self {
        Self(Request::post(url, body))
    }
}

/// wrap for ehttp response
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct RequestResponse(pub Result<Response, String>);

#[derive(Event, DerefMut, Deref)]
pub struct RequestCompleted(pub Result<Response, String>);

/// task for ehttp response result
#[derive(Component)]
pub struct RequestTask {
    pub receiver: Receiver<Result<Response, ehttp::Error>>,
}

impl RequestTask {
    fn poll(&mut self) -> Option<Result<Response, ehttp::Error>> {
        if let Ok(v) = self.receiver.try_recv() {
            Some(v)
        } else {
            None
        }
    }
}

fn handle_request(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    requests: Query<(Entity, &HttpRequest), Without<RequestTask>>,
) {
    for (entity, request) in requests.iter() {
        if req_res.is_available() {
            let req = request.clone();
            {
                let (sender, receiver) = bounded(1);

                IoTaskPool::get()
                    .spawn(async move {
                        let result = ehttp::fetch_async(req.0).await;
                        sender.send(result).ok();
                    })
                    .detach();
                commands
                    .entity(entity)
                    .remove::<HttpRequest>()
                    .insert(RequestTask { receiver });
            }
            req_res.current_clients += 1;
        }
    }
}

fn handle_response(
    mut commands: Commands,
    mut req_res: ResMut<HttpClientSetting>,
    mut request_tasks: Query<(Entity, &mut RequestTask)>,
    mut event_writer: EventWriter<RequestCompleted>,
) {
    for (e, mut task) in request_tasks.iter_mut() {
        if let Some(result) = task.poll() {
            commands
                .entity(e)
                .insert(RequestResponse(result.clone()))
                .remove::<RequestTask>();
            event_writer.send(RequestCompleted(result));
            req_res.current_clients -= 1;
        }
    }
}
