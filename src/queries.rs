use std::fs;
use std::io::prelude::*;
use std::cmp::PartialEq;

use serde::{Deserialize, Serialize};

use super::config::Config;

#[derive(Deserialize, Debug)]
struct UserIdResp {
    data: UpdateUser,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct UpdateUser {
    UpdateUser: UserData,
}

#[derive(Deserialize, Debug)]
pub struct UserData {
    pub id: u32,
    pub name: String
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
pub struct Lists {
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
    progress: u32,
    media: Media,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
enum MediaListStatus {
    COMPLETED,
    CURRENT,
    PLANNING,
    DROPPED,
    PAUSED,
    REPEATING,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Media {
    id: u32,
    idMal: u32,
    title: Title,
    format: MediaFormat,
    episodes: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct Title {
    userPreferred: String,
}

#[derive(Deserialize, Serialize, Debug)]
enum MediaType {
    ANIME,
    MANGA,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Debug, Clone)]
enum MediaFormat {
    TV,
    TV_SHORT,
    MOVIE,
    SPECIAL,
    OVA,
    ONA,
    MUSIC,
    MANGA,
    NOVEL,
    ONE_SHOT
}

#[derive(Deserialize, Serialize, Debug)]
struct UserSection {
    user_id: u32,
    username: String,
    list_type: MediaType,
    total_anime: usize,
    watching: usize,
    completed: usize,
    on_hold: usize,
    dropped: usize,
    planning: usize,
    rewatching: usize
}

impl UserSection {
    fn new(lists: &Lists, user: UserData) -> UserSection {
        let watching_position = lists.lists.iter().position(|x| {
            x.entries[0].status == MediaListStatus::CURRENT
        });
        let watching_len = match watching_position {
            None => 0,
            Some(watching_position) => {
                lists.lists[watching_position].entries.len()
            }
        };

        let completed_position = lists.lists.iter().position(|x| {
            x.entries[0].status == MediaListStatus::COMPLETED
        });
        let completed_len = match completed_position {
            None => 0,
            Some(completed_position) => {
                lists.lists[completed_position].entries.len()
            }
        };

        let paused_position = lists.lists.iter().position(|x| {
            x.entries[0].status == MediaListStatus::PAUSED
        });
        let paused_len = match paused_position {
            None => 0,
            Some(paused_position) => {
                lists.lists[paused_position].entries.len()
            }
        };

        let dropped_position = lists.lists.iter().position(|x| {
            x.entries[0].status == MediaListStatus::DROPPED
        });
        let dropped_len = match dropped_position {
            None => 0,
            Some(dropped_position) => {
                lists.lists[dropped_position].entries.len()
            }
        };

        let planning_position = lists.lists.iter().position(|x| {
            x.entries[0].status == MediaListStatus::PLANNING
        });
        let planning_len = match planning_position {
            None => 0,
            Some(planning_position) => {
                lists.lists[planning_position].entries.len()
            }
        };

        let repeating_position = lists.lists.iter().position(|x| {
            x.entries[0].status == MediaListStatus::REPEATING
        });
        let repeating_len = match repeating_position {
            None => 0,
            Some(repeating_position) => {
                lists.lists[repeating_position].entries.len()
            }
        };

        let total = watching_len + completed_len + paused_len + dropped_len + planning_len + repeating_len;

        UserSection {
            user_id: user.id,
            username: user.name,
            list_type: MediaType::ANIME,
            total_anime: total,
            watching: watching_len,
            completed: completed_len,
            on_hold: paused_len,
            dropped: dropped_len,
            planning: planning_len,
            rewatching: repeating_len
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct EntrySection {
    id: u32,
    id_mal: u32,
    episodes: u32,
    format: MediaFormat,
    status: MediaListStatus,
    score: f32,
    progress: u32,
}

impl EntrySection {
    fn new(entry: &Entry) -> EntrySection {
        EntrySection {
            id: entry.media.id,
            id_mal: entry.media.idMal,
            episodes: entry.media.episodes,
            format: entry.media.format.clone(),
            status: entry.status.clone(),
            score: entry.score,
            progress: entry.progress,
        }
    }
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
        .await;

    let result: UserIdResp = serde_json::from_str(&res.unwrap()).unwrap();
    println!("{}", result.data.UpdateUser.id);
    result.data.UpdateUser
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
        .await;

    let result: ListResp = serde_json::from_str(&res.unwrap()).unwrap();
    result.data.MediaListCollection
}

pub fn write_list_to_file(list: &Lists, user: (u32, &str)) {
    let user = UserData {
        id: user.0,
        name: user.1.to_string()
    };
    // create the file
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("anilist-backup.toml")
        .unwrap();

    // add the user section
    let user_section = UserSection::new(list, user);
    // println!("{:?}", user_section);
    let user_section_toml = toml::to_string(&user_section).unwrap();
    writeln!(&file, "[User Information]").unwrap();
    write!(&file, "{} \n", user_section_toml).unwrap();

    // loop through the list and its entries and write them to the
    // want to sort them by their status first
    for list in list.lists.iter() {
        // want to sort them by their title first
        for entry in list.entries.iter() {
            let entry_section = EntrySection::new(&entry);
            write_media(&mut file, entry_section, &entry.media.title.userPreferred).unwrap();
        }
    }
}

fn write_media(file: &mut std::fs::File, entry: EntrySection, title: &str) -> std::io::Result<()>{
    let entry_toml = toml::to_string(&entry).unwrap();
    writeln!(file, "[{}]", title)?;
    write!(file, "{} \n", entry_toml)?;
    Ok(())
}