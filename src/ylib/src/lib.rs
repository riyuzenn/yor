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
use std::path::Path;
use std::fs;
mod crypto;

pub fn create_db(path: &str) {
    let db = PickleDb::new(
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

pub fn initialize_env() {
    let mut env = dirs::home_dir().unwrap();
    env.push(".yor");
    fs::create_dir_all(env).unwrap();

}