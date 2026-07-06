use crate::error::ClientError;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use serde::Deserialize;
use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug, Clone, Deserialize)]
pub struct StreamDelta {
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamChoice {
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallDelta {
    pub index: usize,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub call_type: Option<String>,
    #[serde(default)]
    pub function: Option<FunctionDelta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FunctionDelta {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arguments: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Content(String),
    ToolCallStart {
        index: usize,
        id: String,
        name: String,
    },
    ToolCallArguments {
        index: usize,
        arguments: String,
    },
    Done {
        finish_reason: Option<String>,
    },
}

pub struct StreamParser<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    inner: S,
    buffer: String,
    tool_calls: Vec<PartialToolCall>,
}

struct PartialToolCall {
    index: usize,
    id: String,
    name: String,
    arguments: String,
}

impl<S> StreamParser<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            buffer: String::new(),
            tool_calls: Vec::new(),
        }
    }

    fn parse_line(&mut self, line: &str) -> Option<Result<StreamEvent, ClientError>> {
        let line = line.trim();
        if line.is_empty() || !line.starts_with("data: ") {
            return None;
        }

        let data = &line[6..];
        if data == "[DONE]" {
            return Some(Ok(StreamEvent::Done {
                finish_reason: None,
            }));
        }

        let delta: StreamDelta = match serde_json::from_str(data) {
            Ok(d) => d,
            Err(_) => return None,
        };

        let choice = delta.choices.first()?;

        if let Some(content) = &choice.delta.content {
            if !content.is_empty() {
                return Some(Ok(StreamEvent::Content(content.clone())));
            }
        }

        if let Some(tool_calls) = &choice.delta.tool_calls {
            for tc in tool_calls {
                if let Some(id) = &tc.id {
                    self.tool_calls.push(PartialToolCall {
                        index: tc.index,
                        id: id.clone(),
                        name: String::new(),
                        arguments: String::new(),
                    });
                }

                if let Some(func) = &tc.function {
                    if let Some(name) = &func.name {
                        if let Some(partial) = self.tool_calls.iter_mut().find(|t| t.index == tc.index) {
                            partial.name = name.clone();
                            return Some(Ok(StreamEvent::ToolCallStart {
                                index: tc.index,
                                id: partial.id.clone(),
                                name: name.clone(),
                            }));
                        }
                    }
                    if let Some(args) = &func.arguments {
                        if let Some(partial) = self.tool_calls.iter_mut().find(|t| t.index == tc.index) {
                            partial.arguments.push_str(args);
                            return Some(Ok(StreamEvent::ToolCallArguments {
                                index: tc.index,
                                arguments: args.clone(),
                            }));
                        }
                    }
                }
            }
        }

        if let Some(reason) = &choice.finish_reason {
            return Some(Ok(StreamEvent::Done {
                finish_reason: Some(reason.clone()),
            }));
        }

        None
    }
}

impl<S> Stream for StreamParser<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<StreamEvent, ClientError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(event) = self.parse_next_event() {
                return Poll::Ready(Some(event));
            }

            match self.inner.poll_next_unpin(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    let chunk = String::from_utf8_lossy(&bytes);
                    self.buffer.push_str(&chunk);
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(ClientError::Http(e))));
                }
                Poll::Ready(None) => {
                    if !self.buffer.is_empty() {
                        let remaining = std::mem::take(&mut self.buffer);
                        for line in remaining.lines() {
                            if let Some(event) = self.parse_line(line) {
                                return Poll::Ready(Some(event));
                            }
                        }
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

impl<S> StreamParser<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    fn parse_next_event(&mut self) -> Option<Result<StreamEvent, ClientError>> {
        while let Some(newline_pos) = self.buffer.find('\n') {
            let line = self.buffer[..newline_pos].to_string();
            self.buffer = self.buffer[newline_pos + 1..].to_string();
            if let Some(event) = self.parse_line(&line) {
                return Some(event);
            }
        }
        None
    }
}
