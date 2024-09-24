#![allow(non_snake_case)]
use discord_presence::{Client, Event};
use reqwest::Error;
use serde::{Deserialize, Serialize};
use std::{
    process::Command,
    str, thread, time,
    time::{SystemTime, UNIX_EPOCH},
};

const CLIENT_ID: u64 = 0; // change this

#[derive(Serialize, Deserialize, Debug)]
struct SongDetails {
    name: String,
    artist: String,
    album: String,
    duration: f32,
    position: f32,
    state: String,
}

#[derive(Deserialize)]
struct ApiResponse {
    resultCount: usize,
    results: Vec<AlbumResult>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(unused)]
struct AlbumResult {
    collectionName: String,
    artistName: String,
    artworkUrl100: String,
    artworkUrl600: Option<String>,
}

async fn fetch_album_cover(
    artist: &str,
    album: &str,
    song: &str,
) -> anyhow::Result<Option<AlbumResult>, Error> {
    let url = format!(
        "https://itunes.apple.com/search?term={}+{}+{}&entity=song",
        artist.replace(' ', "+"),
        album.replace(' ', "+"),
        song.replace(' ', "+")
    );

    let response = reqwest::get(&url).await?.json::<ApiResponse>().await?;

    if response.resultCount > 0 {
        Ok(Some(response.results[0].clone()))
    } else {
        Ok(None)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut drpc = Client::new(CLIENT_ID);

    drpc.on_ready(|_ctx| {
        println!("READY!");
    })
    .persist();

    drpc.on_error(|ctx| {
        eprintln!("An error occurred: {:?}", ctx.event);
    })
    .persist();

    drpc.start();

    drpc.block_until_event(Event::Ready)?;

    assert!(Client::is_ready());

    let mut album_opt: Option<AlbumResult> = None;
    let mut current_song_name: String = String::new();

    let mut song_end: u64 = 0;

    loop {
        let current_time = SystemTime::now();
        let output = Command::new("osascript")
            .arg("-l").arg("JavaScript")
            .arg("-e")
            .arg("var musicApp = Application(\"Music\"); musicApp.running() ? (track = musicApp.currentTrack, console.log(JSON.stringify({name: track.name(), artist: track.artist(), album: track.album(), duration: track.duration(), position: musicApp.playerPosition(), state: musicApp.playerState()}))) : console.log(JSON.stringify({error: \"Music app is not running.\"}))")
            .output()
            .expect("Failed to run song details command.");
        let str_data: String = if output.status.success() {
            // for some reason osascript console.logs go to stderr
            String::from_utf8_lossy(&output.stderr).to_string()
        } else {
            thread::sleep(time::Duration::from_secs(1));
            continue;
        };
        let song_opt: Option<SongDetails> = serde_json::from_str(&str_data)?;
        if let Some(song) = &song_opt {
            let secs = current_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
            let new_end: i128 = ((secs + (song.duration - song.position) as u64) * 1000).into();
            if song.name != current_song_name || new_end - (song_end as i128) > 1000 {
                if song.name != current_song_name {
                    current_song_name = String::from(&song.name);
                    album_opt = fetch_album_cover(&song.artist, &song.album, &song.name).await?;
                }
                println!("NOW PLAYING: {}", &song.name);
                // println!("NEW: {} OLD: {}", new_end, song_end);
                if let (Some(song), Some(album)) = (&song_opt, &album_opt) {
                    let song_started = (secs - song.position as u64) * 1000;
                    song_end = (secs + (song.duration - song.position) as u64) * 1000;
                    drpc.set_activity(|act| {
                        act.state(&song.artist)
                            ._type(discord_presence::models::ActivityType::Listening)
                            .details(song.name.to_string())
                            .instance(true)
                            .timestamps(|t| t.start(song_started).end(song_end))
                            .assets(|a| {
                                a.large_image(
                                    album.artworkUrl600.as_ref().unwrap_or(&album.artworkUrl100),
                                )
                                .large_text(&song.album)
                            })
                        // .append_buttons(|button| button.label("Click Me!").url("https://duckduckgo.com"))
                    })
                    .expect("Failed to set activity");
                }
            }
        }
        thread::sleep(time::Duration::from_secs(1));
    }

    #[allow(unreachable_code)]
    Ok(())
}
