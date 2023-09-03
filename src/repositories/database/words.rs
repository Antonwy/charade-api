use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;

use crate::models::custom_api_errors::Result;
use crate::models::word::{NewWord, Word};
use crate::schema;

use super::Database;

impl Database {
    pub async fn add_word_to_session(&self, new_word: NewWord) -> Result<Word> {
        use schema::words;

        let word = diesel::insert_into(words::table)
            .values(&new_word)
            .returning(Word::as_returning())
            .get_result(&mut self.connection().await?)
            .await?;

        Ok(word)
    }

    pub async fn get_number_of_words_in_session(&self, session_id: &str) -> Result<u16> {
        use schema::words::dsl::{session_id as session_id_column, words};

        let number_of_words: i64 = words
            .filter(session_id_column.eq(session_id))
            .count()
            .get_result(&mut self.connection().await?)
            .await?;

        Ok(number_of_words as u16)
    }

    pub async fn get_words_by_session_id(&self, session_id: &str) -> Result<Vec<Word>> {
        use crate::schema::words::dsl::{session_id as session_id_column, words};

        let w = words
            .filter(session_id_column.eq(session_id))
            .select(Word::as_select())
            .get_results(&mut self.connection().await?)
            .await?;

        Ok(w)
    }
}
