/*
 *
 *  Copyright (c) 2022-present riyuzenn
 *
 *  this program is free software: you can redistribute it and/or modify
 *  it under the terms of the gnu general public license as published by
 *  the free software foundation, either version 3 of the license, or
 *  (at your option) any later version.
 *
 *  this program is distributed in the hope that it will be useful,
 *  but without any warranty; without even the implied warranty of
 *  merchantability or fitness for a particular purpose.  see the
 *  gnu general public license for more details.
 *
 *  you should have received a copy of the gnu general public license
 *  along with this program.  if not, see <https://www.gnu.org/licenses/>.
 *
*/

use anyhow::{bail, ensure, Context, Result};
use base64;
use colored::Colorize;
use dirs;
use getrandom;
use orion::aead::SecretKey;
use pickledb::{PickleDb, PickleDbDumpPolicy, SerializationMethod};
use rand::Rng;
use rpassword;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
    let key =
        XSecretKey::from_slice(key.unprotected_as_bytes()).with_context(|| "Key is invalid")?;

    // Create a Nonce struct from the generated nonce
    let nonce = Nonce::from_slice(&nonce).with_context(|| "Nonce is too short")?;

    // Get the output length
    let output_len = match plaintext
        .len()
        .checked_add(XCHACHA_NONCESIZE + POLY1305_OUTSIZE)
    {
        Some(min_output_len) => min_output_len,
        None => bail!("Plaintext is too long"),
    };

    // Allocate a buffer for the output
    let mut output = vec![0u8; output_len];
    output[..XCHACHA_NONCESIZE].copy_from_slice(nonce.as_ref());

    // Encrypt the plaintext and add it to the end of output buffer
    seal(
        &key,
        &nonce,
        plaintext,
        None,
        &mut output[XCHACHA_NONCESIZE..],
    )
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

    ensure!(
        ciphertext.len() > XCHACHA_NONCESIZE,
        "Ciphertext is too short"
    );

    // Get the key from the password and salt
    let key = get_key_from_password(password, &ciphertext[..XCHACHA_NONCESIZE])?;
    open(&key, ciphertext).with_context(|| "Invalid key password")
}

/// Data enum for handling data types
#[derive(Serialize, Deserialize)]
pub enum YorDataType {
    Bytes(Vec<u8>),
    Str(String),
}
#[derive(Serialize, Deserialize)]
pub struct YorData {
    pub y_data: YorDataType,
    pub y_type: String,
}

pub fn create_db(path: &str) -> PickleDb {
    PickleDb::new(
        path,
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json,
    )
}

pub fn get_password(prompt: &str) -> String {
    rpassword::prompt_password(prompt).unwrap()
}

pub fn load_db(path: &Path) -> Result<PickleDb> {
    PickleDb::load_json(path, PickleDbDumpPolicy::AutoDump)
        .with_context(|| "Database not found. Consider creating using `create`")
}

fn init_config_db() {
    let env = dirs::home_dir().unwrap().as_path().join(".yor");

    if !env.join("config").as_path().exists() {
        let mut db = load_db(env.join("config").as_path())
            .unwrap_or_else(|_| create_db(env.join("config").to_str().unwrap()));

        db.set("db_name", &String::from("default")).unwrap();
        db.set(
            "file_env",
            &String::from(env.join("files").to_str().unwrap()),
        )
        .unwrap();
    }
}

pub fn initialize_env() -> Result<()> {
    let home = dirs::home_dir().unwrap();
    let env = home.as_path().join(".yor");
    let db_path = env.as_path().join("db");
    let default_db = db_path.as_path().join("default");
    let file_path = env.as_path().join("files");

    fs::create_dir_all(env).unwrap();
    fs::create_dir_all(db_path).unwrap();
    fs::create_dir_all(file_path).unwrap();
    init_config_db();

    // Initialize default db

    load_db(&default_db).unwrap_or_else(|_| create_db(&default_db.to_str().unwrap()));

    Ok(())
}

/// Get the config data.
/// # Return (tuple)
/// - `key` - The password key of the given database
/// - `db_name` - The name of the database stored
pub fn get_config_data() -> PickleDb {
    let home = dirs::home_dir().unwrap();
    let cfg_path = home.as_path().join(".yor").join("config");
    load_db(cfg_path.as_path()).unwrap_or_else(|_| {
        println!(
            "{}",
            "Database not found. Consider creating using `create`".truecolor(157, 123, 125)
        );
        std::process::exit(1);
    })
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
    let conf = get_config_data();
    let default_db_name = conf.get::<String>("db_name").unwrap();

    if let Ok(entries) = fs::read_dir(db_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let mut db_name = String::from(entry.file_name().to_str().unwrap());
                if db_name == default_db_name {
                    db_name.push_str(&" (current)".truecolor(164, 141, 110).to_string());

                    // db_name += &" (current)".truecolor(164, 141, 110).to_string();
                }
                println!("{}", db_name.truecolor(172, 138, 172));
            }
        }
    }
}
/// Print all the files that can be found from the environment
/// directories
pub fn print_all_files() {
    let home = dirs::home_dir().unwrap();
    let db_path = home.as_path().join(".yor").join("files");

    if let Ok(entries) = fs::read_dir(db_path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let filename = String::from(entry.file_name().to_str().unwrap());

                println!("{}", filename.truecolor(172, 138, 172));
            }
        }
    }
}

