use std::path::PathBuf;
use crate::cookiejar::CookieJar;
use cookie::*;
use rusqlite::{params, Connection, Result};


mod cookie;
use xbrowser::*;

#[derive(Debug, Clone, Builder)]
pub struct Firefox {
	env: Env,
	#[builder(default)]
	profile: Option<String>,
}

impl Firefox {
	
	pub fn get_cookies_for_domain(&self, domain: &str) -> Result<CookieJar<cookie::FirefoxCookie>, CookieError> {
		let mut path = self.path_profile();
		path.push("cookies.sqlite");

		let con = rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
			.unwrap();

		const Q: &'static str = r##"
			SELECT *
			FROM moz_cookies
			WHERE host = ?;
		"##;
		
		let mut statement = con.prepare(Q)?;
		let mut jar : CookieJar<cookie::FirefoxCookie> = CookieJar::default();
		let cookies = statement.query_and_then([domain], |row| {
			row.try_into()
		} )?;

		for cookie in cookies {
			let cookie: cookie::FirefoxCookie = cookie?;
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
