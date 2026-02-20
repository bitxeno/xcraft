use anyhow::Result;

use crate::destination;

pub fn cmd_destinations() -> Result<()> {
    let dests = destination::list_destinations()?;
    for d in &dests {
        println!("{d}");
    }
    Ok(())
}
