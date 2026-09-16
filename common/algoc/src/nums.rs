/// Gets width of `usize` through division of 10.
pub fn get_num_width_usize(num: usize) -> usize {
    let mut width = 0;
    let mut i = num;
    while i != 0 {
        i /= 10;
        width += 1;
    }
    width
}

/// Gets width of `u32` through division of 10.
pub fn get_num_width_u32(num: u32) -> u32 {
    let mut width = 0;
    let mut i = num;
    while i != 0 {
        i /= 10;
        width += 1;
    }
    width
}
