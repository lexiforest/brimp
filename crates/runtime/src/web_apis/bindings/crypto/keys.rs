use super::*;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use openssl::{
    bn::{BigNumContext, BigNumRef},
    ec::{EcPoint, PointConversionForm},
};

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
        digest(&key.hash)?;
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
fn number(jwk: &Value, name: &str) -> Result<BigNum> {
    let bytes = decode(jwk, name)?;
    require(!bytes.is_empty(), "DataError", "Empty JWK integer")?;
    Ok(BigNum::from_slice(&bytes)?)
}
fn encode(n: &BigNumRef) -> String {
    URL_SAFE_NO_PAD.encode(n.to_vec())
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
            require(public.id() == Id::RSA, "DataError", "Expected RSA key")?;
        } else {
            require(public.id() == Id::EC, "DataError", "Expected EC key")?;
            require(
                public.ec_key()?.group().curve_name() == group(&r.curve)?.curve_name(),
                "DataError",
                "Named curve mismatch",
            )?;
            public.ec_key()?.check_key()?;
            if let Material::Private(p) = &key.material {
                p.ec_key()?.check_key()?;
            }
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
        "spki" => Ok(Material::Public(PKey::public_key_from_der(&r.key)?)),
        "pkcs8" => Ok(Material::Private(PKey::private_key_from_pkcs8(&r.key)?)),
        "raw" if r.algorithm.starts_with("EC") => {
            let g = group(&r.curve)?;
            require(
                r.key.first() == Some(&4) && r.key.len() == 1 + 2 * g.degree().div_ceil(8) as usize,
                "DataError",
                "Expected an uncompressed EC point",
            )?;
            let point = EcPoint::from_bytes(&g, &r.key, &mut *BigNumContext::new()?)?;
            Ok(Material::Public(PKey::from_ec_key(
                EcKey::from_public_key(&g, &point)?,
            )?))
        }
        "jwk" => {
            let j = &r.jwk;
            if r.algorithm.starts_with("RSA") {
                require(
                    j["kty"] == "RSA" && j.get("oth").is_none(),
                    "DataError",
                    "Expected two-prime RSA JWK",
                )?;
                if j.get("d").is_some() {
                    let rsa = Rsa::from_private_components(
                        number(j, "n")?,
                        number(j, "e")?,
                        number(j, "d")?,
                        number(j, "p")?,
                        number(j, "q")?,
                        number(j, "dp")?,
                        number(j, "dq")?,
                        number(j, "qi")?,
                    )?;
                    require(rsa.check_key()?, "DataError", "Invalid RSA private key")?;
                    Ok(Material::Private(PKey::from_rsa(rsa)?))
                } else {
                    Ok(Material::Public(PKey::from_rsa(
                        Rsa::from_public_components(number(j, "n")?, number(j, "e")?)?,
                    )?))
                }
            } else {
                require(
                    j["kty"] == "EC" && j["crv"] == r.curve,
                    "DataError",
                    "EC curve mismatch",
                )?;
                let g = group(&r.curve)?;
                let size = g.degree().div_ceil(8) as usize;
                require(
                    decode(j, "x")?.len() == size && decode(j, "y")?.len() == size,
                    "DataError",
                    "Invalid EC coordinate length",
                )?;
                let public = EcKey::from_public_key_affine_coordinates(
                    &g,
                    &*number(j, "x")?,
                    &*number(j, "y")?,
                )?;
                if j.get("d").is_some() {
                    require(
                        decode(j, "d")?.len() == size,
                        "DataError",
                        "Invalid EC private length",
                    )?;
                    Ok(Material::Private(PKey::from_ec_key(
                        EcKey::from_private_components(&g, &*number(j, "d")?, public.public_key())?,
                    )?))
                } else {
                    Ok(Material::Public(PKey::from_ec_key(public)?))
                }
            }
        }
        _ => Err(error("NotSupportedError", "Unsupported key format")),
    }
}
pub(super) fn export(key: &Key, format: &str) -> Result<Value> {
    match format {
        "raw" => match &key.material {
            Material::Secret(bytes) => Ok(json!(bytes)),
            Material::Public(p) if key.algorithm.starts_with("EC") => {
                let ec = p.ec_key()?;
                Ok(json!(ec.public_key().to_bytes(
                    ec.group(),
                    PointConversionForm::UNCOMPRESSED,
                    &mut *BigNumContext::new()?
                )?))
            }
            _ => Err(error("InvalidAccessError", "Cannot export this key as raw")),
        },
        "pkcs8" => Ok(json!(key.private()?.private_key_to_pkcs8()?)),
        "spki" => {
            require(
                key.kind() == "public",
                "InvalidAccessError",
                "SPKI requires a public key",
            )?;
            Ok(json!(key.public()?.public_key_to_der()?))
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
                    let public = key.public()?.rsa()?;
                    j["kty"] = json!("RSA");
                    j["n"] = json!(encode(public.n()));
                    j["e"] = json!(encode(public.e()));
                    if let Material::Private(p) = &key.material {
                        let rsa = p.rsa()?;
                        j["d"] = json!(encode(rsa.d()));
                        for (name, bn) in [
                            ("p", rsa.p()),
                            ("q", rsa.q()),
                            ("dp", rsa.dmp1()),
                            ("dq", rsa.dmq1()),
                            ("qi", rsa.iqmp()),
                        ] {
                            j[name] = json!(encode(bn.ok_or_else(|| error(
                                "OperationError",
                                "Missing RSA CRT parameter"
                            ))?));
                        }
                    }
                }
                _ => {
                    let public = key.public()?.ec_key()?;
                    let size = public.group().degree().div_ceil(8) as i32;
                    let (mut x, mut y) = (BigNum::new()?, BigNum::new()?);
                    public.public_key().affine_coordinates_gfp(
                        public.group(),
                        &mut x,
                        &mut y,
                        &mut *BigNumContext::new()?,
                    )?;
                    j["kty"] = json!("EC");
                    j["crv"] = json!(key.curve);
                    j["x"] = json!(URL_SAFE_NO_PAD.encode(x.to_vec_padded(size)?));
                    j["y"] = json!(URL_SAFE_NO_PAD.encode(y.to_vec_padded(size)?));
                    if let Material::Private(p) = &key.material {
                        j["d"] = json!(
                            URL_SAFE_NO_PAD.encode(p.ec_key()?.private_key().to_vec_padded(size)?)
                        );
                    }
                }
            }
            Ok(j)
        }
        _ => Err(error("NotSupportedError", "Unsupported export format")),
    }
}
