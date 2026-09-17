use super::TangFs;
use crate::fs::vfs::Result;

pub fn allocate_block(fs: &TangFs) -> Result<u64> {
    let mut sb = fs.superblock.borrow_mut();
    
    if sb.free_blocks == 0 {
        return Err("No free blocks");
    }
    

    let bitmap_block = 2; 
    let mut bitmap = fs.read_block(bitmap_block)?;
    
    for i in 0..bitmap.len() {
        if bitmap[i] != 0xFF {
            for bit in 0..8 {
                if bitmap[i] & (1 << bit) == 0 {
                    bitmap[i] |= 1 << bit;
                    fs.write_block(bitmap_block, &bitmap)?;
                    
                    sb.free_blocks -= 1;
                    sb.write(fs.device)?;
                    
                    let block = (i * 8 + bit) as u64;
                    return Ok(block);
                }
            }
        }
    }
    
    Err("No free blocks found in bitmap")
}

pub fn free_block(fs: &TangFs, block: u64) -> Result<()> {
    let bitmap_block = 2;
    let mut bitmap = fs.read_block(bitmap_block)?;
    
    let byte_index = (block / 8) as usize;
    let bit_index = (block % 8) as u8;
    
    if byte_index >= bitmap.len() {
        return Err("Block out of range");
    }
    
    bitmap[byte_index] &= !(1 << bit_index);
    fs.write_block(bitmap_block, &bitmap)?;
    
    let mut sb = fs.superblock.borrow_mut();
    sb.free_blocks += 1;
    sb.write(fs.device)?;
    
    Ok(())
}