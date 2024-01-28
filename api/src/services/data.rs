use dotenv::dotenv;
use sqlx::{postgres::PgConnection, Connection};
use anyhow::Result;

use crate::{NihongoWordReqWord, NihongoWordReqTense};

pub async fn add_word(word: &NihongoWordReqWord) -> Result<i64> {
    // Fine because we aren't hitting this a lot, don't need a pool, we're just hacking
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let mut connection = PgConnection::connect(db_url.as_str()).await?;

    // wildly inefficient but I don't care, we're just hacking
    let rec = sqlx::query!(
            r#"
                INSERT INTO nihongo_word ( word, definition, sentence )
                VALUES ( $1, $2, $3 )
                RETURNING id
            "#,
            word.word,
            word.definition,
            word.sentence
        )
        .fetch_one(&mut connection)
        .await?;


    Ok(rec.id)
}

pub async fn add_word_tense(id: i64, words: Vec<NihongoWordReqTense>) -> Result<()> {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL")?;
    let mut connection = PgConnection::connect(db_url.as_str()).await?;

    // wildly inefficient but I don't care, we're just hacking
    for w in &words {
        sqlx::query!(
                r#"
                    INSERT INTO nihongo_word_tense ( word_id, word, sentence, tense_type )
                    VALUES ( $1, $2, $3, $4 )
                "#,
                id,
                w.word,
                w.sentence,
                w.tense_type
            )
            .execute(&mut connection)
            .await?;
    }

    Ok(())
}
