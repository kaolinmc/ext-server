use std::collections::HashMap;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::hash::Hash;
use std::io;
use std::io::Write;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::search::index::IndexNode;
use crate::search::SearchError;
use crate::search::token::WordTokenizer;
use crate::types::ExtensionIdentifier;

pub struct SearchHandler<T: Sized> {
    index_node: IndexNode<T>,
    tokenizer: WordTokenizer,
    phantom_data: PhantomData<T>,
}

impl<T> SearchHandler<T>
where
    T: Sized + Clone + Eq + Hash,
{

    pub fn new() -> Result<SearchHandler<T>, SearchError> {
        Ok(SearchHandler {
            index_node: IndexNode::new(),
            tokenizer: WordTokenizer::new().map_err(|_| {
                SearchError::TokenizationError
            })?,
            phantom_data: Default::default(),
        })
    }

    fn group_by<A, K, F>(items: Vec<A>, key_fn: F) -> HashMap<K, Vec<A>>
    where
        K: Eq + Hash,
        F: Fn(&A) -> K,
    {
        let mut map = HashMap::new();
        for item in items {
            let key = key_fn(&item);
            map.entry(key).or_insert_with(Vec::new).push(item);
        }
        map
    }

    pub fn search(
        &self,
        query: &str,
    ) -> Result<Vec<T>, SearchError> {
        let tokens = self.tokenizer.tokenize(query).map_err(|_| {
            SearchError::TokenizationError
        })?;

        let result = Self::group_by(tokens.iter().flat_map(|t| {
            self.index_node.find(t.as_str())
        }).collect(), |t| {
            t.0.clone()
        });

        let mut result: Vec<(u128, &T)> = result.iter().map(|t| {
            (t.1.iter().fold(0u128, |acc, it| {
                (acc) + (it.1 as u128)
            }), t.0)
        }).collect();
        result.sort_by_key(|t| t.0);
        result.reverse();

        Ok(result.iter().map(|t| t.1.clone()).collect())
    }

    pub fn index(
        &mut self,
        content: &str,
        value: T,
        rank: u8,
    ) -> Result<(), SearchError> {
        let tokens = self.tokenizer.tokenize(content).map_err(|e| {
            SearchError::TokenizationError
        })?;

        for token in tokens {
            self.index_node.insert(
                token.as_str(),
                value.clone(),
                rank,
            );
        }

        Ok(())
    }
}

impl SearchHandler<ExtensionIdentifier> {
    pub fn persist_to<P: Into<PathBuf>>(&self, path: P) -> io::Result<()> {
        let path = path.into();
        if !Path::new(&path).exists() {
            if let Some(x) = path.parent() {
                create_dir_all(x.clone())?;
            }
            File::create(path.clone())?;
        };

        let mut file = std::fs::OpenOptions::new().write(true).truncate(true).open(path)?;

        let content = serde_json::to_vec(&self.index_node).unwrap();

        file.write_all(content.deref())
    }

    pub fn hydrate_cache< P: Into<PathBuf>>(path: P) -> Result<SearchHandler<ExtensionIdentifier>, SearchError> {
        let path = path.into();
        let index: IndexNode<ExtensionIdentifier> = if Path::new(&path).exists() {
            let file = File::open(path).map_err(|e| {
                SearchError::IoError(e)
            })?;

            serde_json::from_reader(file).unwrap()
        } else {
            IndexNode::new()
        };

        Ok(SearchHandler {
            index_node: index,
            tokenizer: WordTokenizer::new().map_err(|_| {
                SearchError::TokenizationError
            })?,
            phantom_data: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::search::index::IndexNode;
    use crate::search::search::SearchHandler;
    use crate::search::token::WordTokenizer;

    #[test]
    fn search() {
        let mut handler = SearchHandler {
            index_node: IndexNode::new(),
            tokenizer: WordTokenizer::new().unwrap(),
            phantom_data: Default::default(),
        };

        handler.index(
            "A really cool extension that does a third thing after it does the fourth thing. Also it does a third thing.",
            "2",
            1,
        ).unwrap();

        handler.index(
            "Minecraft is a cool game.",
            "1",
            1,
        ).unwrap();

        let result = handler.search("I want minecraft that has a third and fourth universe").unwrap();

        println!("{:?}", result);
    }
}