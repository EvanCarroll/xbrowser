use std::path::PathBuf;

use crate::cookiejar::CookieJar;
mod cookie;
use cookie::*;
use xbrowser::*;

#[derive(Debug, Clone, Builder)]
pub struct Firefox {
	env: Env,
	#[builder(default)]
	profile: Option<String>,
}

impl Firefox {
	
	pub fn get_all_cookies(&self) -> Result<Vec<FirefoxCookie>, CookieError> {
		let mut path = self.path_profile();
		path.push("cookies.sqlite");

		let path = format!("file://{}?mode=ro&immutable=1", &path.display());
		let con = rusqlite::Connection::open_with_flags(
			path,
			rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI
		).unwrap();

		const Q: &'static str = r##"
			SELECT *
			FROM moz_cookies;
		"##;

		
		let mut statement = con.prepare(Q)?;
		let cookies = statement.query_and_then([], |row| {
			row.try_into()
		} )?;

		let mut vec = Vec::new();
		for cookie in cookies {
			let cookie: cookie::FirefoxCookie = cookie?;
			vec.push(cookie);
		}

		Ok(vec)
	}
	
	pub fn get_cookies_for_domain(&self, domain: &str) -> Result<CookieJar<FirefoxCookie>, CookieError> {
		let mut path = self.path_profile();
		path.push("cookies.sqlite");

		let path = format!("file://{}?mode=ro&immutable=1", &path.display());
		let con = rusqlite::Connection::open_with_flags(
			path,
			rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI
		).unwrap();

		const Q: &'static str = r##"
			SELECT *
			FROM moz_cookies
			WHERE host = ?;
		"##;
		
		let mut statement = con.prepare(Q)?;
		let mut jar : CookieJar<FirefoxCookie> = CookieJar::default();
		let cookies = statement.query_and_then([domain], |row| {
			row.try_into()
		} )?;

		for cookie in cookies {
			let cookie: FirefoxCookie = cookie?;
			jar.add_cookie(cookie.name.clone(), Box::new(cookie));
		}

		Ok(jar)
	}

	fn path_profile(&self) -> PathBuf {
		use ini::Ini;

		let conf = Ini::load_from_file( self.path_install_ini() ).unwrap();
		let profile_name = self.profile.clone()
			.unwrap_or_else( || {
				match conf.len() {
					2 => {
						let mut sec       = conf.sections();
						// the None section
						let _outside      = sec.next();
						let sec           = sec.next().unwrap();
						conf.get_from(sec, "Default").unwrap().to_string()
					}
					_ => todo!("{} Items in installs.ini", conf.len())
				}
			} );
		let mut p = self.path_root();
		p.push( profile_name );
		p
	}

	fn path_root(&self) -> PathBuf {
		let mut p = self.env.path_home();
		p.push( ".mozilla" );
		p.push( "firefox" );
		p
	}

	// fn path_profiles_ini(&self) -> PathBuf {
	// 	match self.env.os {
	// 		Os::Linux => {
	// 			let mut p = self.path_root();
	// 			p.push( "profiles.ini" );
	// 			p
	// 		}
	// 		_ => todo!("Other OSes")
	// 	}
	// }

	// path to install_ini
	fn path_install_ini(&self) -> PathBuf {
		match self.env.os {
			Os::Linux => {
				let mut p = self.path_root();
				p.push( "installs.ini" );
				p
			}
			_ => todo!("Other OSes")
		}
	}

}
