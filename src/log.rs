use core::fmt::{self, Write};

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
            let (payload, rest) = bytes.split_at(len as usize);
            bytes = rest;
            let mut bytes = [0; 120];
            bytes[..payload.len()].copy_from_slice(payload);

            fn b(bytes: &[u8; 120], idx: usize) -> u64 {
                let mut buf = [0; 8];
                buf.copy_from_slice(&bytes[idx * 8..][..8]);
                u64::from_le_bytes(buf)
            }

            macro_rules! b {
                ($b:expr) => {
                    b(&bytes, $b)
                };
            }

            unsafe {
                #[rustfmt::skip]
                log_append(
                    len,
                    b!(0), b!(1), b!(2), b!(3), b!(4), b!(5), b!(6), b!(7),
                    b!(8), b!(9), b!(10), b!(11), b!(12), b!(13), b!(14),
                );
            }
        }

        Ok(())
    }
}

pub fn log(level: i64, format: &fmt::Arguments) {
    extern "C" {
        fn log_emit(level: i64);
    }

    _ = write!(LogWriter, "{format}");

    unsafe { log_emit(level) };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        ::micrortu_sdk::log::log(1, &format_args!($($arg)*))
    }
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        ::micrortu_sdk::log::log(2, &format_args!($($arg)*))
    }
}
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        ::micrortu_sdk::log::log(3, &format_args!($($arg)*))
    }
}
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        ::micrortu_sdk::log::log(4, &format_args!($($arg)*))
    }
}
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        ::micrortu_sdk::log::log(5, &format_args!($($arg)*))
    }
}
