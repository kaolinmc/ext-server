use std::collections::HashMap;
use rocket::futures::SinkExt;

use rocket::serde::Serialize;
use serde::Deserialize;

use crate::types::ExtensionIdentifier;

// pub trait Index<T: Sized> {
//     fn find(
//         &self,
//         token: &str,
//     ) -> &Vec<(T, u8)>;
// }
//
// pub struct ExtensionIndex {
//     top: IndexNode<ExtensionIdentifier>,
// }

// impl Index<ExtensionIdentifier> for ExtensionIndex {
//     fn find(
//         &self,
//         token: &str,
//     ) -> &Vec<(ExtensionIdentifier, u8)> {
//         self.top.find(token)
//     }
// }

#[derive(Serialize, Deserialize)]
pub struct IndexNode<T> {
    path: String,
    rank: Vec<(T, u8)>,
    children: HashMap<char, Box<IndexNode<T>>>,

    empty_vec: Vec<(T, u8)>,
}

impl<T> IndexNode<T> {
    pub fn new() -> IndexNode<T> {
        IndexNode {
            path: "".to_string(),
            rank: vec![],
            children: Default::default(),
            empty_vec: vec![],
        }
    }

    // token must be lowercase
    pub fn find(
        &self,
        token: &str,
    ) -> &Vec<(T, u8)> {
        if token == "" {
            return &self.rank;
        }

        let first_char = token.chars().next().unwrap();

        self.children
            .get(&first_char)
            .map(|t| {
                t.find(&token[1..])
            }).unwrap_or(&self.empty_vec)
    }

    // pub fn find_all(
    //     &self,
    //     token: &str,
    // ) -> Vec<(T, u8)> {
    //     let result = Vec::new();
    //
    //     if token == "" {
    //         result.append(self.rank.clone())
    //         return &self.rank;
    //     }
    //
    //     let first_char = token.chars().next().unwrap();
    //
    //     self.children
    //         .get(&first_char)
    //         .map(|t| {
    //             t.find(&token[1..])
    //         }).unwrap_or(&self.empty_vec)
    // }

    pub fn insert(
        &mut self,
        token: &str,
        value: T,
        rank: u8,
    ) {
        if token == "" {
            self.rank.push((value, rank));
        } else {
            let first_char = token.chars().next().unwrap();

            if !self.children.contains_key(&first_char) {
                self.children.insert(first_char.clone(), Box::new(IndexNode {
                    path: format!("{}{}", self.path, first_char),
                    rank: vec![],
                    children: Default::default(),
                    empty_vec: vec![],
                }));
            }

            self.children.get_mut(&first_char)
                .map(|t| {
                    t.insert(&token[1..], value, rank)
                });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::search::index::IndexNode;

    #[test]
    fn test_tri_insert() {
        let mut node: IndexNode<String> = IndexNode::new();

        node.insert(
            "test-ing",
            "This is the value".to_string(),
            1,
        );
        node.insert(
            "test-ing",
            "This is the value".to_string(),
            4,
        );
        node.insert(
            "test-inga",
            "This is the value".to_string(),
            4,
        );

        println!("{:?}", node.find("test-inga"));
    }
}