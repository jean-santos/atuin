use std::time::Instant;

use futures::StreamExt;
use sqlx::{SqlitePool, sqlite::SqliteRow};

use crate::async_query::{transaction::{Statement, DatabaseTransaction}, stream::get_stream};

mod metric;
mod stream;
mod transaction;

pub async fn async_query<F>(query: &str, pool: &SqlitePool, fun : F) 
-> core::result::Result<usize, sqlx::Error> 
where F: Fn(SqliteRow) -> ()
{
    let stat = Statement { sql: query.into() };

    let start = Instant::now();

    let database_transaction = DatabaseTransaction::new(pool).await;

    let mut stream = get_stream(&database_transaction, stat).await.unwrap();

    let mut count = 0;

    while let Some(item) = stream.next().await {
        match item {
            Ok(row) => {
                count+=1;
                fun(row);
            }
            Err(_e) => {}
        }
    }
    let duration = start.elapsed();
    // println!("Time elapsed in expensive_function() is: {:?}", duration);
    Ok(count)
}
