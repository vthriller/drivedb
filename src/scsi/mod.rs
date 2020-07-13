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
use Device;

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
		// XXX make sure only non-deferred senses are used here
		// XXX it makes no sense (sorry!) to put informational senses here (i.e. sense::SenseKey::{Ok, Recovered, Completed})
		Sense(key: sense::key::SenseKey, asc: u8, ascq: u8) { // XXX do we need additional sense data? descriptors? flags? probably not
			// FIXME no from() here due to sense::Sense lifetimes; for now use Error::from_sense() instead
			description("SCSI error")
			display("SCSI error: {:?} ({})",
				key,
				sense::key::decode_asc(*asc, *ascq)
					.map(|x| x.to_string())
					.unwrap_or_else(|| format!("unknown additional sense code: {:02x} {:02x}", asc, ascq)))
		}
		// this is for Sense::Fixed(FixedData::Invalid(_))
		// pun definitely intented at this point
		Nonsense {}
	}
}

impl Error {
	fn from_sense(sense: &sense::Sense) -> Self {
		match sense.kcq() {
			Some((key, asc, ascq)) =>
				Error::Sense(sense::key::SenseKey::from(key), asc, ascq),
			None =>
				Error::Nonsense,
		}
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

fn read_defect_data_10_cmd(plist: u8, glist: u8, format: AddrDescriptorFormat) -> (Vec<u8>, usize) {
	// we're only interested in the header, not the list itself
	let alloc = 4;
	let cmd = vec![
		0x37, // opcode
		0, // reserved
		(plist << 4) + (glist << 3) + (format as u8), // reserved (3 bits), req_plist, req_glist, defect list format (3 bits)
		0, 0, 0, 0, // reserved
		(alloc >> 8) as u8,
		(alloc & 0xff) as u8,
		0, // control (XXX what's that?!)
	];
	(cmd, alloc)
}

fn read_defect_data_12_cmd(plist: u8, glist: u8, format: AddrDescriptorFormat) -> (Vec<u8>, usize) {
	// we're only interested in the header, not the list itself
	let alloc = 8;
	let cmd = vec![
		0xb7, // opcode
		(plist << 4) + (glist << 3) + (format as u8), // reserved (3 bits), req_plist, req_glist, defect list format (3 bits)
		0, 0, 0, 0, // reserved
		((alloc >> 24) & 0xff) as u8,
		((alloc >> 16) & 0xff) as u8,
		((alloc >>  8) & 0xff) as u8,
		( alloc        & 0xff) as u8,
		0, // reserved
		0, // control (XXX what's that?!)
	];
	(cmd, alloc)
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
