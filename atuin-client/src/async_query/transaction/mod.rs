use futures::lock::Mutex;
use sqlx::SqlitePool;
use std::{future::Future, pin::Pin, sync::Arc};

use sqlx::pool::PoolConnection;
use super::stream::{StreamTrait, transaction::TransactionStream};

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub sql: String,
}

#[derive(Debug)]
pub(crate) enum InnerConnection {
    Sqlite(PoolConnection<sqlx::Sqlite>),
}

pub struct DatabaseTransaction {
    conn: Arc<Mutex<InnerConnection>>,
}

impl DatabaseTransaction {
    pub(crate) async fn new(pool: &SqlitePool) -> Self {
        let pool = pool.acquire().await.unwrap();
        let ic = InnerConnection::Sqlite(pool);
        let mutex_ic = Arc::new(Mutex::new(ic));
        Self { conn: mutex_ic }
    }
}

impl StreamTrait for DatabaseTransaction {
    type Stream<'a> = TransactionStream<'a>;

    fn stream<'a>(
        &'a self,
        stmt: Statement,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, sqlx::Error>> + 'a + Send>> {
        Box::pin(async move {
            let conn = self.conn.lock().await;
            Ok(TransactionStream::build(conn, stmt))
        })
    }
}
