use crate::{HttpRequest, RequestResponseExt, ResponseString};
use bevy_app::App;
use bevy_derive::Deref;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::observer::On;
use bevy_ecs::system::{Commands, Query};
use bevy_ecs::{bundle::Bundle, component::Component};
use ehttp::{Request, Response};
use serde::Deserialize;
use std::marker::PhantomData;

pub trait RegisterRequestTypeTrait {
    fn register_request_type<T: Send + Sync + 'static + for<'a> Deserialize<'a>>(
        &mut self,
    ) -> &mut Self;
}

impl RegisterRequestTypeTrait for App {
    fn register_request_type<T: Send + Sync + 'static + for<'a> Deserialize<'a>>(
        &mut self,
    ) -> &mut Self {
        self.add_observer(on_typed_response::<T>)
    }
}

/// RequestBundle provides easy way to create request that after
/// completing it will add TypedResponseEvent with T type
#[derive(Bundle, Debug, Clone)]
pub struct RequestBundle<T>
where
    T: Send + Sync + 'static,
{
    /// request that will be wrapped up
    pub request: HttpRequest,
    pub request_type: RequestType<T>,
}

impl<T> RequestBundle<T>
where
    T: Send + Sync + 'static,
{
    /// Recomended way to create a new RequestBundle of a given type.
    pub fn new(request: Request) -> Self {
        Self {
            request: HttpRequest(request),
            request_type: RequestType::<T>(PhantomData),
        }
    }

    /// Creates a new RequestBundle that will send a GET request to the given URL.
    pub fn get(url: impl ToString) -> Self {
        let req = ehttp::Request::get(url);
        Self::new(req)
    }

    /// Adds a header to the request.
    pub fn with_header(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.request.headers.insert(key, value);
        self
    }
}

#[derive(Component, Debug, Clone)]
pub struct RequestType<T>(pub PhantomData<T>);

#[derive(EntityEvent, Clone, Debug, Deref)]
pub struct ResponseTyped<T>
where
    T: Send + Sync,
{
    pub request: Result<Response, String>,
    #[deref]
    pub data: Option<T>,
    pub entity: Entity,
}

impl<T> ResponseTyped<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync,
{
    fn new(response: &ResponseString) -> Self {
        let data = Self::try_parse(response);
        ResponseTyped::<T> {
            request: response.response.clone(),
            data,
            entity: response.entity,
        }
    }

    pub fn try_parse(response: &ResponseString) -> Option<T> {
        if let Ok(response) = &**response {
            match response.text() {
                Some(s) => serde_json::from_str::<T>(s).ok(),
                None => None,
            }
        } else {
            None
        }
    }
}

impl<T> RequestResponseExt for ResponseTyped<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync,
{
    fn response(&self) -> &Result<Response, String> {
        &self.request
    }
}

pub fn on_typed_response<T: Send + Sync + 'static + for<'a> Deserialize<'a>>(
    trigger: On<ResponseString>,
    request_tasks: Query<&RequestType<T>>,
    mut commands: Commands,
) {
    let e = trigger.entity;
    if request_tasks.contains(e) {
        commands.trigger(ResponseTyped::<T>::new(&trigger));
    }
}
