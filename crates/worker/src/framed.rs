use std::collections::VecDeque;
use std::io;
use std::sync::Arc;

use brimp_runtime::{AutomationBrowser, PageOptions};
use serde::Serialize;
use serde_json::Value;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, WriteHalf};
use tokio::sync::mpsc;

use crate::dispatch::ConnectionState;
use crate::interception::{InterceptionRegistry, PausedRequest};
use crate::protocol::{ProtocolError, Request, Response};

pub const MAX_FRAME_SIZE: usize = 64 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum FramedError {
    #[error("failed to create browser: {0}")]
    Browser(String),
    #[error("framed CDP I/O failure: {0}")]
    Io(#[from] io::Error),
    #[error("framed CDP message exceeds the 64 MiB limit")]
    Oversized,
    #[error("framed CDP message is not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("framed CDP message must be a JSON object")]
    NotObject,
}

pub async fn serve_framed<S>(stream: S, page_options: PageOptions) -> Result<(), FramedError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let browser = Arc::new(
        AutomationBrowser::new().map_err(|error| FramedError::Browser(error.to_string()))?,
    );
    serve_framed_with_browser(stream, browser, page_options).await
}

pub async fn serve_framed_with_browser<S>(
    stream: S,
    browser: Arc<AutomationBrowser>,
    page_options: PageOptions,
) -> Result<(), FramedError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let (mut stream_reader, mut writer) = tokio::io::split(stream);
    let (incoming_tx, mut reader) = mpsc::channel(64);
    let read_task = tokio::spawn(async move {
        loop {
            let frame = read_frame(&mut stream_reader).await;
            let finished = !matches!(frame, Ok(Some(_)));
            if incoming_tx.send(frame).await.is_err() || finished {
                break;
            }
        }
    });
    let _read_guard = AbortReader(read_task);
    let (interception, mut paused_requests) = InterceptionRegistry::new();
    let mut state = ConnectionState::new(browser, page_options, interception.clone());
    let mut queued_requests = VecDeque::new();
    let mut navigations = tokio::task::JoinSet::new();
    let mut navigating_sessions = std::collections::HashSet::new();

    loop {
        let value = if let Some(value) = queued_requests.pop_front() {
            value
        } else {
            tokio::select! {
                completion = navigations.join_next(), if !navigations.is_empty() => {
                    let (request, navigation): (Request, crate::dispatch::NavigationCompletion) = completion.unwrap().map_err(|e| FramedError::Browser(e.to_string()))?;
                    navigating_sessions.remove(&request.session_id);
                    let response = state.complete_navigation(&request, navigation);
                    write_frame(&mut writer, &response).await?;
                    write_events(&mut writer, state.take_events().collect()).await?;
                    continue;
                }
                Some(paused) = paused_requests.recv() => { write_frame(&mut writer, &paused.event()).await?; continue; }
                incoming = reader.recv() => {
                    match incoming { Some(Ok(Some(value))) => value, Some(Err(error)) => return Err(error), _ => break }
                }
            }
        };
        let request = match serde_json::from_value::<Request>(value) {
            Ok(request) => request,
            Err(error) => {
                write_frame(&mut writer, &parse_error(error)).await?;
                continue;
            }
        };
        if request.method == "Page.navigate" {
            if navigating_sessions.contains(&request.session_id) {
                write_frame(
                    &mut writer,
                    &Response::error(
                        &request,
                        -32600,
                        "navigation already in progress for this session",
                    ),
                )
                .await?;
                continue;
            }
            match state.start_navigation(&request) {
                Ok(job) => {
                    navigating_sessions.insert(request.session_id.clone());
                    navigations.spawn(async move { (request, job.complete().await) });
                }
                Err(response) => write_frame(&mut writer, &response).await?,
            }
            continue;
        }
        let events_before_response = matches!(
            request.method.as_str(),
            "Target.attachToTarget" | "Target.attachToBrowserTarget" | "Target.createTarget"
        );
        let Some(response) = dispatch_while_reading(
            &mut state,
            &request,
            &interception,
            &mut paused_requests,
            &mut reader,
            &mut writer,
            &mut queued_requests,
        )
        .await?
        else {
            break;
        };
        if events_before_response && response.error.is_none() {
            write_events(&mut writer, state.take_events().collect()).await?;
        }
        write_frame(&mut writer, &response).await?;
        write_events(&mut writer, state.take_events().collect()).await?;
    }

    interception.shutdown();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn dispatch_while_reading<R>(
    state: &mut ConnectionState,
    request: &Request,
    interception: &InterceptionRegistry,
    paused_requests: &mut mpsc::UnboundedReceiver<PausedRequest>,
    reader: &mut mpsc::Receiver<Result<Option<Value>, FramedError>>,
    writer: &mut WriteHalf<R>,
    queued_requests: &mut VecDeque<Value>,
) -> Result<Option<Response>, FramedError>
where
    R: AsyncRead + AsyncWrite + Unpin,
{
    let dispatch = state.dispatch(request);
    tokio::pin!(dispatch);
    loop {
        tokio::select! {
            response = &mut dispatch => return Ok(Some(response)),
            Some(paused) = paused_requests.recv() => write_frame(writer, &paused.event()).await?,
            incoming = reader.recv() => {
                let value = match incoming { Some(Ok(Some(value))) => value, Some(Err(error)) => return Err(error), _ => return Ok(None) };
                match serde_json::from_value::<Request>(value.clone()) {
                    Ok(request) => {
                        if let Some(response) = interception.handle_control(&request) {
                            write_frame(writer, &response).await?;
                        } else {
                            interception.prepare_queued_command(&request);
                            queued_requests.push_back(value);
                        }
                    }
                    Err(error) => write_frame(writer, &parse_error(error)).await?,
                }
            }
        }
    }
}

