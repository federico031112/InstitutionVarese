use axum::http::{StatusCode};
use axum::{
    extract::{Path, State},
    Json
};
use std::sync::Arc;
use crate::db::data::SedeIstituzionale;
use crate::db::store::Db;
use crate::auth::jwt::Claims;

pub async fn get_sede_by_id (Path(id): Path<u32>, State(db): State<Arc<Db>>) ->  Result<Json<SedeIstituzionale>, StatusCode>{
    match db.search_by_id(id) {
        Some(sede) => Ok(Json(sede)),
        
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_sede_by_comune (Path(comune): Path<String>, State(db): State<Arc<Db>>) -> Json<Vec<SedeIstituzionale>> {
    Json(db.search_by_comune(comune))
}

pub async fn get_sede_by_name (Path(name): Path<String>, State(db): State<Arc<Db>>) -> Json<Vec<SedeIstituzionale>> {
    Json(db.search_by_name(name))
}

pub async fn get_sede_by_tipology (Path(tipology): Path<String>, State(db): State<Arc<Db>>) -> Json<Vec<SedeIstituzionale>> {
    Json(db.search_by_tipology(tipology))
}

pub async fn create_sede (claims: Claims, State(db): State<Arc<Db>>, Json(sede): Json<SedeIstituzionale>) -> Result<Json<String>, StatusCode> {
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

pub async fn remove_sede (claims: Claims, State(db): State<Arc<Db>>, Path(id): Path<u32>) -> Result<Json<String>, StatusCode> {
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