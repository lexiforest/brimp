"""Editable transport fingerprint data, matching curl_cffi's Fingerprint schema."""

from dataclasses import asdict, dataclass, field


@dataclass
class Fingerprint:
    """Editable TLS, HTTP/2, HTTP/3, QUIC and WebSocket fingerprint data.

    Pass directly as `Session(profile=...)` or place it at
    `profile["values"]["network"]["fingerprint"]`. Use the CLI to manage Pro catalogs.
    """

    client: str = ""
    client_version: str = ""
    os: str = ""
    os_version: str = ""

    # Preferred HTTP version when applied as a Brimp profile.
    http_version: str = "v2"

    tls_version: str = "1.2"
    tls_ciphers: list[str] = field(default_factory=list)
    # the value is [h2, http/1.1], which will be filled by libcurl
    tls_alpn: bool = True
    tls_alps: bool = False
    tls_cert_compression: list[str] = field(default_factory=list)
    tls_signature_hashes: list[str] = field(default_factory=list)  # .sig_hash_args
    tls_key_shares_limit: int = 2  # the default key shares length is 2
    tls_supported_groups: list[str] = field(default_factory=list)  # .curves
    tls_session_ticket: bool = False
    tls_extension_order: str = ""
    tls_delegated_credentials: list[str] = field(default_factory=list)
    tls_record_size_limit: int | None = None
    tls_grease: bool = False
    tls_use_new_alps_codepoint: bool = False
    tls_signed_cert_timestamps: bool = False
    tls_ech: str | None = None
    tls_permute_extensions: bool = False
    tls_trust_anchors: list[str] | None = None

    headers: dict[str, str] = field(default_factory=dict)
    header_order: str = ""
    split_cookies: bool = False
    form_boundary: str = ""

    http2_settings: str = ""
    http2_window_update: int = 0
    http2_pseudo_headers_order: str = ""
    http2_stream_weight: int | None = None
    http2_stream_exclusive: int | None = None
    http2_no_priority: bool = False

    http3_settings: str = ""
    http3_pseudo_headers_order: str = ""
    http3_tls_extension_order: str = ""
    http3_headers: dict[str, str] = field(default_factory=dict)
    http3_header_order: str = ""
    http3_tls_supported_groups: list[str] = field(default_factory=list)
    quic_transport_parameters: str = ""
    quic_cid_length: str | None = None
    quic_initial_packet_number: int | None = None

    ws_headers: dict[str, str] = field(default_factory=dict)
    ws_header_order: str = ""
    ws_disable_session_ticket: bool = False
    ws_tls_cert_compression: list[str] | None = None

    header_lang: str = ""


def profile_json(value):
    if isinstance(value, Fingerprint):
        return asdict(value)
    raise TypeError(f"{type(value).__name__} is not JSON serializable")
