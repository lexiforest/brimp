//! Redirect/cookie policy for realm Fetch and EventSource streams.
use super::{RedirectOptions, redirect_request, refresh_cookie_header, supplied_cookie};
use crate::{network::*, web_apis::BrowsingContext};
use std::sync::{Arc, Mutex};
struct RedirectStream {
    loader: Arc<dyn ResourceLoader>,
    context: Arc<BrowsingContext>,
    options: RedirectOptions,
    callback: Mutex<ResourceStreamCallback>,
    control: ResourceStreamHandle,
    generation: Mutex<usize>,
}
pub(crate) fn fetch_stream_callback(
    loader: Arc<dyn ResourceLoader>,
    context: Arc<BrowsingContext>,
    mut request: ResourceRequest,
    include_credentials: bool,
    callback: ResourceStreamCallback,
) -> Result<ResourceStreamHandle, NetworkError> {
    if include_credentials {
        context.apply_request_identity(&mut request.headers);
    }
    let supplied = supplied_cookie(&request);
    let control = ResourceStreamHandle::proxy();
    let stream = Arc::new(RedirectStream {
        loader,
        context,
        options: RedirectOptions {
            include_credentials,
            ..Default::default()
        },
        callback: Mutex::new(callback),
        control: control.clone(),
        generation: Mutex::new(0),
    });
    stream.start(request, supplied, 0)?;
    Ok(control)
}
impl RedirectStream {
    fn start(
        self: &Arc<Self>,
        mut request: ResourceRequest,
        mut supplied: Option<String>,
        generation: usize,
    ) -> Result<(), NetworkError> {
        if self.options.include_credentials {
            refresh_cookie_header(&self.context, &mut request, supplied.as_deref())?;
        }
        let state = self.clone();
        let original = request.clone();
        let handle = self.loader.fetch_stream_callback(
            request,
            Box::new(move |event, control| {
                if state.control.is_cancelled() || *state.generation.lock().unwrap() != generation {
                    return ResourceStreamDirective::Cancel;
                }
                if let ResourceStreamEvent::Headers {
                    status, headers, ..
                } = &event
                {
                    if state.options.include_credentials {
                        for cookie in headers.get_all(http::header::SET_COOKIE) {
                            if let Ok(cookie) = cookie.to_str() {
                                state.context.store_response_cookie(&original.url, cookie);
                            }
                        }
                    }
                    let mut next = original.clone();
                    match redirect_request(&mut next, status.as_u16(), headers, &mut supplied) {
                        Ok(true) => {
                            if generation >= state.options.limit {
                                state.callback.lock().unwrap()(
                                    ResourceStreamEvent::Error(NetworkError::Transport(
                                        "redirect limit exceeded".into(),
                                    )),
                                    control,
                                );
                                return ResourceStreamDirective::Cancel;
                            }
                            *state.generation.lock().unwrap() = generation + 1;
                            if let Err(error) = state.start(next, supplied.clone(), generation + 1)
                            {
                                state.callback.lock().unwrap()(
                                    ResourceStreamEvent::Error(error),
                                    control,
                                );
                            }
                            return ResourceStreamDirective::Redirect;
                        }
                        Err(error) => {
                            state.callback.lock().unwrap()(
                                ResourceStreamEvent::Error(error),
                                control,
                            );
                            return ResourceStreamDirective::Cancel;
                        }
                        Ok(false) => {}
                    }
                }
                state.callback.lock().unwrap()(event, control)
            }),
        )?;
        let current = self.generation.lock().unwrap();
        if *current == generation {
            self.control.attach(handle);
        } else {
            handle.cancel();
        }
        Ok(())
    }
}
