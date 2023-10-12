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

pub fn read_bool( row: &sqlite::Row, col: &str ) -> Result<bool, CookieError> {
	let val = row.try_read::<i64, _>(col).map_err( |e| CookieError::SqliteInterface(e) )?;
	Ok(val != 0)
}

pub fn read_int( row: &sqlite::Row, col: &str ) -> Result<i64, CookieError> {
	let val = row.try_read::<i64, _>(col).map_err( |e| CookieError::SqliteInterface(e) )?;
	Ok(val)
}

pub fn read_string( row: &sqlite::Row, col: &str ) -> Result<String, CookieError> {
	let val = row.try_read::<&str, _>(col).map_err( |e| CookieError::SqliteInterface(e) )?;
	Ok( val.to_string() )
}

pub fn read_vecu8( row: &sqlite::Row, col: &str ) -> Vec<u8> {
	row[col].clone().try_into().unwrap()
}

#[derive(Debug)]
pub enum CookieError {
	NotEncrypted(String),
	NoValue(String),
	EncryptionError(String, String),
	Egress,
	SqliteInterface(sqlite::Error)
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
