#![allow(unused)]
use model::BlockTextures;
pub use resource_identifier::{MinecraftResourceIdentifier, ParseError};

use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string},
    io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use image::DynamicImage;
use log::warn;
use resource_identifier::ResourceNamespace;
use serde::Deserialize;
use thiserror::Error;

mod model;
mod resource_identifier;

#[cfg(test)]
mod test;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Encountered invalid resource identifier")]
    ResourceIdentifierError(#[from] resource_identifier::ParseError),
    #[error("Encountered an error in filesystem io")]
    IOError(#[from] io::Error),
}

#[derive(Debug, Deserialize)]
struct BlockState {
    variants: Option<HashMap<String, VariantOptions>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum VariantOptions {
    #[serde(alias = "model")]
    Single(Variant),
    Multiple(Vec<Variant>),
}

#[derive(Debug, Deserialize, Clone)]
struct Variant {
    model: MinecraftResourceIdentifier,
}

pub fn get_block_texture_map(
    resource_pack_path: &Path,
) -> Result<HashMap<MinecraftResourceIdentifier, DynamicImage>> {
    let asset_path = resource_pack_path.join("assets");
    let potential_namespaces = read_dir(&asset_path);

    let namespaces: Vec<ResourceNamespace> = potential_namespaces?
        .filter(|path| path.as_ref().unwrap().file_type().unwrap().is_dir())
        .map(|path| Ok(path?.file_name().to_string_lossy().parse()?))
        .collect::<Result<Vec<ResourceNamespace>, Error>>()?;

    let mut blockstates = HashMap::new();

    for namespace in namespaces {
        read_blockstates(&asset_path.join(namespace.to_string()), &mut blockstates)?;
    }

    let mut thingies_2: HashMap<MinecraftResourceIdentifier, BlockTextures> = HashMap::new();

    for (resource_identifier, block_state) in blockstates.into_iter() {
        let Some(mut variant) = block_state.variants else {
            continue;
        };

        let Some(variant) = variant.remove("") else {
            continue;
        };

        let variant = match variant {
            VariantOptions::Single(variant) => variant,
            VariantOptions::Multiple(variants) => variants.first().unwrap().clone(),
        };

        let block_texture = match BlockTextures::from_file(resource_pack_path, variant.model) {
            Ok(block_texture) => block_texture,
            Err(e) => {
                warn!(
                    "Texture {} couldn't be rendered due to {}",
                    resource_identifier, e
                );
                continue;
            }
        };

        if let Some(block_texture) = block_texture {
            thingies_2.insert(resource_identifier, block_texture);
        }
    }

    dbg!(&thingies_2);

    todo!()
}

fn read_blockstates(
    dir: &Path,
    mut map: &mut HashMap<MinecraftResourceIdentifier, BlockState>,
) -> Result<()> {
    for file in read_dir(dir.join("blockstates"))? {
        if !&file.as_ref().unwrap().file_type()?.is_file() {
            continue;
        }

        let path = file?.path();

        let block_state = read_to_string(&path)?;

        let variants: BlockState = match serde_json::from_str(&block_state) {
            Ok(a) => a,
            Err(e) => {
                warn!(
                    "Failed to parse the following blockstate: {} with error: {}",
                    path.to_str().unwrap_or("{non_utf8_filename}"),
                    e
                );
                continue;
            }
        };

        map.insert(
            path.with_extension("")
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .parse()?,
            variants,
        );
    }
    Ok(())
}
