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

use pickledb::{
    PickleDb,
    PickleDbDumpPolicy,
    SerializationMethod
};
use dirs;
use std::path::{
    Path,
    PathBuf
};
use std::fs;
use rpassword;
use anyhow::{Result, Context, bail, ensure};
use getrandom;
use orion::aead::SecretKey;
use colored::Colorize;

#[allow(dead_code)]
fn nonce() -> Result<[u8; 24]> {
    let mut result = [0u8; 24];
    getrandom::getrandom(&mut result).unwrap();
    Ok(result)
}

/// Get a SecretKey that will be used to encrypt/decrypt the data
/// 
/// # Arguments
/// - `password` - The password used to encrypt/decrypt the data
/// - `salt` - The salt used to strengthen the encryption
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



/// Data enum for handling data types
enum YorData {
    Bytes(Vec<u8>),
    Str(String)
}


pub fn create_db(path: &str) {

    PickleDb::new(
        path, 
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json
    );
}

pub fn get_password(prompt: &str) -> String {
    rpassword::prompt_password(prompt).unwrap()
}

pub fn load_db(path: &Path) -> PickleDb {
    if !path.exists() {
        create_db(&path.to_str().unwrap());
    }
    PickleDb::load_json(
        path, 
        PickleDbDumpPolicy::AutoDump
    ).unwrap()
}

fn init_config_db() {
    let env = dirs::home_dir().unwrap()
        .as_path().join(".yor");

    if !env.join("config").as_path().exists() {
        load_db(env.join("config").as_path()).set("db_name", &String::from("default")).unwrap();
    }
}

pub fn initialize_env() -> Result<()> {
    let home = dirs::home_dir().unwrap();
    let env = home.as_path().join(".yor");
    let db_path = env.as_path().join("db");

    fs::create_dir_all(env).unwrap();
    fs::create_dir_all(db_path).unwrap();
    init_config_db();

    Ok(())
}

/// Get the cpnfig data.
/// # Return (tuple)
/// - `key` - The password key of the given database
/// - `db_name` - The name of the database stored
pub fn get_config_data() -> PickleDb {
    let home = dirs::home_dir().unwrap();
    let cfg_path = home.as_path().join(".yor").join("config");
    load_db(cfg_path.as_path())
}

/// Get the db path from the environment given the name
/// 
/// # Arguments
/// - `name` - The name of the database
pub fn get_db_path(name: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap();
    let yor_path = home.as_path().join(".yor");
    let db_path = yor_path.as_path().join("db");
    db_path.join(&name)
}

/// Print all the database that can be found from the environment
/// directories
pub fn print_all_db() {
    let home = dirs::home_dir().unwrap();
    let db_path = home.as_path().join(".yor").join("db");

    if let Ok(entries) = fs::read_dir(db_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                // Here, `entry` is a `DirEntry`.
                println!("{}", entry.file_name().to_str().unwrap());
                
            }
        }
        
    }
}

/// Update or insert the given item
/// 
/// # Arguments
/// - `db_name` - The name of the database (default)
/// - `password` - The password used to encrypt/decrypt the data
/// - `key` - The given key for the value to store
/// - `value` - The given value for the key to store
pub fn upsert_item(db_name: String, password: String, key: String, value: String) {
    let mut db: PickleDb = load_db(&get_db_path(&db_name));

    // Set the Data to DataEnum that has 2 types, Vec<u8> and String since
    // I have no idea how to mutate types in rust.
    let mut data = YorData::Str(value.clone());
    if password  != "" {
        data = YorData::Bytes(encrypt(value, password).unwrap());
    } 
    match data {
        YorData::Bytes(d) => db.set(&key, &d).unwrap(),
        YorData::Str(d) => db.set(&key, &d).unwrap()
    }
   
}


/// Get the value of the given key with the password to decrypt the data
/// 
/// # Arguments
/// - `db_name` - The name of the database (default)
/// - `password` - The password used to encrypt/decrypt the data
/// - `key` - The given key for the value to get
#[allow(unused_assignments)]
pub fn get_item(db_name: String, key: String) -> String {
    let db: PickleDb = load_db(&get_db_path(&db_name));
    let exists = db.exists(&key);

    let mut data = String::from("");
    let mut raw = YorData::Str(String::from(""));

    if exists {
        
        // If the key exist, check if the data is bytes or string.
        // I'm aware that this specific code is a bad practice.
        let x = db.get::<String>(&key);
        if x.is_none() {
            raw = YorData::Bytes(db.get::<Vec<u8>>(&key).unwrap());
        }
        else {
            raw = YorData::Str(x.unwrap());
        }
    }
    match raw {
        YorData::Bytes(d) => {
            let mut tries = 1;
            let mut password = get_password("[yor] password for the key: ");
            let decrypted_data = decrypt(d, password);
   
            // Get the key three times, if it fails then exit
            while !decrypted_data.is_ok() {
                println!("{}", "Password is invalid. Pleae try again".truecolor(157, 123, 125));
                password = get_password("[yor] password for the key: ");
     
                tries += 1; 
                if tries >= 3 {
                    println!("{}", "Woah, chill out. Are you sure the password is correct?.".truecolor(157, 123, 125));
                    std::process::exit(1);
                }

            }
            data = String::from_utf8(decrypted_data.unwrap()).unwrap();
        },
        YorData::Str(d) => {
            data = d;
        }
    }
    return data;
}

/// Remove the given key
pub fn rem_item(db_name: &str, key: &str) -> Result<()> {
    let mut db: PickleDb = load_db(&get_db_path(&db_name));
    let exists = db.exists(&key);
    if !exists {
        println!("Key {} Not found, perhaps it deosn't exist at all?", key);
        std::process::exit(1);
    }
    db.rem(&key).unwrap();
    Ok(())
    
}
