#![allow(unused_variables)]
extern crate byteorder;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;

#[macro_use]
extern crate serde_derive;

extern crate actix;
extern crate actix_web;

mod app;
mod message;
mod reversi;
mod server;

use app::App;

fn main() {
    App::start();
}
