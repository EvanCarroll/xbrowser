#[derive(Debug)]
pub enum CookieError {
	NotEncrypted(String),
	NoValue(String),
	Unsupported(String, String),
}
