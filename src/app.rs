#![allow(non_snake_case)]
use crate::data::*;
use discord_presence::Client;
use std::{
    process::Command,
    str,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct PresenceData {
    name: String,
    artist: String,
    album: AlbumResult,
    start: u64,
    end: u64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AppState {
    Idle,
    Fault,
    Presence(PresenceData),
}

impl AppState {
    pub async fn idle(&self, app: &App) -> Self {
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
            // thread::sleep(time::Duration::from_secs(1));
            return self.fault(app);
        };

        let song_res: Result<SongDetails, _> = serde_json::from_str(&str_data);
        if song_res.is_err() {
            return self.fault(app);
        }

        let song = song_res.unwrap();
        if song.state != "playing" {
            return AppState::Idle;
        }

        let current_time = SystemTime::now();

        let secs = current_time.duration_since(UNIX_EPOCH).unwrap().as_secs();

        let start = (secs - song.position as u64) * 1000;
        let end = (secs + (song.duration - song.position) as u64) * 1000;

        let album_res = fetch_album(&song.album, &song.artist, &song.name).await;

        if album_res.is_err() {
            return self.fault(app);
        }

        let album_opt = album_res.unwrap();

        if album_opt.is_none() {
            return self.fault(app);
        }

        let album = album_opt.unwrap();

        let presence_data = PresenceData {
            name: song.name,
            album,
            artist: song.artist,
            start,
            end,
        };

        println!("TRANSITIONING TO PRESENCE FOR:\n{:?}", presence_data);
        Box::pin(self.presence(app, presence_data)).await
    }

    pub fn fault(&self, _app: &App) -> Self {
        println!("FAULTED!");
        AppState::Idle
    }

    pub async fn presence(&self, app: &App, data: PresenceData) -> Self {
        let output = Command::new("osascript").arg("-l").arg("JavaScript").arg("-e")
            .arg("var musicApp = Application(\"Music\"); musicApp.running() ? (track = musicApp.currentTrack, console.log(`${track.name()}\n${musicApp.playerState()}`)) : console.log(\"No song is playing!\")")
            .output()
            .expect("Failed to run song name command.");
        let str_data: String = if output.status.success() {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        } else {
            return self.fault(app);
        };

        let vals: Vec<&str> = str_data.split("\n").collect();
        let name = vals[0].trim().to_string();
        let state = vals[1].trim();

        if data.name != name || state != "playing" {
            return self.idle(app).await;
        }

        let current_time = SystemTime::now();
        let secs = current_time.duration_since(UNIX_EPOCH).unwrap().as_secs() * 1000;

        if secs > data.end {
            return self.idle(app).await;
        }

        AppState::Presence(data)
    }

    pub async fn handle_update(&self, app: &App) -> Self {
        match self {
            AppState::Idle => self.idle(app).await,
            AppState::Fault => self.fault(app),
            AppState::Presence(data) => self.presence(app, data.clone()).await,
        }
    }
}

pub struct App {
    pub state: AppState,
    pub client: Option<Client>,
}

impl Default for App {
    fn default() -> Self {
        App {
            state: AppState::Idle,
            client: None,
        }
    }
}

impl App {
    pub fn set_client(&mut self, client: Client) {
        self.client = Some(client);
    }

    pub async fn update(&mut self) -> &Self {
        self.state = self.state.handle_update(self).await;
        match &self.state {
            AppState::Presence(data) => {
                if let Some(drpc) = &mut self.client {
                    drpc.set_activity(|act| {
                        act.state(&data.artist)
                            ._type(discord_presence::models::ActivityType::Listening)
                            .details(&format!("{: <3}", data.name))
                            .instance(true)
                            .timestamps(|t| t.start(data.start).end(data.end))
                            .assets(|a| {
                                a.large_image(
                                    data.album
                                        .artworkUrl600
                                        .as_ref()
                                        .unwrap_or(&data.album.artworkUrl100),
                                )
                                .large_text(&data.album.collectionName)
                            })
                            .append_buttons(|button| {
                                button
                                    .label("Open in Apple Music")
                                    .url(generate_share_link(&data.album))
                            })
                    })
                    .expect("Failed to set activity");
                }
            }
            _ => {
                if let Some(drpc) = &mut self.client {
                    let _ = drpc.clear_activity();
                }
            }
        }
        self
    }
}
