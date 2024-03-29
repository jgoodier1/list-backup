use std::fs;
use std::io::prelude::*;

use home;
use serde::{Deserialize, Serialize};

use super::anilist_queries::{Entry, Lists, MediaFormat, MediaListStatus, MediaType, UserData};

#[derive(Deserialize, Serialize, Debug)]
struct UserSection {
    user_id: u32,
    username: String,
    list_type: MediaType,
    total_anime: u32,
    watching: u32,
    completed: u32,
    on_hold: u32,
    dropped: u32,
    planning: u32,
    rewatching: u32,
}

impl UserSection {
    fn new(lists: &Lists, user: UserData, list_type: MediaType) -> UserSection {
        let watching_position = lists
            .lists
            .iter()
            .position(|x| x.entries[0].status == MediaListStatus::Current);
        let watching_len = match watching_position {
            None => 0,
            Some(watching_position) => lists.lists[watching_position].entries.len() as u32,
        };

        let completed_position = lists
            .lists
            .iter()
            .position(|x| x.entries[0].status == MediaListStatus::Completed);
        let completed_len = match completed_position {
            None => 0,
            Some(completed_position) => lists.lists[completed_position].entries.len() as u32,
        };

        let paused_position = lists
            .lists
            .iter()
            .position(|x| x.entries[0].status == MediaListStatus::Paused);
        let paused_len = match paused_position {
            None => 0,
            Some(paused_position) => lists.lists[paused_position].entries.len() as u32,
        };

        let dropped_position = lists
            .lists
            .iter()
            .position(|x| x.entries[0].status == MediaListStatus::Dropped);
        let dropped_len = match dropped_position {
            None => 0,
            Some(dropped_position) => lists.lists[dropped_position].entries.len() as u32,
        };

        let planning_position = lists
            .lists
            .iter()
            .position(|x| x.entries[0].status == MediaListStatus::Planning);
        let planning_len = match planning_position {
            None => 0,
            Some(planning_position) => lists.lists[planning_position].entries.len() as u32,
        };

        let repeating_position = lists
            .lists
            .iter()
            .position(|x| x.entries[0].status == MediaListStatus::Repeating);
        let repeating_len = match repeating_position {
            None => 0,
            Some(repeating_position) => lists.lists[repeating_position].entries.len() as u32,
        };

        let total =
            watching_len + completed_len + paused_len + dropped_len + planning_len + repeating_len;

        UserSection {
            user_id: user.id,
            username: user.name,
            list_type,
            total_anime: total,
            watching: watching_len,
            completed: completed_len,
            on_hold: paused_len,
            dropped: dropped_len,
            planning: planning_len,
            rewatching: repeating_len,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct EntrySection {
    title: String,
    id: u32,
    id_mal: Option<u32>,
    episodes: Option<u32>,
    chapters: Option<u32>,
    format: MediaFormat,
    status: MediaListStatus,
    score: f32,
    progress: u32,
}

impl EntrySection {
    fn new(entry: &Entry) -> EntrySection {
        EntrySection {
            title: entry.media.title.user_preferred.clone(),
            id: entry.media.id,
            id_mal: entry.media.id_mal,
            episodes: entry.media.episodes,
            chapters: entry.media.chapters,
            format: entry.media.format,
            status: entry.status,
            score: entry.score,
            progress: entry.progress,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Current(Option<Vec<EntrySection>>);

#[derive(Deserialize, Serialize, Debug)]
struct Completed(Option<Vec<EntrySection>>);

#[derive(Deserialize, Serialize, Debug)]
struct Planning(Option<Vec<EntrySection>>);

#[derive(Deserialize, Serialize, Debug)]
struct Dropped(Option<Vec<EntrySection>>);

#[derive(Deserialize, Serialize, Debug)]
struct Paused(Option<Vec<EntrySection>>);

#[derive(Deserialize, Serialize, Debug)]
struct Repeating(Option<Vec<EntrySection>>);

#[derive(Deserialize, Serialize, Debug)]
struct BackupToml {
    user_section: UserSection,
    repeating: Repeating,
    current: Current,
    completed: Completed,
    paused: Paused,
    dropped: Dropped,
    planning: Planning,
}

pub fn write_list_to_file(list: &Lists, user: (u32, &str), list_type: MediaType) {
    let user = UserData {
        id: user.0,
        name: user.1.to_string(),
    };

    let mut file_path = home::home_dir().unwrap();
    file_path.push("Documents");
    if list_type == MediaType::ANIME {
        file_path.push("anime-backup");
    } else {
        file_path.push("manga-backup");
    }
    file_path.set_extension("toml");

    // create the file
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .unwrap();

    let user_section = UserSection::new(list, user, list_type);

    let current_list = Current(create_entry_section_vec(&list, MediaListStatus::Current));
    let completed_list = Completed(create_entry_section_vec(&list, MediaListStatus::Completed));
    let planning_list = Planning(create_entry_section_vec(&list, MediaListStatus::Planning));
    let dropped_list = Dropped(create_entry_section_vec(&list, MediaListStatus::Dropped));
    let paused_list = Paused(create_entry_section_vec(&list, MediaListStatus::Paused));
    let repeating_list = Repeating(create_entry_section_vec(&list, MediaListStatus::Repeating));
    let backup = BackupToml {
        user_section,
        current: current_list,
        repeating: repeating_list,
        completed: completed_list,
        paused: paused_list,
        dropped: dropped_list,
        planning: planning_list,
    };
    let backup_toml = toml::to_string(&backup).unwrap();
    writeln!(&file, "{}", backup_toml).unwrap();
}

fn create_entry_section_vec(list: &Lists, status: MediaListStatus) -> Option<Vec<EntrySection>> {
    let list_pos = list
        .lists
        .iter()
        .position(|x| x.entries[0].status == status)
        .unwrap_or_else(|| 999);

    let mut vec = Vec::new();

    if list_pos <= list.lists.len() {
        let list = &list.lists[list_pos].entries;

        for entry in list.iter() {
            let entry_section = EntrySection::new(&entry);
            vec.push(entry_section);
        }
        Some(vec)
    } else {
        None
    }
}
