// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//
// Noise Fuzzing
// =============
//
// This fuzzes the wrappers we have around the Noise library `snow`.
//

use super::Handshake;
use crate::{NoiseParams, NoiseSocket, NOISE_PARAMETER};
use futures::{
    executor::block_on,
    future::join,
    io::{AsyncRead, AsyncWrite},
    ready,
    task::{Context, Poll},
};
use memsocket::MemorySocket;
use once_cell::sync::Lazy;
use rand_core::{impls, CryptoRng, RngCore};
use snow::{
    params::*,
    resolvers::{CryptoResolver, DefaultResolver},
    types::*,
    Builder,
};
use std::{io, pin::Pin};

//
// Corpus generation
// =================
//
// - CountingRng: needed to deterministically generate a keypair with snow.
// - TestResolver: needed to use the deterministic RNG.
// - ExposingSocket: a wrapper around MemorySocket to retrieve handshake messages being sent.
// - NOISE_KEYPAIR: generates the same snow Keypair all the time.
// - generate_first_two_messages: it will generate the first or second message in the handshake.
// - generate_corpus: the function called by our fuzzer to retrieve the corpus.
//

// CountingRng is a deterministic RNG used for deterministic keypair generation
#[derive(Default)]
struct CountingRng(u64);
impl RngCore for CountingRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        Ok(self.fill_bytes(dest))
    }
}
impl CryptoRng for CountingRng {}
impl Random for CountingRng {}

// TestResolver is needed to have a deterministic RNG in keypair generation
struct TestResolver {}
impl CryptoResolver for TestResolver {
    fn resolve_rng(&self) -> Option<Box<dyn Random>> {
        let rng = CountingRng(0);
        Some(Box::new(rng))
    }

    fn resolve_dh(&self, choice: &DHChoice) -> Option<Box<dyn Dh>> {
        DefaultResolver.resolve_dh(choice)
    }

    fn resolve_hash(&self, choice: &HashChoice) -> Option<Box<dyn Hash>> {
        DefaultResolver.resolve_hash(choice)
    }

    fn resolve_cipher(&self, choice: &CipherChoice) -> Option<Box<dyn Cipher>> {
        DefaultResolver.resolve_cipher(choice)
    }
}

// ExposingSocket is needed to retrieve the noise messages peers will write on the socket
struct ExposingSocket {
    pub inner: MemorySocket,
    pub written: Vec<u8>,
}
impl ExposingSocket {
    fn new(memory_socket: MemorySocket) -> Self {
        Self {
            inner: memory_socket,
            written: Vec::new(),
        }
    }
    fn new_pair() -> (Self, Self) {
        let (dialer_socket, listener_socket) = MemorySocket::new_pair();
        (
            ExposingSocket::new(dialer_socket),
            ExposingSocket::new(listener_socket),
        )
    }
}
impl AsyncWrite for ExposingSocket {
    //    use AsyncWrite;
    fn poll_write(
        mut self: Pin<&mut Self>,
        context: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // Note: `let bytes_written = ready!(/* .. */)?` is of a shortcut for
        // let bytes_written = match Pin::new(&mut self.inner).poll_write(context, buf) {
        //     Poll::Ready(Ok(bytes_written)) => bytes_written,
        //     Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
        //     Poll::Pending => return Poll::Pending,
        // };
        let bytes_written = ready!(Pin::new(&mut self.inner).poll_write(context, buf))?;
        self.written.extend_from_slice(&buf[..bytes_written]);
        Poll::Ready(Ok(bytes_written))
    }

    fn poll_flush(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(context)
    }

    /// Attempt to close the channel. Cannot Fail.
    fn poll_close(mut self: Pin<&mut Self>, context: &mut Context) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_close(context)
    }
}
impl AsyncRead for ExposingSocket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_read(context, buf)
    }
}

// take_socket is implemented on parent structure to be able to retrieve our ExposingSocket
impl NoiseSocket<ExposingSocket> {
    fn take_socket(self) -> ExposingSocket {
        self.socket
    }
}

//
// Actual code to generate corpus
//

// let's cache the deterministic keypair
pub static NOISE_KEYPAIR: Lazy<snow::Keypair> = Lazy::new(|| {
    let parameters: NoiseParams = NOISE_PARAMETER.parse().unwrap();
    Builder::with_resolver(parameters, Box::new(TestResolver {}))
        .generate_keypair()
        .unwrap()
});

