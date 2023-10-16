#![feature(inherent_associated_types)]
use std::path::PathBuf;
use clap::ValueEnum;
use std::cmp::Ordering;

#[derive(PartialEq, Debug, Clone, Copy, ValueEnum)]
pub enum Os { Win, Linux, Osx }

#[derive(PartialEq, Debug, Clone, Copy, ValueEnum)]
pub enum Browser { Chrome, Chromium, Firefox, Safari }

// Argument for functions
#[derive(Clone, Debug)]
pub struct Env {
	pub os: Os,
	pub browser: Browser,
	pub user: String
}

impl Env {
	pub fn path_home( &self ) -> PathBuf {
		match self.os {
			Os::Linux => {
				let mut p = PathBuf::new();
				p.push( "/" );
				p.push( "home" );
				p.push( self.user.clone() );
				p
			}
			 _ => todo!("Impl home lookup for other operating systems")
		}
	}
}

pub fn read_bool( row: &rusqlite::Row, col: &str ) -> Result<bool, CookieError> {
	let val: bool = row.get(col)?;
	Ok(val)
}

pub fn read_int( row: &rusqlite::Row, col: &str ) -> Result<i64, CookieError> {
	let val: i64 = row.get(col)?;
	Ok(val)
}

pub fn read_string( row: &rusqlite::Row, col: &str ) -> Result<String, CookieError> {
	let val: String = row.get(col)?;
	Ok( val )
}

pub fn read_vecu8( row: &rusqlite::Row, col: &str ) -> Result<Vec<u8>, CookieError> {
	let val: Vec<u8> = row.get(col)?;
	Ok( val )
}

use thiserror::Error;
#[derive(Error, Debug)]
pub enum CookieError {
	#[error("Can not decrypt {0}")]
	NotEncrypted(String),
	#[error("No value for key {0}")]
	NoValue(String),
	
	#[error("Decryption error")]
	Decryption,

	#[error("Unsupported encryption {0}")]
	ChromeUnsupportedEncryption(String),

	#[error("LibSecret")]
	LibSecret,

	#[error("SQLite")]
	RuSqlite(#[from] rusqlite::Error),
}

pub trait Cookie: std::fmt::Debug {
	fn name(&self) -> String;	
	fn value(&self) -> String;	
}

impl Eq for dyn Cookie {}

impl PartialEq for  dyn Cookie {
    fn eq(&self, other: &Self) -> bool {
        self.name().eq(&other.name())
    }
}

impl PartialOrd for dyn Cookie {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name().partial_cmp(&other.name())
    }
}

impl Ord for dyn Cookie {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(&other.name())
    }
}
