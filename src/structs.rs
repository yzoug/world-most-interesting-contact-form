use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct MessageSubmission {
    #[validate(email)]
    pub reply_to: String,
    #[validate(custom(function = "crate::utils::is_valid_pgp_message"))]
    pub pgp_message: String,
}

static TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9]+$").unwrap());

// regex adapted from https://gist.github.com/johnelliott/cf77003f72f889abbc3f32785fa3df8d?permalink_comment_id=4318506#gistcomment-4318506
static MSG_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$").unwrap()
});

// the length of the token used to validate email
pub const TOKEN_LENGTH: u64 = 30;

#[derive(Deserialize, Debug, Validate)]
pub struct TokenSubmission {
    #[validate(regex(path = *MSG_ID_REGEX))]
    pub msg_id: String,
    #[validate(length(equal = TOKEN_LENGTH), regex(path = *TOKEN_REGEX))]
    pub token: String,
}
