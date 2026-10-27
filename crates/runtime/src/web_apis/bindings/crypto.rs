//! Realm-owned WebCrypto keys. Shared BoringSSL owns cryptographic primitives; this layer
//! owns WebCrypto formats, algorithm binding, usages and extractability.
use native::NativeKey;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
mod cipher;
mod keys;
mod native;

#[derive(Debug)]
struct Error {
    name: &'static str,
    message: String,
}
type Result<T> = std::result::Result<T, Error>;
fn error(name: &'static str, message: impl Into<String>) -> Error {
    Error {
        name,
        message: message.into(),
    }
}
fn require(condition: bool, name: &'static str, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(error(name, message))
    }
}
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct Request {
    operation: String,
    algorithm: String,
    hash: String,
    curve: String,
    handle: u64,
    peer: u64,
    extractable: bool,
    usages: Vec<String>,
    usage: String,
    format: String,
    jwk: Value,
    length: usize,
    modulus_length: u32,
    exponent: Vec<u8>,
    salt_length: i32,
    iterations: usize,
    tag_length: usize,
    key: Vec<u8>,
    data: Vec<u8>,
    signature: Vec<u8>,
    iv: Vec<u8>,
    aad: Vec<u8>,
    salt: Vec<u8>,
    info: Vec<u8>,
    label: Vec<u8>,
    derived: Option<Box<Request>>,
}
enum Material {
    Secret(Vec<u8>),
    Public(NativeKey),
    Private(NativeKey),
}
struct Key {
    material: Material,
    algorithm: String,
    hash: String,
    curve: String,
    length: usize,
    extractable: bool,
    usages: Vec<String>,
}
impl Key {
    fn kind(&self) -> &'static str {
        match self.material {
            Material::Secret(_) => "secret",
            Material::Public(_) => "public",
            Material::Private(_) => "private",
        }
    }
    fn secret(&self) -> Result<&[u8]> {
        match &self.material {
            Material::Secret(b) => Ok(b),
            _ => Err(error("InvalidAccessError", "Expected a secret key")),
        }
    }
    fn private(&self) -> Result<&NativeKey> {
        match &self.material {
            Material::Private(p) => Ok(p),
            _ => Err(error("InvalidAccessError", "Expected a private key")),
        }
    }
    fn public(&self) -> Result<&NativeKey> {
        match &self.material {
            Material::Public(p) => Ok(p),
            Material::Private(p) => Ok(p),
            _ => Err(error("InvalidAccessError", "Expected a public key")),
        }
    }
    fn metadata(&self) -> Result<Value> {
        let mut a = json!({"name":self.algorithm});
        if !self.hash.is_empty() {
            a["hash"] = json!({"name":self.hash});
        }
        if self.algorithm.starts_with("AES-") || self.algorithm == "HMAC" {
            a["length"] = json!(self.length);
        }
        if self.algorithm.starts_with("RSA") {
            let key = self.public()?;
            a["modulusLength"] = json!(key.bits());
            a["publicExponent"] = json!(key.export(11)?);
        }
        if !self.curve.is_empty() {
            a["namedCurve"] = json!(self.curve);
        }
        Ok(a)
    }
}
#[derive(Default)]
pub(super) struct CryptoState {
    keys: HashMap<u64, Key>,
    next: u64,
}
impl CryptoState {
    pub(super) fn execute(&mut self, source: &str) -> std::result::Result<String, String> {
        let request: Request = serde_json::from_str(source).map_err(|e| e.to_string())?;
        let result = match self.run(request) {
            Ok(v) => json!({"value":v}),
            Err(e) => json!({"error":{"name":e.name,"message":e.message}}),
        };
        serde_json::to_string(&result).map_err(|e| e.to_string())
    }
    fn insert(&mut self, key: Key) -> Result<Value> {
        keys::validate_usages(&key)?;
        let algorithm = key.metadata()?;
        self.next += 1;
        let id = self.next;
        let result = json!({"handle":id,"type":key.kind(),"algorithm":algorithm,"extractable":key.extractable,"usages":key.usages});
        self.keys.insert(id, key);
        Ok(result)
    }
    fn get(&self, id: u64) -> Result<&Key> {
        self.keys
            .get(&id)
            .ok_or_else(|| error("InvalidAccessError", "Unknown key"))
    }
    fn run(&mut self, mut r: Request) -> Result<Value> {
        if r.operation == "digest" {
            return Ok(json!(native::digest(&r.algorithm, &r.data)?));
        }
        if r.operation == "import" {
            return self.insert(keys::import(&r)?);
        }
        if r.operation == "generate" {
            if r.algorithm.starts_with("RSA") || r.algorithm == "ECDSA" || r.algorithm == "ECDH" {
                let private = if r.algorithm.starts_with("RSA") {
                    require(
                        (1024..=16384).contains(&r.modulus_length),
                        "OperationError",
                        "Invalid RSA modulus length",
                    )?;
                    NativeKey::generate_rsa(r.modulus_length, &r.exponent)?
                } else {
                    NativeKey::generate_ec(&r.curve)?
                };
                let public = NativeKey::parse(false, &private.export(1)?)?;
                let mut secret = keys::new_key(&r, Material::Private(private));
                let mut visible = keys::new_key(&r, Material::Public(public));
                visible.extractable = true;
                let all = keys::allowed(&r.algorithm, "private")
                    .into_iter()
                    .chain(keys::allowed(&r.algorithm, "public"))
                    .collect::<Vec<_>>();
                require(
                    r.usages.iter().all(|u| all.contains(&u.as_str())),
                    "SyntaxError",
                    "Invalid key usages",
                )?;
                secret
                    .usages
                    .retain(|u| keys::allowed(&r.algorithm, "private").contains(&u.as_str()));
                visible
                    .usages
                    .retain(|u| keys::allowed(&r.algorithm, "public").contains(&u.as_str()));
                // Validate both before inserting either key.
                keys::validate_usages(&secret)?;
                keys::validate_usages(&visible)?;
                return Ok(
                    json!({"privateKey":self.insert(secret)?,"publicKey":self.insert(visible)?}),
                );
            }
            require(
                r.algorithm.starts_with("AES-") || r.algorithm == "HMAC",
                "NotSupportedError",
                "Unsupported generation algorithm",
            )?;
            require(
                r.length > 0 && r.length <= 8 * 1024 * 1024,
                "OperationError",
                "Invalid generated key length",
            )?;
            r.key = vec![0; r.length.div_ceil(8)];
            getrandom::fill(&mut r.key)
                .map_err(|_| error("OperationError", "Random generation failed"))?;
            if !r.length.is_multiple_of(8) {
                *r.key.last_mut().unwrap() &= 255 << (8 - r.length % 8);
            }
            r.format = "raw".into();
            return self.insert(keys::import(&r)?);
        }
        let key = self.get(r.handle)?;
        if r.operation == "export" {
            require(
                key.extractable,
                "InvalidAccessError",
                "Key is not extractable",
            )?;
            return keys::export(key, &r.format);
        }
        require(
            key.algorithm == r.algorithm,
            "InvalidAccessError",
            "Key algorithm mismatch",
        )?;
        let usage = if r.usage.is_empty() {
            r.operation.as_str()
        } else {
            r.usage.as_str()
        };
        require(
            key.usages.iter().any(|u| u == usage),
            "InvalidAccessError",
            "Key does not permit this operation",
        )?;
        match r.operation.as_str() {
            "sign" | "verify" => sign(key, &r),
            "encrypt" | "decrypt" => Ok(json!(cipher::run(key, &r)?)),
            "deriveBits" | "deriveKey" => {
                let bits = self.derive(key, &r)?;
                if r.operation == "deriveBits" {
                    Ok(json!(bits))
                } else {
                    let mut derived = *r
                        .derived
                        .take()
                        .ok_or_else(|| error("TypeError", "Missing derived algorithm"))?;
                    derived.key = bits;
                    derived.format = "raw".into();
                    self.insert(keys::import(&derived)?)
                }
            }
            _ => Err(error("NotSupportedError", "Unsupported crypto operation")),
        }
    }
    fn derive(&self, key: &Key, r: &Request) -> Result<Vec<u8>> {
        require(
            r.length <= 8 * 1024 * 1024,
            "OperationError",
            "Derived output is too large",
        )?;
        if key.algorithm == "ECDH" {
            let peer = self.get(r.peer)?;
            require(
                peer.kind() == "public" && peer.algorithm == "ECDH" && peer.curve == key.curve,
                "InvalidAccessError",
                "ECDH peer key mismatch",
            )?;
            let mut result = key.private()?.ecdh(peer.public()?)?;
            require(
                r.length <= result.len() * 8,
                "OperationError",
                "Derived length exceeds shared secret",
            )?;
            result.truncate(r.length.div_ceil(8));
            if !r.length.is_multiple_of(8)
                && let Some(last) = result.last_mut()
            {
                *last &= 255 << (8 - r.length % 8);
            }
            return Ok(result);
        }
        require(
            r.length.is_multiple_of(8),
            "OperationError",
            "Derived length must be a multiple of eight",
        )?;
        let mut output = vec![0; r.length / 8];
        match key.algorithm.as_str() {
            "PBKDF2" => {
                require(
                    r.iterations > 0 && r.iterations <= i32::MAX as usize,
                    "OperationError",
                    "Invalid PBKDF2 iteration count",
                )?;
                native::derive(
                    2,
                    &r.hash,
                    key.secret()?,
                    &r.salt,
                    &[],
                    r.iterations as u32,
                    &mut output,
                )?;
            }
            "HKDF" => {
                native::derive(1, &r.hash, key.secret()?, &r.salt, &r.info, 0, &mut output)?;
            }
            _ => return Err(error("NotSupportedError", "Unsupported derivation")),
        }
        Ok(output)
    }
}
fn sign(key: &Key, r: &Request) -> Result<Value> {
    let signing = r.operation == "sign";
    let hash = if key.algorithm == "ECDSA" {
        &r.hash
    } else {
        &key.hash
    };
    native::hash_id(hash)?;
    if key.algorithm == "HMAC" {
        let signature = native::hmac(hash, key.secret()?, &r.data)?;
        return Ok(if signing {
            json!(signature)
        } else {
            json!(native::equal(&signature, &r.signature))
        });
    }
    let scheme = match key.algorithm.as_str() {
        "RSASSA-PKCS1-v1_5" => 1,
        "RSA-PSS" => 2,
        "ECDSA" => 3,
        _ => return Err(error("InvalidAccessError", "Not a signing key")),
    };
    if scheme == 2 {
        require(r.salt_length >= 0, "OperationError", "Invalid salt length")?;
    }
    if signing {
        Ok(json!(key.private()?.sign(
            scheme,
            hash,
            r.salt_length,
            &r.data
        )?))
    } else {
        require(
            key.kind() == "public",
            "InvalidAccessError",
            "Expected a public verification key",
        )?;
        Ok(json!(key.public()?.verify(
            scheme,
            hash,
            r.salt_length,
            &r.data,
            &r.signature
        )?))
    }
}
