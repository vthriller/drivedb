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

/**
Use this struct to issue LOG SENSE command against the device and return interpreted log page responses.

See [module documentation](index.html) for example.
*/
#[derive(Debug)]
pub struct SCSIPages<'a, T: SCSICommon + 'a> {
	device: &'a T,
	supported_pages: Vec<u8>,
}

// TODO non-empty autosense errors
impl<'a> SCSIPages<'a, SCSIDevice> {
	// TODO document error type
	pub fn new(device: &'a SCSIDevice) -> Result<Self, Error> {
		// no public method here can work without list of supported pages, so cache it right away or Err() out
		info!("querying list of supported page");
		let supported_pages = Self::get_page_unchecked(device, 0x00)?.data.to_vec();

		Ok(Self {
			device,
			supported_pages,
		})
	}

	pub fn supported_pages(&mut self) -> &[u8] {
		&self.supported_pages
	}

	fn get_page(&mut self, page: u8) -> Result<log_page::Page, Error> {
		if ! self.supported_pages.contains(&page) {
			info!("attemted to query unsupported page {}", page);
			return Err(Error::NotSupported)
		}

		Self::get_page_unchecked(self.device, page)
	}

	fn get_page_unchecked<D: SCSICommon>(device: &D, page: u8) -> Result<log_page::Page, Error> {
		let (_sense, data) = device.log_sense(
			false, // changed
			false, // save_params
			false, // default
			false, // threshold
			page, 0, // page, subpage
			0, // param_ptr
		)?;

		log_page::parse(&data).ok_or(Error::InvalidData("parse log page data"))
	}

	fn get_params(&mut self, page: u8) -> Result<Vec<log_page::Parameter>, Error> {
		let page = self.get_page(page)?;
		page.parse_params().ok_or(Error::InvalidData("parse log page params"))
	}

	/**
	Asks for log page `page` and interprets its contents as a list of error counters

	Use the following instead:

	* [write_error_counters](#method.write_error_counters)
	* [read_error_counters](#method.read_error_counters)
	* [read_reverse_error_counters](#method.read_reverse_error_counters)
	* [verify_error_counters](#method.verify_error_counters)
	*/
	pub fn error_counters(&mut self, page: u8) -> Result<HashMap<ErrorCounter, u64>, Error> {
		info!("querying error counters (page {})", page);

		let params = self.get_params(page)?;

		let counters = params.iter().map(|param| {
			// XXX tell about unexpected params?
			if param.value.len() == 0 { return None; }

			use self::ErrorCounter::*;
			let counter = match param.code {
				0x0000 => CorrectedNoDelay,
				0x0001 => CorrectedDelay,
				0x0002 => Total,
				0x0003 => ErrorsCorrected,
				0x0004 => CRCProcessed,
				0x0005 => BytesProcessed,
				0x0006 => Uncorrected,
				x @ 0x8000...0xffff => VendorSpecific(x),
				x => Reserved(x),
			};
			let value = {
				// read_uint cannot read values larger than 64 bits, and guess what, IBM-ESXS MBF2300RC actually returns 10 and even 16 bytes of data here!
				// TODO for now I just check whether value fits u64 or not, i.e. first bits are 0; should probably read this into u128 instead
				let mut offset = 0;
				if param.value.len() > 8 {
					for i in 0..(param.value.len() - 8) {
						if param.value[i] != 0 {
							warn!("page {} error counter does not fit u64", page);
							return None;
						}
						offset += 1;
					}
				}
				(&param.value[offset..]).read_uint::<BigEndian>(param.value.len() - offset).unwrap()
			};

			Some((counter, value))
		})
		.filter(|kv| kv.is_some())
		.map(|kv| kv.unwrap())
		.collect();

		Ok(counters)
	}

	pub fn write_error_counters(&mut self) -> Result<HashMap<ErrorCounter, u64>, Error> {
		self.error_counters(0x02)
	}
	pub fn read_error_counters(&mut self) -> Result<HashMap<ErrorCounter, u64>, Error> {
		self.error_counters(0x03)
	}
	pub fn read_reverse_error_counters(&mut self) -> Result<HashMap<ErrorCounter, u64>, Error> {
		self.error_counters(0x04)
	}
	pub fn verify_error_counters(&mut self) -> Result<HashMap<ErrorCounter, u64>, Error> {
		self.error_counters(0x05)
	}

	pub fn non_medium_error_count(&mut self) -> Result<u64, Error> {
		info!("querying non-medium error counters");

		let params = self.get_params(0x06)?;

		for param in params {
			// XXX tell about unexpected params?
			if param.value.len() == 0 { continue; }
			if param.code != 0 { continue; }

			return Ok((&param.value[..]).read_uint::<BigEndian>(param.value.len()).unwrap());
		}

		Err(Error::InvalidData("find valid param in the page"))
	}

