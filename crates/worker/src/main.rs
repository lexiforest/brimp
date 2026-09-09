#[cfg(unix)]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    use std::os::fd::{FromRawFd, OwnedFd};
    use std::os::unix::net::UnixStream;

    let mut socket_fd = None;
    let mut window_size = None;
    for argument in std::env::args().skip(1) {
        if argument == "--headless" {
            continue;
        }
        if let Some(value) = argument.strip_prefix("--controller-socket-fd=") {
            if socket_fd.is_some() {
                fail("--controller-socket-fd may be supplied only once");
            }
            let descriptor = value
                .parse::<i32>()
                .unwrap_or_else(|_| fail("invalid controller socket descriptor"));
            if descriptor <= 2 {
                fail("controller socket descriptor must not be stdin, stdout, or stderr");
            }
            socket_fd = Some(descriptor);
            continue;
        }
        if let Some(value) = argument.strip_prefix("--window-size=") {
            window_size = Some(parse_window_size(value).unwrap_or_else(|message| fail(message)));
            continue;
        }
        fail(&format!("unknown argument: {argument}"));
    }
    let descriptor = socket_fd.unwrap_or_else(|| fail("--controller-socket-fd=N is required"));
    // SAFETY: the controller transfers this inherited descriptor to the worker. This is
    // the single ownership conversion in the process, after duplicate arguments are rejected.
    let descriptor = unsafe { OwnedFd::from_raw_fd(descriptor) };
    let stream = UnixStream::from(descriptor);
    stream
        .set_nonblocking(true)
        .unwrap_or_else(|error| fail(&format!("could not configure controller socket: {error}")));
    let stream = tokio::net::UnixStream::from_std(stream)
        .unwrap_or_else(|error| fail(&format!("could not open controller socket: {error}")));
    let mut options = web_runtime::PageOptions::builder();
    if let Some((width, height)) = window_size {
        options = options.viewport(width, height);
    }
    if let Err(error) = lite_worker::serve_framed(stream, options.build()).await {
        fail(&error.to_string());
    }
}

#[cfg(not(unix))]
fn main() {
    fail("lite-worker currently requires the Unix inherited-socket transport");
}

fn parse_window_size(value: &str) -> Result<(u32, u32), &'static str> {
    let (width, height) = value
        .split_once(',')
        .ok_or("window size must be WIDTH,HEIGHT")?;
    let width = width
        .parse()
        .map_err(|_| "window width must be a positive integer")?;
    let height = height
        .parse()
        .map_err(|_| "window height must be a positive integer")?;
    if width == 0 || height == 0 {
        return Err("window dimensions must be positive");
    }
    Ok((width, height))
}

fn fail(message: &str) -> ! {
    eprintln!("lite-worker: {message}");
    std::process::exit(2)
}

#[cfg(test)]
mod tests {
    use super::parse_window_size;

    #[test]
    fn validates_window_size() {
        assert_eq!(parse_window_size("640,480"), Ok((640, 480)));
        assert!(parse_window_size("0,480").is_err());
        assert!(parse_window_size("640x480").is_err());
    }
}
