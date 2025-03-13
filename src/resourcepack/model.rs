use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::from_str;

use super::{MinecraftResourceIdentifier, resource_identifier};

#[derive(Debug)]
pub struct BlockTextures {
    down: MinecraftResourceIdentifier,
    up: MinecraftResourceIdentifier,
    north: MinecraftResourceIdentifier,
    south: MinecraftResourceIdentifier,
    west: MinecraftResourceIdentifier,
    east: MinecraftResourceIdentifier,
}

impl BlockTextures {
    pub fn from_file(
        path: &Path,
        resource_identifier: MinecraftResourceIdentifier,
    ) -> Result<Option<Self>> {
        let data = read_to_string(resource_identifier.into_prefixed_path(path))?;

        let Some(mut raw_model) = RawModel::try_fill_from_parent(from_str(&data)?, path)? else {
            return Ok(None);
        };

        let mut textures = raw_model.textures.unwrap();

        Ok(Some(Self {
            down: textures.remove("down").unwrap().parse()?,
            up: textures.remove("up").unwrap().parse()?,
            north: textures.remove("north").unwrap().parse()?,
            south: textures.remove("south").unwrap().parse()?,
            west: textures.remove("west").unwrap().parse()?,
            east: textures.remove("east").unwrap().parse()?,
        }))
    }
}

#[derive(Debug, Deserialize, Clone)]
struct RawModel {
    parent: MinecraftResourceIdentifier,
    textures: Option<HashMap<String, String>>,
}

impl RawModel {
    pub fn from_namespace(name: MinecraftResourceIdentifier, path: &Path) -> Result<Self> {
        let path = name.into_prefixed_path(path);
        let data = read_to_string(path)?;
        Ok(from_str(&data)?)
    }

    pub fn try_fill_from_parent(mut self, path: &Path) -> Result<Option<Self>> {
        let self_clone = self.clone();

        if path.to_string_lossy().contains("log") {
            dbg!("here");
        }

        if path.ends_with("cube.json") {
            return Ok(Some(self));
        }

        let new_model_path = self.parent.into_prefixed_path(path);
        let data = read_to_string(new_model_path)?;
        let mut new_model: RawModel = from_str(&data)?;

        if new_model.textures.is_none() {
            new_model.textures = Some(self.textures.clone().unwrap())
        }

        new_model
            .textures
            .as_mut()
            .unwrap()
            .values_mut()
            .for_each(|x| {
                if x.starts_with('#') {
                    *x = self
                        .textures
                        .as_mut()
                        .unwrap()
                        .get(x.strip_prefix('#').unwrap())
                        .cloned()
                        .unwrap();
                }
            });

        let path = new_model.parent.clone().into_path();

        new_model.try_fill_from_parent(&path)
    }
}
