fn copy_from_slice_3(x: &[u8]) -> [u8; 3] {
	let mut y = [0; 3];
	y.copy_from_slice(x);
	y
}
fn copy_from_slice_4(x: &[u8]) -> [u8; 4] {
	let mut y = [0; 4];
	y.copy_from_slice(x);
	y
}
