#![warn(
	missing_debug_implementations,
	missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unsafe_code,
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
