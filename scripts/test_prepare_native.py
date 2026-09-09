import tempfile
from pathlib import Path

import prepare_native
import pytest


def test_unpinned_digest_fails_before_download():
    asset = prepare_native.Asset("test.tar.gz", "PENDING", "https://invalid")
    with tempfile.TemporaryDirectory() as temporary:
        with pytest.raises(RuntimeError, match="does not have a pinned SHA-256"):
            prepare_native.download(asset, Path(temporary))


def test_all_release_digests_are_pinned():
    for target in prepare_native.TARGETS.values():
        assert len(target.jsc.sha256) == 64
        assert len(target.curl.sha256) == 64
