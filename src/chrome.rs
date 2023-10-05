use std::path::PathBuf;
use chrono::{DateTime, offset::Utc};
use sqlite::State;
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
			let cookie: cookie::Cookie = row.try_into().unwrap();
			jar.add_cookie(cookie.name.clone(), Box::new(cookie));
		}

		jar
	}


	/// Convert from MS since 1601-01-01 to DateTime
	fn from_epoch( ts: i64 ) -> Option< DateTime<chrono::offset::Utc> > {

		if ts == 0 {
			return None
		}

		// Chrome Epoch is in 1601-01-01.. snowflake garbage
		// https://source.chromium.org/chromium/chromium/src/+/main:base/time/time.h;l=529;drc=131600edcd9395ffa1241050c486e8da3fbfda4f
		const UNIX_EPOCH_OFFSET: i64 = 11644473600;

		DateTime::from_timestamp(
			(ts / 1000000) - UNIX_EPOCH_OFFSET,
			0,
		)
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
