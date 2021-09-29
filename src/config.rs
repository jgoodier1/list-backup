use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::ErrorKind;

use serde::{Deserialize, Serialize};

use super::anilist_queries;

#[derive(Deserialize, Debug, Serialize)]
pub struct TomlConfig {
    pub anilist: Option<AnilistConfig>,
    pub myanimelist: Option<MALConfig>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AnilistConfig {
    pub token_type: String,
    pub expires_in: u32,
    pub access_token: String,
    pub refresh_token: String,
    pub code: String,
    pub user_id: u32,
    pub user_name: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct MALConfig {
    pub token_type: String,
    pub expires_in: u32,
    pub access_token: String,
    pub refresh_token: String,
    pub code: String,
    pub pkce: String,
}

impl MALConfig {
    fn new(res: Response, code: &str, pkce: &str) -> MALConfig {
        MALConfig {
            token_type: res.token_type,
            expires_in: res.expires_in,
            access_token: res.access_token,
            refresh_token: res.refresh_token,
            code: code.to_string(),
            pkce: pkce.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
struct Response {
    token_type: String,
    expires_in: u32,
    access_token: String,
    refresh_token: String,
}

fn write_anilist_config(config: AnilistConfig) {
    let mut file_path = home::home_dir().unwrap();
    file_path.push(".config");
    file_path.push("list-backup");
    file_path.push("config");
    file_path.set_extension("toml");

    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(&file_path);

    match file {
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                println!("The directory doesn't exist");
                create_parent_dir(file_path);
                write_anilist_config(config)
            }
            ErrorKind::PermissionDenied => {
                panic!("You don't have the correct permission to open or read this file");
            }
            other_error => {
                panic!("Unhandled error: {:?}", other_error);
            }
        },
        Ok(file) => {
            println!("Opened the file");
            let toml_config = TomlConfig {
                anilist: Some(config),
                myanimelist: None,
            };
            let toml = toml::to_string(&toml_config).unwrap();
            write!(&file, "{}", toml).unwrap();

            println!("Completed writing to config file");
        }
    }
}

fn create_parent_dir(path: std::path::PathBuf) {
    let path = path.parent().unwrap();
    let created_dir = fs::create_dir_all(path);

    // will error if it already exists
    if let Err(error) = created_dir {
        match error.kind() {
            ErrorKind::AlreadyExists => {
                println!("Dir already exists");
                return;
            }
            other_error => {
                panic!("Unhandled error: {:?}", other_error);
            }
        };
    } else {
        println!("Created the directory");
        return;
    }
}

// haven't actually tested this yet
async fn _refresh_token(ref_token: &str) {
    let mut map = HashMap::new();
    map.insert("grant_type", "refresh_token");
    map.insert("refresh_token", ref_token);

    let client = reqwest::Client::new();
    let res = client
        .post("https://anilist.co/api/v2/oauth/token") // might be a dif url
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&map)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    if let Ok(res) = res {
        let response: Response = serde_json::from_str(&res).unwrap();
        println!("{:?}", response);
    }
}

pub async fn get_anilist_token(code: &str) {
    let secret = env::var("ANILIST_SECRET").unwrap();

    let mut map = HashMap::new();
    map.insert("grant_type", "authorization_code");
    map.insert("client_id", "6593");
    map.insert("client_secret", &secret);
    map.insert("redirect_uri", "http://localhost:5000/anilist");
    map.insert("code", code);

    let client = reqwest::Client::new();
    let res = client
        .post("https://anilist.co/api/v2/oauth/token")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&map)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    if let Ok(res) = res {
        let response: Response = serde_json::from_str(&res).unwrap();
        let user_data = anilist_queries::get_user_id(response.access_token.clone()).await;
        let config = AnilistConfig {
            token_type: response.token_type,
            expires_in: response.expires_in,
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            code: code.to_string(),
            user_id: user_data.id,
            user_name: user_data.name,
        };
        write_anilist_config(config);
    }
}

pub async fn get_mal_token(code: &str, pkce: &str) {
    let id = env::var("MAL_CLIENT_ID").unwrap();
    let secret = env::var("MAL_SECRET").unwrap();

    // refresh token works the same, but `grant_type` is different
    let body = format!(
        "client_id={}&client_secret={}&grant_type=authorization_code&code={}&code_verifier={}",
        id, secret, code, pkce
    );

    let client = reqwest::Client::new();
    let res = client
        .post("https://myanimelist.net/v1/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let response: Response = serde_json::from_str(&res).unwrap();
    let config = MALConfig::new(response, code, pkce);
    write_mal_config(config);
}

fn write_mal_config(config: MALConfig) {
    let mut file_path = home::home_dir().unwrap();
    file_path.push(".config");
    file_path.push("list-backup");
    file_path.push("config");
    file_path.set_extension("toml");

    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(&file_path);

    match file {
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                println!("The directory doesn't exist");
                create_parent_dir(file_path);
                write_mal_config(config)
            }
            ErrorKind::PermissionDenied => {
                panic!("You don't have the correct permission to open or read this file");
            }
            other_error => {
                panic!("Unhandled error: {:?}", other_error);
            }
        },
        Ok(file) => {
            println!("Opened the file");
            let toml_config = TomlConfig {
                anilist: None,
                myanimelist: Some(config),
            };
            let toml = toml::to_string(&toml_config).unwrap();
            write!(&file, "{}", toml).unwrap();

            println!("Completed writing to config file");
        }
    }
}
