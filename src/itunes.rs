use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Library {
    #[serde(rename = "Application Version")]
    pub application_version: String,

    #[serde(rename = "Library Persistent ID")]
    pub persistent_id: String,

    #[serde(rename = "Date")]
    pub date: String,

    #[serde(rename = "Tracks")]
    pub tracks: HashMap<String, Track>,
}

generate_id!(TrackID);
generate_id!(PersistentID);
generate_id!(PlaylistID);

#[derive(Serialize, Deserialize)]
pub struct Track {
    #[serde(rename = "Track ID")]
    pub id: TrackID,

    #[serde(rename = "Persistent ID")]
    pub persistent_id: String,

    #[serde(rename = "Name")]
    pub name: Option<String>,

    #[serde(rename = "Artist")]
    pub artist: Option<String>,

    #[serde(rename = "Composer")]
    pub composer: Option<String>,

    #[serde(rename = "Album")]
    pub album: Option<String>,

    #[serde(rename = "Genre")]
    pub genre: Option<String>,

    #[serde(rename = "Location")]
    pub location: Option<String>,

    #[serde(rename = "Year")]
    pub year: Option<u32>,

    #[serde(rename = "Date Modified")]
    pub date_modified: Option<DateTime<Utc>>,

    #[serde(rename = "Date Added")]
    pub date_added: Option<DateTime<Utc>>,

    #[serde(rename = "Play Count")]
    pub play_count: Option<u32>,

    #[serde(rename = "Play Date UTC")]
    pub play_date: Option<DateTime<Utc>>,

    #[serde(rename = "Track Type")]
    pub track_type: String,

    #[serde(rename = "Kind")]
    pub kind: Option<String>,
}
