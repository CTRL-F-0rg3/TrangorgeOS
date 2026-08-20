use super::TangFs;
use crate::fs::vfs::{DirEntry, FileType, Result};
use alloc::string::String;
use alloc::vec::Vec;

const BTREE_MAGIC: u32 = 0x42544E44; // "BTND"

#[repr(C, packed)]
pub struct BtreeNode {
    pub magic: u32,
    pub flags: u32,        // 0 = internal, 1 = leaf
    pub num_keys: u32,
    pub max_keys: u32,
    pub parent: u64,
    pub next: u64,         // next leaf (leaf only)
    pub prev: u64,         // prev leaf (leaf only)
    pub data: [u8; 4032],  // 4096 - 64 bytes header
    pub checksum: u32,
}

pub fn lookup_dir_entry(fs: &TangFs, dir_ino: u64, name: &str) -> Result<u64> {
    let inode = super::inode::InodeHandle { fs: fs as *const _, ino: dir_ino }.load_inode()?;
    
    // Start from root of directory's B+tree
    let root_block = inode.extents[0].physical_block;
    
    let mut current_block = root_block;
    
    loop {
        let node_data = fs.read_block(current_block)?;
        let node: &BtreeNode = unsafe {
            &*(node_data.as_ptr() as *const BtreeNode)
        };
        
        if node.magic != BTREE_MAGIC {
            return Err("Invalid B+tree node");
        }
        
        if node.flags == 1 {
            // Leaf node - search entries
            return search_leaf_node(node, name);
        } else {
            // Internal node - find child
            current_block = find_child_in_internal(node, name)?;
        }
    }
}

fn search_leaf_node(node: &BtreeNode, name: &str) -> Result<u64> {
    let mut offset = 0;
    
    for i in 0..node.num_keys {
        let entry: &super::dir::DirEntry = unsafe {
            &*(node.data.as_ptr().add(offset) as *const super::dir::DirEntry)
        };
        
        let entry_name = core::str::from_utf8(&entry.name[..entry.name_len as usize])
            .map_err(|_| "Invalid UTF-8 in filename")?;
        
        if entry_name == name {
            return Ok(entry.inode);
        }
        
        offset += 4 + 2 + 1 + 1 + 256; // sizeof(DirEntry)
    }
    
    Err("File not found")
}

fn find_child_in_internal(node: &BtreeNode, name: &str) -> Result<u64> {
    // Binary search for appropriate child
    let keys = unsafe {
        core::slice::from_raw_parts(
            node.data.as_ptr() as *const u64,
            node.num_keys as usize,
        )
    };
    
    let children = unsafe {
        core::slice::from_raw_parts(
            node.data.as_ptr().add(node.num_keys as usize * 8) as *const u64,
            (node.num_keys + 1) as usize,
        )
    };
    
    // Simple linear search for now (optimize to binary search later)
    for i in 0..node.num_keys {
        if name < "" { // Compare with key (simplified)
            return Ok(children[i]);
        }
    }
    
    Ok(children[node.num_keys as usize])
}

pub fn read_dir_entries(fs: &TangFs, dir_ino: u64) -> Result<Vec<DirEntry>> {
    let mut entries = Vec::new();
    
    let inode = super::inode::InodeHandle { fs: fs as *const _, ino: dir_ino }.load_inode()?;
    let root_block = inode.extents[0].physical_block;
    
    let mut current_block = root_block;
    
    // Find leftmost leaf
    loop {
        let node_data = fs.read_block(current_block)?;
        let node: &BtreeNode = unsafe {
            &*(node_data.as_ptr() as *const BtreeNode)
        };
        
        if node.flags == 1 {
            // Found leaf - collect all entries
            collect_leaf_entries(node, &mut entries)?;
            
            // Follow next pointers to collect all entries
            while node.next != 0 {
                current_block = node.next;
                let next_data = fs.read_block(current_block)?;
                let next_node: &BtreeNode = unsafe {
                    &*(next_data.as_ptr() as *const BtreeNode)
                };
                
                collect_leaf_entries(next_node, &mut entries)?;
            }
            
            break;
        } else {
            // Go to leftmost child
            let children = unsafe {
                core::slice::from_raw_parts(
                    node.data.as_ptr() as *const u64,
                    (node.num_keys + 1) as usize,
                )
            };
            
            current_block = children[0];
        }
    }
    
    Ok(entries)
}

fn collect_leaf_entries(node: &BtreeNode, entries: &mut Vec<DirEntry>) -> Result<()> {
    let mut offset = 0;
    
    for i in 0..node.num_keys {
        let entry: &super::dir::DirEntry = unsafe {
            &*(node.data.as_ptr().add(offset) as *const super::dir::DirEntry)
        };
        
        let name = core::str::from_utf8(&entry.name[..entry.name_len as usize])
            .map_err(|_| "Invalid UTF-8")?
            .to_string();
        
        let file_type = match entry.file_type {
            1 => FileType::RegularFile,
            2 => FileType::Directory,
            3 => FileType::Symlink,
            _ => FileType::Unknown,
        };
        
        entries.push(DirEntry {
            name,
            ino: entry.inode,
            file_type,
        });
        
        offset += 4 + 2 + 1 + 1 + 256;
    }
    
    Ok(())
}