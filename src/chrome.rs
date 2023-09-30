use std::path::Path;
use chrono::{DateTime, offset::Utc};
use sqlite::State;
use crate::cookiejar::CookieJar;

mod cookie;

pub fn get_cookies(path: &Path, domain: &str) -> CookieJar {
	let con = sqlite::Connection::open(path)
		.unwrap();

	const Q: &'static str = r##"
		SELECT *
		FROM cookies
		WHERE host_key = ?;
	"##;

	let mut statement = con.prepare(Q).unwrap();

	statement.bind((1,domain)).unwrap();

	let mut jar = crate::cookiejar::CookieJar::default();
	while let Ok(State::Row) = statement.next() {
		let mut cb = cookie::CookieBuilder::default();
		cb.name( statement.read::<String, _>("name").unwrap() );
		cb.encrypted_value( statement.read::<Vec<u8>, _>("encrypted_value").unwrap() );
		cb.path( read_string(&statement, "path") );
		cb.is_secure( read_bool(&statement, "is_secure") );
		cb.is_httponly( read_bool(&statement, "is_httponly") );
		cb.has_expires( read_bool(&statement, "has_expires") );
		cb.is_persistent( read_bool(&statement, "is_persistent") );
		cb.is_same_party( read_bool(&statement, "is_same_party") );
		cb.priority( read_int(&statement, "priority") );
		cb.samesite( (read_int(&statement, "samesite") as i8).into() );
		cb.source_scheme( (read_int(&statement, "source_scheme") as u8).into() );

		// Store as Option where "" is None
		cb.value( Some(read_string(&statement, "value")).filter(|s| !s.is_empty()) );
		cb.source_port( read_int(&statement, "source_port") as u32 );

		{
			let ts = read_int(&statement, "creation_utc");
			cb.creation_utc( from_chrome_epoch(ts).unwrap() );
		}
		
		{
			let ts = read_int(&statement, "last_access_utc");
			cb.last_access_utc( from_chrome_epoch(ts).unwrap() );
		}
		
		{
			let ts = read_int(&statement, "last_update_utc");
			cb.last_update_utc( from_chrome_epoch(ts).unwrap() );
		}
	
		{
			let ts = read_int(&statement, "expires_utc");
			cb.expires_utc( from_chrome_epoch(ts) );
		}


		let cookie = cb.build().unwrap();
		
		// If has_expires is set, ensure that expires_utc is also set
		if cookie.has_expires {
			assert!(matches!( cookie.expires_utc, Some(_) ));
		}
		// If has_expires is *NOT* set, ensure that expires_utc is also NOT set
		else {
			assert!(matches!( cookie.expires_utc, None ));
		}

		jar.add_cookie(cookie.name.clone(), Box::new(cookie));
	}

	jar
}


/// Convert from MS since 1601-01-01 to DateTime
fn from_chrome_epoch( ts: i64 ) -> Option< DateTime<chrono::offset::Utc> > {

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

fn read_bool(statement: &sqlite::Statement, col: &str ) -> bool {
	let res = statement.read::<i64, _>(col).unwrap();
	res != 0
}

fn read_int(statement: &sqlite::Statement, col: &str ) -> i64 {
	statement.read::<i64, _>(col).unwrap()
}

fn read_string(statement: &sqlite::Statement, col: &str ) -> String {
	statement.read::<String, _>(col).unwrap()
}
