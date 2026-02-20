use anyhow::Result;

use crate::cache;

pub fn cmd_reset() -> Result<()> {
    let root = cache::CachedState::root()?;
    if cache::CachedState::reset(&root)? {
        eprintln!("Cache cleared.");
    } else {
        eprintln!("No cache to clear.");
    }
    Ok(())
}
