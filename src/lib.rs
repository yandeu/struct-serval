#![forbid(unsafe_code)]
// test code in README file
#[doc = include_str!("../README.md")]
#[allow(unused_imports)]
#[macro_use]
pub extern crate validator;

pub extern crate sanitizer;

extern crate rocket;

use rocket::{
    data::{Data, FromData, Outcome as DataOutcome},
    http::Status,
    outcome::Outcome,
    request::Request,
    serde::{json::Json, Serialize},
};
use sanitizer::Sanitize;
use std::fmt::Debug;
pub use validator::{Validate, ValidationErrors};

///  Struct used for Request Guards
#[derive(Clone, Debug)]
pub struct Validated<T>(pub T);

///  Impl to get type T of `Json`
impl<T> Validated<Json<T>> {
    pub fn into_deep_inner(self) -> T {
        self.0 .0
    }
}

/// Struct representing errors sent by the catcher
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Error<'a> {
    code: u128,
    message: &'a str,
    errors: Option<&'a ValidationErrors>,
}

///  Wrapper used to store `ValidationErrors` within the scope of the request
#[derive(Clone)]
pub struct CachedValidationErrors(pub Option<ValidationErrors>);

///  Implementation of `Validated` for `Json`
//
///  An example with `Json`
///  ```rust
///  # #[macro_use] extern crate rocket;
///  use struct_serval::sanitizer::prelude::*;
///  use struct_serval::{Validate, Validated};
///  use serde::{Deserialize, Serialize};
///  use rocket::serde::json::Json;
///  
///  #[derive(Debug, Deserialize, Serialize, Sanitize, Validate)]
///  pub struct HelloData {
///      #[sanitize(trim)]
///      #[validate(length(min = 1))]
///      name: String,
///      #[validate(range(min = 0, max = 100))]
///      age: u8,
///  }
//
///  #[post("/hello", format = "application/json", data = "<data>")]
///  fn validated_hello(data: Validated<Json<HelloData>>) -> Json<HelloData> {
///      Json(data.into_deep_inner())
///  }
///  ```
#[rocket::async_trait]
impl<'r, D: Validate + Sanitize + rocket::serde::Deserialize<'r>> FromData<'r>
    for Validated<Json<D>>
{
    type Error = Result<ValidationErrors, rocket::serde::json::Error<'r>>;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> DataOutcome<'r, Self> {
        let data_outcome = <Json<D> as FromData<'r>>::from_data(req, data).await;

        match data_outcome {
            Outcome::Failure((status, err)) => Outcome::Failure((status, Err(err))),
            Outcome::Forward(err) => Outcome::Forward(err),
            Outcome::Success(mut data) => {
                data.sanitize();

                return match data.validate() {
                    Ok(_) => Outcome::Success(Validated(data)),
                    Err(err) => {
                        req.local_cache(|| CachedValidationErrors(Some(err.to_owned())));
                        Outcome::Failure((Status::BadRequest, Ok(err)))
                    }
                };
            }
        }
    }
}
