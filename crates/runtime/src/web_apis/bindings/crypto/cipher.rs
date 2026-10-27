use super::*;
pub(super) fn run(key: &Key, r: &Request) -> Result<Vec<u8>> {
    let encrypt = r.operation == "encrypt";
    if key.algorithm == "RSA-OAEP" {
        let native = if encrypt {
            require(
                key.kind() == "public",
                "InvalidAccessError",
                "Encryption requires a public key",
            )?;
            key.public()?
        } else {
            key.private()?
        };
        return native.oaep(encrypt, &key.hash, &r.label, &r.data);
    }
    let bytes = key.secret()?;
    if key.algorithm == "AES-KW" {
        require(
            ["wrapKey", "unwrapKey"].contains(&r.usage.as_str()),
            "InvalidAccessError",
            "AES-KW is only for key wrapping",
        )?;
        return native::cipher(4, encrypt, bytes, &[], &[], &r.data, 0);
    }
    if key.algorithm == "AES-GCM" {
        require(
            [32, 64, 96, 104, 112, 120, 128].contains(&r.tag_length) && !r.iv.is_empty(),
            "OperationError",
            "Invalid GCM parameters",
        )?;
        return native::cipher(1, encrypt, bytes, &r.iv, &r.aad, &r.data, r.tag_length / 8);
    }
    require(
        r.iv.len() == 16,
        "OperationError",
        "IV/counter must contain 16 bytes",
    )?;
    if key.algorithm == "AES-CTR" {
        require(
            (1..=128).contains(&r.length),
            "OperationError",
            "Invalid counter length",
        )?;
        let blocks = r.data.len().div_ceil(16) as u128;
        require(
            r.length == 128 || blocks <= 1u128 << r.length,
            "OperationError",
            "Counter would repeat",
        )?;
        let counter = u128::from_be_bytes(r.iv.as_slice().try_into().unwrap());
        let mask = if r.length == 128 {
            u128::MAX
        } else {
            (1u128 << r.length) - 1
        };
        let remaining = (mask - (counter & mask)).checked_add(1);
        let split = remaining
            .filter(|n| *n < blocks)
            .map(|n| n as usize * 16)
            .unwrap_or(r.data.len());
        let mut out = native::cipher(3, true, bytes, &r.iv, &[], &r.data[..split], 0)?;
        if split < r.data.len() {
            out.extend(native::cipher(
                3,
                true,
                bytes,
                &(counter & !mask).to_be_bytes(),
                &[],
                &r.data[split..],
                0,
            )?);
        }
        return Ok(out);
    }
    require(
        key.algorithm == "AES-CBC",
        "NotSupportedError",
        "Unsupported cipher",
    )?;
    native::cipher(2, encrypt, bytes, &r.iv, &[], &r.data, 0)
}
