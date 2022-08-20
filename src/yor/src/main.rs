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

use std::fs;
use dialoguer::Confirm;
use clap::Parser;
use colored::Colorize;
mod args;
mod lib;

fn main() {
    lib::initialize_env().unwrap();
    let a: args::YorParser = args::YorParser::parse();
    match a.command {
        args::Op::Get(v) => {
            let conf = lib::get_config_data();
            let db_name = conf.get::<String>("db_name").unwrap();
            let data = lib::get_item(db_name, v.key);
            println!("{}", data.truecolor(138, 172, 171));
        }
        args::Op::Set(v) => {
            let db = lib::get_config_data();
            let mut db_name = db.get::<String>("db_name").unwrap();
            let mut pwd = db.get::<String>("db_key").unwrap_or(String::from(""));
            let r#type = v.r#type.unwrap_or("data/str".to_string());
            if pwd  == "" && !v.no_password {
                pwd = lib::get_password("[yor] password to be set: ");
            }
            if !v.db.is_none() {
                db_name = v.db.unwrap();
            }
            
            lib::upsert_item(db_name, pwd, v.key, v.value, r#type);
        }
        args::Op::SetDb(v) => {
            let mut db = lib::get_config_data();
            let path = lib::get_db_path(v.name.as_str());
         
            if !path.exists() {
                println!("Database: {} not found, perhaps it doesn't exist at all?", v.name.truecolor(172, 138, 140));
                std::process::exit(1);
            }
            db.set("db_name", &v.name).expect("Cannot set the database name");
            println!("Successfully set the database to: {}", v.name.truecolor(172, 169, 138));
        },
        args::Op::Rem(v) => {
            let db = lib::get_config_data();
            let db_name = db.get::<String>("db_name").unwrap();
            lib::rem_item(&db_name, &v.key).unwrap();
        }
        args::Op::Delete(v) => {
            let path = lib::get_db_path(v.name.as_str());

            if !path.exists() {
                println!("Database {} doesn't exist at all", v.name.truecolor(172, 138, 140));
                std::process::exit(1);
            }
            
            if Confirm::new().with_prompt(
                format!(
                    "Are you sure you want to delete: {}? (action can't be undone)", v.name
                )
            ).interact().unwrap() {
                fs::remove_file(path).expect("Failed to remove the file");   
                println!("Database: {} is removed.", v.name.truecolor(172, 138, 140));
            } else {
                println!("{}", "Ignoring the deletion request.".truecolor(172, 138, 140));
            }
            
        },
        args::Op::Create(v) => {
            let path = lib::get_db_path(v.name.as_str());
            if path.exists() {
                println!("It looks like database: {} is already created.", v.name.truecolor(172, 138, 140));
                std::process::exit(1);
            }
            lib::create_db(path.to_str().unwrap());
        },
        args::Op::Clear(v) => {
            let env = dirs::home_dir().unwrap()
                .as_path().join(".yor");

            let dir = env.join(&v.name);
            if !dir.exists() {
                println!("Cannot clear environment: `{}`. Not found", v.name);
                std::process::exit(1);
            } 
            // Delete & Create the directory instead of deleting all the files
            fs::remove_dir_all(dir.clone()).unwrap();
            fs::create_dir_all(dir).unwrap();
        },
        args::Op::ListKeys(v) => {
            let conf = lib::get_config_data();
            let mut db_name = conf.get::<String>("db_name").unwrap();
            if !v.db.is_none() {
                db_name = v.db.unwrap();
            }

            let db = lib::load_db(&lib::get_db_path(db_name.as_str())).unwrap_or_else(|_| {
                println!("{}", "Database not found. Consider creating using `create`".truecolor(157, 123, 125));
                std::process::exit(1);
            });

            for key in db.get_all() {
                let db = db.get::<lib::YorData>(&key).unwrap();
                let mut data_type = db.y_type; 
                if data_type == "bytes" {
                    data_type = "password protected".to_string();
                }
                println!("{} ({})", key.truecolor(172, 138, 172), data_type.truecolor(172, 169, 138));
            }  
             
        },
        args::Op::ListDb => lib::print_all_db(),
        args::Op::ListFiles => lib::print_all_files(),
        args::Op::About => about()
        
    }
}

fn about() {
    let ascii = concat!(
        "▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄\n",
        "█░██░█▀▄▄▀█░▄▄▀█\n",
        "█░▀▀░█░██░█░▀▀▄█\n",
        "█▀▀▀▄██▄▄██▄█▄▄█\n",
        "▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀\n",
    );
    println!("\n{}", ascii.truecolor(172, 138, 172));
    println!("{}{}", "Yor v".truecolor(198, 166, 121), env!("CARGO_PKG_VERSION").truecolor(138, 172, 171));
    println!("{}", "─".repeat(16).truecolor(138, 152, 172));
    println!("{}", "Yet another secure personal key-value storage vault\nfor folks who store sensitive information.\n".truecolor(160, 160, 160))
}
