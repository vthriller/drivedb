use std::fmt;

// TODO make sure characters are in the range of 0x20 to (and including) 0x7e
// (this is in the standard, and also to make std::String safe again)
fn read_string(arr: &Vec<u16>, start: usize, fin: usize) -> String {
	let mut output = String::with_capacity((fin - start) * 2);

	for i in start..(fin+1) {
		output.push((arr[i] >> 8) as u8 as char);
		output.push((arr[i] & 0xff) as u8 as char);
	}

	String::from(output.trim())
}

fn is_set(word: u16, bit: usize) -> bool {
	word & (1<<bit) != 0
}
