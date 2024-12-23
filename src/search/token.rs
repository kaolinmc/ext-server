use std::collections::HashSet;
use rust_stemmers::{Algorithm, Stemmer};
use tokenizers::{Encoding, Tokenizer};

pub struct WordTokenizer(Tokenizer, &'static str);

impl WordTokenizer {
    pub fn new() -> Result<WordTokenizer, tokenizers::Error> {
        let tokenizer = Tokenizer::from_pretrained("bert-base-uncased", None)?;
        Ok(WordTokenizer(
            tokenizer,
            "##",
        ))
    }

    pub fn tokenize(&self, text: &str) -> Result<Vec<String>, tokenizers::Error> {
        let text = text.to_lowercase();

        let encoding: Encoding = self.0.encode(text, false)?;
        let tokens = encoding.get_tokens();

        let tokens = self.combine_tokens(tokens);

        let stopwords: HashSet<&str> = vec![
            "i", "me", "my", "myself", "we", "our", "ours", "ourselves", "you", "your", "yours", "yourself", "yourselves",
            "he", "him", "his", "himself", "she", "her", "hers", "herself", "it", "its", "itself", "they", "them", "their",
            "theirs", "themselves", "what", "which", "who", "whom", "this", "that", "these", "those", "am", "is", "are",
            "was", "were", "be", "been", "being", "have", "has", "had", "having", "do", "does", "did", "doing", "a", "an",
            "the", "and", "but", "if", "or", "because", "as", "until", "while", "of", "at", "by", "for", "with", "about",
            "against", "between", "into", "through", "during", "before", "after", "above", "below", "to", "from", "up",
            "down", "in", "out", "on", "off", "over", "under", "again", "further", "then", "once", "here", "there", "when",
            "where", "why", "how", "all", "any", "both", "each", "few", "more", "most", "other", "some", "such", "no", "nor",
            "not", "only", "own", "same", "so", "than", "too", "very", "s", "t", "can", "will", "just", "don", "should", "now",
            ".", ",", ":", ";", "!", "?",
        ].into_iter().collect();

        let stemmer = Stemmer::create(Algorithm::English);

        let tokens: Vec<String> = tokens.iter()
            .filter(|&token| !stopwords.contains(token.as_str()))
            .map(|token| stemmer.stem(token).to_string())  // Stem the words
            .map(|t| t.clone())
            .collect();

        if tokens.len() == 0 {
            return Ok(vec!["".to_string()])
        }

        Ok(tokens)
    }

    fn combine_tokens(&self, tokens: &[String]) -> Vec<String> {
        let mut vec: Vec<String> = Vec::new();
        for x in tokens {
            if let Some(x) = x.strip_prefix(&self.1) {
                let last = vec.pop().unwrap_or("".to_string());
                vec.push(format!("{}{}", last, x))
            } else {
                vec.push(x.clone());
            }
        }
        vec
    }
}


#[cfg(test)]
mod tests {
    use crate::search::token::WordTokenizer;

    #[test]
    fn test_tokenizing() {
        // Example input text
        let text = "this is an example of a searchable index creation in Rust.";

        let tokenizer = WordTokenizer::new().unwrap();
        let tokens = tokenizer.tokenize(text);


        // Print the result
        println!("{:?}", tokens);
    }
}