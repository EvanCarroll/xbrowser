#![feature(split_array,never_type)]
#[macro_use]
extern crate derive_builder;

use clap::{Parser, Subcommand, ValueEnum, Args};
mod chrome;
mod firefox;
mod cookiejar;

use xbrowser::*;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(value_enum)]
	action: ExportImport,
	
	#[arg(value_enum, short, default_value="plain-text")]
	format: ExportFormat,

	#[arg(value_enum, long, default_value = "linux")]
	/// Browser operating system which created the profile
	os: Os,

	#[arg()]
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

	#[command(subcommand)]
	command: Commands,

}

#[derive(Debug, Default, Clone, ValueEnum)]
enum ExportFormat {
	#[default]
	PlainText,
	Json,
}

// https://source.chromium.org/chromium/chromium/src/+/main:components/os_crypt/sync/key_storage_util_linux.h;drc=f5bdc89c7395ed24f1b8d196a3bdd6232d5bf771;bpv=1;bpt=1;l=20
#[derive(Debug, Default, Clone, ValueEnum)]
enum ChromiumLinuxPasswordStore {
	#[default]
	GnomeLibsecret,
	Kwallet5,
	Kwallet6,
	Kwallet,
	BasicText,
	Deferred,
}

#[derive(Debug, Default, Clone, ValueEnum)]
enum ExportImport {
	#[default]
	Export,
	Import	
}

#[derive(Subcommand, Debug)]
enum Commands {
	/// Decodes Cookies
	Cookies {
		#[command(flatten)]
		browser: ChromeCrypto,
	
		/// Domain you wish you to query
		domain: Option<String>,
	},
}

#[derive(Args, Debug, Clone)]
struct ChromeCrypto {
	/// Defaults based on operating system
	#[arg(long)]
	chrome_v11_master_key: Option<String>,
	
	#[arg(long, default_value="peanuts")]
	chrome_v10_master_key: String,

	/// Default set based on the operating system
	#[arg(long)]
	chrome_pbkdf2_iterations: Option<u32>,
	
	#[arg(long, default_value="saltysalt")]
	chrome_pbkdf2_salt: String,
	
	/// Defaults set based on the operating system
	#[arg(long)]
	chrome_password_store: Option<ChromiumLinuxPasswordStore>,
}

impl From<Cli> for Env {
	fn from( args: Cli ) -> Self {
		Self {
			os: args.os,
			browser: args.browser,
			user: args.user,
		}
	}
}

fn main() {
	let args = Cli::parse();

	let env: Env = Env {
		browser: args.browser,
		user: args.user,
		os: args.os,
	};
	
	match &args.command {
		Commands::Cookies { domain, .. } => {
			let profile = args.profile.clone();
			let export_format = args.format.clone();
			let path_config = args.path_config.clone();

			match env.browser {
				Browser::Chromium | Browser::Chrome => {
					let mut builder = chrome::ChromeBuilder::default();
					builder.env(env);
					builder.profile(profile);
					if let Some(path) = path_config {
						builder.path_config(Some(path.as_str().into()));
					}
					let browser = builder.build().unwrap();
					match domain {
						Some(domain) => {
							let jar = browser.get_cookies_for_domain( domain ).unwrap();

							match export_format {
								ExportFormat::Json => println!("{}", serde_json::to_string(&jar).unwrap() ),
								ExportFormat::PlainText => println!("{}", &jar)
							}
						}
						None => {
							let vec = browser.get_all_cookies().unwrap();
							match export_format {
								ExportFormat::Json => println!("{}", serde_json::to_string(&vec).unwrap() ),
								_ => panic!("We only support dumping in JSON")
							}
						}
					}
				}
				Browser::Firefox => {
					let mut builder = firefox::FirefoxBuilder::default();
					builder.env(env);
					builder.profile(profile);
					let browser = builder.build().unwrap();
					match domain {
						Some(domain) => {
							let jar = browser.get_cookies_for_domain( domain ).unwrap();

							match export_format {
								ExportFormat::Json => println!("{}", serde_json::to_string(&jar).unwrap() ),
								ExportFormat::PlainText => println!("{}", &jar)
							}
						}
						None => {
							let vec = browser.get_all_cookies().unwrap();
							match export_format {
								ExportFormat::Json => println!("{}", serde_json::to_string(&vec).unwrap() ),
								_ => panic!("We only support dumping in JSON")
							}
						}
					}
				}
				_ => todo!( "Chill cowboy, the requested browser isn't implemented yet" )
			}
		}

	}
	
}
