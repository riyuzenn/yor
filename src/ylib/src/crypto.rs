/*
 *
 *  Copyright (C) 2022-present riyuzenn
 *  
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
*/

use anyhow::{Result, Context, bail, ensure};
use getrandom;
use orion::aead::SecretKey;


/// Get a SecretKey that will be used to encrypt/decrypt the data
/// 
/// # Arguments
/// - `password` - The password used to encrypt/decrypt the data
/// - `salt` - The salt used to strengthen the encryption
#[allow(dead_code)]
fn get_key_from_password(password: &str, salt: &[u8]) -> Result<SecretKey> {
    use orion::hazardous::stream::chacha20::CHACHA_KEYSIZE;
    use orion::kdf::{derive_key, Password, Salt};
    let password = Password::from_slice(password.as_bytes()).with_context(|| "Password error")?;
    let salt = Salt::from_slice(salt).with_context(|| "Salt is too short")?;
    let kdf_key = derive_key(&password, &salt, 15, 1024, CHACHA_KEYSIZE as u32)
        .with_context(|| "Could not derive key from password")?;
    let key = SecretKey::from_slice(kdf_key.unprotected_as_bytes())
        .with_context(|| "Could not convert key")?;
    Ok(key)
}

/// Encrypts the plaintext with the given password and returns the ciphertext. The nonce is generated at each call to strengthen the encryption. 
/// Otherwise there's a chance the key is weakened if the same nonce is used. 
/// The nonce is 24 byte (following the XCHACHA_NONCESIZE property). 
/// The ciphertext will be 40 bytes longer than the plaintext because of the XCHACHA_NONCESIZE + POLY1305_OUTSIZE size.
/// 
/// ## Format
/// 
/// {0,24: nonce} {24,: ciphertext} ...
/// 
/// ## Arguments
/// - `plaintext`: The plaintext to encrypt
/// - `password`: The password to use for the encryption
/// - `salt`: The salt to use for the encryption
/// 
/// ## Returns
/// The ciphertext
#[allow(dead_code)]
pub fn encrypt(plaintext: impl AsRef<[u8]>, password: impl AsRef<str>) -> Result<Vec<u8>> {
    use orion::hazardous::{
        aead::xchacha20poly1305::{seal, Nonce, SecretKey as XSecretKey},
        mac::poly1305::POLY1305_OUTSIZE,
        stream::xchacha20::XCHACHA_NONCESIZE,
    };
    // Fetch param as refs
    let plaintext = plaintext.as_ref();
    let password = password.as_ref();
    let mut nonce = [0u8; 24];
    getrandom::getrandom(&mut nonce).unwrap();

    // Get high-level API key
    let key = get_key_from_password(password, &nonce)?;
    // Convert high-level API key to low-level API key
    let key = XSecretKey::from_slice(key.unprotected_as_bytes()).with_context(|| "Key is invalid")?;

    // Create a Nonce struct from the generated nonce
    let nonce = Nonce::from_slice(&nonce).with_context(|| "Nonce is too short")?;

    // Get the output length
    let output_len = match plaintext.len().checked_add(XCHACHA_NONCESIZE + POLY1305_OUTSIZE) {
        Some(min_output_len) => min_output_len,
        None => bail!("Plaintext is too long"),
    };

    // Allocate a buffer for the output
    let mut output = vec![0u8; output_len];
    output[..XCHACHA_NONCESIZE].copy_from_slice(nonce.as_ref());

    // Encrypt the plaintext and add it to the end of output buffer
    seal(&key, &nonce, plaintext, None, &mut output[XCHACHA_NONCESIZE..])
        .with_context(|| "Could not convert key")?;

    Ok(output)
}


/// Decrypts the ciphertext with the given password and returns the plaintext. 
/// 
/// ## Arguments
/// - `ciphertext`: The ciphertext to decrypt
/// - `password`: The password to use for the decryption
/// 
/// ## Returns
/// The plaintext as bytes
#[allow(dead_code)]
pub fn decrypt(ciphertext: impl AsRef<[u8]>, password: impl AsRef<str>) -> Result<Vec<u8>> {
    use orion::aead::open;
    use orion::hazardous::stream::xchacha20::XCHACHA_NONCESIZE;

    let ciphertext = ciphertext.as_ref();
    let password = password.as_ref();

    ensure!(ciphertext.len() > XCHACHA_NONCESIZE, "Ciphertext is too short");

    // Get the key from the password and salt 
    let key = get_key_from_password(password, &ciphertext[..XCHACHA_NONCESIZE])?;
    open(&key, ciphertext).with_context(|| "Invalid key password")
}