use std::fmt;
use itertools::Itertools;
use crate::*;


use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CookieJar<C>
	where C: Cookie + std::cmp::Ord
{
	#[serde(flatten)]
	inner: Option<std::collections::HashMap<String, Box<C>>>
}

impl<C> Default for CookieJar<C>
	where C: Cookie + std::cmp::Ord
{
	fn default() -> Self {
		Self { inner: None }
	}
}

impl<C> CookieJar<C>
	where C: Cookie + std::cmp::Ord
{
	pub fn add_cookie(&mut self, key: String, value: Box<C>)
	{
		match &mut self.inner {
			Some(ref mut col) => {
				col.insert(key, value);
			}
			None => {
				let mut col = std::collections::HashMap::new();
				col.insert(key,value);
				self.inner = Some(col);
			}
		}
	}
}

/// Display as `set-cookie-string`
/// https://datatracker.ietf.org/doc/html/rfc6265
impl<C> fmt::Display for CookieJar<C>
	where C: Cookie + std::cmp::Ord
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match &self.inner {
			Some(inner) => {
				let iter = inner.iter().sorted();
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
			},
			None => Ok(())
		}
	}
}
