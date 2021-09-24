#![allow(dead_code)]

use serde::Deserialize;

use super::config::Config;

#[derive(Deserialize, Debug)]
struct UserIdResp {
    data: UpdateUser,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct UpdateUser {
    UpdateUser: UpdateUserData,
}

#[derive(Deserialize, Debug)]
struct UpdateUserData {
    id: u32,
}

#[derive(Deserialize, Debug)]
struct ListResp {
    data: MediaListCollection,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct MediaListCollection {
    MediaListCollection: Lists,
}

#[derive(Deserialize, Debug)]
struct Lists {
    lists: Vec<Entries>,
}

#[derive(Deserialize, Debug)]
struct Entries {
    entries: Vec<Entry>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Entry {
    status: MediaListStatus,
    score: f32,
    progress: i32,
    startedAt: FuzzyDate,
    completedAt: FuzzyDate,
    updatedAt: i32,
    media: Media,
}

#[derive(Deserialize, Debug)]
enum MediaListStatus {
    COMPLETED,
    CURRENT,
    PLANNING,
    DROPPED,
    PAUSED,
    REPEATING,
}

#[derive(Deserialize, Debug)]
struct FuzzyDate {
    year: Option<i32>,
    month: Option<i32>,
    day: Option<i32>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Media {
    id: i32,
    idMal: i32,
    title: Title,
    r#type: MediaType,
    episodes: i32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Title {
    userPreferred: String,
}

#[derive(Deserialize, Debug)]
enum MediaType {
    ANIME,
    MANGA,
}

const GET_USER_ID: &str = "
mutation {
    UpdateUser {
        id
    }
}
";

const GET_LIST: &str = "
query ($id: Int) {
	MediaListCollection(userId: $id, type: ANIME) {
    lists {
      entries {
        status
        score
        progress
        startedAt {
          year
          month
          day
        }
        completedAt {
          year
          month
          day
        }
        updatedAt
        media {
          id
          idMal
          title {
            userPreferred
          }
          type
          episodes
        }
      }
    }
  }
}
";

pub async fn get_user_id(access_token: String) -> u32 {
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
        .await;

    let result: UserIdResp = serde_json::from_str(&res.unwrap()).unwrap();
    println!("{}", result.data.UpdateUser.id);
    result.data.UpdateUser.id
}

pub async fn get_list(config: &Config) {
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
        .await;

    let result: ListResp = serde_json::from_str(&res.unwrap()).unwrap();
    println!(
        "{:?}",
        result.data.MediaListCollection.lists[0].entries[0]
            .media
            .title
            .userPreferred
    );
}
