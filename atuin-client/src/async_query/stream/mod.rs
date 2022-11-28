use futures::{Future, Stream, TryStreamExt};
use std::pin::Pin;
use sqlx::sqlite::SqliteRow;
pub mod transaction;

use super::transaction::{Statement, DatabaseTransaction};

pub trait StreamTrait: Send + Sync {
    type Stream<'a>: Stream<Item = Result<sqlx::sqlite::SqliteRow, sqlx::Error>> + Send
    where
        Self: 'a;

    fn stream<'a>(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, sqlx::Error>> + 'a + Send>>;
}

pub async fn get_stream<'a: 'b, 'b>(
    db: &'a DatabaseTransaction,
    stmt: Statement,
) -> Result<Pin<Box<dyn Stream<Item = Result<SqliteRow, sqlx::Error>> + 'b + Send>>, sqlx::Error> {
    let stream = db.stream(stmt).await?;
    Ok(Box::pin(
        stream.and_then(|row| futures::future::ready(Ok(row))),
    ))
}
