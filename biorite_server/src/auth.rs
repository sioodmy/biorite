use actix_web::{
    error, post,
    web::{self, Data},
    App, Error, HttpResponse,
};
use ed25519_dalek::{ PublicKey, Signature, Verifier};
use futures::StreamExt;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{Challenges, ChallengeData};

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Serialize, Deserialize)]
struct Welcome {
    key: PublicKey,
    uuid: String,
}

#[derive(Serialize, Deserialize)]
struct SignedChallenge {
    uuid: String,
    sign: Signature,
}

fn generate_challenge() -> [u8; 30] {
    let mut rng = rand::thread_rng();
    let mut arr = [0_u8; 30];

    for i in 0..arr.len() {
        arr[i] = rng.gen();
    }

    arr
}

#[post("/auth/challenge")]
pub async fn challenge(
    mut payload: web::Payload,
    data: Data<Challenges>,
) -> Result<HttpResponse, Error> {
    let mut challenges = data.0.lock().unwrap();
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    let obj = serde_json::from_slice::<SignedChallenge>(&body)?;
    if let Some(entry) = challenges.remove(&obj.uuid) {

    let secret = 2137;
    if entry.key.verify(&entry.bytes, &obj.sign).is_ok(){
        Ok(HttpResponse::Ok().json(secret)) 
    } else {
        Ok(HttpResponse::Forbidden().body("Authentication failed"))
    }
    } else {
        Ok(HttpResponse::Forbidden().body("Authentication failed"))
    }
}

#[post("/auth/key")]
pub async fn public_key(
    mut payload: web::Payload,
    data: Data<Challenges>,
) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();
    let mut challenges = data.0.lock().unwrap();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }

    let obj = serde_json::from_slice::<Welcome>(&body)?;
    println!("amogus {:?}", obj.key);

    let bytes = generate_challenge();
    challenges.entry(obj.uuid).or_insert(ChallengeData { bytes, key: obj.key });

    Ok(HttpResponse::Ok().json(bytes))
}
