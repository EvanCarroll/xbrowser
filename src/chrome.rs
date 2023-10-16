use std::path::PathBuf;
use crate::cookiejar::CookieJar;

use xbrowser::*;
mod cookie;

#[derive(Debug, Clone, Builder)]
pub struct Chrome {
	env: Env,
	#[builder(default)]
	profile: Option<String>,
	#[builder(default)]
	path_config: Option<PathBuf>,
}

impl Chrome {
	pub fn get_cookies_for_domain(&self, domain: &str) -> Result<CookieJar, CookieError> {
		let mut path = self.path_profile();
		path.push("Cookies");

		let con = rusqlite::Connection::open(&path)
			.unwrap();

		const Q: &'static str = r##"
			SELECT *
			FROM cookies
			WHERE host_key = ?;
		"##;

		let mut statement = con.prepare(Q)?;
		let mut jar = CookieJar::default();
		let cookies = statement.query_and_then([domain], |row| {
			row.try_into()
		} )?;

		for cookie in cookies {
			let cookie: cookie::ChromeCookie = cookie?;
			jar.add_cookie(cookie.name.clone(), Box::new(cookie));
		}

		Ok(jar)
	}


	fn path_root(&self) -> PathBuf {
		let p = self.path_config.clone().unwrap_or_else( || {
			let mut p = self.env.path_home();
			p.push( ".config" );
			p.push( "chromium" );
			p
		} );
		p.to_path_buf()
	}

	fn path_profile( &self ) -> PathBuf {
		let profile_name = self.profile.clone()
			.unwrap_or_else( || "Default".to_owned() );
		let mut p = self.path_root();
		p.push( profile_name );
		p
	}

}