fn encrypt_file(path: &str, key: &str) -> Vec<u8> {
    let data = fs::read(Path::new(path)).unwrap();
    encrypt(base64::encode(data), key).unwrap()
}
fn write_file(path: &str, data: String) -> Result<()> {
    let path = Path::new(path);
    let raw = base64::decode(data).unwrap();
    fs::write(path, raw).with_context(|| "Cannot write the file")?;
    Ok(())
}
#[allow(dead_code)] // for future use
fn gen_random(len: usize) -> String {
    rand::thread_rng()
        .sample_iter::<char, _>(rand::distributions::Standard)
        .take(len)
        .collect()
}
#[allow(dead_code)] // for future use
fn generate_file_session(filename: &str) -> String {
    let random = gen_random(5);
    format!("{0}-{1}", filename, random).to_string()
}
fn split_type(string: &str) -> Vec<&str> {
    string.split("/").collect()
}

/// Update or insert the given item
///
/// # Arguments
/// - `db_name` - The name of the database (default)
/// - `password` - The password used to encrypt/decrypt the data
/// - `key` - The given key for the value to store
/// - `value` - The given value for the key to store
pub fn upsert_item(db_name: String, password: String, key: String, value: String, r#type: String) {
    let supported_types = ["image", "video", "file", "data"];
    let file_types = ["video", "file", "image"];
    if !supported_types.iter().any(|&i| i == split_type(&r#type)[0]) {
        println!("{}", r#type);
        println!("Data type is not supported");
        std::process::exit(1);
    }

    let mut db: PickleDb = load_db(&get_db_path(&db_name)).unwrap_or_else(|_| {
        println!(
            "{}",
            "Database not found. Consider creating using `create`".truecolor(157, 123, 125)
        );
        std::process::exit(1);
    });

    // Set the Data to DataEnum that has 2 types, Vec<u8> and String since
    // I have no idea how to mutate types in rust.
    let mut data = YorDataType::Str(value.clone());
    let mut _type = r#type;
    if password != "" {
        data = YorDataType::Bytes(encrypt(value.clone(), password.clone()).unwrap());
        if split_type(&_type)[1] == "str" {
            _type = String::from("data/byte");
        }
    }
    /*
    match data {
        YorDataType::Bytes(d) => db.set(&key, &d).unwrap(),
        YorDataType::Str(d) => db.set(&key, &d).unwrap()
    }
    */

    let mut yordata = YorData {
        y_data: data,
        y_type: _type.clone(),
    };
    if file_types.iter().any(|&i| i == split_type(&_type)[0]) {
        if password != "" {
            yordata.y_data = YorDataType::Bytes(encrypt_file(&value, &password));
        }
        let d = fs::read(Path::new(&value)).unwrap();
        yordata.y_data = YorDataType::Str(base64::encode(d));
    }
    db.set(&key, &yordata).unwrap();
}

/// Get the value of the given key with the password to decrypt the data
///
/// # Arguments
/// - `db_name` - The name of the database (default)
/// - `password` - The password used to encrypt/decrypt the data
/// - `key` - The given key for the value to get
#[allow(unused_assignments)]
pub fn get_item(db_name: String, key: String) -> String {
    let file_types = ["video", "file", "image"];
    let db: PickleDb = load_db(&get_db_path(&db_name)).unwrap_or_else(|_| {
        println!("Database not found. Consider creating using `create`");
        std::process::exit(1);
    });
    let exists = db.exists(&key);

    let mut data = String::from("");
    let mut raw = YorDataType::Str(String::from(""));
    let mut y_type = String::from("data/str");

    if exists {
        let yor = db.get::<YorData>(&key).unwrap();
        raw = yor.y_data;
        y_type = yor.y_type;
    }
    let splitted_type = split_type(&y_type);
    let configdb = get_config_data();
    let pathstr = configdb.get::<String>("file_env").unwrap();
    let mut path = Path::new(&pathstr).join(format!("{}.{}", &key, splitted_type[1]));

    if splitted_type[1] == "bin" {
        path = Path::new(&pathstr).join(&key);
    }

    match raw {
        YorDataType::Bytes(d) => {
            let mut tries = 1;
            let mut password = get_password("[yor] password for the key: ");

            let decrypted_data = decrypt(d, password);

            // Get the key three times, if it fails then exit
            while !decrypted_data.is_ok() {
                println!(
                    "{}",
                    "Password is invalid. Pleae try again".truecolor(157, 123, 125)
                );
                password = get_password("[yor] password for the key: ");

                tries += 1;
                if tries >= 3 {
                    println!(
                        "{}",
                        "Woah, chill out. Are you sure the password is correct?."
                            .truecolor(157, 123, 125)
                    );
                    std::process::exit(1);
                }
            }

            println!("{:?}", splitted_type);
            if file_types.iter().any(|&i| i == splitted_type[0]) {
                // writing the file

                write_file(
                    path.to_str().unwrap(),
                    String::from_utf8(decrypted_data.unwrap()).unwrap(),
                )
                .unwrap();
                data = String::from(path.to_str().unwrap());
            } else {
                data = String::from_utf8(decrypted_data.unwrap()).unwrap();
            }
        }
        YorDataType::Str(d) => {
            if file_types.iter().any(|&i| i == splitted_type[0]) {
                write_file(&path.to_str().unwrap(), d).unwrap();
                data = String::from(path.to_str().unwrap());
            } else {
                data = d;
            }
        }
    }
    return data;
}

/// Remove the given key
pub fn rem_item(db_name: &str, key: &str) -> Result<()> {
    let mut db = load_db(&get_db_path(&db_name)).unwrap_or_else(|_| {
        println!("Database not found. Consider creating using `create`");
        std::process::exit(1);
    });

    let exists = db.exists(&key);
    if !exists {
        println!("Key {} Not found, perhaps it deosn't exist at all?", key);
        std::process::exit(1);
    }
    db.rem(&key)?;
    Ok(())
}
