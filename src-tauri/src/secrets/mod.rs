#[derive(Debug, serde::Deserialize)]
pub struct SecretForm {
    kind: String,
    name: String,
    value: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Secret {
    id: String,
    kind: String,
    name: String,
    value: String,
}

#[tauri::command]
pub fn create_secret(data: SecretForm) -> Result<String, String> {
    println!("Received secret: {:?}", data);
    Ok("Submitted secret".to_string())
}

#[tauri::command]
pub fn get_secrets() -> Result<Vec<Secret>, String> {
    Ok(vec![
        Secret {
            id: "1".to_string(),
            kind: "login".to_string(),
            name: "secret1".to_string(),
            value: "password".to_string(),
        },
        Secret {
            id: "2".to_string(),
            kind: "login".to_string(),
            name: "secret2".to_string(),
            value: "password".to_string(),
        },
        Secret {
            id: "3".to_string(),
            kind: "login".to_string(),
            name: "secret3".to_string(),
            value: "password".to_string(),
        },
    ])
}

#[tauri::command]
pub fn get_secret(id: String) -> Result<Secret, String> {
    get_secrets()
        .unwrap()
        .into_iter()
        .find(|secret| secret.id == id)
        .ok_or_else(|| "Secret not found".to_string())
}
