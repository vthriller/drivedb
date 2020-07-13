use std::cmp::{min, max};
use drivedb;
use std::fmt;

// Initially I used `BigEndian` from the `byteorder` crate; however, it quickly resulted in an iterator mess (`.chunks()`, `.take()`, `.skip()`, `.map()`, `.unwrap()` et al.), and it also did not help with 24-bit and 48-bit packed values at all.
fn read(data: &[u8], bits: usize) -> u64 {
	let mut out: u64 = 0;
	for i in 0..(bits/8) {
		out <<= 8;
		out += data[i] as u64;
	}
	out
}

fn write_vec<T>(f: &mut fmt::Formatter, vec: &Vec<T>) -> fmt::Result
where T: fmt::Display {
	let mut values = vec.iter();
	if let Some(i) = values.next() { write!(f, "{}", i)?; }
	for i in values { write!(f, " {}", i)?; }
	Ok(())
}

// In smartmontools, they first apply byte order attribute to get the u64, which in turn is used to get separate u8/u16s for RAWFMT_RAW8/RAWFMT_RAW16; there's also different byte order defaults for different formats. Oh for crying out loud…

// `data` is a slice that contains all the attribute data, including attribute id
fn reorder(data: &[u8], byte_order: &str) -> Vec<u8> {
	byte_order.chars().map(|c| match c {
		'v' => data[3], // value
		'w' => data[4], // worst
		'0' => data[5],
		'1' => data[6],
		'2' => data[7],
		'3' => data[8],
		'4' => data[9],
		'5' => data[10],
		'r' => data[11], // reserved byte
		// smartmontools defaults to 0 for any unrecognized character;
		// we'll use '_' later for padding
		_ => 0,
	}).collect()
}
