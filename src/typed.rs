use crate::{HttpRequest, OnResponseString, RequestResponseExt};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EntityEvent;
use bevy::ecs::observer::On;
use bevy::prelude::{App, Commands, Deref};
use bevy::prelude::{Component, Query};
use ehttp::{Request, Response};
use serde::Deserialize;
use std::marker::PhantomData;
use std::ops::Deref;

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
pub struct OnResponseTyped<T>
where
    T: Send + Sync,
{
    pub request: Result<Response, String>,
    #[deref]
    pub data: Option<T>,
    pub entity: Entity,
}

impl<T> OnResponseTyped<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync,
{
    fn new(response: &OnResponseString) -> Self {
        let data = Self::try_parse(response);
        OnResponseTyped::<T> {
            request: response.response.clone(),
            data,
            entity: response.entity,
        }
    }

    pub fn try_parse(response: &OnResponseString) -> Option<T> {
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

impl<T> RequestResponseExt for OnResponseTyped<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync,
{
    fn response(&self) -> &Result<Response, String> {
        &self.request
    }
}

pub fn on_typed_response<T: Send + Sync + 'static + for<'a> Deserialize<'a>>(
    trigger: On<OnResponseString>,
    request_tasks: Query<&RequestType<T>>,
    mut commands: Commands,
) {
    let e = trigger.entity;
    if request_tasks.contains(e) {
        commands.trigger(OnResponseTyped::<T>::new(trigger.deref()));
    }
}
