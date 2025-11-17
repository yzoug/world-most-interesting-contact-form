use crate::structs::*;
use crate::utils::{email_message, email_token};

use std::path::Path;
use tokio::fs::{read_to_string, remove_file, write};

use axum::Json;
use axum::http::StatusCode;

use log::{debug, error, warn};

use rand::Rng;
use uuid::Uuid;
use validator::Validate;

pub async fn post_pgp_message(
    Json(payload): Json<MessageSubmission>,
) -> Result<String, StatusCode> {
    debug!("HIT on post_pgp_message");

    // verify the user input
    if let Err(e) = payload.validate() {
        warn!("BAD REQUEST: received PGP payload is not valid. Error: {e}");
        return Err(StatusCode::BAD_REQUEST);
    };

    // generate a unique identifier for this message
    // this is what we'll return to the user if everything goes well
    let message_uuid = Uuid::new_v4().to_string();

    // generate a random token
    // sent by email to the user to verify the supplied email address
    let mut token = String::new();
    {
        // the "rng" variable has to go out of scope before any await
        let mut rng = rand::rng();
        for _ in 0..TOKEN_LENGTH {
            token.push(rng.sample(rand::distr::Alphanumeric) as char);
        }
    }

    debug!("Token for submitted PGP payload generated");

    // store the message submission as JSON to disk
    let Ok(json_message) = serde_json::to_string(&payload) else {
        error!("INTERNAL SERVER ERROR: couldn't convert the PGP payload to JSON string");
        error!("Corresponding PGP payload: {:?}", payload);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(storage_path) = std::env::var("AXUM_BACKEND_STORAGE_PATH") else {
        error!("INTERNAL SERVER ERROR: AXUM_BACKEND_STORAGE_PATH env var not set");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    match write(
        format!("{}/{}_{}.pgp_message", storage_path, message_uuid, token),
        json_message,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            error!("INTERNAL SERVER ERROR: couldn't write the PGP payload to disk");
            error!("Error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // email the token to the user
    // if successful, return the message UUID to the user
    match email_token(&payload.reply_to, &token).await {
        StatusCode::OK => {
            debug!("Token corresponding to received PGP payload successfully sent");
            Ok(message_uuid)
        }
        s => Err(s),
    }
}

pub async fn post_pgp_token(Json(payload): Json<TokenSubmission>) -> StatusCode {
    debug!("HIT on post_pgp_token");

    // verify the user input
    if let Err(e) = payload.validate() {
        warn!("BAD REQUEST: received token payload is not valid. Error: {e}");
        return StatusCode::BAD_REQUEST;
    };

    // check if a PGP message with the ID and corresponding token exists
    let Ok(storage_path) = std::env::var("AXUM_BACKEND_STORAGE_PATH") else {
        error!("INTERNAL SERVER ERROR: AXUM_BACKEND_STORAGE_PATH env var not set");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    let file_path = format!(
        "{}/{}_{}.pgp_message",
        storage_path, payload.msg_id, payload.token
    );

    if !Path::new(&file_path).exists() {
        warn!("NOT FOUND: received ID/token pair not found on disk");
        return StatusCode::NOT_FOUND;
    }

    // read the data
    let Ok(file_data) = read_to_string(&file_path).await else {
        error!("INTERNAL SERVER ERROR: couldn't read existing PGP message from disk");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(pgp_message) = serde_json::from_str(&file_data) else {
        error!("INTERNAL SERVER ERROR: couldn't convert existing PGP message to struct");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };

    // delete the file
    if let Err(e) = remove_file(file_path).await {
        error!(
            "INTERNAL SERVER ERROR: couldn't delete the file containing PGP message, after reading it. Error: {e}"
        );
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    email_message(pgp_message).await
}