async fn write_events<W>(
    writer: &mut W,
    events: Vec<crate::protocol::Event>,
) -> Result<(), FramedError>
where
    W: AsyncWrite + Unpin,
{
    for event in events {
        write_frame(writer, &event).await?;
    }
    Ok(())
}

async fn read_frame<R>(reader: &mut R) -> Result<Option<Value>, FramedError>
where
    R: AsyncRead + Unpin,
{
    let mut header = [0; 4];
    if reader.read(&mut header[..1]).await? == 0 {
        return Ok(None);
    }
    reader.read_exact(&mut header[1..]).await?;
    let length = u32::from_be_bytes(header) as usize;
    if length > MAX_FRAME_SIZE {
        return Err(FramedError::Oversized);
    }
    let mut payload = vec![0; length];
    reader.read_exact(&mut payload).await?;
    let value: Value = serde_json::from_slice(&payload)?;
    if !value.is_object() {
        return Err(FramedError::NotObject);
    }
    Ok(Some(value))
}

async fn write_frame<W, T>(writer: &mut W, value: &T) -> Result<(), FramedError>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let payload = serde_json::to_vec(value)?;
    if payload.len() > MAX_FRAME_SIZE {
        return Err(FramedError::Oversized);
    }
    writer
        .write_all(&(payload.len() as u32).to_be_bytes())
        .await?;
    writer.write_all(&payload).await?;
    writer.flush().await?;
    Ok(())
}

fn parse_error(error: serde_json::Error) -> Response {
    Response {
        id: 0,
        result: None,
        error: Some(ProtocolError {
            data: None,
            code: -32700,
            message: format!("Parse error: {error}"),
        }),
        session_id: None,
    }
}

struct AbortReader(tokio::task::JoinHandle<()>);
impl Drop for AbortReader {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use brimp_network::{NetworkError, ResourceLoader, ResourceRequest, ResourceResponse};
    use brimp_runtime::AutomationBrowser;
    use serde_json::json;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    struct DataLoader;

