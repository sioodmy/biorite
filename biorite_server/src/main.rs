#![feature(addr_parse_ascii)]
#[cfg(not(target_os = "windows"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use config::Args;
use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::net::NetworkServerPlugin;
use actix_web::{
    get, web, App as ActixApp, HttpResponse, HttpServer, Responder,
};
use bevy::{log::LogPlugin, prelude::*, render::settings::WgpuSettings};
use bevy_rapier3d::prelude::*;
use bevy_renet::renet::generate_random_bytes;
use biorite_generator::SaveFile;
use clap::Parser;
use ed25519_dalek::PublicKey;


use std::{
    sync::{
        Arc, Mutex,
    },
};

mod auth;
pub mod block_update;
mod chunks;
mod collider;
mod config;
mod net;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Clone)]
pub struct ChallengeData {
    bytes: [u8; 30],
    key: PublicKey,
}

#[derive(Default, Clone)]
pub struct Challenges(Arc<Mutex<HashMap<String, ChallengeData>>>);

lazy_static! {
    pub static ref PRIVATE_KEY: [u8; 32] = generate_random_bytes();
    pub static ref ARGS: Args = Args::parse();
}

#[actix_web::main]
async fn actix_main() -> std::io::Result<()> {
    let hashmap = Challenges(Arc::new(Mutex::new(HashMap::new())));

    HttpServer::new(move || {
        ActixApp::new()
            .service(index)
            .service(auth::public_key)
            .service(auth::challenge)
            .app_data(web::Data::new(hashmap.clone()))
    })
    .bind(("127.0.0.1", 8080))?
    .disable_signals()
    .run()
    .await
}

#[tokio::main]
async fn main() {
    println!("{:?}", *PRIVATE_KEY);
    let args = config::Args::parse();
    std::thread::spawn(actix_main);
    App::new()
        .insert_resource(WgpuSettings {
            backends: None,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: bevy::log::Level::ERROR,
            filter:
                "error,wgpu_core=warn,wgpu_hal=warn,biorite_server=info".into(),
        }))
        .insert_resource(args)
        .insert_resource(SaveFile::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(NetworkServerPlugin)
        .run();
}
