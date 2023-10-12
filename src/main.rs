#![feature(split_array,never_type)]
#[macro_use]
extern crate derive_builder;

use xbrowser::*;
mod chrome;
mod firefox;
mod cookiejar;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(value_enum, long, default_value = "linux")]
	/// Browser operating system which created the profile
	os: Os,

	#[arg(short, long, default_value = "chromium")]
	browser: Browser,

	#[arg(short, long)]
	/// Name of the profile in the browser
	profile: Option<String>,
	
	#[arg(long)]
	/// Parent directory of the profile data
	path_config: Option<String>,
	
	#[arg(short, long, env)]
	/// User's home directory, used to compute a default path to your
	/// browser's config. Defaults to the ENV{USER}
	user: String,
	
	#[arg(short, long)]
	/// Domain you wish you to query
	domain: String,
}

impl From<Args> for Env {
	fn from( args: Args ) -> Self {
		Self {
			os: args.os,
			browser: args.browser,
			user: args.user,
		}
	}
}

fn main() {
	let args = Args::parse();
	let domain = args.domain.clone();
	let profile = args.profile.clone();
	let path_config = args.path_config.clone();
	let env: Env = args.into();

	match env.browser {
		Browser::Chromium | Browser::Chrome => {
			let mut builder = chrome::ChromeBuilder::default();
			builder.env(env);
			builder.profile(profile);
			if let Some(path) = path_config {
				builder.path_config(Some(path.as_str().into()));
			}
			let browser = builder.build().unwrap();
			let jar = browser.get_cookies( &domain );
			println!("{}", jar)
		}
		Browser::Firefox => {
			let mut builder = firefox::FirefoxBuilder::default();
			builder.env(env);
			builder.profile(profile);
			let browser = builder.build().unwrap();
			let jar = browser.get_cookies( &domain );
			println!("{}", jar)
		}
		_ => todo!( "Chill cowboy, the requested browser isn't implemented yet" )
	}
	
}
