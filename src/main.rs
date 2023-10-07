#![feature(split_array,never_type)]
#[macro_use]
extern crate derive_builder;

use browser_cookie::*;
mod chrome;
mod firefox;
mod cookiejar;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[arg(value_enum, long, default_value = "linux")]
	os: Os,

	#[arg(short, long, default_value = "chromium")]
	browser: Browser,
	
	#[arg(short, long, env)]
	user: String,
	
	#[arg(short, long)]
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
	let env: Env = args.into();

	match env.browser {
		Browser::Chromium | Browser::Chrome => {
			let browser = chrome::ChromeBuilder::default().env(env).build().unwrap();
			let jar = browser.get_cookies( &domain );
			println!("{}", jar)
		}
		Browser::Firefox => {
			let browser = firefox::FirefoxBuilder::default().env(env).build().unwrap();
			let jar = browser.get_cookies( &domain );
			println!("{}", jar)
		}
		_ => todo!( "Chill cowboy, the requested browser isn't implemented yet" )
	}
	
}
