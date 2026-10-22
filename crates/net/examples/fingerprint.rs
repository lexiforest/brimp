use std::env;

use brimp_net::{Client, Config};
use http::Request;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| "https://fp.impersonate.pro/api/http2".to_string());
    let impersonation_target =
        env::var("BRIMP_IMPERSONATION_TARGET").unwrap_or_else(|_| "chrome150".to_string());
    let client = Client::new(Config {
        impersonation_target,
        default_headers: true,
        ..Config::default()
    });
    let request = Request::builder().uri(url).body(None)?;
    let response = client.send_collect(request)?;
    print!("{}", String::from_utf8_lossy(&response.body));
    Ok(())
}
