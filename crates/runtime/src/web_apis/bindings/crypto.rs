//! Realm-owned WebCrypto keys. OpenSSL owns cryptographic primitives; this layer
//! owns WebCrypto formats, algorithm binding, usages and extractability.
use openssl::{
    bn::BigNum,
    derive::Deriver,
    ec::{EcGroup, EcKey},
    ecdsa::EcdsaSig,
    hash::{MessageDigest, hash},
    md::Md,
    nid::Nid,
    pkey::{Id, PKey, Private, Public},
    pkey_ctx::PkeyCtx,
    rsa::{Padding, Rsa},
    sign::{RsaPssSaltlen, Signer, Verifier},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
mod cipher;
mod keys;

#[derive(Debug)]
struct Error {
    name: &'static str,
    message: String,
}
type Result<T> = std::result::Result<T, Error>;
impl From<openssl::error::ErrorStack> for Error {
    fn from(_: openssl::error::ErrorStack) -> Self {
        error("OperationError", "Cryptographic operation failed")
    }
}
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
    Public(PKey<Public>),
    Private(PKey<Private>),
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
    fn private(&self) -> Result<&PKey<Private>> {
        match &self.material {
            Material::Private(p) => Ok(p),
            _ => Err(error("InvalidAccessError", "Expected a private key")),
        }
    }
    fn public(&self) -> Result<PKey<Public>> {
        match &self.material {
            Material::Public(p) => Ok(p.clone()),
            Material::Private(p) => Ok(PKey::public_key_from_der(&p.public_key_to_der()?)?),
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
            let rsa = self.public()?.rsa()?;
            a["modulusLength"] = json!(rsa.n().num_bits());
            a["publicExponent"] = json!(rsa.e().to_vec());
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
            return Ok(json!(hash(digest(&r.algorithm)?, &r.data)?.to_vec()));
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
                    let e = BigNum::from_slice(&r.exponent)?;
                    PKey::from_rsa(Rsa::generate_with_e(r.modulus_length, &e)?)?
                } else {
                    PKey::from_ec_key(EcKey::generate(&*group(&r.curve)?)?)?
                };
                let public = PKey::public_key_from_der(&private.public_key_to_der()?)?;
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
            let public = peer.public()?;
            let mut derive = Deriver::new(key.private()?)?;
            derive.set_peer(&public)?;
            let mut result = derive.derive_to_vec()?;
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
                openssl::pkcs5::pbkdf2_hmac(
                    key.secret()?,
                    &r.salt,
                    r.iterations,
                    digest(&r.hash)?,
                    &mut output,
                )?;
            }
            "HKDF" => {
                let mut ctx = PkeyCtx::new_id(Id::HKDF)?;
                ctx.derive_init()?;
                ctx.set_hkdf_md(md(&r.hash)?)?;
                ctx.set_hkdf_key(key.secret()?)?;
                ctx.set_hkdf_salt(&r.salt)?;
                ctx.add_hkdf_info(&r.info)?;
                ctx.derive(Some(&mut output))?;
            }
            _ => return Err(error("NotSupportedError", "Unsupported derivation")),
        }
        Ok(output)
    }
}
fn digest(name: &str) -> Result<MessageDigest> {
    match name {
        "SHA-1" => Ok(MessageDigest::sha1()),
        "SHA-256" => Ok(MessageDigest::sha256()),
        "SHA-384" => Ok(MessageDigest::sha384()),
        "SHA-512" => Ok(MessageDigest::sha512()),
        _ => Err(error("NotSupportedError", "Unsupported hash")),
    }
}
fn md(name: &str) -> Result<&'static openssl::md::MdRef> {
    match name {
        "SHA-1" => Ok(Md::sha1()),
        "SHA-256" => Ok(Md::sha256()),
        "SHA-384" => Ok(Md::sha384()),
        "SHA-512" => Ok(Md::sha512()),
        _ => Err(error("NotSupportedError", "Unsupported hash")),
    }
}
fn group(curve: &str) -> Result<EcGroup> {
    Ok(EcGroup::from_curve_name(match curve {
        "P-256" => Nid::X9_62_PRIME256V1,
        "P-384" => Nid::SECP384R1,
        "P-521" => Nid::SECP521R1,
        _ => return Err(error("NotSupportedError", "Unsupported named curve")),
    })?)
}
fn sign(key: &Key, r: &Request) -> Result<Value> {
    let signing = r.operation == "sign";
    let hash_name = if key.algorithm == "ECDSA" {
        &r.hash
    } else {
        &key.hash
    };
    let hash = digest(hash_name)?;
    if key.algorithm == "HMAC" {
        let pkey = PKey::hmac(key.secret()?)?;
        let mut signer = Signer::new(hash, &pkey)?;
        let signature = signer.sign_oneshot_to_vec(&r.data)?;
        return Ok(if signing {
            json!(signature)
        } else {
            json!(
                signature.len() == r.signature.len()
                    && openssl::memcmp::eq(&signature, &r.signature)
            )
        });
    }
    require(
        ["RSA-PSS", "RSASSA-PKCS1-v1_5", "ECDSA"].contains(&key.algorithm.as_str()),
        "InvalidAccessError",
        "Not a signing key",
    )?;
    if key.algorithm == "RSA-PSS" {
        require(r.salt_length >= 0, "OperationError", "Invalid salt length")?;
    }
    if signing {
        let mut signer = Signer::new(hash, key.private()?)?;
        if key.algorithm == "RSA-PSS" {
            signer.set_rsa_padding(Padding::PKCS1_PSS)?;
            signer.set_rsa_pss_saltlen(RsaPssSaltlen::custom(r.salt_length))?;
            signer.set_rsa_mgf1_md(hash)?;
        }
        let signature = signer.sign_oneshot_to_vec(&r.data)?;
        if key.algorithm == "ECDSA" {
            let sig = EcdsaSig::from_der(&signature)?;
            let size = group(&key.curve)?.degree().div_ceil(8) as i32;
            let mut bytes = sig.r().to_vec_padded(size)?;
            bytes.extend(sig.s().to_vec_padded(size)?);
            return Ok(json!(bytes));
        }
        Ok(json!(signature))
    } else {
        require(
            key.kind() == "public",
            "InvalidAccessError",
            "Expected a public verification key",
        )?;
        let public = key.public()?;
        let mut verifier = Verifier::new(hash, &public)?;
        if key.algorithm == "RSA-PSS" {
            verifier.set_rsa_padding(Padding::PKCS1_PSS)?;
            verifier.set_rsa_pss_saltlen(RsaPssSaltlen::custom(r.salt_length))?;
            verifier.set_rsa_mgf1_md(hash)?;
        }
        let signature = if key.algorithm == "ECDSA" {
            let size = group(&key.curve)?.degree().div_ceil(8) as usize;
            if r.signature.len() != 2 * size {
                return Ok(json!(false));
            }
            EcdsaSig::from_private_components(
                BigNum::from_slice(&r.signature[..size])?,
                BigNum::from_slice(&r.signature[size..])?,
            )?
            .to_der()?
        } else {
            r.signature.clone()
        };
        Ok(json!(
            verifier
                .verify_oneshot(&signature, &r.data)
                .unwrap_or(false)
        ))
    }
}
