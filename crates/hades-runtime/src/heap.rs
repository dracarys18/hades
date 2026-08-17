pub fn alloc(size: usize, align: usize) -> *mut u8 {
    let _ = (size, align);
    core::ptr::null_mut()
}
