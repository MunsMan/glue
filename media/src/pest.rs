use std::num::ParseIntError;

use pest::Parser;
use pest_derive::Parser;
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

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Unknown Metadata Type: {}", .0)]
    UnknownMetadataType(String),
    #[error("Unknown Metadata Type: {}", .0)]
    PestError(pest::error::Error<Rule>),
    #[error("Expected {} to be {}: {}", .1, .0, .2)]
    ParsingError(&'static str, String, ParseIntError),
}

#[derive(Debug)]
pub(crate) struct Metadata {
    source: String,
    protocol: String,
    value: MetadataType,
}

pub fn parse_metadata(input: &str) -> Result<Vec<Metadata>, MetadataError> {
    let mut parsed = MetadataParser::parse(Rule::input, input).map_err(MetadataError::PestError)?;
    let mut res = Vec::new();

    for line in parsed.next().unwrap().into_inner() {
        if line.as_rule() == Rule::line {
            let mut inner_rules = line.into_inner();
            let source = inner_rules.next().unwrap().as_str().to_string();

            let protocol = inner_rules.next().unwrap().as_str().to_string();
            let key = inner_rules.next().unwrap().as_str();
            let value = inner_rules.next().unwrap().as_str();
            res.push(Metadata {
                source,
                protocol,
                value: MetadataType::try_from((key, value))?,
            })
        }
    }

    Ok(res)
}
