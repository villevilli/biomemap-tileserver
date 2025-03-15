use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::from_str;

use super::{MinecraftResourceIdentifier, resource_identifier};

#[derive(Debug, Deserialize, Clone)]
pub struct BlockTextures {
    down: Option<Face>,
    up: Option<Face>,
    north: Option<Face>,
    south: Option<Face>,
    west: Option<Face>,
    east: Option<Face>,
}

impl Model {}

#[derive(Debug, Deserialize, Clone)]
pub struct Model {
    parent: Option<MinecraftResourceIdentifier>,
    textures: Option<HashMap<String, String>>,
    elements: Option<Vec<Element>>,
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
        dbg!(&resource_identifier);
        let data = read_to_string(resource_identifier.into_prefixed_path(path))?;

        Model::try_fill_from_parent(from_str(&data)?, path)
    }

    pub fn from_namespace(name: MinecraftResourceIdentifier, path: &Path) -> Result<Self> {
        let path = name.into_prefixed_path(path);
        let data = read_to_string(path)?;
        Ok(from_str(&data)?)
    }

    pub fn try_fill_from_parent(mut self, path: &Path) -> Result<Option<Self>> {
        let self_clone = self.clone();
        dbg!(self_clone);

        let Some(parent) = self.parent else {
            eprintln!(
                "RETURNED-------------------RETURNED-------------------RETURNED-------------------RETURNED"
            );
            return Ok(Some(self));
        };

        let new_model_path = parent.clone().into_prefixed_path(path);
        let data = read_to_string(new_model_path)?;

        let mut new_model: Model = from_str(&data)?;

        if let Some(new_textures) = new_model.textures.as_mut() {
            new_textures.values_mut().for_each(|new_texture| {
                if new_texture.starts_with('#') {
                    *new_texture = self
                        .textures
                        .as_mut()
                        .unwrap()
                        .get(new_texture.strip_prefix('#').unwrap())
                        .cloned()
                        .unwrap();
                }
            });
        }

        if let Some(mut elemnts) = self.elements {
            let mut e = new_model.elements.unwrap_or_default();
            e.append(&mut elemnts);
            new_model.elements = Some(e);
        }
        if let Some(textures) = self.textures {
            if let Some(x) = new_model.textures.as_mut() {
                textures.into_iter().for_each(|(k, v)| {
                    x.try_insert(k, v);
                });
            } else {
                new_model.textures = Some(textures);
            }
        }

        new_model.try_fill_from_parent(path)
    }
}
