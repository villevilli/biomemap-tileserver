use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use image::RgbaImage;
use serde::Deserialize;
use serde_json::from_str;

use super::{Facing, MinecraftResourceIdentifier, resource_identifier};

#[derive(Debug, Deserialize, Clone)]
pub struct BlockTextures {
    down: Option<Face>,
    up: Option<Face>,
    north: Option<Face>,
    south: Option<Face>,
    west: Option<Face>,
    east: Option<Face>,
}

impl BlockTextures {
    pub fn get(&self, facing: Facing) -> Option<Face> {
        match facing {
            Facing::Up => self.up.clone(),
            Facing::Down => self.down.clone(),
            Facing::North => self.north.clone(),
            Facing::South => self.south.clone(),
            Facing::East => self.east.clone(),
            Facing::West => self.west.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Model {
    pub(crate) parent: Option<MinecraftResourceIdentifier>,
    pub(crate) textures: Option<HashMap<String, String>>,
    pub(crate) elements: Option<Vec<Element>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Element {
    faces: BlockTextures,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Face {
    texture: String,
}

impl Model {
    pub fn from_file(
        path: &Path,
        resource_identifier: MinecraftResourceIdentifier,
    ) -> Result<Option<Self>> {
        let data = read_to_string(resource_identifier.into_prefixed_path(path))?;

        Model::try_fill_from_parent(from_str(&data)?, path)
    }

    pub(crate) fn get_side(&self, facing: super::Facing) -> Option<MinecraftResourceIdentifier> {
        for element in self.elements.as_ref()? {
            if let Some(face) = element.faces.get(facing) {
                if let Some(path) = self
                    .textures
                    .as_ref()
                    .unwrap()
                    .get(face.texture.strip_prefix('#').unwrap_or(&face.texture))
                {
                    return Some(path.parse().unwrap());
                }
            }
        }
        None
    }

    pub fn from_namespace(name: MinecraftResourceIdentifier, path: &Path) -> Result<Self> {
        let path = name.into_prefixed_path(path);
        let data = read_to_string(path)?;
        Ok(from_str(&data)?)
    }

    pub fn try_fill_from_parent(mut self, path: &Path) -> Result<Option<Self>> {
        let Some(parent) = self.parent else {
            return Ok(Some(self));
        };

        let new_model_path = parent.clone().into_prefixed_path(path);
        let data = read_to_string(new_model_path)?;

        let mut new_model: Model = from_str(&data)?;

        if let Some(textures) = self.textures {
            if let Some(x) = new_model.textures.as_mut() {
                textures.into_iter().for_each(|(k, v)| {
                    x.try_insert(k, v);
                });
            } else {
                new_model.textures = Some(textures);
            }
        }

        if let Some(mut new_textures) = new_model.textures.as_mut() {
            let refs: Vec<(String, String)> = new_textures
                .iter()
                .filter(|(k, v)| v.starts_with('#'))
                .map(|(k, v)| (k.clone(), v.strip_prefix('#').unwrap().to_owned()))
                .collect();

            for (set_key, ref_key) in refs {
                let set_value = new_textures.get(&ref_key).unwrap();
                new_textures.insert(set_key, set_value.clone());
            }
        }

        if let Some(mut elemnts) = self.elements {
            let mut e = new_model.elements.unwrap_or_default();
            e.append(&mut elemnts);
            new_model.elements = Some(e);
        }

        new_model.try_fill_from_parent(path)
    }
}
