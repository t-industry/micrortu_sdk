use core::convert::Infallible;

use ufmt::uWrite;

#[doc(hidden)]
pub struct LogWriter;

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
fn log_append(len: u64, b1: u64, b2: u64, b3: u64, b4: u64, b5: u64,
    b6: u64, b7: u64, b8: u64, b9: u64, b10: u64, b11: u64,
    b12: u64, b13: u64, b14: u64, b15: u64) {
    extern "C" {
        fn log_append(len: u64, b1: u64, b2: u64, b3: u64, b4: u64, b5: u64,
            b6: u64, b7: u64, b8: u64, b9: u64, b10: u64, b11: u64,
            b12: u64, b13: u64, b14: u64, b15: u64);
    }
    unsafe {
        log_append(len, b1, b2, b3, b4, b5, b6, b7, b8, b9, b10, b11, b12, b13, b14, b15)
    }
}

#[doc(hidden)]
pub fn log_emit(level: i64) {
    extern "C" {
        fn log_emit(level: i64);
    }
    unsafe { log_emit(level) }
}

impl uWrite for LogWriter {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
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

            #[rustfmt::skip]
            log_append(
                len,
                b!(0), b!(1), b!(2), b!(3), b!(4), b!(5), b!(6), b!(7),
                b!(8), b!(9), b!(10), b!(11), b!(12), b!(13), b!(14),
            );
        }

        Ok(())
    }
}

#[cfg(not(feature = "home"))]
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {{
        _ = ::micrortu_sdk::ufmt::uwrite!(::micrortu_sdk::log::LogWriter, $($arg)*);
        ::micrortu_sdk::log::log_emit($level);
    }}
}

#[cfg(feature = "home")]
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {{
        match $level {
            1 => log::error!($($arg)*),
            2 => log::warn!($($arg)*),
            3 => log::info!($($arg)*),
            4 => log::debug!($($arg)*),
            5 => log::trace!($($arg)*),
            _ => unreachable!(),
        }
    }}
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        ::micrortu_sdk::log!(1, $($arg)*)
    }
}
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        ::micrortu_sdk::log!(2, $($arg)*)
    }
}
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        ::micrortu_sdk::log!(3, $($arg)*)
    }
}
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        ::micrortu_sdk::log!(4, $($arg)*)
    }
}
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        ::micrortu_sdk::log!(5, $($arg)*)
    }
}
