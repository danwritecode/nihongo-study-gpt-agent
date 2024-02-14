use std::convert::Into;
use serde::{Serialize, Deserialize};

use super::db::{NihongoWordInsert, NihongoWordTenseInsert};

#[derive(Clone, Debug)]
pub struct Prompt {
    pub system_prompt: String,
    pub user_prompt: String,
    pub model: String,
    pub provider: ModelProvider 
}

#[derive(Clone, Debug)]
pub enum ModelProvider {
    OpenAi,
    Mistral
}


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NihongoWordOpenAiRes {
    pub word: String,
    pub is_kanji: bool,
    pub word_reading: String,
    pub definition: String,
    pub sentence: String,
    pub sentence_translation: String,
    pub kanji_mnemonic: Option<String>,
    pub spoken_mnemonic: Option<String>,
    pub word_tenses: Vec<NihongoWordOpenAiResTense>
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NihongoWordOpenAiResTense {
    pub word: String,
    pub sentence: String,
    pub tense_type: String
}


impl Into<NihongoWordInsert> for NihongoWordOpenAiRes {
    fn into(self) -> NihongoWordInsert {
        let mut tenses = vec![];

        for t in self.word_tenses {
            let new_tense: NihongoWordTenseInsert = t.into();
            tenses.push(new_tense);
        }

        NihongoWordInsert {
            word: self.word,
            is_kanji: self.is_kanji,
            word_reading: self.word_reading,
            definition: self.definition,
            sentence: self.sentence,
            sentence_translation: self.sentence_translation,
            kanji_mnemonic: self.kanji_mnemonic,
            spoken_mnemonic: self.spoken_mnemonic,
            word_tenses: tenses
        }
    }
}

impl Into<NihongoWordTenseInsert> for NihongoWordOpenAiResTense {
    fn into(self) -> NihongoWordTenseInsert {
        NihongoWordTenseInsert {
            word: self.word,
            sentence: self.sentence,
            tense_type: self.tense_type
        }
    }
}
