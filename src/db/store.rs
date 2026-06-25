use super::data::SedeIstituzionale;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;
use std::fs::File;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DbState {
    key_value_store: HashMap<u32, SedeIstituzionale>,
    free_id: Vec<u32>,
    max_id: u32,
    unique_values: HashSet<String>
}

pub struct Db {
    file_path: String,
    key_value_store: RwLock<DbState>
}

impl Db {
    pub fn new(path: &str) -> Self {
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

    pub fn save_on_disk(&self) -> Result<(), std::io::Error>{
        if let Ok(lock) = self.key_value_store.read() {
            if let Ok(json) = serde_json::to_string_pretty(&*lock) {
                if let Ok(mut f) = File::create(&self.file_path) {
                    f.write_all(json.as_bytes())?;

                }
            }
        }
        Ok(())
    }

    pub fn insert(&self, sede: SedeIstituzionale) -> Option<String>{
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

    pub fn search_by_id(&self, id: u32) -> Option<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.key_value_store.get(&id).cloned();
        }
        return None;
    }

    pub fn search_by_name(&self, nome: String) -> Vec<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.key_value_store.values().filter(|s| s.nome.to_lowercase().contains(&nome.to_lowercase())).cloned().collect();
        }
        return Vec::new();
    }

    pub fn search_by_tipology(&self, tipologia: String) -> Vec<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.key_value_store.values().filter(|s| s.tipologia.to_lowercase() == tipologia.to_lowercase()).cloned().collect();
        }
        return Vec::new();
    }

    pub fn search_by_comune(&self, comune: String) -> Vec<SedeIstituzionale> {
        if let Ok(lock) = self.key_value_store.read() {
            return lock.key_value_store.values().filter(|s| s.comune.to_lowercase() == comune.to_lowercase()).cloned().collect();
        }
        Vec::new()
    }

    pub fn remove(&self, id: u32) -> Option<String> {
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