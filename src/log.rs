use core::fmt::{self, Write};

pub(crate) struct MicroRTULog;
pub(crate) static LOGGER: MicroRTULog = MicroRTULog;

struct LogWriter;
impl Write for LogWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        #[rustfmt::skip]
        extern "C" {
            fn log_append(len: u64, b1: u64, b2: u64, b3: u64, b4: u64, b5: u64,
                b6: u64, b7: u64, b8: u64, b9: u64, b10: u64, b11: u64,
                b12: u64, b13: u64, b14: u64, b15: u64);
        }
        let mut bytes = s.as_bytes();

        while !bytes.is_empty() {
            let len = bytes.len().min(120) as u64;
            let (bytes_120, rest) = bytes.split_at(len as usize);
            bytes = rest;

            macro_rules! b {
                ($b:expr) => {
                    'blk: {
                        let Some(b) = bytes_120.get(($b - 1) * 8..) else {
                            break 'blk 0;
                        };
                        let Some(b) = b.get(..8) else {
                            break 'blk 0;
                        };
                        u64::from_ne_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
                    }
                };
            }

            unsafe {
                #[rustfmt::skip]
                log_append(
                    len,
                    b!(1), b!(2), b!(3), b!(4), b!(5), b!(6), b!(7), b!(8),
                    b!(9), b!(10), b!(11), b!(12), b!(13), b!(14), b!(15),
                );
            }
        }

        Ok(())
    }
}

impl log::Log for MicroRTULog {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        extern "C" {
            fn log_emit(level: i64);
        }

        _ = write!(LogWriter, "{}", record.args());

        let level = match record.level() {
            log::Level::Error => 1,
            log::Level::Warn => 2,
            log::Level::Info => 3,
            log::Level::Debug => 4,
            log::Level::Trace => 5,
        };

        unsafe {
            log_emit(level);
        }
    }

    fn flush(&self) {}
}
