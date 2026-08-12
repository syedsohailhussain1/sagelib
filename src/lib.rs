#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn hello_world(name: String) -> String {
  format!("Hello {} from sagelib's Rust Core!", name)
}
