use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::ErrorKind;

use serde::{Deserialize, Serialize};

use super::queries;

#[derive(Deserialize, Debug, Serialize)]
pub struct TomlConfig {
    pub anilist: Option<Config>,
    // myanimelist: Option<Config>
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Config {
    pub token_type: String,
    pub expires_in: u32,
    pub access_token: String,
    pub refresh_token: String,
    pub code: String,
    pub user_id: u32,
}

#[derive(Deserialize, Debug)]
struct Response {
    token_type: String,
    expires_in: u32,
    access_token: String,
    refresh_token: String,
}

fn write_config_file(config: Config) {
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(".config/list-backup/config.toml");
    match file {
        Err(error) => match error.kind() {
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
            };
            let toml = toml::to_string(&toml_config).unwrap();
            write!(&file, "{}", toml).unwrap();

            println!("Completed writing to config file");
        }
    }
}

fn get_config_dir(config: Config) {
    let home_dir = home::home_dir().unwrap();
    env::set_current_dir(home_dir).unwrap();

    // try to create config dir
    let created_dir = fs::create_dir(".config/list-backup");

    // will error if it already exists
    if let Err(error) = created_dir {
        match error.kind() {
            ErrorKind::AlreadyExists => {
                println!("Dir already exists");
                write_config_file(config);
            }
            other_error => {
                panic!("Unhandled error: {:?}", other_error);
            }
        };
    } else {
        println!("Created the directory");
        write_config_file(config);
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

pub async fn get_token(code: &str) {
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
        let user_id = queries::get_user_id(response.access_token.clone()).await;
        let config = Config {
            token_type: response.token_type,
            expires_in: response.expires_in,
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            code: code.to_string(),
            user_id,
        };
        get_config_dir(config);
    }
}
