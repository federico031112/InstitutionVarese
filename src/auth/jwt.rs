use serde::{Serialize, Deserialize};
use axum::extract::FromRequestParts;
use axum::http::{StatusCode, request::Parts, header::AUTHORIZATION};
use jsonwebtoken::{DecodingKey, Validation, decode};


const JWT_KEY: &[u8] = b"chiave-super-segreta";

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub role: String,
    pub exp: u64
}

impl <S> FromRequestParts <S> for Claims 
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let authentication = parts.headers.get(AUTHORIZATION).and_then(|value| value.to_str().ok()).ok_or((StatusCode::UNAUTHORIZED, "Token mancante"))?;

        if !authentication.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Formato token non valido (usa Bearer)"));
        }

        let token = &authentication[7..]; 

        // 3. Decodifica e verifica del JWT
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_KEY),
            &Validation::default(),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Token non valido o scaduto"))?;

        // Restituisce i dati utente validati
        Ok(token_data.claims)
    }
}