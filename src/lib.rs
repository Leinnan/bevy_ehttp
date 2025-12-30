#![doc = include_str!("../README.md")]

#[cfg(feature = "asset_loading")]
mod asset_reader;
mod typed;

#[cfg(feature = "asset_loading")]
use asset_reader::WebAssetReader;
use bevy_app::{App, Plugin, Update};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::EntityEvent,
    prelude::Query,
    query::Without,
    resource::Resource,
    system::{Commands, ResMut},
};
use bevy_tasks::IoTaskPool;
use crossbeam_channel::{Receiver, bounded};

use ehttp::{Request, Response};

pub mod prelude {
    #[cfg(feature = "response_as_component")]
    pub use super::RequestResponse;
    pub use super::typed::{RegisterRequestTypeTrait, RequestBundle, ResponseTyped};
    pub use super::{
        HttpClientSetting, HttpPlugin, HttpRequest, RequestResponseExt, RequestTask, ResponseString,
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
///     .add_plugins((HttpPlugin,DefaultPlugins))
///     .run();
/// ```
#[derive(Default)]
pub struct HttpPlugin;

impl Plugin for HttpPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<HttpClientSetting>() {
            app.init_resource::<HttpClientSetting>();
        }
        app.add_systems(Update, (handle_request, handle_response));

        #[cfg(feature = "asset_loading")]
        {
            use bevy_asset::{AssetApp, io::AssetSourceBuilder};

            app.register_asset_source(
                "http",
                AssetSourceBuilder::new(|| Box::<WebAssetReader<false>>::default()),
            )
            .register_asset_source(
                "https",
                AssetSourceBuilder::new(|| Box::<WebAssetReader<true>>::default()),
            );
        }
    }
}

/// Settings of http client.
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

pub trait RequestResponseExt {
    fn response(&self) -> &Result<Response, String>;

    /// Did we get a 2xx response code?
    fn success(&self) -> bool {
        self.response().as_ref().is_ok_and(|f| f.ok)
    }
    /// The URL we ended up at. This can differ from the request url when we have followed redirects.
    fn url(&self) -> Result<&String, &String> {
        self.response().as_ref().map(|e| &e.url)
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

#[cfg(feature = "response_as_component")]
/// wrap for ehttp response
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct RequestResponse(pub Result<Response, String>);

/// Wraps ehttp response without parsing it.
/// For parsed version use ``
#[derive(EntityEvent, DerefMut, Deref)]
pub struct ResponseString {
    #[deref]
    pub response: Result<Response, String>,
    pub entity: Entity,
}

impl RequestResponseExt for ResponseString {
    fn response(&self) -> &Result<Response, String> {
        &self.response
    }
}

/// task for ehttp response result
#[derive(Component)]
pub struct RequestTask {
    pub receiver: Receiver<Result<Response, ehttp::Error>>,
}

impl RequestTask {
    fn poll(&mut self) -> Option<Result<Response, ehttp::Error>> {
        self.receiver.try_recv().ok()
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
) {
    for (e, mut task) in request_tasks.iter_mut() {
        if let Some(result) = task.poll() {
            let mut cmd = commands.entity(e);
            #[cfg(feature = "response_as_component")]
            cmd.insert(RequestResponse(result.clone()));
            cmd.remove::<RequestTask>();
            cmd.trigger(|entity| ResponseString {
                response: result,
                entity,
            });
            req_res.current_clients -= 1;
        }
    }
}
