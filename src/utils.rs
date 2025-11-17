use crate::structs::MessageSubmission;

use axum::http::StatusCode;
use lettre::message::{Message, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use log::{debug, error};
use validator::ValidationError;

pub fn is_valid_pgp_message(pgp_message: &str) -> Result<(), ValidationError> {
    debug!("Validating received PGP message");

    // discard messages that are too long
    // https://www.youtube.com/watch?v=cJRcxmNAiQo
    let line_count = pgp_message.lines().count();
    if line_count > 200 {
        return Err(ValidationError::new("PGP message is too long"));
    }

    // then validate the PGP message line by line
    let mut iterator = pgp_message.lines();

    // the first line should contain the PGP header only
    if iterator.next() != Some("-----BEGIN PGP MESSAGE-----") {
        return Err(ValidationError::new(
            "Malformed PGP message: first line doesn't contain PGP header",
        ));
    }

    // the second line should be an empty line
    if iterator.next() != Some("") {
        return Err(ValidationError::new(
            "Malformed PGP message: second line isn't empty",
        ));
    }

    // all other lines should contain only alphanumerical, '-', '+', '/' or '=' characters
    for (index, line) in iterator.enumerate() {
        // discard message if a line is longer than 60 characters (what OpenPGPjs produces)
        if line.len() > 60 {
            return Err(ValidationError::new(
                "Malformed PGP message: line length greater than 60 characters",
            ));
        }

        // only these characters are allowed
        // we also allow ' ' and '-' for the last line header
        if !line.chars().all(|c| {
            c.is_alphanumeric() || c == '+' || c == '/' || c == '=' || c == '-' || c == ' '
        }) {
            return Err(ValidationError::new(
                "Malformed PGP message: unexpected character in message",
            ));
        }

        // the last line should contain the PGP end header
        // "- 3" because the first two lines of the message where already consumed
        // before entering into this loop
        if index == line_count - 3 && line != "-----END PGP MESSAGE-----" {
            return Err(ValidationError::new(
                "Malformed PGP message: last line doesn't contain PGP header",
            ));
        }
    }
    Ok(())
}

pub async fn email_token(user_email_address: &str, token: &str) -> StatusCode {
    debug!("Building email containing the token");
    let Ok(from_email_address) = std::env::var("AXUM_BACKEND_EMAIL_FROM") else {
        error!("INTERNAL SERVER ERROR: AXUM_BACKEND_EMAIL_FROM env var not set");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let email_body = format!(
        "Hello, and thank you for using the zoug.fr contact form (https://zoug.fr/world-most-interesting-contact-form). You'll need to input this token to verify your email address:\n\n{token}\n\nYou can safely ignore this email if you didn't use the form."
    );
    let email = Message::builder()
        .from(from_email_address.parse().unwrap())
        .subject("Your submission token")
        .to(user_email_address.parse().unwrap())
        .header(ContentType::TEXT_PLAIN)
        .body(email_body)
        .unwrap();

    // Send the email
    send_email(&email).await
}

pub async fn email_message(payload: MessageSubmission) -> StatusCode {
    debug!("Building email containing the PGP message");
    let Ok(my_email_address) = std::env::var("AXUM_BACKEND_EMAIL_RECIPIENT") else {
        error!("INTERNAL SERVER ERROR: AXUM_BACKEND_EMAIL_RECEIPIENT env var not set");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(from_email_address) = std::env::var("AXUM_BACKEND_EMAIL_FROM") else {
        error!("INTERNAL SERVER ERROR: AXUM_BACKEND_EMAIL_FROM env var not set");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let email = Message::builder()
        .from(from_email_address.parse().unwrap())
        .subject("Email from your contact form")
        // the email address has already been validated
        .reply_to((payload.reply_to).parse().unwrap())
        .to(format!("Me <{}>", my_email_address).parse().unwrap())
        // sending our PGP message in the body with a text/plain content-type works,
        // but the modern way of doing this is through MIME
        .header(ContentType::TEXT_PLAIN)
        .body(payload.pgp_message)
        .unwrap();

    // Send the email
    send_email(&email).await
}

async fn send_email(payload: &Message) -> StatusCode {
    debug!("Sending email");
    // SMTP credentials
    let Ok(smtp_username) = std::env::var("AXUM_BACKEND_SMTP_USERNAME") else {
        error!("INTERNAL SERVER ERROR: AXUM_BACKEND_SMTP_USERNAME env var not set");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let Ok(smtp_token) = std::env::var("AXUM_BACKEND_SMTP_TOKEN") else {
        error!("INTERNAL SERVER ERROR: AXUM_BACKEND_SMTP_TOKEN env var not set");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let creds = Credentials::new(smtp_username.to_owned(), smtp_token.to_owned());

    // Open a remote connection to your SMTP relay
    let Ok(smtp_server) = std::env::var("AXUM_BACKEND_SMTP_SERVER") else {
        error!("INTERNAL SERVER ERROR: AXUM_BACKEND_SMTP_SERVER env var not set");
        return StatusCode::INTERNAL_SERVER_ERROR;
    };
    let mailer = SmtpTransport::relay(&smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(payload) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            error!("INTERNAL SERVER ERROR: Couldn't send the email. Error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
