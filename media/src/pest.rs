use std::{collections::HashMap, num::ParseIntError};

use pest::Parser;
use pest_derive::Parser;
use serde::Serialize;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "metadata.pest"]
pub struct MetadataParser;

#[derive(Debug)]
enum MetadataType {
    Trackid(String),
    Title(String),
    Album(String),
    Artist(String),
    ArtUrl(String),
    Url(String),
    Length(usize),
}

impl TryFrom<(&str, &str)> for MetadataType {
    type Error = MetadataError;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        match value.0 {
            "trackid" => Ok(MetadataType::Trackid(value.1.to_string())),
            "title" => Ok(MetadataType::Title(value.1.to_string())),
            "album" => Ok(MetadataType::Album(value.1.to_string())),
            "artist" => Ok(MetadataType::Artist(value.1.to_string())),
            "artUrl" => Ok(MetadataType::ArtUrl(value.1.to_string())),
            "url" => Ok(MetadataType::Url(value.1.to_string())),
            "length" => Ok(MetadataType::Length(value.1.parse::<usize>().map_err(
                |err| MetadataError::ParsingError("usize", value.1.to_string(), err),
            )?)),
            _ => Err(MetadataError::UnknownMetadataType(value.0.to_string())),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    player: String,
    trackid: Option<String>,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    art_url: Option<String>,
    url: Option<String>,
    length: Option<String>,
}

impl Metadata {
    fn new(player: String) -> Self {
        Self {
            player,
            trackid: None,
            title: None,
            artist: None,
            album: None,
            art_url: None,
            url: None,
            length: None,
        }
    }
}

impl TryFrom<&[MetadataEntry]> for Metadata {
    type Error = MetadataError;

    fn try_from(value: &[MetadataEntry]) -> Result<Self, Self::Error> {
        let mut metadata = match value.first() {
            Some(meta) => Metadata::new(meta.source.clone()),
            None => return Err(MetadataError::NoPlayer),
        };
        for entry in value.iter().filter(|entry| metadata.player == entry.source) {
            match &entry.value {
                MetadataType::Trackid(trackid) => metadata.trackid = Some(trackid.to_string()),
                MetadataType::Title(title) => metadata.title = Some(title.to_string()),
                MetadataType::Album(album) => metadata.album = Some(album.to_string()),
                MetadataType::Artist(artist) => metadata.artist = Some(artist.to_string()),
                MetadataType::ArtUrl(art_url) => metadata.art_url = Some(art_url.to_string()),
                MetadataType::Url(url) => metadata.url = Some(url.to_string()),
                MetadataType::Length(length) => metadata.length = Some(length.to_string()),
            }
        }
        Ok(metadata)
    }
}

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Unknown Metadata Type: {}", .0)]
    UnknownMetadataType(String),
    #[error("Unknown Metadata Type: {}", .0)]
    PestError(Box<pest::error::Error<Rule>>),
    #[error("Expected {} to be {}: {}", .1, .0, .2)]
    ParsingError(&'static str, String, ParseIntError),
    #[error("No Player found")]
    NoPlayer,
}

#[derive(Debug)]
pub(crate) struct MetadataEntry {
    source: String,
    value: MetadataType,
}

pub fn parse_metadata(input: &str) -> Result<Vec<Result<Metadata, MetadataError>>, MetadataError> {
    let mut parsed = MetadataParser::parse(Rule::input, input)
        .map_err(|err| MetadataError::PestError(Box::new(err)))?;
    let mut entries = Vec::new();

    for line in parsed.next().unwrap().into_inner() {
        if line.as_rule() == Rule::line {
            let mut inner_rules = line.into_inner();
            let source = inner_rules.next().unwrap().as_str().to_string();

            // let protocol = inner_rules.next().unwrap().as_str().to_string();
            let key = inner_rules.next().unwrap().as_str();
            let value = inner_rules.next().unwrap().as_str();
            entries.push(MetadataEntry {
                source,
                value: MetadataType::try_from((key, value))?,
            })
        }
    }
    let mut grouped: HashMap<String, Vec<MetadataEntry>> = HashMap::new();

    for obj in entries {
        grouped
            .entry(obj.source.clone())
            .or_insert_with(Vec::new)
            .push(obj);
    }
    let res = grouped
        .values()
        .map(|value| Metadata::try_from(value.as_slice()))
        .collect();
    Ok(res)
}
