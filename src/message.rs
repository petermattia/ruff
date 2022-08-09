use colored::Colorize;
use rustpython_parser::ast::Location;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
#[serde(remote = "Location")]
struct LocationDef {
    #[serde(getter = "Location::row")]
    row: usize,
    #[serde(getter = "Location::column")]
    column: usize,
}

impl From<LocationDef> for Location {
    fn from(def: LocationDef) -> Location {
        Location::new(def.row, def.column)
    }
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    ImportStarUsage {
        filename: PathBuf,
        #[serde(with = "LocationDef")]
        location: Location,
    },
}

impl Message {
    /// A four-letter shorthand code for the message.
    pub fn code(&self) -> &'static str {
        match self {
            Message::ImportStarUsage {
                filename: _,
                location: _,
            } => "F403",
        }
    }

    /// The body text for the message.
    pub fn body(&self) -> &'static str {
        match self {
            Message::ImportStarUsage {
                filename: _,
                location: _,
            } => "Unable to detect undefined names",
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Message::ImportStarUsage { filename, location } => write!(
                f,
                "{}{}{}{}{}\t{}\t{}",
                filename.to_string_lossy().white().bold(),
                ":".cyan(),
                location.column(),
                ":".cyan(),
                location.row(),
                self.code().red().bold(),
                self.body()
            ),
        }
    }
}
