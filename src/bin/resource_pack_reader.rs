use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use biomemap_tileserver::resourcepack;

fn main() -> Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
    env_logger::init();

    let path = PathBuf::from_str("./resourcepack/VanillaDefault 1.21.3")?;

    let resource_pack_map = resourcepack::get_block_texture_map(&path)?;

    dbg!(resource_pack_map);

    Ok(())
}
