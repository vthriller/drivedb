/*!
Functions implementing typical log page queries

The reason why this is implemented as a wrapper type instead of a trait is because it needs to cache and keep around the list of log pages that this device supports.

## Example

```
use hdd::Device;
use hdd::scsi::SCSIDevice;
use hdd::scsi::pages::{Pages, page_name};

...

let pages = dev.supported_pages().unwrap();

if pages.contains(0x03) {
	println!("{}:", page_name(0x03));
	println!("{:#?}\n", dev.read_error_counters()),
}
```
*/

use scsi;
use scsi::{SCSIDevice, SCSICommon};
use scsi::data::log_page;

extern crate byteorder;
use byteorder::{ReadBytesExt, BigEndian};

use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ErrorCounter {
	/// Errors corrected without substantial delay; smartctl name: Errors Corrected by ECC (fast)
	CorrectedNoDelay,
	/// Errors corrected with possible delays; smartctl name: Errors Corrected by ECC (delayed)
	CorrectedDelay,
	/// Total (e.g., rewrites or rereads); smartctl name: Errors Corrected by rereads/rewrites
	Total, // XXX total what?
	/// Total errors corrected; smartctl name: Total errors corrected
	ErrorsCorrected,
	/// Total times correction algorithm processed; smartctl name: Correction algorithm invocations
	CRCProcessed,
	/// Total bytes processed; smartctl name: Bytes processed
	BytesProcessed,
	/// Total uncorrected errors; smartctl name: Total uncorrected errors
	Uncorrected,
	VendorSpecific(u16),
	Reserved(u16),
}

/// For a given page number `page`, return its name
pub fn page_name(page: u8) -> &'static str {
	match page {
		0x00 => "Supported Log Pages",
		0x02 => "Write Error Counter",
		0x03 => "Read Error Counter",
		0x04 => "Read Reverse Error Counter",
		0x05 => "Verify Error Counter",
		0x06 => "Non-Medium Error",
		0x0d => "Temperature",
		0x0e => "Start-Stop Cycle Counter",
		0x10 => "Self-Test results",
		0x2f => "Informational Exceptions",
		0x30...0x3e => "(Vendor-Specific)",
		0x3f => "(Reserved)",
		// TODO Option<>?
		_ => "?",
	}
}

quick_error! {
	#[derive(Debug)]
	pub enum Error {
		NotSupported {}
		SCSI(err: scsi::Error) {
			from()
			display("{}", err)
		}
		/// failed to parse page data
		InvalidData(what: &'static str) {
			display("Unable to {}", what)
		}
	}
}
