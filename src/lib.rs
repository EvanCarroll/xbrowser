use std::path::PathBuf;
use clap::ValueEnum;

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
	pub fn home_path( &self ) -> PathBuf {
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

pub fn read_bool( row: &sqlite::Row, col: &str ) -> bool {
	let res = row.read::<i64, _>(col);
	res != 0
}

pub fn read_int( row: &sqlite::Row, col: &str ) -> i64 {
	row.read::<i64, _>(col)
}

pub fn read_string( row: &sqlite::Row, col: &str ) -> String {
	row.read::<&str, _>(col).to_string()
}

pub fn read_vecu8( row: &sqlite::Row, col: &str ) -> Vec<u8> {
	row[col].clone().try_into().unwrap()
}

#[derive(Debug)]
pub enum CookieError {
	NotEncrypted(String),
	NoValue(String),
	Unsupported(String, String),
	Egress,
}
