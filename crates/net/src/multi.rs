use std::collections::HashMap;
use std::ptr;
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use tokio::sync::{mpsc as tokio_mpsc, oneshot};

use crate::body::Body;
use crate::error::Error;
use crate::ffi::{
    CURLM_CALL_MULTI_PERFORM, CURLM_OK, CURLMOPT_PIPELINING, CURLMSG_DONE, CURLPIPE_MULTIPLEX,
    CurlMulti, curl_multi_add_handle, curl_multi_cleanup, curl_multi_error, curl_multi_info_read,
    curl_multi_init, curl_multi_perform, curl_multi_poll, curl_multi_remove_handle,
    curl_multi_setopt, global_init,
};
use crate::transfer::CurlTransfer;

const MULTI_POLL_TIMEOUT_MS: i32 = 20;

/// Only curl_multi_wakeup may cross threads. The mutex prevents cleanup while
/// another thread wakes the coordinator; it never exposes an easy handle.
#[derive(Debug, Default)]
pub(crate) struct Wake(Mutex<usize>);
impl Wake {
    pub(crate) fn notify(&self) {
        let handle = self.0.lock().expect("curl wake lock");
        if *handle != 0 {
            unsafe {
                crate::ffi::curl_multi_wakeup(*handle as *mut CurlMulti);
            }
        }
    }
}

pub(crate) struct StreamingTransfer {
    pub(crate) transfer: Box<CurlTransfer>,
    pub(crate) wake: Arc<Wake>,
    pub(crate) response_sender: oneshot::Sender<Result<http::Response<Body>, Error>>,
    pub(crate) body_sender: tokio_mpsc::Sender<bytes::Bytes>,
    pub(crate) body_receiver: tokio_mpsc::Receiver<bytes::Bytes>,
    pub(crate) completion_sender: oneshot::Sender<Result<(), Error>>,
    pub(crate) completion_receiver: oneshot::Receiver<Result<(), Error>>,
}

enum Command {
    Submit(StreamingTransfer),
    Shutdown,
}

#[derive(Debug)]
pub(crate) struct MultiExecutor {
    sender: SyncSender<Command>,
    worker: Mutex<Option<JoinHandle<()>>>,
    pub(crate) wake: Arc<Wake>,
}

impl MultiExecutor {
    pub(crate) fn new(queue_capacity: usize) -> Self {
        let (sender, receiver) = mpsc::sync_channel(queue_capacity.max(1));
        let wake = Arc::new(Wake::default());
        let worker_wake = Arc::clone(&wake);
        let worker = std::thread::Builder::new()
            .name("brimp-curl-multi".to_string())
            .spawn(move || run(receiver, worker_wake))
            .expect("failed to start brimp curl multi worker");
        Self {
            sender,
            worker: Mutex::new(Some(worker)),
            wake,
        }
    }

    pub(crate) fn submit(&self, transfer: StreamingTransfer) -> Result<(), Error> {
        self.sender
            .try_send(Command::Submit(transfer))
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => {
                    Error::Network("curl submission queue is full".into())
                }
                mpsc::TrySendError::Disconnected(_) => {
                    Error::Initialization("curl multi worker exited".into())
                }
            })?;
        self.wake.notify();
        Ok(())
    }
}

fn run(receiver: Receiver<Command>, wake: Arc<Wake>) {
    global_init();
    let multi = unsafe { curl_multi_init() };
    if multi.is_null() {
        reject_commands(receiver, "failed to initialize curl multi handle");
        return;
    }

    *wake.0.lock().expect("curl wake lock") = multi as usize;
    unsafe {
        let _ = curl_multi_setopt(multi, CURLMOPT_PIPELINING, CURLPIPE_MULTIPLEX);
    }

    let mut active = HashMap::<usize, Box<CurlTransfer>>::new();
    let mut disconnected = false;
    loop {
        if active.is_empty() {
            match receiver.recv() {
                Ok(Command::Shutdown) => break,
                Ok(command) => add_command(multi, command, &mut active),
                Err(_) => break,
            }
        }

        loop {
            match receiver.try_recv() {
                Ok(Command::Shutdown) => {
                    disconnected = true;
                    break;
                }
                Ok(command) => add_command(multi, command, &mut active),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    disconnected = true;
                    break;
                }
            }
        }

        if disconnected {
            fail_all(multi, &mut active, Error::Network("client closed".into()));
            break;
        }
        service_consumers(multi, &mut active);
        drive(multi, &mut active);
        if !active.is_empty() {
            let mut ready = 0;
            unsafe {
                let _ =
                    curl_multi_poll(multi, ptr::null_mut(), 0, MULTI_POLL_TIMEOUT_MS, &mut ready);
            }
        }
    }

    let mut handle = wake.0.lock().expect("curl wake lock");
    *handle = 0;
    unsafe {
        let _ = curl_multi_cleanup(multi);
    }
}

