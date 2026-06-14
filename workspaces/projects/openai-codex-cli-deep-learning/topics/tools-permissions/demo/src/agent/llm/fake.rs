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

/// 测试用 LLM。
///
/// 每次 `stream` 消费一组预设事件，同时记录收到的 messages，
/// 用来验证 ReAct loop 是否把 tool observation 正确回灌给下一轮模型调用。
pub(crate) struct FakeLlm {
    turns: Mutex<VecDeque<Vec<StreamEvent>>>,
    requests: Mutex<Vec<Vec<Value>>>,
}

impl FakeLlm {
    /// 创建一个按 turn 回放的 fake model。
    pub(crate) fn new(turns: Vec<Vec<StreamEvent>>) -> Arc<Self> {
        Arc::new(Self {
            turns: Mutex::new(VecDeque::from(turns)),
            requests: Mutex::new(vec![]),
        })
    }

    /// 读取每一轮模型调用收到的 messages。
    pub(crate) fn requests(&self) -> Vec<Vec<Value>> {
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .clone()
    }
}

#[async_trait]
impl Llm for FakeLlm {
    /// 返回下一轮预设事件流。
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
