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

use clap::{
    Args,
    Parser,
    Subcommand    
};


#[derive(Debug, Parser)]
#[clap(name = "Yor", version, about = "Secure personal Key-Value storage system")]
pub struct YorParser {
    #[clap(subcommand)]
    pub command: Op,
}

#[derive(Debug, Subcommand)]
pub enum Op {
    #[clap(about = "Get the version and check for available updates")]
    Version,
    #[clap(about = "Information about the app.")]
    About,
    Set(SetCommand),
    Get(GetCommand),
    SetDb(SetDbCommand),
    Create(CreateCommand),
    Delete(DeleteCommand),
}

#[derive(Debug, Args)]
#[clap(about = "Set the given key and value")]
pub struct SetCommand {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Args)]
#[clap(about = "Get the value of a given key")]
pub struct GetCommand {
    pub key: String
}

#[derive(Debug, Args)]
#[clap(about = "Set the default database")]
pub struct SetDbCommand {
    // The name of the database
    pub name: String,
}

#[derive(Debug, Args)]
#[clap(about = "Create a new empty database")]
pub struct CreateCommand {
    // The name of the database
    pub name: String,
    
}

#[derive(Debug, Args)]
#[clap(about = "Delete the given database name")]
pub struct DeleteCommand {
    // The name of the database
    pub name: String,

}