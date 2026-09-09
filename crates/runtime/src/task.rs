use std::{
    collections::VecDeque,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::Page;

type PageTask = Box<dyn FnOnce(&mut Page) + Send>;

pub struct TaskQueue {
    pending: VecDeque<PageTask>,
    sender: Sender<PageTask>,
    receiver: Receiver<PageTask>,
}

#[derive(Clone)]
pub struct TaskSender {
    sender: Sender<PageTask>,
}

impl Default for TaskQueue {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            pending: VecDeque::new(),
            sender,
            receiver,
        }
    }
}

impl TaskQueue {
    pub fn push<F>(&mut self, task: F)
    where
        F: FnOnce(&mut Page) + Send + 'static,
    {
        self.pending.push_back(Box::new(task));
    }

    pub(crate) fn pop(&mut self) -> Option<PageTask> {
        self.pending
            .pop_front()
            .or_else(|| self.receiver.try_recv().ok())
    }

    pub fn sender(&self) -> TaskSender {
        TaskSender {
            sender: self.sender.clone(),
        }
    }

    pub fn is_empty(&mut self) -> bool {
        if self.pending.is_empty()
            && let Ok(task) = self.receiver.try_recv()
        {
            self.pending.push_back(task);
        }
        self.pending.is_empty()
    }
}

impl TaskSender {
    pub fn post<F>(&self, task: F) -> Result<(), TaskSendError>
    where
        F: FnOnce(&mut Page) + Send + 'static,
    {
        self.sender.send(Box::new(task)).map_err(|_| TaskSendError)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskSendError;

impl std::fmt::Display for TaskSendError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("page task queue is no longer available")
    }
}

impl std::error::Error for TaskSendError {}
