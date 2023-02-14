use bincode::serialize;
use ed25519_dalek::{Keypair, Signature, Signer};
use reqwest::{Client, StatusCode};
use seed15::{
    keypair::keypair_from_seed,
    phrase::{seed_phrase_to_seed, seed_to_seed_phrase},
    random_seed,
};
use uuid::Uuid;
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

pub async fn send_public_key() -> Result<(), reqwest::Error> {
    let client = Client::new();

    let key = key_gen();
    let uuid = Uuid::new_v4();
    let response = client
        .post("http://127.0.0.1:8080/auth/key")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "key": key.public,
            "uuid": uuid,
        }))
        .send()
        .await?;

    let bytes= serde_json::from_str::<Vec<u8>>(&response.text().await?).unwrap();

    let challenge= client
        .post("http://127.0.0.1:8080/auth/challenge")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "uuid": uuid,
            "sign": key.sign(&bytes),
        }))
        .send()
        .await?;

    if challenge.status() == StatusCode::OK {
        println!("noji fariancik");
    }
    Ok(())
}
