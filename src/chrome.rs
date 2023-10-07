use std::path::PathBuf;
use crate::cookiejar::CookieJar;

use browser_cookie::*;
mod cookie;

#[derive(Debug, Clone, Builder)]
pub struct Chrome {
	env: Env,
	#[builder(default)]
	path_profile: Option<PathBuf>
}

impl Chrome {
	pub fn get_cookies(&self, domain: &str) -> CookieJar {
		let mut path = self.path_profile.clone()
			.unwrap_or_else(|| self.default_profile_path() );
		path.push("Cookies");

		let con = sqlite::Connection::open(path)
			.unwrap();

		const Q: &'static str = r##"
			SELECT *
			FROM cookies
			WHERE host_key = ?;
		"##;

		let mut statement = con.prepare(Q).unwrap();

		statement.bind((1,domain)).unwrap();

		let mut jar = CookieJar::default();
		for row in statement.iter() {
			let row = row.unwrap();
			let cookie: cookie::ChromeCookie = row.try_into().unwrap();
			jar.add_cookie(cookie.name.clone(), Box::new(cookie));
		}

		jar
	}


	fn path_root(&self) -> PathBuf {
		let mut p = self.env.home_path();
		p.push( ".config" );
		p.push( "chromium" );
		p
	}

	fn default_profile_path( &self ) -> PathBuf {
		match self.env.os {
			Os::Linux => {
				let mut p = self.path_root();
				p.push( "Default" );
				p
			}
			_ => todo!("Other OSes")
		}
	}

}
