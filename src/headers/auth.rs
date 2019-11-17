use sha::utils::Digest;
use sha::utils::DigestExt;
use sha::sha256::Sha256;
use sha::sha512::Sha512;

use std::fmt;
use std::collections::HashMap;
use std::io::Result as IoResult;

use crate::Uri;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Schema {
    Digest
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Schema::Digest => write!(f, "Digest")
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct AuthHeader(pub Schema, pub HashMap<String, String>);

impl fmt::Display for AuthHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)?;
        for (index, (key, value)) in self.1.iter().enumerate() {
            if index == 0 {
                if key == &"qop".to_string() {
                    write!(f, "{}={}", key, value)?;
                } else if key == &"auth".to_string() {
                    write!(f, "{}={:08}", key, value)?;
                } else {
                    write!(f, " {}=\"{}\"", key, value)?;
                }
            } else {
                if key == &"qop".to_string() {
                    write!(f, ", {}={}", key, value)?;
                } else if key == &"auth".to_string() {
                    write!(f, ", {}={:08}", key, value)?;
                } else {
                    write!(f, ", {}=\"{}\"", key, value)?;
                }
            }
        }
        Ok(())
    }
}

pub struct AuthContext<'a> {
    pub user: &'a str,
    pub pass: &'a str,
    pub nc: u32,
    pub uri: &'a Uri
}

impl AuthHeader {

    pub fn authenticate<'a>(&self, ctx: AuthContext<'a>) -> IoResult<AuthHeader> {
        match self.0 {
            Schema::Digest => self.handle_digest_auth(ctx)
        }
    }

    fn handle_digest_auth<'a>(&self, ctx: AuthContext<'a>) -> IoResult<AuthHeader> {
        let qop = self.1.get("qop").expect("Auth header does not contain a qop field");
        match qop.as_ref() {
            "auth" => self.handle_digest_qop_auth(ctx),
            qop => panic!("unknown auth qop: {}", qop)
        }
    }

    fn handle_digest_qop_auth<'a>(&self, ctx: AuthContext<'a>) -> IoResult<AuthHeader> {
        let alg = self.1.get("algorithm")
            .map(|item| item.to_string())
            .unwrap_or("md5".to_string());
        match alg.as_ref() {
            "md5" | "MD5" => self.handle_md5_digest_auth(ctx),
            "sha-256" | "SHA-256" => self.handle_sha256_digest_auth(ctx),
            "sha-512-256" | "SHA-512-256" => self.handle_sha512_digest_auth(ctx),
            alg => panic!("Unknown Auth alogirithm: {}", alg)
        }
    }

    fn handle_md5_digest_auth<'a>(&self, ctx: AuthContext<'a>) -> IoResult<AuthHeader> {
        let realm = self.1.get("realm").expect("Auth header does not contain a realm");
        let nonce = self.1.get("nonce").expect("Auth header does not contain a nonce");
        let mut map: HashMap<String, String> = HashMap::new();
        let cnonce = self.generate_cnonce();
        map.insert("username".into(), ctx.user.to_string());
        map.insert("nonce".into(), format!("{}", nonce));
        map.insert("realm".into(), realm.clone());
        map.insert("uri".into(), format!("{}", ctx.uri));
        map.insert("qop".into(), self.1.get("qop").unwrap().clone());
        map.insert("algorithm".into(), "MD5".into());
        map.insert("cnonce".into(), format!("{:x}", cnonce));
        map.insert("nc".into(), format!("{:08}", ctx.nc));
        let ha1 = md5::compute(&format!("{}:{}:{}", ctx.user, realm, ctx.pass));
        let ha2 = md5::compute(format!("REGISTER:{}", ctx.uri));
        let digest = format!("{:x}:{}:{:08}:{:x}:auth:{:x}", ha1, nonce, ctx.nc, cnonce, ha2);
        let pass = md5::compute(digest);
        map.insert("response".into(), format!("{:x}", pass));
        Ok(AuthHeader(Schema::Digest, map))
    }

    fn handle_sha256_digest_auth<'a>(&self, ctx: AuthContext<'a>) -> IoResult<AuthHeader> {
        let realm = self.1.get("realm").expect("Auth header does not contain a realm");
        let nonce = self.1.get("nonce").expect("Auth header does not contain a nonce");
        let opaque = self.1.get("opaque").expect("Auth header does not contain a opaque");
        let mut map: HashMap<String, String> = HashMap::new();
        let cnonce = self.generate_cnonce();
        map.insert("username".into(), ctx.user.to_string());
        map.insert("nonce".into(), format!("{}", nonce));
        map.insert("realm".into(), realm.clone());
        map.insert("uri".into(), format!("{}", ctx.uri));
        map.insert("qop".into(), self.1.get("qop").unwrap().clone());
        map.insert("algorithm".into(), "SHA-512-256".into());
        map.insert("cnonce".into(), format!("{:x}", cnonce));
        map.insert("nc".into(), format!("{:08}", ctx.nc));
        map.insert("opaque".into(), opaque.into());
        let mut ha1_hasher = Sha256::default();
        let ha1 = ha1_hasher.digest(format!("{}:{}:{}", ctx.user, realm, ctx.pass).as_ref());
        let mut ha2_hasher = Sha256::default();
        let ha2 = ha2_hasher.digest(format!("REGISTER:{}", ctx.uri).as_ref());
        let digest = format!("{}:{}:{:08}:{:x}:auth:{}", ha1.to_hex(), nonce, ctx.nc, cnonce, ha2.to_hex());
        let pass = md5::compute(digest);
        map.insert("response".into(), format!("{:x}", pass));
        Ok(AuthHeader(Schema::Digest, map))
    }

    fn handle_sha512_digest_auth<'a>(&self, ctx: AuthContext<'a>) -> IoResult<AuthHeader> {
        let realm = self.1.get("realm").expect("Auth header does not contain a realm");
        let nonce = self.1.get("nonce").expect("Auth header does not contain a nonce");
        let opaque = self.1.get("opaque").expect("Auth header does not contain a opaque");
        let mut map: HashMap<String, String> = HashMap::new();
        let cnonce = self.generate_cnonce();
        map.insert("username".into(), ctx.user.to_string());
        map.insert("nonce".into(), format!("{}", nonce));
        map.insert("realm".into(), realm.clone());
        map.insert("uri".into(), format!("{}", ctx.uri));
        map.insert("qop".into(), self.1.get("qop").unwrap().clone());
        map.insert("algorithm".into(), "SHA-512".into());
        map.insert("cnonce".into(), format!("{:x}", cnonce));
        map.insert("nc".into(), format!("{:08}", ctx.nc));
        map.insert("opaque".into(), opaque.into());
        let mut ha1_hasher = Sha512::default();
        let ha1 = ha1_hasher.digest(format!("{}:{}:{}", ctx.user, realm, ctx.pass).as_ref());
        let mut ha2_hasher = Sha512::default();
        let ha2 = ha2_hasher.digest(format!("REGISTER:{}", ctx.uri).as_ref());
        let digest = format!("{}:{}:{:08}:{:x}:auth:{}", ha1.to_hex(), nonce, ctx.nc, cnonce, ha2.to_hex());
        let pass = md5::compute(digest);
        map.insert("response".into(), format!("{:x}", pass));
        Ok(AuthHeader(Schema::Digest, map))
    }

    fn generate_cnonce(&self) -> md5::Digest {
        md5::compute(rand::random::<[u8 ; 16]>())
    }
}
