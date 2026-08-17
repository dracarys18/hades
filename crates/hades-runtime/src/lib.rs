#![no_std]
#![no_main]

mod heap;

use core::ffi::{c_char, c_int};

extern "C" {
    fn real_main() -> c_int;
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main(argc: c_int, argv: *const *const c_char) -> c_int {
    rt_init(argc, argv);
    let code = unsafe { real_main() };
    rt_shutdown();
    code
}

#[no_mangle]
pub extern "C" fn rt_init(_argc: c_int, _argv: *const *const c_char) {}

#[no_mangle]
pub extern "C" fn rt_shutdown() {}

#[no_mangle]
pub extern "C" fn alloc(size: usize, align: usize) -> *mut u8 {
    heap::alloc(size, align)
}
