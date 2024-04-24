/// Advances through `array` until the null terminator element is
/// reached and returns the address of that element.
pub(super) unsafe fn find_term(array: *const *const u8) -> *const *const u8 {
    let mut ptr = array;
    while !(*ptr).is_null() {
        ptr = ptr.add(1);
    }
    ptr
}
