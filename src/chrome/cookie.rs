use num_enum::{IntoPrimitive, FromPrimitive};

use chrono::{DateTime, offset::Utc};
use aes::cipher::{block_padding:: NoPadding, BlockDecryptMut, KeyIvInit};

mod error;
use error::CookieError;

/// Rust port of GenerateEncryptionKey
/// https://source.chromium.org/chromium/chromium/src/+/main:components/os_crypt/sync/os_crypt_linux.cc;bpv=1;bpt=1
fn get_key () -> [u8; 16] {
	use pbkdf2::pbkdf2_hmac;
	use sha1::Sha1;
	const PASSWORD: &[u8; 7] = b"peanuts";
	const SALT: &[u8; 9] = b"saltysalt";
	const ITER: u32 = 1;
	let mut key = [0u8; 16];
	pbkdf2_hmac::<Sha1>(PASSWORD, SALT, ITER, &mut key);
	key
}

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
pub struct Cookie {
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

impl crate::cookiejar::Cookie for Cookie {
	fn name (&self) -> String { self.name.clone() }
	fn value (&self) -> String {
		self.value.clone()
			.unwrap_or( self.decrypt().unwrap_or("".to_string()) )
	}

}

impl Cookie {
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
