#[cfg(not(target_os = "windows"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::collections::HashMap;

use crate::net::NetworkServerPlugin;
use actix_web::{
    get, web, App as ActixApp, HttpRequest, HttpResponse, HttpServer, Responder,
};
use bevy::{log::LogPlugin, prelude::*, render::settings::WgpuSettings};
use bevy_rapier3d::prelude::*;
use biorite_generator::SaveFile;
use ed25519_dalek::PublicKey;
use std::cell::Cell;

use std::{
    io,
    sync::{
        atomic::{AtomicU32, Ordering::Relaxed},
        Arc, Mutex,
    },
};

mod auth;
pub mod block_update;
mod chunks;
mod collider;
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
    std::thread::spawn(|| actix_main());
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
        .insert_resource(SaveFile::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(NetworkServerPlugin)
        .run();
}
