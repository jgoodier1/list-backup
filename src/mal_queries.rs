// temp
#![allow(dead_code)]

use super::config::MALConfig;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Response {
  data: Vec<Node>,
  paging: Paging
}

#[derive(Deserialize, Debug)]
struct Node {
  id: u32,
  title: String,
  main_picture: Picture,
  list_status: ListStatus
}

#[derive(Deserialize, Debug)]
struct Picture {
  medium: String,
  large: String
}

#[derive(Deserialize, Debug)]
struct ListStatus {
  // change to enum
  status: String,
  score: u8,
  num_episodes_watched: u32,
  is_rewatching: bool,
  updated_at: String
}

#[derive(Deserialize, Debug)]
struct Paging {
  next: Option<String>
}

pub async fn get_list(config: &MALConfig) {
  let auth_header = format!("Bearer {}", config.access_token);

  let client = reqwest::Client::new();
  let res = client.get("https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=10")
  .header("Authorization", auth_header)
  .send()
  .await
  .unwrap()
  .text()
  .await
  .unwrap();

  println!("{}", res);

  // this will error when the token expires
  let result: Response = serde_json::from_str(&res).unwrap();
  println!("{:?}", result.data[0]);
}