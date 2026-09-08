#!/bin/sh
set -eu

expected_version=0.19.3
expected_bundle_sha256=50ac3cec17c11139833a05cf0f61a812f89c06019c3af190284b49c093091294
vendor_dir=$(CDPATH= cd -- "$(dirname -- "$0")/$expected_version" && pwd)

if command -v sha256sum >/dev/null 2>&1; then
    actual_bundle_sha256=$(sha256sum "$vendor_dir/index.full.js" | awk '{print $1}')
else
    actual_bundle_sha256=$(shasum -a 256 "$vendor_dir/index.full.js" | awk '{print $1}')
fi

if [ "$actual_bundle_sha256" != "$expected_bundle_sha256" ]; then
    echo "Defuddle bundle checksum mismatch" >&2
    echo "expected: $expected_bundle_sha256" >&2
    echo "actual:   $actual_bundle_sha256" >&2
    exit 1
fi

for license in \
    defuddle-LICENSE \
    mathml-to-latex-LICENSE \
    temml-LICENSE \
    turndown-LICENSE \
    xmldom-LICENSE
do
    test -s "$vendor_dir/licenses/$license" || {
        echo "missing Defuddle bundle notice: $license" >&2
        exit 1
    }
done

echo "Defuddle $expected_version bundle and notices verified"
