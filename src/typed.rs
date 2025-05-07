use crate::{HttpRequest, OnResponseString, RequestResponseExt};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::event::Event;
use bevy::ecs::observer::Trigger;
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
            .add_event::<OnResponseTyped<T>>()
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
}

#[derive(Component, Debug, Clone)]
pub struct RequestType<T>(pub PhantomData<T>);

#[derive(Event, Clone, Debug, Deref)]
pub struct OnResponseTyped<T>
where
    T: Send + Sync,
{
    pub request: Result<Response, String>,
    #[deref]
    pub data: Option<T>,
}

impl<T> OnResponseTyped<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync,
{
    fn new(response: &OnResponseString) -> Self {
        let data = Self::try_parse(response);
        OnResponseTyped::<T> {
            request: response.0.clone(),
            data,
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
    trigger: Trigger<OnResponseString>,
    request_tasks: Query<&RequestType<T>>,
    mut commands: Commands,
) {
    let e = trigger.target();
    if request_tasks.contains(e) {
        commands.trigger_targets(OnResponseTyped::<T>::new(trigger.deref()), e);
    }
}
