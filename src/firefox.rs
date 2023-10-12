use std::path::PathBuf;
use crate::cookiejar::CookieJar;
use cookie::*;

mod cookie;
use xbrowser::*;

#[derive(Debug, Clone, Builder)]
pub struct Firefox {
	env: Env,
	#[builder(default)]
	profile: Option<String>,
}

impl Firefox {
	pub fn get_cookies(&self, domain: &str) -> CookieJar {
		let mut path = self.path_profile();
		path.push("cookies.sqlite");

		let con = sqlite::Connection::open(&path)
			.unwrap();

		const Q: &'static str = r##"
			SELECT *
			FROM moz_cookies
			WHERE host = ?;
		"##;

		let mut statement = con.prepare(Q)
			.expect(&format!("Error preparing statement with {:?}", &path.clone().as_os_str()));
		statement.bind((1,domain)).unwrap();

		let mut jar = CookieJar::default();
		for row in statement.iter() {
			let row = row.unwrap();
			let cookie: FirefoxCookie = row.try_into().unwrap();
			jar.add_cookie(cookie.name.clone(), Box::new(cookie));
		}
		jar
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
