# Struct Serval

Struct Validator and Sanitizer using https://crates.io/crates/validator and https://crates.io/crates/sanitizer.

Inspired by https://crates.io/crates/rocket-validation

## Example

```rust
#[macro_use] extern crate rocket;
use struct_serval::sanitizer::prelude::*;
use struct_serval::{Validate, Validated};
use serde::{Deserialize, Serialize};
use rocket::serde::json::Json;

#[derive(Debug, Default, Deserialize, Serialize, Sanitize, Validate)]
pub struct User {
    #[sanitize(trim)]
    #[validate(length(min = 6))]
    name: String,
    #[sanitize(trim, lower_case)]
    #[validate(email)]
    email: String,
    #[validate(range(min = 0, max = 100))]
    age: Option<u8>,
}

#[post("/hello", format = "application/json", data = "<data>")]
fn validated_hello(data: Validated<Json<User>>) -> Json<User> {
    Json(data.into_deep_inner())
}

// test
fn main() {
    let mut hello = User {
        name: "John Doe".to_string(),
        email: " John.Doe@example.com".to_string(),
        ..Default::default()
    };
    hello.sanitize();
    hello.validate();
    assert_eq!(hello.email, "john.doe@example.com".to_string());
}
```
