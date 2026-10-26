use encoding_rs::{Encoding, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252, X_USER_DEFINED};
use std::borrow::Cow;

/// Decode HTML before parser scripts run. The document retains this canonical
/// encoding; DOM mutations and public JS properties cannot change it.
pub(crate) fn decode_html<'a>(
    bytes: &'a [u8],
    content_type: Option<&str>,
) -> Result<(Cow<'a, str>, &'static Encoding), brimp_lexbor::Error> {
    let encoding = if let Some((encoding, _)) = Encoding::for_bom(bytes) {
        encoding
    } else if let Some(encoding) = content_type
        .and_then(|value| value.parse::<data_url::mime::Mime>().ok())
        .and_then(|mime| {
            mime.get_parameter("charset")
                .and_then(|label| Encoding::for_label(label.as_bytes()))
        })
    {
        encoding
    } else if let Some(label) = brimp_lexbor::html_meta_charset(bytes, |label| {
        Encoding::for_label(label.as_bytes()).is_some()
    })? {
        let encoding = Encoding::for_label(label.as_bytes()).expect("validated HTML encoding");
        if encoding == UTF_16LE || encoding == UTF_16BE {
            UTF_8
        } else if encoding == X_USER_DEFINED {
            WINDOWS_1252
        } else {
            encoding
        }
    } else {
        // Installed Safari's default for unlabelled top-level HTTP HTML.
        WINDOWS_1252
    };
    Ok((encoding.decode_with_bom_removal(bytes).0, encoding))
}
