use crate::Client;
use futures::future::BoxFuture;
use futures::prelude::*;
use phabricator_api::types::Cursor;
use phabricator_api::RequestError;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use thiserror::Error;

pub type QueryData<T> = (Vec<T>, Option<Cursor>);

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Request Failed: {0}")]
    RequestError(#[from] reqwest::Error),
}

enum QueryState<'f, T> {
    Cached(Vec<T>),
    Data(Vec<T>, Option<Cursor>),
    Next(BoxFuture<'f, Result<QueryData<T>, RequestError>>),
    Finished,
}

type Getter<S, T> = Box<
    dyn Fn(&Client, Arc<S>, Option<Cursor>) -> BoxFuture<Result<QueryData<T>, RequestError>> + Send,
>;
pub struct Search<'c, S, T> {
    client: &'c Client,
    search: Option<Arc<S>>,
    state: QueryState<'c, T>,
    getter: Getter<S, T>,
}

impl<'c, S, T> Search<'c, S, T> {
    pub fn new(
        client: &'c Client,
        cached: Vec<T>,
        search: Option<S>,
        getter: Getter<S, T>,
    ) -> Self {
        let search = search.map(Arc::new);

        Search {
            client,
            search,
            state: QueryState::Cached(cached),
            getter,
        }
    }
}

impl<'c, S, T> Stream for Search<'c, S, T>
where
    T: Unpin,
    S: 'c,
{
    type Item = Result<T, RequestError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let me = self.get_mut();

        loop {
            match me.state {
                QueryState::Cached(ref mut tasks) => {
                    if let Some(task) = tasks.pop() {
                        return Poll::Ready(Some(Ok(task)));
                    }

                    me.state = if let Some(s) = &me.search {
                        let f = (me.getter)(&me.client, s.clone(), None);
                        QueryState::Next(f)
                    } else {
                        QueryState::Finished
                    };
                }
                QueryState::Data(ref mut data, ref mut cursor) => {
                    if let Some(t) = data.pop() {
                        return Poll::Ready(Some(Ok(t)));
                    }
                    me.state = if let Some(cursor) = cursor.take() {
                        let f = (me.getter)(
                            me.client,
                            me.search.as_ref().unwrap().clone(),
                            Some(cursor),
                        );
                        QueryState::Next(f)
                    } else {
                        QueryState::Finished
                    };
                }
                QueryState::Next(ref mut f) => {
                    let (tasks, cursor) = futures::ready!(f.as_mut().poll(cx))?;
                    me.state = QueryState::Data(tasks, cursor);
                }
                QueryState::Finished => return Poll::Ready(None),
            }
        }
    }
}
