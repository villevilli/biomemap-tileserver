use std::{char, fmt::Display, str::FromStr};

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

/// Represents a valid minecraft resource identifier
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct MinecraftResourceIdentifier {
    namespace: String,
    path: String,
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
                namespace: "minecraft".to_string(),
                path: Self::is_valid_path(parts.pop().unwrap())?,
            }),
            2 => Ok(Self {
                path: Self::is_valid_path(parts.pop().unwrap())?,
                namespace: Self::is_valid_namespace(parts.pop().unwrap())?,
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

impl MinecraftResourceIdentifier {
    fn is_valid_path(path: String) -> Result<String, ParseError> {
        for char in path.chars() {
            Self::is_valid_path_character(&char)?;
        }
        Ok(path)
    }

    fn is_valid_path_character(char: &char) -> Result<(), ParseError> {
        if !matches!(char, '0'..='9' | 'a'..='z' | '_' | '-' | '.') {
            Err(ParseError::InvalidCharactersInPath)
        } else {
            Ok(())
        }
    }

    fn is_valid_namespace(namespace: String) -> Result<String, ParseError> {
        let namespace =
            Self::is_valid_path(namespace).map_err(|_| ParseError::InvalidCharactersInNamespace)?;
        if namespace.contains('/') {
            Err(ParseError::InvalidCharactersInNamespace)
        } else {
            Ok(namespace)
        }
    }
}
