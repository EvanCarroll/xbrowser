use chrono::{DateTime, offset::Utc};
use sqlite::State;
use crate::cookiejar::CookieJar;

use browser_cookie::*;
use super::Firefox;

#[derive(Debug, Builder)]
pub struct Cookie {
	pub id: u64,
	pub origin_attributes: String,
	pub name: String,
	pub value: String,
	pub host: String,
	pub path: String,
	pub expiry: DateTime<Utc>,
	pub last_accessed: DateTime<Utc>,
	pub creation_time: DateTime<Utc>,
	pub is_secure: bool,
	pub is_http_only: bool,
	pub in_browser_element: bool,
	pub same_site: bool,
	pub raw_same_site: bool,
	pub scheme_map: bool,
}

impl crate::cookiejar::Cookie for Cookie {
	fn name (&self) -> String { self.name.clone() }
	fn value (&self) -> String { self.value.clone() }
}

impl TryFrom<sqlite::Row> for Cookie {
	type Error = CookieError;
	fn try_from( row: sqlite::Row ) -> Result<Cookie, Self::Error> {
		let mut cb = CookieBuilder::default();
		cb.id( read_int(&row, "id") as u64 );
		cb.origin_attributes( read_string(&row, "originAttributes") );
		cb.name( read_string(&row, "name") );
		cb.value( read_string(&row, "value") );
		cb.host( read_string(&row, "host") );
		cb.path( read_string(&row, "path") );

		{
			let secs = read_int(&row, "expiry") as i64;
			let ts = Firefox::from_epoch_seconds(secs).unwrap();
			cb.expiry( ts );
		}
		{
			let msecs = read_int(&row, "lastAccessed") as i64;
			let ts = Firefox::from_epoch_microseconds(msecs).unwrap();
			cb.last_accessed( ts );
		}
		{
			let msecs = read_int(&row, "creationTime") as i64;
			let ts = Firefox::from_epoch_microseconds(msecs).unwrap();
			cb.creation_time( ts );
		}
		cb.is_secure( read_bool(&row, "isSecure") );
		cb.is_http_only( read_bool(&row, "isHttpOnly") );
		cb.in_browser_element( read_bool(&row, "inBrowserElement") );
		cb.same_site( read_bool(&row, "sameSite") );
		cb.raw_same_site( read_bool(&row, "rawSameSite") );
		cb.scheme_map( read_bool(&row, "schemeMap") );

		let cookie = cb.build().unwrap();

		Ok(cookie)
	}
}


