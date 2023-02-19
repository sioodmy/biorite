use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use bevy_renet::renet::ConnectToken;
use bincode::serialize;
use ed25519_dalek::{Keypair, Signer};
use reqwest::StatusCode;
use seed15::{
    keypair::keypair_from_seed,
    phrase::{seed_phrase_to_seed, seed_to_seed_phrase},
    random_seed,
};
use uuid::Uuid;

use crate::config::Args;
// use seed15::keypair::*;

pub fn key_gen() -> Keypair {
    // seed.
    let new_seed = random_seed();
    let phrase = seed_to_seed_phrase(new_seed);
    let seed = seed_phrase_to_seed(&phrase).unwrap();

    // Use the seed to create an ed25519 keypair.
    let keypair = keypair_from_seed(seed);

    println!("{:?} {:?}", phrase, serialize(&keypair.public));
    keypair
}

pub fn send_public_key(args: &Args) -> Result<ConnectToken> {
    let client = reqwest::blocking::Client::new();

    let seed = if let Some(phrase) = &args.seed {
        println!("found seed");
        seed_phrase_to_seed(phrase).unwrap()
    } else {
        println!("generating seed");
        let random = random_seed();
        let phrase = seed_to_seed_phrase(random);
        println!("{phrase}");
        random
    };
    let key = keypair_from_seed(seed);
    let uuid = Uuid::new_v4();
    let response = client
        .post("http://127.0.0.1:8080/auth/key")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "key": key.public,
            "uuid": uuid,
        }))
        .send()?;

    let bytes = general_purpose::STANDARD.decode(&response.text()?)?;

    let challenge = client
        .post("http://127.0.0.1:8080/auth/challenge")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "uuid": uuid,
            "sign": key.sign(&bytes),
        }))
        .send()?;

    if challenge.status() == StatusCode::OK {
        let bytes = challenge.bytes()?;
        let token = ConnectToken::read(&mut bytes.as_ref())?;
        return Ok(token);
    }
    Err(anyhow!("Failed to authenticate"))
}
