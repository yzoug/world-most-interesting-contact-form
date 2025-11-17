mod routes;
mod structs;
mod utils;

use crate::routes::*;

use axum::{Router, routing::post};

#[tokio::main]
async fn main() {
    // initialize our logger
    env_logger::init();

    // create our app router
    let app = Router::new()
        .route("/pgp/post", post(post_pgp_message))
        .route("/pgp/token", post(post_pgp_token));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
