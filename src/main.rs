use std::fs;
use std::io::prelude::*;
use std::io::ErrorKind;

use rocket::Config as RocketConfig;
use rocket::Shutdown;
use home;
use dotenv::dotenv;

mod config;
mod queries;
use config::TomlConfig;

#[rocket::get("/anilist?<code>")]
async fn anilist(code: &str, shutdown: Shutdown) -> &'static str {
    shutdown.notify();

    config::get_token(code).await;
    "You may close this page now and return to the terminal"
}

async fn start_rocket() {
    let rocket_config = RocketConfig {
        port: 5000,
        log_level: rocket::config::LogLevel::Off,
        ..RocketConfig::debug_default()
    };

    let server = rocket::custom(&rocket_config)
        .mount("/", rocket::routes![anilist])
        .launch()
        .await;

    if let Err(error) = server {
        panic!("There was an error: {}", error);
    }
}

#[rocket::main]
async fn main() {
    dotenv().unwrap();

    // need to first read the config file
    let mut file_path = home::home_dir().unwrap();
    file_path.push(".config");
    file_path.push("list-backup");
    file_path.push("config");
    file_path.set_extension("toml");

    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path);
    match file {
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => {
                    // also want to go here after parsing the file for the token
                    println!("Go here to authenticate: https://anilist.co/api/v2/oauth/authorize?client_id=6593&redirect_uri=http://localhost:5000/anilist&response_type=code");
                    start_rocket().await;
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
                let list = queries::get_list(&anilist).await;
                queries::write_list_to_file(&list, (anilist.user_id, &anilist.user_name));
            }
        }
    }
}
