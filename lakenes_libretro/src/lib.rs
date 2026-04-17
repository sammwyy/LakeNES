use lakenes_core::NES;
use libc::{c_char, c_float, c_uint, c_void, size_t};
use parking_lot::Mutex;
use std::ptr;

mod logger;

// --- Libretro Constants ---
pub const RETRO_API_VERSION: c_uint = 1;

pub const RETRO_DEVICE_MASK: c_uint = 0xff;
pub const RETRO_DEVICE_NONE: c_uint = 0;
pub const RETRO_DEVICE_JOYPAD: c_uint = 1;

pub const RETRO_DEVICE_ID_JOYPAD_B: c_uint = 0;
pub const RETRO_DEVICE_ID_JOYPAD_Y: c_uint = 1;
pub const RETRO_DEVICE_ID_JOYPAD_SELECT: c_uint = 2;
pub const RETRO_DEVICE_ID_JOYPAD_START: c_uint = 3;
pub const RETRO_DEVICE_ID_JOYPAD_UP: c_uint = 4;
pub const RETRO_DEVICE_ID_JOYPAD_DOWN: c_uint = 5;
pub const RETRO_DEVICE_ID_JOYPAD_LEFT: c_uint = 6;
pub const RETRO_DEVICE_ID_JOYPAD_RIGHT: c_uint = 7;
pub const RETRO_DEVICE_ID_JOYPAD_A: c_uint = 8;
pub const RETRO_DEVICE_ID_JOYPAD_X: c_uint = 9;
pub const RETRO_DEVICE_ID_JOYPAD_L: c_uint = 10;
pub const RETRO_DEVICE_ID_JOYPAD_R: c_uint = 11;
pub const RETRO_DEVICE_ID_JOYPAD_L2: c_uint = 12;
pub const RETRO_DEVICE_ID_JOYPAD_R2: c_uint = 13;
pub const RETRO_DEVICE_ID_JOYPAD_L3: c_uint = 14;
pub const RETRO_DEVICE_ID_JOYPAD_R3: c_uint = 15;

pub const RETRO_REGION_NTSC: c_uint = 0;
pub const RETRO_REGION_PAL: c_uint = 1;

pub const RETRO_ENVIRONMENT_GET_LOG_INTERFACE: c_uint = 27;
pub const RETRO_ENVIRONMENT_SET_PIXEL_FORMAT: c_uint = 10;

pub enum retro_pixel_format {
    RETRO_PIXEL_FORMAT_0RGB1555 = 0,
    RETRO_PIXEL_FORMAT_XRGB8888 = 1,
    RETRO_PIXEL_FORMAT_RGB565 = 2,
}

