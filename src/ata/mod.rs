/*!
All things ATA.

* Import [`ATADevice`](struct.ATADevice.html) to start sending ATA commands to the [`Device`](../device/index.html) or [`SCSIDevice`](../scsi/index.html).
* Use [`data` module](data/index.html) to parse various low-level structures found in ATA command replies.
* Import traits from porcelain modules (currently that's just [`misc`](misc/index.html)) to do typical tasks without needing to compose commands and parse responses yourself.
*/

pub mod data;
pub mod misc;

use Direction;
use scsi;

#[derive(Debug)]
pub struct ATADevice<T> {
	device: T,
}

impl<T> ATADevice<T> {
	pub fn new(device: T) -> Self {
		Self { device }
	}
}

// using macro here instead of trait because we don't want to define yet another trait just to declare that `T` in `ATADevice<T>` is capable of `ata_platform_do()`,
// which is an implementation detail that would leak everywhere as part of a public interface
// besides, we really only need this method for just, like, two types: `ATADevice<Device>` and `ATADevice<SCSIDevice>`
macro_rules! ata_do { ($Err:ty) => {
	pub fn ata_do(&self, dir: Direction, regs: &::ata::RegistersWrite) -> Result<(::ata::RegistersRead, Vec<u8>), $Err> {
		info!("issuing cmd: dir={:?} regs={:?}", dir, regs);

		// this one is implemented in `mod {linux,freebsd}`, and here for `T: SCSIDevice`
		let ret = Self::ata_platform_do(self, dir, regs);
		match &ret {
			Ok((regs, data)) => {
				debug!("cmd reply: regs={:?}", regs);
				// XXX does it make sense to use hexdump_16() instead of hexdump_8() if cmd is not IDENTIFY DEVICE?
				debug!("cmd data: {}", ::utils::hexdump_16be(&::utils::bytes_to_be_words(data)));
			},
			err => {
				debug!("cmd error: {:?}", err);
			},
		}
		ret
	}
} }

/*
One might notice there's no linux support here. There's a couple of reasons for that:
- generally available ioctls like HDIO_DRIVE_{CMD,TASK} are too specialized and unsuitable for writing generic code
- more generic ioctl, HDIO_DRIVE_TASKFILE is not available if kernel is not built with CONFIG_IDE_TASK_IOCTL (and it is indeed absent in modern mainstream distros)
- all HDIO_* ioctls are full of quirks like conditionally pre-filled and masked registers (see Documentation/ioctl/hdio.txt)
- CONFIG_IDE is disabled for a really long time in modern distros, and support for most of HDIO_* ioctls is absent from libata in favour of issuing ATA commangs through SG_IO, which is already covered in scsi module of this crate
*/
