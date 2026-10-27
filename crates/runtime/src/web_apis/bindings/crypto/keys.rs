use super::*;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
pub(super) fn allowed(algorithm: &str, kind: &str) -> Vec<&'static str> {
    match (algorithm, kind) {
        ("HMAC", "secret") => vec!["sign", "verify"],
        ("AES-KW", "secret") => vec!["wrapKey", "unwrapKey"],
        ("AES-GCM" | "AES-CBC" | "AES-CTR", "secret") => {
            vec!["encrypt", "decrypt", "wrapKey", "unwrapKey"]
        }
        ("HKDF" | "PBKDF2", "secret") | ("ECDH", "private") => vec!["deriveKey", "deriveBits"],
        ("RSA-PSS" | "RSASSA-PKCS1-v1_5" | "ECDSA", "private") => vec!["sign"],
        ("RSA-PSS" | "RSASSA-PKCS1-v1_5" | "ECDSA", "public") => vec!["verify"],
        ("RSA-OAEP", "private") => vec!["decrypt", "unwrapKey"],
        ("RSA-OAEP", "public") => vec!["encrypt", "wrapKey"],
        _ => vec![],
    }
}
pub(super) fn validate_usages(key: &Key) -> Result<()> {
    require(
        key.kind() == "public" || !key.usages.is_empty(),
        "SyntaxError",
        "Secret/private keys require usages",
    )?;
    let allowed = allowed(&key.algorithm, key.kind());
    require(
        key.usages.iter().all(|u| allowed.contains(&u.as_str())),
        "SyntaxError",
        "Invalid key usages",
    )?;
    if key.algorithm.starts_with("RSA") || key.algorithm == "HMAC" {
        native::hash_id(&key.hash)?;
    }
    Ok(())
}
pub(super) fn new_key(r: &Request, material: Material) -> Key {
    Key {
        material,
        algorithm: r.algorithm.clone(),
        hash: r.hash.clone(),
        curve: r.curve.clone(),
        length: r.length,
        extractable: r.extractable,
        usages: r.usages.iter().fold(Vec::new(), |mut usages, usage| {
            if !usages.contains(usage) {
                usages.push(usage.clone());
            }
            usages
        }),
    }
}
fn decode(jwk: &Value, name: &str) -> Result<Vec<u8>> {
    let text = jwk[name]
        .as_str()
        .ok_or_else(|| error("DataError", format!("Missing JWK {name}")))?;
    URL_SAFE_NO_PAD
        .decode(text)
        .map_err(|_| error("DataError", "Invalid JWK base64url"))
}
fn jwk_algorithm(key: &Key) -> Option<String> {
    let hash = key.hash.strip_prefix("SHA-").unwrap_or("");
    Some(match key.algorithm.as_str() {
        "HMAC" => format!("HS{hash}"),
        "RSA-PSS" => format!("PS{hash}"),
        "RSASSA-PKCS1-v1_5" => format!("RS{hash}"),
        "RSA-OAEP" => {
            if hash == "1" {
                "RSA-OAEP".into()
            } else {
                format!("RSA-OAEP-{hash}")
            }
        }
        "AES-GCM" => format!("A{}GCM", key.length),
        "AES-CBC" => format!("A{}CBC", key.length),
        "AES-CTR" => format!("A{}CTR", key.length),
        "AES-KW" => format!("A{}KW", key.length),
        "ECDSA" => match key.curve.as_str() {
            "P-256" => "ES256",
            "P-384" => "ES384",
            _ => "ES512",
        }
        .into(),
        _ => return None,
    })
}
pub(super) fn import(r: &Request) -> Result<Key> {
    let imported = import_material(r).map_err(|e| {
        if e.name == "OperationError" {
            error("DataError", "Invalid key data")
        } else {
            e
        }
    })?;
    let mut key = new_key(r, imported);
    if let Material::Secret(bytes) = &mut key.material {
        if r.algorithm.starts_with("AES-") {
            require(
                [16, 24, 32].contains(&bytes.len()),
                "DataError",
                "Invalid AES key length",
            )?;
            key.length = bytes.len() * 8;
        } else if r.algorithm == "HMAC" {
            require(!bytes.is_empty(), "DataError", "Empty HMAC key")?;
            key.length = if r.length == 0 {
                bytes.len() * 8
            } else {
                r.length
            };
            require(
                key.length > bytes.len() * 8 - 8 && key.length <= bytes.len() * 8,
                "DataError",
                "Invalid HMAC key length",
            )?;
            if !key.length.is_multiple_of(8) {
                *bytes.last_mut().unwrap() &= 255 << (8 - key.length % 8);
            }
        } else {
            require(
                !r.extractable,
                "SyntaxError",
                "Derivation base keys must not be extractable",
            )?;
        }
    } else {
        let public = key.public()?;
        if r.algorithm.starts_with("RSA") {
            require(public.kind() == 1, "DataError", "Expected RSA key")?;
        } else {
            require(public.kind() == 2, "DataError", "Expected EC key")?;
            require(
                public.curve() == native::curve_id(&r.curve)?,
                "DataError",
                "Named curve mismatch",
            )?;
        }
    }
    validate_usages(&key)?;
    if r.format == "jwk" {
        let j = &r.jwk;
        require(
            j["ext"] != false || !r.extractable,
            "DataError",
            "JWK is not extractable",
        )?;
        if let Some(ops) = j.get("key_ops") {
            let ops = ops
                .as_array()
                .ok_or_else(|| error("DataError", "Invalid JWK key_ops"))?;
            let mut seen = std::collections::HashSet::new();
            require(
                ops.iter()
                    .all(|v| v.as_str().is_some_and(|v| seen.insert(v))),
                "DataError",
                "Duplicate/invalid JWK key_ops",
            )?;
            require(
                r.usages.iter().all(|u| seen.contains(u.as_str())),
                "DataError",
                "JWK usage mismatch",
            )?;
        }
        if let Some(use_) = j.get("use") {
            let expected = if ["HMAC", "ECDSA", "RSA-PSS", "RSASSA-PKCS1-v1_5"]
                .contains(&r.algorithm.as_str())
            {
                "sig"
            } else {
                "enc"
            };
            require(
                use_.as_str() == Some(expected),
                "DataError",
                "JWK use mismatch",
            )?;
        }
        if let Some(alg) = j.get("alg") {
            require(
                jwk_algorithm(&key).as_deref() == alg.as_str(),
                "DataError",
                "JWK algorithm mismatch",
            )?;
        }
    }
    Ok(key)
}
fn import_material(r: &Request) -> Result<Material> {
    let symmetric = [
        "HMAC", "AES-GCM", "AES-CBC", "AES-CTR", "AES-KW", "HKDF", "PBKDF2",
    ]
    .contains(&r.algorithm.as_str());
    if symmetric {
        let bytes = match r.format.as_str() {
            "raw" => r.key.clone(),
            "jwk" if r.algorithm != "HKDF" && r.algorithm != "PBKDF2" => {
                require(r.jwk["kty"] == "oct", "DataError", "Expected oct JWK")?;
                decode(&r.jwk, "k")?
            }
            _ => return Err(error("NotSupportedError", "Unsupported key format")),
        };
        return Ok(Material::Secret(bytes));
    }
    require(
        ["RSA-PSS", "RSASSA-PKCS1-v1_5", "RSA-OAEP", "ECDSA", "ECDH"]
            .contains(&r.algorithm.as_str()),
        "NotSupportedError",
        "Unsupported key algorithm",
    )?;
    match r.format.as_str() {
        "spki" => Ok(Material::Public(NativeKey::parse(false, &r.key)?)),
        "pkcs8" => Ok(Material::Private(NativeKey::parse(true, &r.key)?)),
        "raw" if r.algorithm.starts_with("EC") => {
            let size = (native::curve_id(&r.curve)? as usize).div_ceil(8);
            require(
                r.key.first() == Some(&4) && r.key.len() == 1 + 2 * size,
                "DataError",
                "Expected an uncompressed EC point",
            )?;
            Ok(Material::Public(NativeKey::ec(
                &r.curve,
                &r.key[1..1 + size],
                &r.key[1 + size..],
                &[],
            )?))
        }
        "jwk" => {
            let j = &r.jwk;
            let private = j.get("d").is_some();
            let material = if r.algorithm.starts_with("RSA") {
                require(
                    j["kty"] == "RSA" && j.get("oth").is_none(),
                    "DataError",
                    "Expected two-prime RSA JWK",
                )?;
                let mut parts: [Vec<u8>; 8] = Default::default();
                for (i, name) in ["n", "e", "d", "p", "q", "dp", "dq", "qi"]
                    .iter()
                    .enumerate()
                    .take(if private { 8 } else { 2 })
                {
                    parts[i] = decode(j, name)?;
                    require(!parts[i].is_empty(), "DataError", "Empty JWK integer")?;
                }
                NativeKey::rsa(&parts)?
            } else {
                require(
                    j["kty"] == "EC" && j["crv"] == r.curve,
                    "DataError",
                    "EC curve mismatch",
                )?;
                let size = (native::curve_id(&r.curve)? as usize).div_ceil(8);
                let x = decode(j, "x")?;
                let y = decode(j, "y")?;
                let d = if private { decode(j, "d")? } else { Vec::new() };
                require(
                    x.len() == size && y.len() == size && (!private || d.len() == size),
                    "DataError",
                    "Invalid EC component length",
                )?;
                NativeKey::ec(&r.curve, &x, &y, &d)?
            };
            Ok(if private {
                Material::Private(material)
            } else {
                Material::Public(material)
            })
        }
        _ => Err(error("NotSupportedError", "Unsupported key format")),
    }
}
pub(super) fn export(key: &Key, format: &str) -> Result<Value> {
    match format {
        "raw" => match &key.material {
            Material::Secret(bytes) => Ok(json!(bytes)),
            Material::Public(p) if key.algorithm.starts_with("EC") => Ok(json!(p.export(3)?)),
            _ => Err(error("InvalidAccessError", "Cannot export this key as raw")),
        },
        "pkcs8" => Ok(json!(key.private()?.export(2)?)),
        "spki" => {
            require(
                key.kind() == "public",
                "InvalidAccessError",
                "SPKI requires a public key",
            )?;
            Ok(json!(key.public()?.export(1)?))
        }
        "jwk" => {
            let mut j = json!({"key_ops":key.usages,"ext":key.extractable});
            if let Some(alg) = jwk_algorithm(key) {
                j["alg"] = json!(alg);
            }
            match &key.material {
                Material::Secret(bytes) => {
                    j["kty"] = json!("oct");
                    j["k"] = json!(URL_SAFE_NO_PAD.encode(bytes));
                }
                _ if key.algorithm.starts_with("RSA") => {
                    j["kty"] = json!("RSA");
                    for (i, name) in ["n", "e", "d", "p", "q", "dp", "dq", "qi"]
                        .iter()
                        .enumerate()
                        .take(if key.kind() == "private" { 8 } else { 2 })
                    {
                        j[name] =
                            json!(URL_SAFE_NO_PAD.encode(key.public()?.export(10 + i as i32)?));
                    }
                }
                _ => {
                    j["kty"] = json!("EC");
                    j["crv"] = json!(key.curve);
                    for (i, name) in ["x", "y", "d"]
                        .iter()
                        .enumerate()
                        .take(if key.kind() == "private" { 3 } else { 2 })
                    {
                        j[name] =
                            json!(URL_SAFE_NO_PAD.encode(key.public()?.export(20 + i as i32)?));
                    }
                }
            }
            Ok(j)
        }
        _ => Err(error("NotSupportedError", "Unsupported export format")),
    }
}