#[repr(C)]
pub struct retro_system_info {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

#[repr(C)]
pub struct retro_game_geometry {
    pub base_width: c_uint,
    pub base_height: c_uint,
    pub max_width: c_uint,
    pub max_height: c_uint,
    pub aspect_ratio: c_float,
}

#[repr(C)]
pub struct retro_system_timing {
    pub fps: f64,
    pub sample_rate: f64,
}

#[repr(C)]
pub struct retro_system_av_info {
    pub geometry: retro_game_geometry,
    pub timing: retro_system_timing,
}

#[repr(C)]
pub struct retro_game_info {
    pub path: *const c_char,
    pub data: *const c_void,
    pub size: size_t,
    pub meta: *const c_char,
}

// --- Callbacks ---
pub type retro_environment_t = extern "C" fn(cmd: c_uint, data: *mut c_void) -> bool;
pub type retro_video_refresh_t =
    extern "C" fn(data: *const c_void, width: c_uint, height: c_uint, pitch: size_t);
pub type retro_audio_sample_t = extern "C" fn(left: i16, right: i16);
pub type retro_audio_sample_batch_t = extern "C" fn(data: *const i16, frames: size_t) -> size_t;
pub type retro_input_poll_t = extern "C" fn();
pub type retro_input_state_t =
    extern "C" fn(port: c_uint, device: c_uint, index: c_uint, id: c_uint) -> i16;

static mut ENV_CB: Option<retro_environment_t> = None;
static mut VIDEO_CB: Option<retro_video_refresh_t> = None;
static mut AUDIO_CB: Option<retro_audio_sample_t> = None;
static mut AUDIO_BATCH_CB: Option<retro_audio_sample_batch_t> = None;
static mut POLL_CB: Option<retro_input_poll_t> = None;
static mut INPUT_CB: Option<retro_input_state_t> = None;

// --- Global State ---
struct LibretroState {
    nes: NES,
    audio_buffer: Vec<i16>,
}

static STATE: Mutex<Option<LibretroState>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn retro_init() {}

#[no_mangle]
pub extern "C" fn retro_deinit() {
    let mut state = STATE.lock();
    *state = None;
}

#[no_mangle]
pub extern "C" fn retro_api_version() -> c_uint {
    RETRO_API_VERSION
}

#[no_mangle]
pub extern "C" fn retro_set_environment(cb: retro_environment_t) {
    unsafe {
        ENV_CB = Some(cb);

        // 1. Initialize logging
        let mut log_cb = logger::retro_log_callback { log: None };
        if cb(
            RETRO_ENVIRONMENT_GET_LOG_INTERFACE,
            &mut log_cb as *mut _ as *mut c_void,
        ) {
            logger::init(log_cb.log);
            log::info!("Logger initialized");
        }
    }
}

#[no_mangle]
pub extern "C" fn retro_set_video_refresh(cb: retro_video_refresh_t) {
    unsafe {
        VIDEO_CB = Some(cb);
    }
}

#[no_mangle]
pub extern "C" fn retro_set_audio_sample(cb: retro_audio_sample_t) {
    unsafe {
        AUDIO_CB = Some(cb);
    }
}

#[no_mangle]
pub extern "C" fn retro_set_audio_sample_batch(cb: retro_audio_sample_batch_t) {
    unsafe {
        AUDIO_BATCH_CB = Some(cb);
    }
}

#[no_mangle]
pub extern "C" fn retro_set_input_poll(cb: retro_input_poll_t) {
    unsafe {
        POLL_CB = Some(cb);
    }
}

#[no_mangle]
pub extern "C" fn retro_set_input_state(cb: retro_input_state_t) {
    unsafe {
        INPUT_CB = Some(cb);
    }
}

#[no_mangle]
pub extern "C" fn retro_get_system_info(info: *mut retro_system_info) {
    unsafe {
        (*info).library_name = b"lakenes_libretro\0".as_ptr() as *const c_char;
        (*info).library_version = b"0.1.0\0".as_ptr() as *const c_char;
        (*info).valid_extensions = b"nes|fds\0".as_ptr() as *const c_char;
        (*info).need_fullpath = true;
        (*info).block_extract = false;
    }
}

#[no_mangle]
pub extern "C" fn retro_get_system_av_info(info: *mut retro_system_av_info) {
    unsafe {
        (*info).geometry = retro_game_geometry {
            base_width: 256,
            base_height: 240,
            max_width: 256,
            max_height: 240,
            aspect_ratio: 4.0 / 3.0,
        };
        (*info).timing = retro_system_timing {
            fps: 60.09881186, // NTSC
            sample_rate: 44100.0,
        };
    }
}

#[no_mangle]
pub extern "C" fn retro_set_controller_port_device(_port: c_uint, _device: c_uint) {}

#[no_mangle]
pub extern "C" fn retro_reset() {
    let mut state = STATE.lock();
    if let Some(ref mut s) = *state {
        s.nes.hard_reset();
    }
}

#[no_mangle]
pub extern "C" fn retro_load_game(info: *const retro_game_info) -> bool {
    if info.is_null() {
        return false;
    }

    let game = unsafe { &*info };
    let mut rom_data: Vec<u8> = Vec::new();

    // 1. Try to load from memory buffer
    if !game.data.is_null() && game.size > 0 {
        rom_data =
            unsafe { std::slice::from_raw_parts(game.data as *const u8, game.size) }.to_vec();
    }
    // 2. Fallback to loading from path if memory buffer is missing
    else if !game.path.is_null() {
        let path = unsafe { std::ffi::CStr::from_ptr(game.path) }.to_string_lossy();
        match std::fs::read(path.as_ref()) {
            Ok(data) => rom_data = data,
            Err(_) => return false,
        }
    }

    if rom_data.is_empty() {
        return false;
    }

    // Set pixel format
    unsafe {
        if let Some(cb) = ENV_CB {
            let mut format = retro_pixel_format::RETRO_PIXEL_FORMAT_XRGB8888 as u32;
            cb(
                RETRO_ENVIRONMENT_SET_PIXEL_FORMAT,
                &mut format as *mut _ as *mut c_void,
            );
        }
    }

    // Catch panics from lakenes_core (e.g. .expect() during ROM parsing)
    let nes_result = std::panic::catch_unwind(|| NES::new(&rom_data));

    match nes_result {
        Ok(nes) => {
            let mut state = STATE.lock();
            *state = Some(LibretroState {
                nes,
                audio_buffer: Vec::with_capacity(4096),
            });
            true
        }
        Err(_) => false,
    }
}

#[no_mangle]
pub extern "C" fn retro_load_game_special(
    _game_type: c_uint,
    _info: *const retro_game_info,
    _num_info: size_t,
) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn retro_unload_game() {
    let mut state = STATE.lock();
    *state = None;
}

#[no_mangle]
pub extern "C" fn retro_get_region() -> c_uint {
    RETRO_REGION_NTSC
}

#[no_mangle]
pub extern "C" fn retro_get_memory_data(_id: c_uint) -> *mut c_void {
    ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn retro_get_memory_size(_id: c_uint) -> size_t {
    0
}

#[no_mangle]
pub extern "C" fn retro_run() {
    let mut state_lock = STATE.lock();
    let state = match state_lock.as_mut() {
        Some(s) => s,
        None => return,
    };

    // 1. Poll input
    unsafe {
        if let Some(poll) = POLL_CB {
            poll();
        }

        if let Some(input) = INPUT_CB {
            let mut buttons = 0u8;
            if input(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_A) != 0 {
                buttons |= 0x01;
            }
            if input(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_B) != 0 {
                buttons |= 0x02;
            }
            if input(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_SELECT) != 0 {
                buttons |= 0x04;
            }
            if input(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_START) != 0 {
                buttons |= 0x08;
            }
            if input(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_UP) != 0 {
                buttons |= 0x10;
            }
            if input(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_DOWN) != 0 {
                buttons |= 0x20;
            }
            if input(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_LEFT) != 0 {
                buttons |= 0x40;
            }
            if input(0, RETRO_DEVICE_JOYPAD, 0, RETRO_DEVICE_ID_JOYPAD_RIGHT) != 0 {
                buttons |= 0x80;
            }

            // lakenes_core currently reads input from player 1.
            state.nes.update_joypad(1, buttons);
        }
    }

    // 2. Step frame
    state.nes.step_frame();

    // 3. Render video
    let framebuffer = state.nes.get_frame_buffer();
    unsafe {
        if let Some(video) = VIDEO_CB {
            video(framebuffer.as_ptr() as *const c_void, 256, 240, 256 * 4);
        }
    }

    // 4. Render audio
    state.audio_buffer.clear();
    while state.nes.audio_buffer_len() > 0 {
        let sample = state.nes.get_audio_sample();
        // Convert f32 [-1.0, 1.0] to i16
        let sample_i16 = (sample * 32767.0) as i16;
        state.audio_buffer.push(sample_i16); // Left
        state.audio_buffer.push(sample_i16); // Right (Mono)
    }

    unsafe {
        if let Some(audio_batch) = AUDIO_BATCH_CB {
            audio_batch(state.audio_buffer.as_ptr(), state.audio_buffer.len() / 2);
        }
    }
}

#[no_mangle]
pub extern "C" fn retro_serialize_size() -> size_t {
    0
}

#[no_mangle]
pub extern "C" fn retro_serialize(_data: *mut c_void, _size: size_t) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn retro_unserialize(_data: *const c_void, _size: size_t) -> bool {
    false
}

#[no_mangle]
pub extern "C" fn retro_cheat_reset() {}

#[no_mangle]
pub extern "C" fn retro_cheat_set(_index: c_uint, _enabled: bool, _code: *const c_char) {}
