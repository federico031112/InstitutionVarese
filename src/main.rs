use axum::{
    routing::{get, post},
    Router
};
use std::sync::Arc;
use std::net::SocketAddr;
use colored::{self, Colorize};

mod auth;

mod db;
use db::store::Db;

mod api;
use api::handler::{get_sede_by_comune, get_sede_by_name, get_sede_by_tipology, get_sede_by_id, remove_sede, create_sede};


#[tokio::main]
async fn main() {
    let database = Arc::new(Db::new("sedi.json"));

    let app = Router::new()
        .route("/sedi/:id", get(get_sede_by_id).delete(remove_sede))
        .route("/sedi/comuni/:comune", get(get_sede_by_comune))
        .route("/sedi/nomi/:name", get(get_sede_by_name))
        .route("/sedi/tipology/:tipology", get(get_sede_by_tipology))
        .route("/sedi", post(create_sede))
        .with_state(database);

    let address = SocketAddr::from(([127,0,0,1], 3000));
    println!("{}",("Microservice online on http://127.0.0.1:3000").bright_blue());

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
