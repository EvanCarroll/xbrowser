#![feature(split_array)]
#[macro_use]
extern crate derive_builder;

mod chrome;
mod cookiejar;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(value_enum, long)]
	os: Os,

	#[arg(short, long, default_value = "chromium")]
	browser: Browsers,
	
	#[arg(short, long, env)]
	user: String,
	
	#[arg(short, long)]
	domain: String,
}

fn main() {
	let cli = Cli::parse();
	let path = build_path( cli.os, &cli.user, None );
	let jar = chrome::get_cookies(&path, &cli.domain );
	println!("{}", jar);
}

#[derive(PartialEq, Debug, Clone, ValueEnum)]
enum Os { Win, Linux, Osx }

#[derive(PartialEq, Debug, Clone, ValueEnum)]
enum Browsers { Chrome, Chromium, Firefox, Safari }

/// Path to sql lite database
fn build_path( os: Os, user: &str, profile: Option<&str> ) -> PathBuf {
	match os {
		Os::Linux => {
			let mut p = PathBuf::new();
			p.push( "/" );
			p.push( "home" );
			p.push( user );
			p.push( ".config" );
			p.push( "chromium" );
			p.push( profile.unwrap_or("Default") );
			p.push( "Cookies" );
			p
		}
		_ => todo!("Other OSes")
	}
}
