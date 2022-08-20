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
#[clap(propagate_version = true)]
pub struct YorParser {
    #[clap(subcommand)]
    pub command: Op,
}

#[derive(Debug, Subcommand)]
pub enum Op {
    #[clap(about = "Information about the app.")]
    About,
    #[clap(about = "List all database avaialable")]
    ListDb,
    #[clap(about = "List all files avaialable from the file environment")]
    ListFiles,
    Set(SetCommand),
    Get(GetCommand),
    Rem(RemCommand),
    SetDb(SetDbCommand),
    ListKeys(ListKeysCommand),
    Create(CreateCommand),
    Delete(DeleteCommand),
    Clear(ClearCommand),
}

#[derive(Debug, Args)]
#[clap(about = "Set the given key and value")]
pub struct SetCommand {
    pub key: String,
    pub value: String,
    #[clap(short, long)]
    pub no_password: bool,
    #[clap(short, long)]
    pub r#type: Option<String>,
    
    
    #[clap(short, long)]
    pub db: Option<String>,
    
}

#[derive(Debug, Args)]
#[clap(about = "Get the value of a given key")]
pub struct GetCommand {
    pub key: String,
}

#[derive(Debug, Args)]
#[clap(about = "Set the default database")]
pub struct SetDbCommand {
    // The name of the database
    pub name: String,
    
}

#[derive(Debug, Args)]
#[clap(about = "Remove a key from the database")]
pub struct RemCommand {
    // The key to be remove
    pub key: String,
    
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

#[derive(Debug, Args)]
#[clap(about = "Clear the given environment")]
pub struct ClearCommand {
    pub name: String,
}

#[derive(Debug, Args)]
#[clap(about = "List all keys avaialable from the database")]
pub struct ListKeysCommand {
    #[clap(short, long)]
    pub db: Option<String>
}
