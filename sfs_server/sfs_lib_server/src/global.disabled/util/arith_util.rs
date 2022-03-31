
pub fn is_pow_of_2(n: u64) -> bool{
    n != 0 && (n & (n - 1) == 0)
}
pub fn block_overrun(offset: i64, chunk_size: u64) -> u64{
    assert!(is_pow_of_2(chunk_size));
    offset as u64 & (chunk_size - 1)
}
pub fn block_index(offset: i64, chunk_size: u64) -> u64{
    assert!(is_pow_of_2(chunk_size));
    align_left(offset, chunk_size) >> ((chunk_size as f64).log2() as u64)
}
pub fn chunk_align_down(offset: i64, chunk_size: u64) -> i64{
    offset & !(chunk_size - 1) as i64
}
pub fn chunk_align_up(offset: i64, chunk_size: u64) -> i64{
    chunk_align_down(offset + chunk_size as i64, chunk_size)
}
pub fn offset_to_chunk_id(offset: i64, chunk_size: u64) -> u64{
    //(chunk_align_down(offset, chunk_size) >> ((chunk_size as f64).log2() as i64)) as u64
    offset as u64 / chunk_size
}
pub fn align_left(offset: i64, chunk_size: u64) -> u64{
    assert!(is_pow_of_2(chunk_size));
    offset as u64 & !(chunk_size - 1)
}
pub fn chunk_lpad(offset: i64, chunk_size: u64) -> u64{
    offset as u64 % chunk_size
}
pub fn chunk_rpad(offset: i64, chunk_size: u64) -> u64{
    let res = - offset % chunk_size as i64;
    if res == 0{
        res as u64
    }
    else{
        (res + chunk_size as i64) as u64
    }
}