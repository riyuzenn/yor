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
use anyhow::{Result};

mod crypto;

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

pub fn get_password() -> String {
    rpassword::prompt_password("[yor] password: ").unwrap()
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

pub fn initialize_env() -> Result<()> {
    let home = dirs::home_dir().unwrap();
    let env = home.as_path().join(".yor");
    let db_path = env.as_path().join("db");

    if !env.join("config").as_path().exists() {
        load_db(env.join("config").as_path()).set("db_name", &String::from("default")).unwrap();
    }

    fs::create_dir_all(env).unwrap();
    fs::create_dir_all(db_path).unwrap();

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
        data = YorData::Bytes(crypto::encrypt(value, password).unwrap());
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
            let password = get_password();
            let decrypted_data = crypto::decrypt(d, password);
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
