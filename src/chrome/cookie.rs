use num_enum::{IntoPrimitive, FromPrimitive};
use chrono::{DateTime, offset::Utc};
use aes::cipher::{block_padding:: NoPadding, BlockDecryptMut, KeyIvInit};
use once_cell::sync::OnceCell;

use xbrowser::*;


/// Rust port of GenerateEncryptionKey
/// https://source.chromium.org/chromium/chromium/src/+/main:components/os_crypt/sync/os_crypt_linux.cc;bpv=1;bpt=1
/// ```
/// assert!( get_key_v10(), CHROME_V10_KEY_LINUX_POSIX )
/// ```
// Can't be made const.. yet! https://github.com/rust-lang/rust/issues/57349
#[allow(dead_code)]
fn get_key_v10() -> [u8; 16] {
	const PASSWORD: &[u8; 7] = b"peanuts";
	pbkdf2(PASSWORD)
}


// It's safe to precalculate the v10 key since it's hard coded in the source
static CHROME_V10_KEY_LINUX_POSIX: [u8;16] = [253, 98, 31, 229, 162, 180, 2, 83, 157, 250, 20, 124, 169, 39, 39, 120];
static CHROME_V11_KEY_LINUX_POSIX: OnceCell<[u8;16]> = OnceCell::new();


/// Currently we only handle v11 key retreval on Linux
// https://source.chromium.org/chromium/chromium/src/+/main:components/os_crypt/sync/key_storage_libsecret.cc;l=17
fn get_key_v11() -> Result<[u8; 16], CookieError> {
	//let collection = libsecret::COLLECTION_DEFAULT;
	let mut attributes = std::collections::HashMap::new();
	attributes.insert("application", libsecret::SchemaAttributeType::String);
	let schema = libsecret::Schema::new(
		"chrome_libsecret_os_crypt_password_v2",
		libsecret::SchemaFlags::DONT_MATCH_NAME,
		attributes
	);

	let mut q = std::collections::HashMap::new();
	q.insert("application", "chromium");

	let cancellable = gio::Cancellable::new();
	let lookup = libsecret::password_lookup_sync(Some(&schema), q, Some(&cancellable))
		.map_err(|_| CookieError::LibSecret)?
		.unwrap();

	Ok(pbkdf2( lookup.as_bytes() ))
}

fn pbkdf2(password: &[u8]) -> [u8; 16] {
	use pbkdf2::pbkdf2_hmac;
	use sha1::Sha1;
	const SALT: &[u8; 9] = b"saltysalt";
	const ITER: u32 = 1;
	let mut key = [0u8; 16];
	pbkdf2_hmac::<Sha1>(password, SALT, ITER, &mut key);
	key
}

#[allow(dead_code)]
fn base64decode(encoded: String) -> [u8; 16] {
	assert!( encoded.len() < 32, "Encoded must be len 32 {:?}", encoded.len() );
	use base64::{Engine as _, engine::general_purpose};
	let mut decoded: [u8; 16] = [0;16];
	general_purpose::STANDARD.decode_slice_unchecked(encoded, &mut decoded).unwrap();
	decoded
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
			.unwrap_or(
				self.decrypt().or_else(|err| {
					match err {
						CookieError::NoValue(_) => Ok("".to_string()),
						CookieError::NotEncrypted(_) => Ok("".to_string()),
						_ => Err(err),
					}
				} ).expect("Error decrypting to value")
			)
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
				chrome_decrypt( &CHROME_V10_KEY_LINUX_POSIX, data )
			}
			"v11" => {
				let key = CHROME_V11_KEY_LINUX_POSIX.get_or_try_init( || get_key_v11() )?;
				chrome_decrypt( key, data )
			}
			other => Err(
				CookieError::ChromeUnsupportedEncryption(other.to_string())
			)
		}
	}
}

fn chrome_decrypt( key: &[u8;16], data: &[u8] ) -> Result<String, CookieError> {
	let plaintext = aes128_cbc_decrypt( key, data )?;
	let trim = trim_padding( &plaintext, data.len() );
	Ok( trim.iter().map(|b| *b as char).collect::<String>() )
}

// This returns a static buffer of 2048 bytes
fn aes128_cbc_decrypt( key: &[u8;16], data: &[u8] ) -> Result<[u8;2048], CookieError> {
	let iv = [b' '; 16];
	let mut buf = [0u8; 2048];
	buf[..data.len()].copy_from_slice(data);
	type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
	Aes128CbcDec::new( key.into(), &iv.into() )
		.decrypt_padded_mut::<NoPadding>(&mut buf)
		.map_err( |_| CookieError::Decryption )?;
	Ok(buf)
}

/// Takes a padded buffer, and the length of the input ciphertext
fn trim_padding<'a>( pt: &'a [u8;2048], cyphertext_len: usize ) -> &'a [u8] {
	// The padding lengnth is stored in last byte
	let padding = pt[cyphertext_len-1] as usize;
	let len = cyphertext_len - padding;
	&pt[..cyphertext_len - (pt[len] as usize) ]
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
