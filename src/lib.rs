#![warn(
	missing_debug_implementations,
	// TODO
	//missing_docs,
	// XXX how to limit this to C-like enums? I'd like to #[derive(Copy)] them
	// see also https://github.com/rust-lang-nursery/rust-clippy/issues/2222
	//missing_copy_implementations,
	trivial_casts,
	trivial_numeric_casts,
	// XXX this crate is all about unsafe code, but we should probably limit that to certain modules
	//unsafe_code,
	unstable_features,
	unused_import_braces,
	unused_qualifications,
)]

#[macro_use]
extern crate quick_error;

#[macro_use]
extern crate nom;
extern crate regex;

pub mod drivedb;
