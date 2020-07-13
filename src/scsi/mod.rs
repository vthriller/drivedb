/*!
All things SCSI.

* Use [`data` module](data/index.html) to parse various low-level structures found in SCSI command replies.
* Import traits from porcelain modules (like [`pages`](pages/index.html)) to do typical tasks without needing to compose commands and parse responses yourself.
  * You can also use [`module ata`](../ata/index.html) to issue ATA commands using ATA PASS-THROUGH.
*/

pub mod data;
pub mod pages;

use std::io;
use ata;
use byteorder::{ReadBytesExt, BigEndian};
use self::data::sense;

use Direction;

use utils::hexdump_8;

quick_error! {
	#[derive(Debug)]
	pub enum Error {
		IO(err: io::Error) {
			from()
			display("IO error: {}", err)
			description(err.description())
			cause(err)
		}
		// this is for Sense::Fixed(FixedData::Invalid(_))
		// pun definitely intented at this point
		Nonsense {}
	}
}

// FIXME naming: this isn't about ATA-level error, this is error related to ATA PASS-THROUGH command
quick_error! {
	#[derive(Debug)]
	pub enum ATAError {
		SCSI(err: Error) {
			from()
			from(err: io::Error) -> (Error::IO(err))
			display("{}", err)
		}
		/// Device does not support ATA PASS-THROUGH command
		NotSupported {}
		// no non-deferred sense is available, or there's no descriptors for ATA registers to be found
		NoRegisters {}
	}
}

// The following return tuple of (format, glistv, plistv, len)
fn parse_defect_data_10(data: &[u8]) -> Option<(u8, bool, bool, u16)> {
	if data.len() >= 4 {
		// byte 0: reserved

		// > A device server unable to return the requested format shall return the defect list in its default format and indicate that format in the DEFECT LIST FORMAT field in the defect list header
		let format = data[1] & 0b111;
		let glistv = data[1] & 0b1000 != 0;
		let plistv = data[1] & 0b10000 != 0;
		// byte 1 bits 5..7: reserved

		let len = (&data[2..4]).read_u16::<BigEndian>().unwrap();

		// the rest is the address list itself

		return Some((format, glistv, plistv, len));
	}

	None
}
fn parse_defect_data_12(data: &[u8]) -> Option<(u8, bool, bool, u32)> {
	if data.len() >= 8 {
		// byte 0: reserved

		// > A device server unable to return the requested format shall return the defect list in its default format and indicate that format in the DEFECT LIST FORMAT field in the defect list header
		let format = data[1] & 0b111;
		let glistv = data[1] & 0b1000 != 0;
		let plistv = data[1] & 0b10000 != 0;
		// byte 1 bits 5..7: reserved

		// bytes 2, 3: reserved

		let len = (&data[4..8]).read_u32::<BigEndian>().unwrap();

		// the rest is the address list itself

		return Some((format, glistv, plistv, len));
	}

	None
}
