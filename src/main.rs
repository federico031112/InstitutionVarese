use axum::http::{StatusCode};
use axum::{
    routing::{get, post},
    extract::{Path, State},
    Json,
    Router
};
use std::sync::Arc;
use std::net::SocketAddr;
use colored::{self, Colorize};


mod auth;
use auth::jwt::{Claims};
mod db;
use db::store::Db;
use db::data::SedeIstituzionale;

async fn get_sede_by_id (Path(id): Path<u32>, State(db): State<Arc<Db>>) ->  Result<Json<SedeIstituzionale>, StatusCode>{
    match db.search_by_id(id) {
        Some(sede) => Ok(Json(sede)),
        
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_sede_by_comune (Path(comune): Path<String>, State(db): State<Arc<Db>>) -> Json<Vec<SedeIstituzionale>> {
    Json(db.search_by_comune(comune))
}

async fn get_sede_by_name (Path(name): Path<String>, State(db): State<Arc<Db>>) -> Json<Vec<SedeIstituzionale>> {
    Json(db.search_by_name(name))
}

async fn get_sede_by_tipology (Path(tipology): Path<String>, State(db): State<Arc<Db>>) -> Json<Vec<SedeIstituzionale>> {
    Json(db.search_by_tipology(tipology))
}

async fn create_sede (claims: Claims, State(db): State<Arc<Db>>, Json(sede): Json<SedeIstituzionale>) -> Result<Json<String>, StatusCode> {
    if claims.role != "admin"{
        return Err(StatusCode::FORBIDDEN);
    }
    
    let res = db.insert(sede);
    match res {
        Some(res) => {
            let _ = db.save_on_disk();
            Ok(Json(res))
        }

        None => {
            Err(StatusCode::CONFLICT)
        }
    }
    
}

async fn remove_sede (claims: Claims, State(db): State<Arc<Db>>, Path(id): Path<u32>) -> Result<Json<String>, StatusCode> {
   if claims.role != "admin"{
        return Err(StatusCode::FORBIDDEN);
    }
    
    
    let res = db.remove(id);
    match res {
        Some(res) => {
            let _ = db.save_on_disk();
            Ok(Json(res))
        }

        None => {
            Err(StatusCode::NOT_FOUND)
        }
    }
}

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
