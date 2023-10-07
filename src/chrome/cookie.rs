use num_enum::{IntoPrimitive, FromPrimitive};
use chrono::{DateTime, offset::Utc};
use aes::cipher::{block_padding:: NoPadding, BlockDecryptMut, KeyIvInit};

use browser_cookie::*;

/// Rust port of GenerateEncryptionKey
/// https://source.chromium.org/chromium/chromium/src/+/main:components/os_crypt/sync/os_crypt_linux.cc;bpv=1;bpt=1
// Can't be made const.. yet! https://github.com/rust-lang/rust/issues/57349
fn get_key() -> [u8; 16] {
	use pbkdf2::pbkdf2_hmac;
	use sha1::Sha1;
	const PASSWORD: &[u8; 7] = b"peanuts";
	const SALT: &[u8; 9] = b"saltysalt";
	const ITER: u32 = 1;
	let mut key = [0u8; 16];
	pbkdf2_hmac::<Sha1>(PASSWORD, SALT, ITER, &mut key);
	key
}

// const VERSION_10_KEY: [u8;16] = get_key();

#[derive(Debug, Clone, IntoPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum CookieSourceScheme {
	#[num_enum(default)]
	Unset = 0,
	NonSecure = 1,
	Secure = 2,
}

// See https://tools.ietf.org/html/draft-ietf-httpbis-cookie-same-site-00
// and https://tools.ietf.org/html/draft-ietf-httpbis-rfc6265bis for
// information about same site cookie restrictions.
// Note: Don't renumber, as these values are persisted to a database.
#[repr(i8)]
#[derive(Debug, Clone, IntoPrimitive, FromPrimitive)]
pub enum CookieSameSite {
	#[num_enum(default)]
	Unspecified = -1,
	NoRestriction = 0,
	LaxMode = 1,
	StrictMode = 2,
	// Reserved 3 (was EXTENDED_MODE), next number is 4.
}

#[derive(Debug, Builder)]
pub struct ChromeCookie {
	pub name: String,
	pub path: String,
	pub value: Option<String>,
	pub encrypted_value: Vec<u8>,
	pub creation_utc: DateTime<Utc>,
	pub last_access_utc: DateTime<Utc>,
	pub last_update_utc: DateTime<Utc>,
	pub has_expires: bool,
	pub expires_utc: Option<DateTime<Utc>>,
	pub source_port: u32,
	pub is_secure: bool,
	pub is_httponly: bool,
	pub is_persistent: bool,
	pub is_same_party: bool,
	pub priority: i64,
	pub samesite: CookieSameSite,
	pub source_scheme: CookieSourceScheme,
}

impl Cookie for ChromeCookie {
	fn name (&self) -> String { self.name.clone() }
	fn value (&self) -> String {
		self.value.clone()
			.unwrap_or( self.decrypt().unwrap_or("".to_string()) )
	}
}

impl ChromeCookie {
	pub fn decrypt(&self) -> Result<String, CookieError> {
		
		if self.encrypted_value.len() == 0 {
			return Err( CookieError::NoValue(self.name.clone()) )
		}
		if self.encrypted_value.len() < 3 {
			return Err( CookieError::NotEncrypted(self.name.clone()) )
		}

		let (version,data) = &self
			.encrypted_value
			.split_array_ref::<3>();

		let version = version
			.iter()
			.map(|b| *b as char)
			.collect::<String>();

		match version.as_str() {
			"v10" => {
				let iv = [b' '; 16];
				let mut buf = [0u8; 2048];
				let len = data.len();
				buf[..len].copy_from_slice(data);

				type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
				let pt = Aes128CbcDec::new( &get_key().into(), &iv.into() )
					.decrypt_padded_mut::<NoPadding>(&mut buf).unwrap();
				
				let padding = pt[len-1] as usize;
				let trim = &pt[..len - (pt[ len - padding] as usize) ];

				let value = trim
					.iter()
					.map(|b| *b as char).collect::<String>();
				
				Ok( value.to_owned() )
			}
			other => Err(
				CookieError::Unsupported(other.to_string(), self.name.clone())
			)
		}
	}
}


impl TryFrom<sqlite::Row> for ChromeCookie {
	type Error = CookieError;
	fn try_from( row: sqlite::Row ) -> Result<ChromeCookie, Self::Error> {
		let mut cb = ChromeCookieBuilder::default();
		cb.name( row.read::<&str, _>("name").to_string() );
		cb.encrypted_value( read_vecu8(&row, "encrypted_value") );
		cb.path( read_string(&row, "path")? );
		cb.is_secure( read_bool(&row, "is_secure")? );
		cb.is_httponly( read_bool(&row, "is_httponly")? );
		cb.has_expires( read_bool(&row, "has_expires")? );
		cb.is_persistent( read_bool(&row, "is_persistent")? );
		cb.is_same_party( read_bool(&row, "is_same_party")? );
		cb.priority( read_int(&row, "priority")? );
		cb.samesite( (read_int(&row, "samesite")? as i8).into() );
		cb.source_scheme( (read_int(&row, "source_scheme")? as u8).into() );

		// Store as Option where "" is None
		cb.value( Some(read_string(&row, "value")?).filter(|s| !s.is_empty()) );
		cb.source_port( read_int(&row, "source_port")? as u32 );

		{
			let ts = read_int(&row, "creation_utc")?;
			cb.creation_utc( from_epoch(ts).unwrap() );
		}
		
		{
			let ts = read_int(&row, "last_access_utc")?;
			cb.last_access_utc( from_epoch(ts).unwrap() );
		}
		
		{
			let ts = read_int(&row, "last_update_utc")?;
			cb.last_update_utc( from_epoch(ts).unwrap() );
		}
		
		{
			let ts = read_int(&row, "expires_utc")?;
			cb.expires_utc( from_epoch(ts) );
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

		Ok(cookie)
	}
}
	
/// Convert from MS since 1601-01-01 to DateTime
/// https://source.chromium.org/chromium/chromium/src/+/main:base/time/time.h;l=529;drc=131600edcd9395ffa1241050c486e8da3fbfda4f
fn from_epoch( ts: i64 ) -> Option< DateTime<chrono::offset::Utc> > {
	if ts == 0 {
		return None
	}
	const UNIX_EPOCH_OFFSET: i64 = 11644473600;
	DateTime::from_timestamp(
		(ts / 1000000) - UNIX_EPOCH_OFFSET,
		0,
	)
}
