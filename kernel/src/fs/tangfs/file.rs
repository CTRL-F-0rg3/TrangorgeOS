use super::{TangFs, inode::Inode};
use crate::fs::vfs::Result;

pub fn read_file(fs: &TangFs, inode: &Inode, mut offset: u64, buf: &mut [u8]) -> Result<usize> {
    if offset >= inode.size {
        return Ok(0);
    }
    
    let end = core::cmp::min(offset + buf.len() as u64, inode.size);
    let mut bytes_read = 0;
    
    while offset < end {
        let logical_block = offset / 4096;
        let block_offset = (offset % 4096) as usize;
        
        let mut physical_block = None;
        
        for extent in &inode.extents {
            if extent.length == 0 {
                continue;
            }
            
            if logical_block >= extent.logical_block
                && logical_block < extent.logical_block + extent.length as u64
            {
                physical_block = Some(
                    extent.physical_block + (logical_block - extent.logical_block)
                );
                break;
            }
        }
        
        let physical_block = physical_block.ok_or("Block not mapped")?;
        
        let block_data = fs.read_block(physical_block)?;
        
        let bytes_in_block = core::cmp::min(4096 - block_offset, buf.len() - bytes_read);
        let copy_len = core::cmp::min(bytes_in_block, (end - offset) as usize);
        
        buf[bytes_read..bytes_read + copy_len]
            .copy_from_slice(&block_data[block_offset..block_offset + copy_len]);
        
        bytes_read += copy_len;
        offset += copy_len as u64;
    }
    
    Ok(bytes_read)
}

pub fn write_file(fs: &TangFs, inode: &mut Inode, mut offset: u64, data: &[u8]) -> Result<usize> {
    let mut bytes_written = 0;
    
    while bytes_written < data.len() {
        let logical_block = offset / 4096;
        let block_offset = (offset % 4096) as usize;
        
        let physical_block = allocate_block_for_inode(fs, inode, logical_block)?;
        
        let mut block_data = if block_offset > 0 {
            fs.read_block(physical_block)?
        } else {
            alloc::vec![0u8; 4096]
        };
        
        let bytes_in_block = core::cmp::min(4096 - block_offset, data.len() - bytes_written);
        
        block_data[block_offset..block_offset + bytes_in_block]
            .copy_from_slice(&data[bytes_written..bytes_written + bytes_in_block]);
        
        fs.write_block(physical_block, &block_data)?;
        
        bytes_written += bytes_in_block;
        offset += bytes_in_block as u64;
    }
    
    // Update size if needed
    if offset > inode.size {
        inode.size = offset;
        inode.mtime = crate::time::current_timestamp();
    }
    
    Ok(bytes_written)
}

fn allocate_block_for_inode(fs: &TangFs, inode: &mut Inode, logical_block: u64) -> Result<u64> {
    for extent in &inode.extents {
        if extent.length == 0 {
            continue;
        }
        
        if logical_block >= extent.logical_block
            && logical_block < extent.logical_block + extent.length as u64
        {
            return Ok(extent.physical_block + (logical_block - extent.logical_block));
        }
    }
    
    let new_block = super::extent::allocate_block(fs)?;
    

    for extent in &mut inode.extents {
        if extent.length == 0 {
            extent.logical_block = logical_block;
            extent.physical_block = new_block;
            extent.length = 1;
            extent.flags = 0;
            return Ok(new_block);
        }
    }
    
    Err("No free extent slots in inode")
}