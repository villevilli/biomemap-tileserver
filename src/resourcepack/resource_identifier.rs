use std::{
    char,
    fmt::{Debug, Display, write},
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::Deserialize;
use serde_with::DeserializeFromStr;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseError {
    #[error("String contained more than 1 colon")]
    TooManyColons,
    #[error("Namespace contains disallowed characters")]
    InvalidCharactersInNamespace,
    #[error("Path contains disallowed characters")]
    InvalidCharactersInPath,
    #[error("Empty string cannot be parsed")]
    EmptyString,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, DeserializeFromStr)]
pub(crate) struct ResourcePath(String);

impl FromStr for ResourcePath {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(is_valid_path(s.to_string())?))
    }
}

impl Display for ResourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for ResourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, DeserializeFromStr)]
pub struct ResourceNamespace(String);

impl FromStr for ResourceNamespace {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(is_valid_namespace(s.to_string())?))
    }
}

impl Display for ResourceNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Debug for ResourceNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
/// Represents a valid minecraft resource identifier
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, DeserializeFromStr)]
pub struct MinecraftResourceIdentifier {
    namespace: ResourceNamespace,
    path: ResourcePath,
}

impl FromStr for MinecraftResourceIdentifier {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseError::EmptyString);
        }

        let mut parts: Vec<String> = s.split(':').map(|x| x.to_owned()).collect();

        match parts.len() {
            0 => Err(ParseError::EmptyString),
            1 => Ok(Self {
                namespace: "minecraft".parse()?,
                path: parts.pop().unwrap().parse()?,
            }),
            2 => Ok(Self {
                path: parts.pop().unwrap().parse()?,
                namespace: parts.pop().unwrap().parse()?,
            }),
            _ => Err(ParseError::TooManyColons),
        }
    }
}

impl Display for MinecraftResourceIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}
impl Debug for MinecraftResourceIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
impl MinecraftResourceIdentifier {
    pub fn into_prefixed_path(self, path: &Path) -> PathBuf {
        let mut path = path.to_owned();
        path.push(self.into_path());
        path
    }

    pub fn into_path(self) -> PathBuf {
        let mut path = PathBuf::from("assets");
        path.push(self.namespace.0);
        let mut splits = self.path.0.split('/');

        let Some(split) = splits.next() else {
            unreachable!()
        };

        match split {
            "block" => {
                path.push("models/block");
                path.push(splits.next().unwrap());
                path.set_extension("json");
            }
            _ => panic!(),
        }

        path
    }

    pub fn into_texture_path(self) -> PathBuf {
        let mut path = PathBuf::from("assets");
        path.push(self.namespace.0);
        path.push("textures");
        path.push(self.path.0);
        path.with_extension("png")
    }
}

fn is_valid_path(path: String) -> Result<String, ParseError> {
    for char in path.chars() {
        is_valid_path_character(&char)?;
    }
    Ok(path)
}

fn is_valid_path_character(char: &char) -> Result<(), ParseError> {
    if !matches!(char, '0'..='9' | 'a'..='z' | '_' | '-' | '.' | '/') {
        Err(ParseError::InvalidCharactersInPath)
    } else {
        Ok(())
    }
}

fn is_valid_namespace(namespace: String) -> Result<String, ParseError> {
    let namespace =
        is_valid_path(namespace).map_err(|_| ParseError::InvalidCharactersInNamespace)?;
    if namespace.contains('/') {
        Err(ParseError::InvalidCharactersInNamespace)
    } else {
        Ok(namespace)
    }
}