	/**
	Returns tuple of `(temp, ref_temp)`, where:

	* `temp`: current temperature, °C,
	* `ref_temp`: reference temperature, °C; maximum temperature at which device is capable of operating continuously without degrading
	*/
	pub fn temperature(&mut self) -> Result<(Option<u8>, Option<u8>), Error> {
		info!("querying device temperature");

		let params = self.get_params(0x0d)?;

		let mut temp = None;
		let mut ref_temp = None;

		for param in params {
			// XXX tell about unexpected params?
			if param.value.len() < 2 { continue; }

			// value[0] is reserved
			let value = match param.value[1] {
				0xff => None, // unable to return temperature despite including this param in the answer
				x => Some(x),
			};

			match param.code {
				0x0000 => { temp = value },
				0x0001 => { ref_temp = value },
				_ => (),
			};
		}

		Ok((temp, ref_temp))
	}

	/// In SPC-4, this is called Start-Stop Cycle Counter
	pub fn dates_and_cycle_counters(&mut self) -> Result<DatesAndCycleCounters, Error> {
		info!("querying cycle counters");

		let params = self.get_params(0x0e)?;

		let mut result = DatesAndCycleCounters {
			manufacturing_date: None,
			accounting_date: None,
			lifetime_start_stop_cycles: None,
			start_stop_cycles: None,
			lifetime_load_unload_cycles: None,
			load_unload_cycles: None,
		};

		for param in params {
			match param.code {
				0x0001 => {
					// XXX tell about unexpected params?
					if param.value.len() < 6 { continue; }

					result.manufacturing_date = Some(Date {
						year: String::from_utf8(param.value[0..4].to_vec()).unwrap(), // ASCII
						week: String::from_utf8(param.value[4..6].to_vec()).unwrap(), // ASCII
					});
				},
				0x0002 => {
					// XXX tell about unexpected params?
					if param.value.len() < 6 { continue; }

					result.accounting_date = Some(Date {
						year: String::from_utf8(param.value[0..4].to_vec()).unwrap(), // ASCII, might be all-spaces
						week: String::from_utf8(param.value[4..6].to_vec()).unwrap(), // ASCII, might be all-spaces
					});
				},
				0x0003 => {
					// XXX tell about unexpected params?
					if param.value.len() < 4 { continue; }

					result.lifetime_start_stop_cycles = Some(
						(&param.value[0 .. 4]).read_u32::<BigEndian>().unwrap()
					);
				},
				0x0004 => {
					// XXX tell about unexpected params?
					if param.value.len() < 4 { continue; }

					result.start_stop_cycles = Some(
						(&param.value[0 .. 4]).read_u32::<BigEndian>().unwrap()
					);
				},
				0x0005 => {
					// XXX tell about unexpected params?
					if param.value.len() < 4 { continue; }

					result.lifetime_load_unload_cycles = Some(
						(&param.value[0 .. 4]).read_u32::<BigEndian>().unwrap()
					);
				},
				0x0006 => {
					// XXX tell about unexpected params?
					if param.value.len() < 4 { continue; }

					result.load_unload_cycles = Some(
						(&param.value[0 .. 4]).read_u32::<BigEndian>().unwrap()
					);
				},
				_ => {
					// XXX tell about unexpected params?
				},
			}
		}

		Ok(result)
	}

	pub fn self_test_results(&mut self) -> Result<Vec<SelfTest>, Error> {
		info!("querying self-test results");

		let params = self.get_params(0x10)?;

		let self_tests = params.iter().map(|param| {
			// XXX tell about unexpected params?
			if param.code == 0 || param.code > 0x0014 { return None; }
			if param.value.len() < 0x10 { return None; }

			// unused self-test log parameter is all zeroes
			if *param.value.iter().max().unwrap() == 0 { return None }

			use self::SelfTestResult::*;
			Some(SelfTest {
				result: match param.value[0] & 0b111 {
					0 => NoError,
					1 => Aborted { explicitly: true },
					2 => Aborted { explicitly: false },
					3 => UnknownError,
					4...7 => Failed,
					15 => InProgress,
					x => Reserved(x),
				},
				code: (param.value[0] & 0b1110_0000) >> 5,
				number: param.value[1],
				power_on_hours: (&param.value[2..4]).read_u16::<BigEndian>().unwrap(),
				first_failure_lba: (&param.value[4..12]).read_u64::<BigEndian>().unwrap(),
				sense_key: param.value[12] & 0b1111,
				sense_asc: param.value[13],
				sense_ascq: param.value[14],
				vendor_specific: param.value[15],
			})
		})
		.filter(|kv| kv.is_some())
		.map(|kv| kv.unwrap())
		.collect();

		Ok(self_tests)
	}

	pub fn informational_exceptions(&mut self) -> Result<Vec<InformationalException>, Error> {
		info!("querying informational exceptions");

		let params = self.get_params(0x2f)?;

		let exceptions = params.iter().map(|param| {
			// XXX tell about unexpected params?
			if param.code != 0 { return None; }
			if param.value.len() < 3 { return None; }

			Some(InformationalException {
				asc: param.value[0],
				ascq: param.value[1],
				recent_temperature_reading: match param.value[2] {
					0xff => None,
					x => Some(x),
				},
				vendor_specific: param.value[3..].to_vec(),
			})
		})
		.filter(|kv| kv.is_some())
		.map(|kv| kv.unwrap())
		.collect();

		Ok(exceptions)
	}
}
