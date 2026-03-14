use anyhow::Result;
use serde::Serialize;

use crate::destination::{self, Destination};

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum DestinationEntry {
    Simulator {
        name: String,
        os: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<String>,
        arg: String,
    },
    Device {
        name: String,
        device_type: String,
        arg: String,
    },
    #[serde(rename = "macos")]
    MacOS { arch: String, arg: String },
}

impl From<Destination> for DestinationEntry {
    fn from(d: Destination) -> Self {
        let arg = d.spec();
        match d {
            Destination::Simulator {
                name, os, state, ..
            } => DestinationEntry::Simulator {
                name,
                os,
                state,
                arg,
            },
            Destination::Device {
                name, device_type, ..
            } => DestinationEntry::Device {
                name,
                device_type,
                arg,
            },
            Destination::MacOS { arch } => DestinationEntry::MacOS { arch, arg },
        }
    }
}

pub fn cmd_destinations(json: bool) -> Result<()> {
    let dests = destination::list_destinations()?;
    if json {
        let entries: Vec<DestinationEntry> =
            dests.into_iter().map(DestinationEntry::from).collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for d in &dests {
            println!("{d}");
        }
    }
    Ok(())
}
