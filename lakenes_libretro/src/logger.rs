use log::{Level, Metadata, Record};
use libc::{c_char, c_uint, c_void};
use std::ffi::CString;

pub type retro_log_printf_t = extern "C" fn(level: c_uint, fmt: *const c_char, ...);

#[repr(C)]
pub struct retro_log_callback {
    pub log: Option<retro_log_printf_t>,
}

pub struct LibretroLogger {
    pub printf: Option<retro_log_printf_t>,
}

impl log::Log for LibretroLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if let Some(printf) = self.printf {
            let level = match record.level() {
                Level::Error => 0,
                Level::Warn => 1,
                Level::Info => 2,
                Level::Debug => 3,
                Level::Trace => 3,
            };
            
            // Using a simple format for the message
            let msg = format!("[LakeNES] {}\0", record.args());
            if let Ok(c_msg) = CString::new(msg) {
                // We can't easily pass variadic arguments from Rust to C variadic functions
                // without using some tricks or just passing a single %s string.
                // libretro log_printf is usually: void log_printf(enum retro_log_level level, const char *fmt, ...)
                unsafe {
                    printf(level, b"%s\0".as_ptr() as *const c_char, c_msg.as_ptr());
                }
            }
        }
    }

    fn flush(&self) {}
}

static mut GLOBAL_LOGGER: LibretroLogger = LibretroLogger { printf: None };

pub unsafe fn init(printf: Option<retro_log_printf_t>) {
    GLOBAL_LOGGER.printf = printf;
    let _ = log::set_logger(&GLOBAL_LOGGER);
    log::set_max_level(log::LevelFilter::Info);
}
