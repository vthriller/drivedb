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

pub fn parse(data: &[u8]) -> Option<FixedData> {
	if data.len() < 18 {
		return None;
	}
	if data[0] & 0b1000_0000 != 0 {
		return Some(FixedData::Invalid(data));
	}

	// data[7] is Additional Sense Length, starting from data[8],
	let len = (data[7] + 8) as usize;

	Some(FixedData::Valid {
		file_mark: data[2] & 0b1000_0000 != 0,
		eom: data[2] & 0b0100_0000 != 0,
		incorrect_length: data[2] & 0b0010_0000 != 0,
		key: data[2] & 0b1111,

		info: copy_from_slice_4(&data[3..7]),
		cmd_info: copy_from_slice_4(&data[8..12]),
		asc: data[12],
		ascq: data[13],
		fruc: data[14],
		sks: copy_from_slice_3(&data[15..18]),

		more: if len > data.len() {
			// sense reports more data than `data` buffer actually fits
			return None
		} else {
			&data[18 .. len]
		}
	})
}
