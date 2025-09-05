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

use clap::Parser;
use colored::Colorize;
use dialoguer::Confirm;
use std::fs;
mod args;
mod yor;

fn main() {
    yor::initialize_env().unwrap();
    let a: args::YorParser = args::YorParser::parse();
    match a.command {
        args::Op::Get(v) => {
            let conf = yor::get_config_data();
            let db_name = conf.get::<String>("db_name").unwrap();
            let data = yor::get_item(db_name, v.key, v.out);
            println!("{}", data.truecolor(138, 172, 171));
        }
        args::Op::Set(v) => {
            let db = yor::get_config_data();
            let mut db_name = db.get::<String>("db_name").unwrap();
            let mut pwd = db.get::<String>("db_key").unwrap_or(String::from(""));
            let r#type = v.r#type.unwrap_or("data/str".to_string());
            if pwd == "" && !v.no_password {
                let _pwd = yor::get_password("[yor] password to be set: ");
                if _pwd != "" {
                    let _confirm_pwd = yor::get_password("[yor] confirm password: ");
                    if _pwd == _confirm_pwd {
                        pwd = _pwd
                    } else {
                        println!("{}", "Password does not match.".truecolor(157, 123, 125));
                        std::process::exit(1);
                    }
                } else {

                    pwd = _pwd

                }
            }
            if !v.db.is_none() {
                db_name = v.db.unwrap();
            }

            yor::upsert_item(db_name, pwd, v.key, v.value, r#type);
        }
        args::Op::SetDb(v) => {
            let mut db = yor::get_config_data();
            let path = yor::get_db_path(v.name.as_str());

            if !path.exists() {
                println!(
                    "Database: {} not found, perhaps it doesn't exist at all?",
                    v.name.truecolor(172, 138, 140)
                );
                std::process::exit(1);
            }
            db.set("db_name", &v.name)
                .expect("Cannot set the database name");
            println!(
                "Successfully set the database to: {}",
                v.name.truecolor(172, 169, 138)
            );
        }
        args::Op::Rem(v) => {
            let db = yor::get_config_data();
            let db_name = db.get::<String>("db_name").unwrap();
            if Confirm::new()
                .with_prompt(format!(
                    "Are you sure you want to remove: {}? (action can't be undone)",
                    v.key
                ))
                .interact()
                .unwrap()
            {
                yor::rem_item(&db_name, &v.key).unwrap();
                println!(
                    "Key: {} from Database: {} is successfully removed.", 
                    v.key.truecolor(172, 138, 140),
                    db_name.truecolor(172,138,140)
                );
            } else {
                println!(
                    "{}",
                    "Ignoring the key removal request.".truecolor(172, 138, 140)
                );
            }
        }
        args::Op::Delete(v) => {
            let path = yor::get_db_path(v.name.as_str());

            if !path.exists() {
                println!(
                    "Database {} doesn't exist at all",
                    v.name.truecolor(172, 138, 140)
                );
                std::process::exit(1);
            }

            if Confirm::new()
                .with_prompt(format!(
                    "Are you sure you want to delete: {}? (action can't be undone)",
                    v.name
                ))
                .interact()
                .unwrap()
            {
                fs::remove_file(path).expect("Failed to remove the file");
                println!("Database: {} is removed.", v.name.truecolor(172, 138, 140));
            } else {
                println!(
                    "{}",
                    "Ignoring the deletion request.".truecolor(172, 138, 140)
                );
            }
        }
        args::Op::Create(v) => {
            let path = yor::get_db_path(v.name.as_str());
            if path.exists() {
                println!(
                    "It looks like database: {} is already created.",
                    v.name.truecolor(172, 138, 140)
                );
                std::process::exit(1);
            }
            yor::create_db(path.to_str().unwrap());
        }
        args::Op::Clear(v) => {
            let env = dirs::home_dir().unwrap().as_path().join(".yor");

            let dir = env.join(&v.name);
            if !dir.exists() {
                println!("Cannot clear environment: `{}`. Not found", v.name);
                std::process::exit(1);
            }
            // Delete & Create the directory instead of deleting all the files
            fs::remove_dir_all(dir.clone()).unwrap();
            fs::create_dir_all(dir).unwrap();
        }
        args::Op::Ls(v) => {
            let conf = yor::get_config_data();
            let mut db_name = conf.get::<String>("db_name").unwrap();
            if !v.db.is_none() {
                db_name = v.db.unwrap();
            }

            let db = yor::load_db(&yor::get_db_path(db_name.as_str())).unwrap_or_else(|_| {
                println!(
                    "{}",
                    "Database not found. Consider creating using `create`".truecolor(157, 123, 125)
                );
                std::process::exit(1);
            });

            for key in db.get_all() {
                let db = db.get::<yor::YorData>(&key).unwrap();
                let mut data_type = db.y_type;
                if data_type == "bytes" {
                    data_type = "password protected".to_string();
                }
                println!(
                    "{} ({})",
                    key.truecolor(172, 138, 172),
                    data_type.truecolor(172, 169, 138)
                );
            }
        }
        args::Op::LoadEnv => yor::load_env(),
        args::Op::LsDb => yor::print_all_db(),
        args::Op::LsFile => yor::print_all_files(),
        args::Op::About => about(),
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
    println!(
        "{}{}",
        "Yor v".truecolor(198, 166, 121),
        env!("CARGO_PKG_VERSION").truecolor(138, 172, 171)
    );
    println!("{}", "─".repeat(16).truecolor(138, 152, 172));
    println!("{}", "Yet another secure personal key-value storage vault\nfor folks who store sensitive information.\n".truecolor(160, 160, 160))
}
