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
use std::io::*;
use dirs;
use std::path::{
    Path,
    PathBuf
};
use std::fs;
use std::env;
mod crypto;

fn get_db_path(name: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap();
    let db_path = home.as_path().join("db");
    db_path.join(&name)
}


fn create_db(path: &str) {
    PickleDb::new(
        Path::new(path), 
        PickleDbDumpPolicy::AutoDump,
        SerializationMethod::Json
    );
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

    fs::create_dir_all(env).unwrap();
    fs::create_dir_all(db_path).unwrap();
    Ok(())
}

/// Get the session data stored in environment variables.
/// # Return (tuple)
/// - `key` - The password key of the given database
/// - `db_name` - The name of the database stored
pub fn get_session_data() -> (String, String) {
    let key = env::var("YOR_SESSION_KEY").unwrap_or(String::from(""));
    let db_name = env::var("YOR_SESSION_DB").unwrap_or(String::from("default"));
    (key, db_name)
}

/// Update or insert the given item
/// 
/// # Arguments
/// - `db_name` - The name of the database (default)
/// - `password` - The password used to encrypt/decrypt the data
/// - `key` - The given key for the value to store
/// - `value` - The given value for the key to store
pub fn upsert_item(db_name: &str, password: &str, key: &str, value: &str) {
    let mut db: PickleDb = load_db(&get_db_path(&db_name));
    let data = crypto::encrypt(value, password).unwrap();
    db.set(&key, &data).unwrap();
}


/// Get the value of the given key with the password to decrypt the data
/// 
/// # Arguments
/// - `db_name` - The name of the database (default)
/// - `password` - The password used to encrypt/decrypt the data
/// - `key` - The given key for the value to get
pub fn get_item(db_name: &str, password: &str, key: &str) -> String {
    let db: PickleDb = load_db(&get_db_path(&db_name));
    let raw = db.get::<Vec<u8>>(&key).unwrap();
    String::from_utf8(
        crypto::decrypt(raw, password).unwrap()
    ).unwrap()
}
