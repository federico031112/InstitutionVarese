use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SedeIstituzionale {
    pub id: u32,
    pub nome: String,
    pub comune: String,
    pub indirizzo: String,
    pub tipologia: String
}