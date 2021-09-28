use std::cmp::PartialEq;

use serde::{Deserialize, Serialize};

use super::config::Config;

#[derive(Deserialize, Debug)]
struct UserIdResp {
    data: UpdateUser,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct UpdateUser {
    update_user: UserData,
}

#[derive(Deserialize, Debug)]
pub struct UserData {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize, Debug)]
struct ListResp {
    data: MediaListCollection,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct MediaListCollection {
    media_list_collection: Lists,
}

#[derive(Deserialize, Debug)]
pub struct Lists {
    pub lists: Vec<Entries>,
}

#[derive(Deserialize, Debug)]
pub struct Entries {
    pub entries: Vec<Entry>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Entry {
    pub status: MediaListStatus,
    pub score: f32,
    pub progress: u32,
    pub media: Media,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE", serialize = "PascalCase"))]
pub enum MediaListStatus {
    Completed,
    Current,
    Planning,
    Dropped,
    Paused,
    Repeating,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct Media {
    pub id: u32,
    pub id_mal: u32,
    pub title: Title,
    pub format: MediaFormat,
    pub episodes: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct Title {
    pub user_preferred: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE", serialize = "PascalCase"))]
pub enum MediaType {
    Anime,
    Manga,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE", serialize = "PascalCase"))]
pub enum MediaFormat {
    Tv,
    TvShort,
    Movie,
    Special,
    Ova,
    Ona,
    Music,
    Manga,
    Novel,
    OneShot,
}

const GET_USER_ID: &str = "
mutation {
    UpdateUser {
        id
        name
    }
}
";

pub async fn get_user_id(access_token: String) -> UserData {
    let auth_header = format!("Bearer {}", access_token);

    let json = serde_json::json!({ "query": GET_USER_ID });

    let client = reqwest::Client::new();
    let res = client
        .post("https://graphql.anilist.co")
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let result: UserIdResp = serde_json::from_str(&res).unwrap();
    println!("{}", result.data.update_user.id);
    result.data.update_user
}

const GET_LIST: &str = "
query ($id: Int) {
	MediaListCollection(userId: $id, type: ANIME) {
    lists {
      entries {
        status
        score
        progress
        media {
          id
          idMal
          title {
            userPreferred
          }
          format
          episodes
        }
      }
    }
  }
}
";

pub async fn get_list(config: &Config) -> Lists {
    let auth_header = format!("Bearer {}", config.access_token);

    let json = serde_json::json!({"query": GET_LIST, "variables" : {"id": config.user_id}});

    let client = reqwest::Client::new();
    let res = client
        .post("https://graphql.anilist.co")
        .header("Authorization", auth_header)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(json.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let result: ListResp = serde_json::from_str(&res).unwrap();
    result.data.media_list_collection
}