    #[async_trait]
    impl ResourceLoader for DataLoader {
        async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
            Err(NetworkError::InvalidRequest(request.url))
        }
    }

    #[tokio::test]
    async fn framed_transport_runs_the_existing_cdp_dispatcher() {
        let (mut client, server) = tokio::io::duplex(4096);
        let browser = Arc::new(AutomationBrowser::with_resource_loader(Arc::new(
            DataLoader,
        )));
        let task = tokio::spawn(serve_framed_with_browser(
            server,
            browser,
            PageOptions::default(),
        ));
        send(
            &mut client,
            json!({"id":7,"method":"Browser.getVersion","params":{}}),
        )
        .await;
        let response = receive(&mut client).await;
        assert_eq!(response["id"], 7);
        assert_eq!(response["result"]["product"], "Brimp/0.1.0");
        drop(client);
        task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn framed_transport_enforces_its_size_and_object_bounds() {
        let (mut client, server) = tokio::io::duplex(16);
        let task = tokio::spawn(async move {
            let (mut reader, _) = tokio::io::split(server);
            read_frame(&mut reader).await
        });
        client
            .write_all(&((MAX_FRAME_SIZE as u32) + 1).to_be_bytes())
            .await
            .unwrap();
        assert!(matches!(task.await.unwrap(), Err(FramedError::Oversized)));

        let (mut client, server) = tokio::io::duplex(16);
        let task = tokio::spawn(async move {
            let (mut reader, _) = tokio::io::split(server);
            read_frame(&mut reader).await
        });
        client.write_all(&2_u32.to_be_bytes()).await.unwrap();
        client.write_all(b"[]").await.unwrap();
        assert!(matches!(task.await.unwrap(), Err(FramedError::NotObject)));
    }

    #[tokio::test]
    async fn framed_transport_distinguishes_clean_eof_from_corrupt_frames() {
        let (client, server) = tokio::io::duplex(16);
        drop(client);
        let (mut reader, _) = tokio::io::split(server);
        assert_eq!(read_frame(&mut reader).await.unwrap(), None);

        let (mut client, server) = tokio::io::duplex(16);
        let task = tokio::spawn(async move {
            let (mut reader, _) = tokio::io::split(server);
            read_frame(&mut reader).await
        });
        client.write_all(&3_u32.to_be_bytes()).await.unwrap();
        client.write_all(b"{").await.unwrap();
        drop(client);
        assert!(matches!(task.await.unwrap(), Err(FramedError::Io(_))));

        let (mut client, server) = tokio::io::duplex(16);
        let task = tokio::spawn(async move {
            let (mut reader, _) = tokio::io::split(server);
            read_frame(&mut reader).await
        });
        client.write_all(&1_u32.to_be_bytes()).await.unwrap();
        client.write_all(b"{").await.unwrap();
        assert!(matches!(task.await.unwrap(), Err(FramedError::Json(_))));
    }

    struct ConcurrentLoader(tokio::sync::Barrier);
    #[async_trait]
    impl ResourceLoader for ConcurrentLoader {
        async fn fetch(&self, request: ResourceRequest) -> Result<ResourceResponse, NetworkError> {
            tokio::time::timeout(std::time::Duration::from_secs(3), self.0.wait())
                .await
                .map_err(|_| {
                    NetworkError::InvalidRequest("page navigations were serialized".into())
                })?;
            Ok(ResourceResponse {
                status: http::StatusCode::OK,
                headers: {
                    let mut headers = brimp_network::HeaderList::new();
                    headers.append("content-type", http::HeaderValue::from_static("text/html"));
                    headers
                },
                body: b"<title>Concurrent</title>".to_vec(),
                effective_url: request.url,
                metadata: Default::default(),
            })
        }
    }

    #[tokio::test]
    async fn concurrent_navigations_preserve_a_partially_read_next_frame() {
        let (mut client, server) = tokio::io::duplex(65536);
        let browser = Arc::new(AutomationBrowser::with_resource_loader(Arc::new(
            ConcurrentLoader(tokio::sync::Barrier::new(2)),
        )));
        let task = tokio::spawn(serve_framed_with_browser(
            server,
            browser,
            PageOptions::default(),
        ));
        let mut sessions = Vec::new();
        for index in 0..2 {
            let id = index * 3;
            send(
                &mut client,
                json!({"id":id,"method":"Target.createTarget","params":{"url":"about:blank"}}),
            )
            .await;
            let target = response(&mut client, id).await["result"]["targetId"].clone();
            send(&mut client, json!({"id":id+1,"method":"Target.attachToTarget","params":{"targetId":target,"flatten":true}})).await;
            let session = response(&mut client, id + 1).await["result"]["sessionId"].clone();
            send(
                &mut client,
                json!({"id":id+2,"method":"Page.enable","sessionId":session,"params":{}}),
            )
            .await;
            let enabled = response(&mut client, id + 2).await;
            assert!(enabled.get("error").is_none(), "{enabled}");
            sessions.push(session);
        }
        for (index, session) in sessions.into_iter().enumerate() {
            send(&mut client, json!({"id":10+index,"method":"Page.navigate","sessionId":session,"params":{"url":format!("https://fixture.test/{index}")}})).await;
        }
        let next = serde_json::to_vec(&json!({"id":20,"method":"Browser.getVersion","params":{}}))
            .unwrap();
        let header = (next.len() as u32).to_be_bytes();
        client.write_all(&header[..2]).await.unwrap();
        let mut responses = 0;
        while responses < 2 {
            let event =
                tokio::time::timeout(std::time::Duration::from_secs(5), receive(&mut client))
                    .await
                    .unwrap();
            if event.get("id").is_some() {
                assert!(event.get("error").is_none(), "{event}");
                assert!(event["id"] == 10 || event["id"] == 11);
                responses += 1;
            }
        }
        client.write_all(&header[2..]).await.unwrap();
        client.write_all(&next).await.unwrap();
        loop {
            let event = receive(&mut client).await;
            if event.get("id").is_some() {
                assert_eq!(event["id"], 20);
                assert_eq!(event["result"]["protocolVersion"], "1.3");
                break;
            }
        }
        drop(client);
        task.await.unwrap().unwrap();
    }

    async fn response(stream: &mut tokio::io::DuplexStream, id: u64) -> Value {
        loop {
            let value = receive(stream).await;
            if value["id"] == id {
                return value;
            }
        }
    }

    async fn send(stream: &mut tokio::io::DuplexStream, value: Value) {
        let payload = serde_json::to_vec(&value).unwrap();
        stream
            .write_all(&(payload.len() as u32).to_be_bytes())
            .await
            .unwrap();
        stream.write_all(&payload).await.unwrap();
    }

    async fn receive(stream: &mut tokio::io::DuplexStream) -> Value {
        let length = stream.read_u32().await.unwrap() as usize;
        let mut payload = vec![0; length];
        stream.read_exact(&mut payload).await.unwrap();
        serde_json::from_slice(&payload).unwrap()
    }
}
