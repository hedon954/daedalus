use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use futures_util::stream;
use serde_json::Value;

use crate::agent::{
    llm::{LLmRequest, Llm},
    stream_event::{EventStream, StreamEvent},
};

pub(crate) struct FakeLlm {
    turns: Mutex<VecDeque<Vec<StreamEvent>>>,
    requests: Mutex<Vec<Vec<Value>>>,
}

impl FakeLlm {
    pub(crate) fn new(turns: Vec<Vec<StreamEvent>>) -> Arc<Self> {
        Arc::new(Self {
            turns: Mutex::new(VecDeque::from(turns)),
            requests: Mutex::new(vec![]),
        })
    }

    pub(crate) fn requests(&self) -> Vec<Vec<Value>> {
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .clone()
    }
}

#[async_trait]
impl Llm for FakeLlm {
    async fn stream(&self, request: LLmRequest) -> anyhow::Result<EventStream> {
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .push(request.messages);

        let turn = self
            .turns
            .lock()
            .expect("turns lock poisoned")
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("fake llm has no more turns"))?;

        Ok(Box::pin(stream::iter(turn.into_iter().map(Ok))))
    }
}
