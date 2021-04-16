#[cfg(target_arch="wasm32")]
extern crate alloc;

#[cfg(target_arch="wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type Position = u8;

mod board;

#[no_mangle]
pub unsafe extern fn alloc_memory(byte_size: u64) -> *mut u8 {
    vec![0u8; byte_size as _].leak() as *const _ as _
}

#[no_mangle]
pub unsafe extern fn free_memory(ptr: *mut u8, byte_size: u64) {
    Vec::from_raw_parts(ptr, byte_size as _, byte_size as _);
}

