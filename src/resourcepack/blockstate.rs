use std::{hash::Hash, str::FromStr};

use serde_with::DeserializeFromStr;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Hash, Error)]
pub enum ParseError {
    #[error("Key number {0} has either 0 or too many equal signs")]
    TooManyEq(usize),
}

#[derive(Debug, Hash, PartialEq, Eq, DeserializeFromStr)]
pub struct BlockState(Vec<String>);

impl BlockState {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.0.push(format!("{key}={value}"));
        self.0.sort_unstable();
    }
}

impl Default for BlockState {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for BlockState {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut block_state = Self::new();

        if s.is_empty() {
            return Ok(block_state);
        }

        let splits = s.split(',');

        for (i, split) in splits.enumerate() {
            let (k, v) = split.split_once('=').ok_or(ParseError::TooManyEq(i))?;
            block_state.insert(k, v);
        }

        Ok(block_state)
    }
}