fn generate_first_two_messages() -> (Vec<u8>, Vec<u8>) {
    // build
    let parameters: NoiseParams = NOISE_PARAMETER.parse().unwrap();
    let initiator = snow::Builder::new(parameters.clone())
        // why not use the same as the responder :D
        .local_private_key(&NOISE_KEYPAIR.private)
        .remote_public_key(&NOISE_KEYPAIR.public)
        .build_initiator()
        .unwrap();
    let responder = snow::Builder::new(parameters)
        .local_private_key(&NOISE_KEYPAIR.private)
        .build_responder()
        .unwrap();

    // create exposing socket
    let (dialer_socket, listener_socket) = ExposingSocket::new_pair();
    let (dialer, listener) = (
        NoiseSocket::new(dialer_socket, initiator),
        NoiseSocket::new(listener_socket, responder),
    );

    let (dialer, listener) = (Handshake(dialer), Handshake(listener));

    let (dialer_result, listener_result) =
        block_on(join(dialer.handshake_1rt(), listener.handshake_1rt()));

    // take result
    let init_msg = dialer_result.unwrap().take_socket().written;
    let resp_msg = listener_result.unwrap().take_socket().written;

    (init_msg, resp_msg)
}

pub fn generate_corpus(gen: &mut libra_proptest_helpers::ValueGenerator) -> Vec<u8> {
    let (init_msg, resp_msg) = generate_first_two_messages();
    // choose a random one
    let strategy = proptest::arbitrary::any::<bool>();
    if gen.generate(strategy) {
        init_msg
    } else {
        resp_msg
    }
}

//
// Fuzzing
// =======
//
// - FakeSocket: used to quickly consume fuzzing data and dismiss socket writes.
// - fuzz_initiator: fuzzes the second message of the handshake, received by the initiator.
// - fuzz_responder: fuzzes the first message of the handshake, received by the responder.
//

struct FakeSocket<'a> {
    pub content: &'a [u8],
}
impl<'a> AsyncWrite for FakeSocket<'a> {
    fn poll_write(
        self: Pin<&mut Self>,
        _context: &mut Context,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _context: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    /// Attempt to close the channel. Cannot Fail.
    fn poll_close(self: Pin<&mut Self>, _context: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
impl<'a> AsyncRead for FakeSocket<'a> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        mut _context: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // if nothing left, just return 0
        if self.content.is_empty() {
            return Poll::Ready(Ok(0));
        }
        // if something left, return what's asked
        let to_read = std::cmp::min(buf.len(), self.content.len());
        buf[..to_read].copy_from_slice(&self.content[..to_read]);
        // update internal state
        self.content = &self.content[to_read..self.content.len()];
        // return length read
        Poll::Ready(Ok(to_read))
    }
}

pub fn fuzz_initiator(data: &[u8]) {
    // setup initiator
    let parameters: NoiseParams = NOISE_PARAMETER.parse().unwrap();
    let initiator = snow::Builder::new(parameters)
        .local_private_key(&NOISE_KEYPAIR.private)
        .remote_public_key(&NOISE_KEYPAIR.public)
        .build_initiator()
        .unwrap();
    // setup NoiseSocket
    let fake_socket = FakeSocket { content: data };
    let handshake = Handshake::new(fake_socket, initiator);
    // send a message, then read fuzz data
    let _ = block_on(handshake.handshake_1rt());
}

pub fn fuzz_responder(data: &[u8]) {
    // setup responder
    let parameters: NoiseParams = NOISE_PARAMETER.parse().unwrap();
    let responder = snow::Builder::new(parameters)
        .local_private_key(&NOISE_KEYPAIR.private)
        .build_responder()
        .unwrap();
    // setup NoiseSocket
    let fake_socket = FakeSocket { content: data };
    let handshake = Handshake::new(fake_socket, responder);
    // read fuzz data
    let _ = block_on(handshake.handshake_1rt());
}

//
// Tests
// =====
//
// To ensure fuzzers will not break, this test the fuzzers.
//

#[test]
fn test_noise_fuzzer() {
    let (init_msg, resp_msg) = generate_first_two_messages();
    fuzz_responder(&init_msg);
    fuzz_initiator(&resp_msg);
}
