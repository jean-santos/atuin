use futures::Stream;
use futures::{lock::MutexGuard, TryStreamExt};
use sqlx::Executor;
use std::ops::DerefMut;
use std::{pin::Pin, task::Poll};
use crate::async_query::transaction::Statement;
use crate::async_query::transaction::InnerConnection;
use crate::async_query::metric::MetricStream;

#[ouroboros::self_referencing]
pub struct TransactionStream<'a> {
    stmt: Statement,
    conn: MutexGuard<'a, InnerConnection>,
    #[borrows(mut conn, stmt)]
    #[not_covariant]
    stream: MetricStream<'this>,
}

impl<'a> TransactionStream<'a> {
    #[allow(unused_variables)]
    pub(crate) fn build(
        conn: MutexGuard<'a, InnerConnection>,
        stmt: Statement,
    ) -> TransactionStream<'a> {
        TransactionStreamBuilder {
            stmt,
            conn,
            stream_builder: |conn, stmt| match conn.deref_mut() {
                InnerConnection::Sqlite(c) => {
                    let query = sqlx::query(&stmt.sql);
                    let stream = c
                        .fetch(query)
                        .map_ok(Into::into)
                        .map_err(|x| sqlx::Error::PoolTimedOut);
                    MetricStream::new(stmt, stream)
                }
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            },
        }
        .build()
    }
}

impl<'a> Stream for TransactionStream<'a> {
    type Item = Result<sqlx::sqlite::SqliteRow, sqlx::Error>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.with_stream_mut(|stream| Pin::new(stream).poll_next(cx))
    }
}
