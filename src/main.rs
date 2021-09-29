use std::env;
use std::fs;
use std::io::prelude::*;
use std::io::ErrorKind;

use clap::{App, Arg, SubCommand};
use dotenv::dotenv;
use home;
use rand::{thread_rng, Rng};
use rocket::Config as RocketConfig;
use rocket::Shutdown;

mod config;
use config::TomlConfig;
mod anilist_queries;
use anilist_queries::MediaType;
mod save_to_file;
mod mal_queries;

pub struct PKCE {
    code_challenge: String,
}

const CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ\
    abcdefghijklmnopqrstuvwxyz\
    0123456789-.~_";

#[rocket::get("/anilist?<code>")]
async fn anilist(code: &str, shutdown: Shutdown) -> &'static str {
    shutdown.notify();

    config::get_anilist_token(code).await;
    "You may close this page now and return to the terminal"
}

#[rocket::get("/myanimelist?<code>")]
async fn myanimelist(code: &str, pkce: &rocket::State<PKCE>, shutdown: Shutdown) -> &'static str {
    shutdown.notify();

    config::get_mal_token(code, &pkce.code_challenge).await;
    "You may close this page now and return to the terminal"
}

async fn start_rocket(pkce: PKCE) {
    let rocket_config = RocketConfig {
        port: 5000,
        log_level: rocket::config::LogLevel::Off,
        ..RocketConfig::debug_default()
    };

    let server = rocket::custom(&rocket_config)
        .mount("/", rocket::routes![anilist, myanimelist])
        .manage(pkce)
        .launch()
        .await;

    if let Err(error) = server {
        panic!("There was an error: {}", error);
    }
}

fn create_code_challenge() -> String {
    let mut rng = thread_rng();

    let chars = CHARS.as_bytes();

    let buf: Vec<u8> = (0..128)
        .map(|_| {
            let i = rng.gen_range(0..CHARS.len());
            chars[i]
        })
        .collect();
    let s = String::from_utf8_lossy(&buf).into_owned();
    s
}

#[rocket::main]
async fn main() {
    dotenv().unwrap();

    let matches = App::new("List Backup")
        .version("0.1.0")
        .about("Does stuff with your anime/manga lists")
        .subcommand(
            SubCommand::with_name("backup")
                .about("Backups your list to a file")
                .arg(
                    Arg::with_name("list type")
                        .help("The type of list to backup. Either 'anime' or 'manga'")
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("update").about("Updates MAL with your list from Anilist"))
        .get_matches();

    let pkce = PKCE {
        code_challenge: create_code_challenge(),
    };

    //the config file
    let mut file_path = home::home_dir().unwrap();
    file_path.push(".config");
    file_path.push("list-backup");
    file_path.push("config");
    file_path.set_extension("toml");

    match matches.subcommand() {
        ("backup", Some(backup_matches)) => {
            if let Some(list_type) = backup_matches.value_of("list type") {
                let list_type_uppercase = list_type.to_uppercase();
                let list_type: MediaType;
                if list_type_uppercase == "ANIME" {
                    list_type = MediaType::ANIME
                } else if list_type_uppercase == "MANGA" {
                    list_type = MediaType::MANGA
                } else {
                    panic!("The value for 'list type' needs to be either 'anime' or 'manga', case insensitive")
                }

                let file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(file_path);
                match file {
                    Err(error) => {
                        match error.kind() {
                            ErrorKind::NotFound => {
                                println!("Go here to authenticate: https://anilist.co/api/v2/oauth/authorize?client_id=6593&redirect_uri=http://localhost:5000/anilist&response_type=code");
                                start_rocket(pkce).await;
                            }
                            ErrorKind::PermissionDenied => {
                                panic!("You don't have permission to open the config file")
                            }
                            other_error => {
                                panic!("Unhandled error: {:?}", other_error);
                            }
                        }
                    }
                    Ok(mut file) => {
                        let mut file_string = String::new();
                        file.read_to_string(&mut file_string).unwrap();
                        let config: TomlConfig = toml::from_str(&file_string).unwrap();
                        if let Some(anilist) = config.anilist {
                            let list = anilist_queries::get_list(&anilist, list_type).await;
                            save_to_file::write_list_to_file(
                                &list,
                                (anilist.user_id, &anilist.user_name),
                                list_type,
                            );
                        } else {
                            println!("Go here to authenticate: https://anilist.co/api/v2/oauth/authorize?client_id=6593&redirect_uri=http://localhost:5000/anilist&response_type=code");
                            start_rocket(pkce).await;
                        }
                    }
                }
            }
        }
        ("update", Some(_)) => {
            let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(file_path);

            match file {
                Err(error) => {
                    match error.kind() {
                        ErrorKind::NotFound => {
                            let mut auth_link = String::new();
                            auth_link.push_str(
                                "https://myanimelist.net/v1/oauth2/authorize?response_type=code&client_id=",
                            );
                            auth_link.push_str(&env::var("MAL_CLIENT_ID").unwrap());
                            auth_link.push_str("&code_challenge=");
                            auth_link.push_str(&pkce.code_challenge);
                            println!("Go here to authenticate: {}", auth_link);

                            start_rocket(pkce).await;

                        }
                        ErrorKind::PermissionDenied => {
                            panic!("You don't have permission to open the config file")
                        }
                        other_error => {
                            panic!("Unhandled error: {:?}", other_error);
                        }
                    }
                }
                Ok(mut file) => {
                    let mut file_string = String::new();
                    file.read_to_string(&mut file_string).unwrap();
                    let config: TomlConfig = toml::from_str(&file_string).unwrap();
                    // just going to get the current list from mal before trying to update anything
                    if let Some(mal) = config.myanimelist {
                        mal_queries::get_list(&mal).await;
                    } else {
                        let mut auth_link = String::new();
                        auth_link.push_str(
                            "https://myanimelist.net/v1/oauth2/authorize?response_type=code&client_id=",
                        );
                        auth_link.push_str(&env::var("MAL_CLIENT_ID").unwrap());
                        auth_link.push_str("&code_challenge=");
                        auth_link.push_str(&pkce.code_challenge);
                        println!("Go here to authenticate: {}", auth_link);

                        start_rocket(pkce).await;
                    }
                }
            }

        }
        _ => {
            panic!("No matches");
        }
    }
}
