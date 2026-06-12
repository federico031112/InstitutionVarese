use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::fs::File;
use std::io::{Read, Write};
use axum::http::{StatusCode, HeaderMap};
use serde::{Serialize, Deserialize};
use axum::{
    routing::{get, post},
    extract::{Path, State},
    Json,
    Router
};
use std::sync::Arc;
use std::net::SocketAddr;
use colored::{self, Colorize};

const ADMIN_TOKEN: &str = "tokenadminsegreto131203";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SedeIstituzionale {
    id: u32,
    nome: String,
    comune: String,
    indirizzo: String,
    tipologia: String
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
struct DbState {
    key_value_store: HashMap<u32, SedeIstituzionale>,
    free_id: Vec<u32>,
    max_id: u32,
    unique_values: HashSet<String>
}


struct Db {
    file_path: String,
    key_value_store: RwLock<DbState>
}



impl Db {
    fn new(path: &str) -> Self {
        let mut store = DbState::default();

        if let Ok(mut f) = File::open(path) {
            let mut contenuto = String::new();
            if let Ok(_) = f.read_to_string(&mut contenuto) {
                if let Ok(data) = serde_json::from_str(&contenuto) {
                    store = data;
                }
            }
        }

        Db {
            file_path: path.to_string(),
            key_value_store: RwLock::new(store)
        }
        
    }

    fn save_on_disk(&self) -> Result<(), std::io::Error>{
        if let Ok(lock) = self.key_value_store.read() {
            if let Ok(json) = serde_json::to_string_pretty(&*lock) {
                if let Ok(mut f) = File::create(&self.file_path) {
                    f.write_all(json.as_bytes())?;

                }
            }
        }
        Ok(())
    }

    fn insert(&self, sede: SedeIstituzionale) -> Option<String>{
        if let Ok(mut lock) = self.key_value_store.write() {
            let mut sede_con_id = sede;
            let firma = format!("{}:{}:{}",sede_con_id.nome.to_lowercase(),sede_con_id.comune.to_lowercase(),sede_con_id.indirizzo.to_lowercase());
            if lock.unique_values.contains(&firma) {
                return None;
            }
            if lock.free_id.is_empty() {
                sede_con_id.id = lock.max_id + 1;
                lock.max_id = sede_con_id.id;
            }else {
                if let Some(newid) = lock.free_id.pop() {
                    sede_con_id.id = newid;
                }
            }
            lock.key_value_store.insert(sede_con_id.id, sede_con_id);
            lock.unique_values.insert(firma);
            return Some("OK".to_string())
        }
        None
    }

    fn search_by_id(&self, id: u32) -> Option<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.key_value_store.get(&id).cloned();
        }
        return None;
    }

    fn search_by_name(&self, nome: String) -> Vec<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.key_value_store.values().filter(|s| s.nome.to_lowercase().contains(&nome.to_lowercase())).cloned().collect();
        }
        return Vec::new();
    }

    fn search_by_tipology(&self, tipologia: String) -> Vec<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.key_value_store.values().filter(|s| s.tipologia.to_lowercase() == tipologia.to_lowercase()).cloned().collect();
        }
        return Vec::new();
    }

    fn search_by_comune(&self, comune: String) -> Vec<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.key_value_store.values().filter(|s| s.comune.to_lowercase() == comune.to_lowercase()).cloned().collect();
        }
        Vec::new()
    }

    fn remove(&self, id: u32) -> Option<String> {
        if let Ok(mut lock) = self.key_value_store.write() {
            let sede: Option<SedeIstituzionale>;
            if let Some(sede_con_id) = lock.key_value_store.remove(&id) {
                let firma = format!("{}:{}:{}",sede_con_id.nome.to_lowercase(),sede_con_id.comune.to_lowercase(),sede_con_id.indirizzo.to_lowercase());
                lock.unique_values.remove(&firma);
                sede = Some(sede_con_id);
            }else {
                sede = None;
            }
            let flagrem = sede.is_some();
            if flagrem {
                lock.free_id.push(id);
                if id == lock.max_id {
                    if let Some(newmaxid) = lock.key_value_store.keys().max(){
                        lock.max_id = *newmaxid;
                    }else {
                        lock.max_id = 0;
                    }
                }
                

            }else {
                return None;
            }
            return Some("OK".to_string());
        }
        None
    }

}

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

async fn create_sede (headers: HeaderMap, State(db): State<Arc<Db>>, Json(sede): Json<SedeIstituzionale>) -> Result<Json<String>, StatusCode> {
    if let Some(token) = headers.get("X-Admin-Token") {
        if token != ADMIN_TOKEN {
            return Err(StatusCode::UNAUTHORIZED); 
        }
    } else {
        return Err(StatusCode::UNAUTHORIZED); 
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

async fn remove_sede (headers: HeaderMap, State(db): State<Arc<Db>>, Path(id): Path<u32>) -> Result<Json<String>, StatusCode> {
    if let Some(token) = headers.get("X-Admin-Token") {
        if token != ADMIN_TOKEN {
            return Err(StatusCode::UNAUTHORIZED); 
        }
    } else {
        return Err(StatusCode::UNAUTHORIZED); 
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
