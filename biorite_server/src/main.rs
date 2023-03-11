#![warn(clippy::disallowed_types)]
#![feature(addr_parse_ascii)]
#[cfg(not(target_os = "windows"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use actix_extensible_rate_limit::{
    backend::{memory::InMemoryBackend, SimpleInputFunctionBuilder},
    RateLimiter,
};
use config::Args;
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use std::net::SocketAddr;

use crate::net::NetworkServerPlugin;
use actix_web::{
    get, web, App as ActixApp, HttpResponse, HttpServer, Responder,
};
use bevy::{log::LogPlugin, prelude::* };
use bevy_rapier3d::prelude::*;
use bevy_renet::renet::generate_random_bytes;
use biorite_generator::SaveFile;
use clap::Parser;
use ed25519_dalek::PublicKey;
use std::time::Duration;

use std::sync::{Arc, Mutex};

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
pub struct Challenges(Arc<Mutex<FxHashMap<String, ChallengeData>>>);

lazy_static! {
    pub static ref PRIVATE_KEY: [u8; 32] = generate_random_bytes();
    pub static ref ARGS: Args = Args::parse();
    // Parse ip's and fail early if required
    pub static ref ADDR_AUTH: SocketAddr = SocketAddr::parse_ascii(ARGS.auth_ip.as_bytes()).expect("Failed to parse auth server IP");
    pub static ref ADDR: SocketAddr = SocketAddr::parse_ascii(ARGS.ip.as_bytes()).expect("Failed to parse server IP");
}

#[actix_web::main]
async fn actix_main() -> std::io::Result<()> {
    let hashmap = Challenges(Arc::new(Mutex::new(FxHashMap::default())));

    let backend = InMemoryBackend::builder().build();
    HttpServer::new(move || {
        let input = SimpleInputFunctionBuilder::new(Duration::from_secs(15), 4)
            .real_ip_key()
            .build();
        let middleware = RateLimiter::builder(backend.clone(), input)
            .add_headers()
            .build();
        ActixApp::new()
            .service(index)
            .service(auth::public_key)
            .service(auth::challenge)
            .app_data(web::Data::new(hashmap.clone()))
            .wrap(middleware)
    })
    .bind(*ADDR_AUTH)?
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
