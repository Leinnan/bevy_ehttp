use crate::{HttpRequest, RequestResponse};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::event::{Event, EventWriter};
use bevy::ecs::query::{Added, QueryData};
use bevy::prelude::{App, Update};
use bevy::prelude::{Component, Entity, Query};
use ehttp::{Request, Response};
use serde::Deserialize;
use std::marker::PhantomData;

pub trait RegisterRequestTypeTrait {
    fn register_request_type<T: Send + Sync + 'static>(&mut self) -> &mut Self;
}

impl RegisterRequestTypeTrait for App {
    fn register_request_type<T: Send + Sync + 'static>(&mut self) -> &mut Self {
        self.add_systems(Update, handle_typed_response::<T>)
            .add_event::<TypedResponseEvent<T>>()
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

#[derive(Event, Clone, Debug)]
pub struct TypedResponseEvent<T>
where
    T: Send + Sync,
{
    pub result: Result<Response, String>,
    res: PhantomData<T>,
}

impl<T> TypedResponseEvent<T>
where
    T: for<'a> Deserialize<'a> + Send + Sync,
{
    pub fn parse(&self) -> Option<T> {
        if let Ok(response) = &self.result {
            match response.text() {
                Some(s) => match serde_json::from_str::<T>(s) {
                    Ok(val) => Some(val),
                    _ => None,
                },
                None => None,
            }
        } else {
            None
        }
    }
}

#[derive(QueryData)]
#[query_data(mutable, derive(Debug))]
pub struct TypedRequestQuery<T: Send + Sync + 'static> {
    pub entity: Entity,
    pub response: &'static RequestResponse,
    pub type_info: &'static RequestType<T>,
}

pub fn handle_typed_response<T: Send + Sync + 'static>(
    request_tasks: Query<TypedRequestQuery<T>, Added<RequestResponse>>,
    mut event_writer: EventWriter<TypedResponseEvent<T>>,
) {
    for entry in request_tasks.iter() {
        event_writer.send(TypedResponseEvent::<T> {
            result: entry.response.0.clone(),
            res: PhantomData,
        });
    }
}
