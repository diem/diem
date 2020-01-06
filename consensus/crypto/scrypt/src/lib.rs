use std::{mem::size_of, os::raw::c_int};

#[link(name = "scrypt-kdf", kind = "static")]
extern "C" {
    fn crypto_scrypt(
        passwd: *const u8,
        passwdlen: usize,
        salt: *const u8,
        saltlen: usize,
        n: u64,
        r: u32,
        p: u32,
        buf: *mut u8,
        buflen: usize,
    ) -> c_int;
}

#[derive(Clone, Copy, Debug)]
pub struct ScryptParams {
    /// Number of iterations
    pub n: u64,
    /// Block size for the underlying hash
    pub r: u32,
    /// Parallelization factor
    pub p: u32,
}

impl ScryptParams {
    pub fn new(n: u64, r: u32, p: u32) -> ScryptParams {
        assert!(r > 0);
        assert!(p > 0);
        assert!(n > 0);
        assert!(
            size_of::<usize>() >= size_of::<u32>() || (r <= std::u32::MAX && p < std::u32::MAX)
        );

        ScryptParams { n, r, p }
    }
}

pub fn scrypt(passwd: &[u8], salt: &[u8], params: &ScryptParams, output: &mut [u8]) {
    unsafe {
        crypto_scrypt(
            passwd.as_ptr(),
            passwd.len(),
            salt.as_ptr(),
            passwd.len(),
            params.n,
            params.r,
            params.p,
            output.as_mut_ptr(),
            output.len(),
        );
    }
}
/*
Implement scrypt call in litecoin
N = 1024;
r = 1;
p = 1;
salt is the same 80 bytes as the input
output is 256 bits (32 bytes)

https://litecoin.info/index.php/Scrypt
*/
pub fn scrypt_1024_1_1_256(input: &[u8], output: &mut [u8]) {
    assert_eq!(32, output.len());
    let param = ScryptParams::new(1024, 1, 1);
    scrypt(input, input, &param, output);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_crypto_scrypt() {
        let passwd = "1234";
        let mut output = vec![0u8; 32];
        let params = ScryptParams::new(128, 80240, 1);
        scrypt(passwd.as_bytes(), passwd.as_bytes(), &params, &mut output);
    }

    #[test]
    fn test_scrypt_1024_1_1_256() {
        let passwd = "1234";
        let mut output = vec![0u8; 32];
        scrypt_1024_1_1_256(passwd.as_bytes(), &mut output);
    }
}
