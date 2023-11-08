

use aes::cipher::{KeyIvInit};
use anyhow::{Result};
use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use hmac::{Mac};
use lz_fnv::{Fnv1a, FnvHasher};
use md5::{Digest, Md5};
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha2::Sha256;
use uuid::Uuid;

use crate::session::{SocksAddr, SocksAddrWireType};

use super::aead_header::seal_vmess_aead_header;

type RequestCommand = u8;

pub const REQUEST_COMMAND_TCP: RequestCommand = 0x01;
pub const REQUEST_COMMAND_UDP: RequestCommand = 0x02;

type Security = u8;

pub const SECURITY_TYPE_AES128_GCM: Security = 0x03;
pub const SECURITY_TYPE_CHACHA20_POLY1305: Security = 0x04;

type RequestOption = u8;

pub const REQUEST_OPTION_CHUNK_STREAM: RequestOption = 0x01;
pub const REQUEST_OPTION_CHUNK_MASKING: RequestOption = 0x04;
pub const REQUEST_OPTION_GLOBAL_PADDING: RequestOption = 0x08;

pub struct RequestHeader {
    pub version: u8,
    pub command: RequestCommand,
    pub option: u8,
    pub security: Security,
    pub address: SocksAddr,
    pub uuid: Uuid,
}

impl RequestHeader {
    pub fn set_option(&mut self, opt: RequestOption) {
        self.option |= opt;
    }

    pub fn encode(&self, buf: &mut BytesMut, sess: &ClientSession) -> Result<()> {
        buf.put_u8(self.version);
        buf.put_slice(&sess.request_body_iv);
        buf.put_slice(&sess.request_body_key);
        buf.put_u8(sess.response_header);
        buf.put_u8(self.option);

        let padding_len = StdRng::from_entropy().gen_range(0..16) % 16_u8;
        let security = (padding_len << 4) | self.security;

        buf.put_u8(security);
        buf.put_u8(0);
        buf.put_u8(self.command);

        self.address.write_buf(buf, SocksAddrWireType::PortFirst);

        // add random bytes
        if padding_len > 0 {
            let mut padding_bytes = BytesMut::with_capacity(padding_len as usize);
            unsafe { padding_bytes.set_len(padding_len as usize) };
            let mut rng = StdRng::from_entropy();
            for i in 0..padding_bytes.len() {
                padding_bytes[i] = rng.gen();
            }
            buf.put_slice(&padding_bytes);
        }

        // checksum
        let mut hasher = Fnv1a::<u32>::default();
        hasher.write(buf);
        let h = hasher.finish();
        let buf_size = buf.len();
        buf.resize(buf_size + 4, 0);
        BigEndian::write_u32(&mut buf[buf_size..], h);
        let mut hasher = Md5::new();
        hasher.update(self.uuid.as_bytes());
        hasher.update(
            "c48619fe-8f02-49e0-b9e9-edf763e17e21"
                .to_string()
                .as_bytes(),
        );
        let key = hasher.finalize();

        let aead_buf = seal_vmess_aead_header(&key, buf);
        *buf = aead_buf;
        Ok(())
    }
}

pub struct ClientSession {
    pub request_body_key: Vec<u8>,
    pub request_body_iv: Vec<u8>,
    pub response_body_key: Vec<u8>,
    pub response_body_iv: Vec<u8>,
    pub response_header: u8,
}

impl ClientSession {
    pub fn new() -> Self {
        let mut salt = [0u8; 64];
        super::aead_helper::random_iv_or_salt(&mut salt);
        let respv = salt[32];
        let resp_body_key = sha256(&salt[16..32]);
        let resp_body_iv = sha256(&salt[0..16]);
        salt[32..48].copy_from_slice(&resp_body_key[..16]);
        salt[48..64].copy_from_slice(&resp_body_iv[..16]);
        let req_body_iv = &salt[0..16];
        let req_body_key = &salt[16..32];
        let resp_body_key = &salt[32..48];
        let resp_body_iv = &salt[48..];

        let mut request_body_key = vec![0u8; 16];
        let mut request_body_iv = vec![0u8; 16];
        let response_header: u8 = respv;

        request_body_key[..].copy_from_slice(req_body_key);
        request_body_iv[..].copy_from_slice(req_body_iv);

        let response_body_key = resp_body_key[..16].to_vec();
        let response_body_iv = resp_body_iv[..16].to_vec();

        ClientSession {
            request_body_key,
            request_body_iv,
            response_body_key,
            response_body_iv,
            response_header,
        }
    }
}

pub fn sha256(b: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b);
    hasher.finalize().into()
}
