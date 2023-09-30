use std::fmt;

pub trait Cookie: std::fmt::Debug {
	fn name(&self) -> String;	
	fn value(&self) -> String;	
}

#[derive(Default, Debug)]
pub struct CookieJar {
	_inner: std::collections::HashMap<String, Box<dyn Cookie>> 
}

impl CookieJar {
	pub fn add_cookie(&mut self, key: String, value: Box<dyn Cookie>) {
		self._inner.insert(key, value);
	}
}

/// Display as `set-cookie-string`
/// https://datatracker.ietf.org/doc/html/rfc6265
impl fmt::Display for CookieJar {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let iter = self._inner.iter();
		let mut count = iter.clone().count();
		let mut res = Ok(());
		for (k,v) in iter {
			if count > 1 {
				res = write!(f, "{}={}; ", k, v.value() )
			}
			else {
				res = write!(f, "{}={}", k, v.value() )
			}
			count = count - 1;
		}
		res
	}
}
