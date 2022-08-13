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
use ylib;
use dialoguer::Confirm;
use clap::Parser;
use colored::Colorize;
mod args;


fn main() {
    ylib::initialize_env().unwrap();
    let a: args::YorParser = args::YorParser::parse();
    match a.command {
        args::Op::Get(v) => {
            let conf = ylib::get_config_data();
            let db_name = conf.get::<String>("db_name").unwrap();
            let db = ylib::load_db(&ylib::get_db_path(db_name.as_str()));
            if v.keys {
                println!("{:?}", db.get_all());
                std::process::exit(1);
            }
            let key = v.key.unwrap_or_else(|| { println!("No key prompted"); std::process::exit(1); });
            let data = ylib::get_item(db_name, key);
            println!("{}", data.truecolor(138, 172, 171));
        }
        args::Op::Set(v) => {
            let db = ylib::get_config_data();
            let db_name = db.get::<String>("db_name").unwrap();
            let mut pwd = db.get::<String>("db_key").unwrap_or(String::from(""));
            if pwd  == "" && !v.no_password {
                pwd = ylib::get_password();
            }
            ylib::upsert_item(db_name, pwd, v.key, v.value);
        }
        args::Op::SetDb(v) => {
            let mut db = ylib::get_config_data();
            let path = ylib::get_db_path(v.name.as_str());
         
            if !path.exists() {
                println!("Database: {} not found, perhaps it doesn't exist at all?", v.name.truecolor(172, 138, 140));
                std::process::exit(1);
            }
            db.set("db_name", &v.name).expect("Cannot set the database name");
            println!("Successfully set the database to: {}", v.name.truecolor(172, 169, 138));
        },
        args::Op::Rem(v) => {
            let db = ylib::get_config_data();
            let db_name = db.get::<String>("db_name").unwrap();
            ylib::rem_item(&db_name, &v.key).unwrap();
        }
        args::Op::Delete(v) => {
            let path = ylib::get_db_path(v.name.as_str());

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
            let path = ylib::get_db_path(v.name.as_str());
            if path.exists() {
                println!("It looks like database: {} is already created.", v.name.truecolor(172, 138, 140));
                std::process::exit(1);
            }
            ylib::create_db(path.to_str().unwrap());
        },
        args::Op::ListDb => {
            ylib::print_all_db();
        }
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
