use reqwest::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SongDetails {
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: f32,
    pub position: f32,
    pub state: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ApiResponse {
    pub resultCount: usize,
    pub results: Vec<AlbumResult>,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(unused)]
pub struct AlbumResult {
    pub trackName: String,
    pub collectionName: String,
    pub artistName: String,
    pub artworkUrl100: String,
    pub artworkUrl600: Option<String>,
    pub collectionId: u32,
    pub trackId: u32,
}

pub async fn fetch_album(
    album: &str,
    artist: &str,
    song: &str,
) -> anyhow::Result<Option<AlbumResult>, Error> {
    let url = format!(
        "https://itunes.apple.com/search?term={}+{}+{}&entity=song&limit=200",
        artist.replace(' ', "+"),
        song.replace(' ', "+"),
        album.replace(' ', "+")
    );

    let response = reqwest::get(&url).await?.json::<ApiResponse>().await?;
    let filtered: Vec<&AlbumResult> = response
        .results
        .iter()
        .filter(|a| a.trackName == song && a.artistName == artist && a.collectionName == album)
        .collect();

    if !filtered.is_empty() {
        Ok(Some(filtered[0].clone()))
    } else if !response.results.is_empty() {
        Ok(Some(response.results[0].clone()))
    } else {
        Ok(None)
    }
}

pub fn generate_share_link(album: &AlbumResult) -> String {
    format!(
        "https://music.apple.com/us/album/{}/{}?i={}&ls=1&app=music",
        album.trackName.replace(' ', "-"),
        album.collectionId,
        album.trackId
    )
}
