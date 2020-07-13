fn is_set(x: u8, bit: usize) -> bool {
	x & (1<<bit) != 0
}
