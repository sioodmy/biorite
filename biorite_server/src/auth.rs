use std::time::SystemTime;

use actix_web::{
    error, post,
    web::{self, Data},
    Error, HttpResponse,
};
use base64::{engine::general_purpose, Engine as _};
use bevy_renet::renet::ConnectToken;
use biorite_shared::net::protocol::{UserData, PROTOCOL_ID};
use ed25519_dalek::{PublicKey, Signature, Verifier};
use futures::StreamExt;

use rand::Rng;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ChallengeData, Challenges, ADDR, PRIVATE_KEY};

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

    for i in &mut arr {
        *i = rng.gen();
    }

    arr
}

fn get_uuid(conn: &Connection, key: PublicKey) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT uuid FROM keys WHERE key = ?")?;
    stmt.query_row([key.to_bytes()], |row| row.get(0))
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
        let _secret = 2137;
        if entry.key.verify(&entry.bytes, &obj.sign).is_ok() {
            let conn = Connection::open("keys.db").unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS keys(
                    id   INTEGER PRIMARY KEY,
                    uuid TEXT NOT NULL,
                    key BLOB
                )",
                (), // empty list of parameters.
            )
            .unwrap();
            let _o = get_uuid(&conn, entry.key);
            let uuid = if let Ok(Some(uuid)) = get_uuid(&conn, entry.key) {
                uuid
            } else {
                let uuid = Uuid::new_v4();
                conn.execute(
                    "INSERT INTO keys (uuid, key) VALUES (?1, ?2)",
                    (&uuid.to_string(), &entry.key.to_bytes()),
                )
                .unwrap();
                uuid.to_string()
            };

            let user_data = UserData(uuid).to_netcode_user_data();
            let current_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();
            let client_id = current_time.as_millis() as u64;

            let connect_token = ConnectToken::generate(
                current_time,
                PROTOCOL_ID,
                300,
                client_id,
                15,
                vec![*ADDR],
                Some(&user_data),
                &PRIVATE_KEY,
            )
            .expect("Failed to generate connection token");

            let mut bytes = Vec::new();
            connect_token.write(&mut bytes).unwrap();

            Ok(HttpResponse::Ok().body(bytes))
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

    let bytes = generate_challenge();
    challenges.entry(obj.uuid).or_insert(ChallengeData {
        bytes,
        key: obj.key,
    });


    Ok(HttpResponse::Ok().body(general_purpose::STANDARD.encode(bytes)))
}
