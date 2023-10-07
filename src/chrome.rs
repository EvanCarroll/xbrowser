use std::path::PathBuf;
use crate::cookiejar::CookieJar;

use browser_cookie::*;
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
	pub fn get_cookies(&self, domain: &str) -> CookieJar {
		let mut path = self.path_profile();
		path.push("Cookies");

		let con = sqlite::Connection::open(&path)
			.unwrap();

		const Q: &'static str = r##"
			SELECT *
			FROM cookies
			WHERE host_key = ?;
		"##;

		let mut statement = con.prepare(Q)
			.expect(&format!("Error preparing statement with {:?}", &path.clone().as_os_str()));
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