fn add_command(
    multi: *mut CurlMulti,
    command: Command,
    active: &mut HashMap<usize, Box<CurlTransfer>>,
) {
    let Command::Submit(mut streaming) = command else {
        return;
    };
    if let Err(error) = streaming.transfer.prepare_streaming(
        streaming.response_sender,
        streaming.body_sender,
        streaming.body_receiver,
        streaming.completion_sender,
        streaming.completion_receiver,
        streaming.wake,
    ) {
        streaming.transfer.finish_streaming(Err(error));
        return;
    }
    let handle = streaming.transfer.handle();
    let result = unsafe { curl_multi_add_handle(multi, handle) };
    if result != CURLM_OK {
        streaming
            .transfer
            .finish_streaming(Err(Error::Network(curl_multi_error(result))));
        return;
    }
    active.insert(handle as usize, streaming.transfer);
}

fn drive(multi: *mut CurlMulti, active: &mut HashMap<usize, Box<CurlTransfer>>) {
    let mut running = 0;
    loop {
        let result = unsafe { curl_multi_perform(multi, &mut running) };
        if result == CURLM_CALL_MULTI_PERFORM {
            continue;
        }
        if result != CURLM_OK {
            let error = Error::Network(curl_multi_error(result));
            fail_all(multi, active, error);
        }
        break;
    }

    loop {
        let mut queued = 0;
        let message = unsafe { curl_multi_info_read(multi, &mut queued) };
        if message.is_null() {
            break;
        }
        let message = unsafe { &*message };
        if message.message != CURLMSG_DONE {
            continue;
        }
        let handle = message.easy_handle;
        let result = unsafe { message.data.result };
        unsafe {
            let _ = curl_multi_remove_handle(multi, handle);
        }
        if let Some(mut transfer) = active.remove(&(handle as usize)) {
            transfer.finish_streaming(if result == 0 {
                Ok(())
            } else {
                Err(Error::Network(crate::ffi::curl_error(handle, result)))
            });
        }
    }
}

fn fail_all(multi: *mut CurlMulti, active: &mut HashMap<usize, Box<CurlTransfer>>, error: Error) {
    let message = error.to_string();
    for (_, mut transfer) in active.drain() {
        unsafe {
            let _ = curl_multi_remove_handle(multi, transfer.handle());
        }
        transfer.finish_streaming(Err(Error::Network(message.clone())));
    }
}

fn reject_commands(receiver: Receiver<Command>, message: &str) {
    while let Ok(Command::Submit(streaming)) = receiver.recv() {
        let _ = streaming
            .response_sender
            .send(Err(Error::Initialization(message.to_string())));
    }
}

impl Drop for MultiExecutor {
    fn drop(&mut self) {
        let _ = self.sender.send(Command::Shutdown);
        self.wake.notify();
        if let Some(worker) = self.worker.get_mut().expect("curl worker lock").take() {
            let _ = worker.join();
        }
    }
}

fn service_consumers(multi: *mut CurlMulti, active: &mut HashMap<usize, Box<CurlTransfer>>) {
    let cancelled: Vec<_> = active
        .iter()
        .filter(|(_, t)| t.is_cancelled())
        .map(|(id, _)| *id)
        .collect();
    for id in cancelled {
        if let Some(mut transfer) = active.remove(&id) {
            unsafe {
                curl_multi_remove_handle(multi, transfer.handle());
            }
            transfer.finish_streaming(Err(Error::Network("request cancelled".into())));
        }
    }
    // Collect raw handles first. Unpausing can invoke callbacks synchronously;
    // do not retain a Rust reference to their transfer state across that call.
    let ready: Vec<_> = active
        .values_mut()
        .filter_map(|t| t.resume_handle())
        .collect();
    for handle in ready {
        let result = unsafe { crate::ffi::curl_easy_pause(handle, crate::ffi::CURLPAUSE_CONT) };
        if result != crate::ffi::CURLE_OK {
            unsafe {
                curl_multi_remove_handle(multi, handle);
            }
            if let Some(mut transfer) = active.remove(&(handle as usize)) {
                transfer
                    .finish_streaming(Err(Error::Network(crate::ffi::curl_error(handle, result))));
            }
        }
    }
}
