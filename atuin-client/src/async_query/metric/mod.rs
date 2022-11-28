use futures::Stream;
use std::{pin::Pin, task::Poll};

use super::transaction::Statement;

pub(crate) struct MetricStream<'a> {
    stmt: &'a Statement,
    stream: Pin<Box<dyn Stream<Item = Result<sqlx::sqlite::SqliteRow, sqlx::Error>> + 'a + Send>>,
}

impl<'a> Stream for MetricStream<'a> {
    type Item = Result<sqlx::sqlite::SqliteRow, sqlx::Error>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let res = Pin::new(&mut this.stream).poll_next(cx);
        res
    }
}

impl<'a> MetricStream<'a> {
    #[allow(dead_code)]
    pub(crate) fn new<S>(stmt: &'a Statement, stream: S) -> Self
    where
        S: Stream<Item = Result<sqlx::sqlite::SqliteRow, sqlx::Error>> + 'a + Send,
    {
        MetricStream {
            stmt,
            stream: Box::pin(stream),
        }
    }
}
