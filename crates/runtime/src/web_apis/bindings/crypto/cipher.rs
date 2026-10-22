use super::*;
use openssl::{
    aes,
    encrypt::{Decrypter, Encrypter},
    symm::{self, Cipher},
};
pub(super) fn run(key: &Key, r: &Request) -> Result<Vec<u8>> {
    let encrypt = r.operation == "encrypt";
    if key.algorithm == "RSA-OAEP" {
        let hash = digest(&key.hash)?;
        if encrypt {
            require(
                key.kind() == "public",
                "InvalidAccessError",
                "Encryption requires a public key",
            )?;
            let p = key.public()?;
            let mut c = Encrypter::new(&p)?;
            c.set_rsa_padding(Padding::PKCS1_OAEP)?;
            c.set_rsa_oaep_md(hash)?;
            c.set_rsa_mgf1_md(hash)?;
            if !r.label.is_empty() {
                c.set_rsa_oaep_label(&r.label)?;
            }
            let mut out = vec![0; c.encrypt_len(&r.data)?];
            let n = c.encrypt(&r.data, &mut out)?;
            out.truncate(n);
            return Ok(out);
        }
        let mut c = Decrypter::new(key.private()?)?;
        c.set_rsa_padding(Padding::PKCS1_OAEP)?;
        c.set_rsa_oaep_md(hash)?;
        c.set_rsa_mgf1_md(hash)?;
        if !r.label.is_empty() {
            c.set_rsa_oaep_label(&r.label)?;
        }
        let mut out = vec![0; c.decrypt_len(&r.data)?];
        let n = c.decrypt(&r.data, &mut out)?;
        out.truncate(n);
        return Ok(out);
    }
    let bytes = key.secret()?;
    if key.algorithm == "AES-KW" {
        require(
            ["wrapKey", "unwrapKey"].contains(&r.usage.as_str()),
            "InvalidAccessError",
            "AES-KW is only for key wrapping",
        )?;
        require(
            r.data.len().is_multiple_of(8) && r.data.len() >= if encrypt { 16 } else { 24 },
            "OperationError",
            "Invalid AES-KW input length",
        )?;
        let mut out = vec![
            0;
            if encrypt {
                r.data.len() + 8
            } else {
                r.data.len() - 8
            }
        ];
        let n = if encrypt {
            aes::wrap_key(
                &aes::AesKey::new_encrypt(bytes)
                    .map_err(|_| error("OperationError", "Invalid AES key"))?,
                None,
                &mut out,
                &r.data,
            )
        } else {
            aes::unwrap_key(
                &aes::AesKey::new_decrypt(bytes)
                    .map_err(|_| error("OperationError", "Invalid AES key"))?,
                None,
                &mut out,
                &r.data,
            )
        }
        .map_err(|_| error("OperationError", "Key wrapping failed"))?;
        out.truncate(n);
        return Ok(out);
    }
    let cipher = match (key.algorithm.as_str(), bytes.len()) {
        ("AES-GCM", 16) => Cipher::aes_128_gcm(),
        ("AES-GCM", 24) => Cipher::aes_192_gcm(),
        ("AES-GCM", 32) => Cipher::aes_256_gcm(),
        ("AES-CBC", 16) => Cipher::aes_128_cbc(),
        ("AES-CBC", 24) => Cipher::aes_192_cbc(),
        ("AES-CBC", 32) => Cipher::aes_256_cbc(),
        ("AES-CTR", 16) => Cipher::aes_128_ctr(),
        ("AES-CTR", 24) => Cipher::aes_192_ctr(),
        ("AES-CTR", 32) => Cipher::aes_256_ctr(),
        _ => return Err(error("NotSupportedError", "Unsupported cipher")),
    };
    if key.algorithm == "AES-GCM" {
        require(
            [32, 64, 96, 104, 112, 120, 128].contains(&r.tag_length) && !r.iv.is_empty(),
            "OperationError",
            "Invalid GCM parameters",
        )?;
        let size = r.tag_length / 8;
        if encrypt {
            let mut tag = vec![0; size];
            let mut data =
                symm::encrypt_aead(cipher, bytes, Some(&r.iv), &r.aad, &r.data, &mut tag)?;
            data.extend(tag);
            return Ok(data);
        }
        require(
            r.data.len() >= size,
            "OperationError",
            "Ciphertext is shorter than the tag",
        )?;
        let (data, tag) = r.data.split_at(r.data.len() - size);
        return Ok(symm::decrypt_aead(
            cipher,
            bytes,
            Some(&r.iv),
            &r.aad,
            data,
            tag,
        )?);
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
        let mut out = symm::encrypt(cipher, bytes, Some(&r.iv), &r.data[..split])?;
        if split < r.data.len() {
            out.extend(symm::encrypt(
                cipher,
                bytes,
                Some(&(counter & !mask).to_be_bytes()),
                &r.data[split..],
            )?);
        }
        return Ok(out);
    }
    Ok(if encrypt {
        symm::encrypt(cipher, bytes, Some(&r.iv), &r.data)?
    } else {
        symm::decrypt(cipher, bytes, Some(&r.iv), &r.data)?
    })
}
