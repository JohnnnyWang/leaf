use aes_gcm::Aes128Gcm;
use bytes::{BytesMut};
use chacha20poly1305::ChaCha20Poly1305;


use std::task::{Poll};
use std::{io};


pub struct VmessAeadWriter {
    security: VmessSecurity,
    buffer: BytesMut,
    nonce: [u8; 32],
    pos: usize,
    iv: BytesMut,
    count: u16,
    data_len: usize,
    state: u32, // for state machine generator use
    write_res: Poll<io::Result<usize>>,
}
pub enum VmessSecurity {
    Aes128Gcm(Aes128Gcm),
    ChaCha20Poly1305(ChaCha20Poly1305),
}

impl VmessSecurity {
    #[inline(always)]
    pub fn overhead_len(&self) -> usize {
        16
    }
    #[inline(always)]
    pub fn nonce_len(&self) -> usize {
        12
    }
    #[inline(always)]
    pub fn tag_len(&self) -> usize {
        16
    }
}

pub struct VmessAeadReader {
    security: VmessSecurity,
    pub buffer: BytesMut, // pub for replace buffer
    state: u32,           // for state machine generator use
    read_res: Poll<io::Result<()>>,
    nonce: [u8; 32],
    iv: BytesMut,
    data_length: usize,
    count: u16,
    minimal_data_to_put: usize,
    read_zero: bool,
}

impl VmessAeadReader {
    pub fn new(iv: &[u8], security: VmessSecurity) -> VmessAeadReader {
        let iv = BytesMut::from(iv);
        let buffer = BytesMut::new();
        VmessAeadReader {
            security,
            buffer,
            state: 0,
            read_res: Poll::Pending,
            nonce: [0u8; 32],
            iv,
            data_length: 0,
            count: 0,
            minimal_data_to_put: 0,
            read_zero: false,
        }
    }
}
