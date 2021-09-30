// temp
#![allow(dead_code)]

use super::config::MALConfig;

use serde::Deserialize;

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
    main_picture: Picture,
}

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
    pub num_episodes_watched: u32,
    pub is_rewatching: bool,
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
}

#[derive(Deserialize, Debug)]
pub struct Paging {
    pub next: Option<String>,
}

// will probably need a different function for manga because the return fields are different
// or maybe just make this one do more ???
pub async fn get_list(config: &MALConfig) -> List {
    let auth_header = format!("Bearer {}", config.access_token);

    let client = reqwest::Client::new();
    let res = client
        .get("https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=1000")
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

pub async fn update_entry(config: &MALConfig, id: u32, status: Status, progress: u32, score: u8) {
    let url = format!("https://api.myanimelist.net/v2/anime/{}/my_list_status", id);
    let auth_header = format!("Bearer {}", config.access_token);
    let body = format!(
        "status={:?}&score={}&num_watched_episodes={}",
        status, score, progress
    );
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

    println!("res: {:#?}", res);
}
