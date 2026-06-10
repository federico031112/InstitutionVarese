use std::collections::HashMap;
use std::sync::RwLock;
use std::fs::File;
use std::io::{Read, Write};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SedeIstituzionale {
    id: u32,
    nome: String,
    comune: String,
    indirizzo: String,
    tipologia: String
}



struct Db {
    file_path: String,
    key_value_store: RwLock<HashMap<u32, SedeIstituzionale>>
}



impl Db {
    fn new(path: &str) -> Self {
        let mut store = HashMap::new();

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
            sede_con_id.id = lock.len() as u32 + 1;
            lock.insert(sede_con_id.id, sede_con_id);
            return Some("OK".to_string())
        }
        None
    }

    fn search_by_id(&self, id: u32) -> Option<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.get(&id).cloned();
        }
        return None;
    }

    fn search_by_name(&self, nome: String) -> Vec<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.values().filter(|s| s.nome.to_lowercase().contains(&nome.to_lowercase())).cloned().collect();
        }
        return Vec::new();
    }

    fn search_by_tipology(&self, tipologia: String) -> Vec<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.values().filter(|s| s.tipologia.to_lowercase() == tipologia.to_lowercase()).cloned().collect();
        }
        return Vec::new();
    }

    fn search_by_comune(&self, comune: String) -> Vec<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.values().filter(|s| s.comune.to_lowercase() == comune.to_lowercase()).cloned().collect();
        }
        Vec::new()
    }

}

async fn get_sede_by_id (Path(id): Path<u32>, State(db): State<Arc<Db>>) ->  Json<Option<SedeIstituzionale>>{
    Json(db.search_by_id(id))
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

async fn create_sede (State(db): State<Arc<Db>>, Json(sede): Json<SedeIstituzionale>) -> Json<Option<String>> {
    let res = db.insert(sede);
    let _ = db.save_on_disk();
    Json(res)
}

#[tokio::main]
async fn main() {
    let database = Arc::new(Db::new("sedi.json"));

    let app = Router::new()
        .route("/sedi/:id", get(get_sede_by_id))
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
