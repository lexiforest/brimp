use brimp_cdp::{ServerConfig, parse_bind, start};

#[tokio::main]
async fn main() {
    let mut bind = "127.0.0.1:9222".to_owned();
    let mut allow_non_loopback = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => {
                bind = args
                    .next()
                    .unwrap_or_else(|| usage("--bind requires HOST:PORT"))
            }
            "--allow-non-loopback" => allow_non_loopback = true,
            "--help" | "-h" => usage(""),
            _ => usage(&format!("unknown argument: {arg}")),
        }
    }
    let bind = parse_bind(&bind).unwrap_or_else(|error| usage(&error));
    let server = start(ServerConfig {
        bind,
        allow_non_loopback,
    })
    .await
    .unwrap_or_else(|error| {
        eprintln!("brimp-cdp: {error}");
        std::process::exit(1);
    });
    println!("{}", server.browser_websocket_url());
    std::future::pending::<()>().await;
}

fn usage(error: &str) -> ! {
    if !error.is_empty() {
        eprintln!("brimp-cdp: {error}");
    }
    eprintln!("usage: brimp-cdp [--bind HOST:PORT] [--allow-non-loopback]");
    std::process::exit(if error.is_empty() { 0 } else { 2 });
}
