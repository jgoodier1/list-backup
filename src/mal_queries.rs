use serde::Deserialize;

use super::anilist_queries::MediaType;
use super::config::MALConfig;

#[derive(Deserialize, Debug)]
pub struct List {
    pub data: Vec<MALEntry>,
    pub paging: Paging,
}

#[derive(Deserialize, Debug)]
pub struct MALEntry {
    pub node: Node,
    pub list_status: ListStatus,
}

#[derive(Deserialize, Debug)]
pub struct Node {
    pub id: u32,
    pub title: String,
    #[allow(dead_code)] // query comes with picture that I don't need
    main_picture: Picture,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Picture {
    medium: String,
    large: String,
}

#[derive(Deserialize, Debug)]
pub struct ListStatus {
    // change to enum
    pub status: Status,
    pub score: u8,
    pub num_episodes_watched: Option<u32>,
    pub num_chapters_read: Option<u32>,
    pub num_volumes_read: Option<u32>,
    pub is_rewatching: Option<bool>,
    pub updated_at: String,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Copy, Clone)]
pub enum Status {
    watching,
    completed,
    on_hold,
    dropped,
    plan_to_watch,
    reading,
    plan_to_read,
}

#[derive(Deserialize, Debug)]
pub struct Paging {
    pub next: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Error {
    message: String,
    error: String,
}

// will probably need a different function for manga because the return fields are different
// or maybe just make this one do more ???
pub async fn get_list(config: &MALConfig, list_type: MediaType) -> List {
    let auth_header = format!("Bearer {}", config.access_token);

    let url = match list_type {
        MediaType::ANIME => {
            "https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=1000"
        }
        MediaType::MANGA => {
            "https://api.myanimelist.net/v2/users/@me/mangalist?fields=list_status&limit=1000"
        }
    };

    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("Authorization", auth_header)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    // this will error when the token expires
    let result: List = serde_json::from_str(&res).unwrap();

    result
}

pub async fn update_entry(
    config: &MALConfig,
    id: u32,
    status: Status,
    progress: u32,
    score: u8,
    list_type: MediaType,
) {
    let auth_header = format!("Bearer {}", config.access_token);

    let url = match list_type {
        MediaType::ANIME => {
            format!("https://api.myanimelist.net/v2/anime/{}/my_list_status", id)
        }
        MediaType::MANGA => {
            format!("https://api.myanimelist.net/v2/manga/{}/my_list_status", id)
        }
    };

    let body = match list_type {
        MediaType::ANIME => {
            format!(
                "status={:?}&score={}&num_watched_episodes={}",
                status, score, progress
            )
        }
        MediaType::MANGA => {
            format!(
                "status={:?}&score={}&num_chapters_read={}",
                status, score, progress
            )
        }
    };

    let client = reqwest::Client::new();
    let res = client
        .patch(url)
        .header("Authorization", auth_header)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let result: serde_json::Result<Error> = serde_json::from_str(&res);
    match result {
        Ok(error) => {
            println!("\n Error: {}. {} \n", error.error, error.message);
        }
        Err(_) => {
            println!("\n Update complete \n");
        }
    };
}
