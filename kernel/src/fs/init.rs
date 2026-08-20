pub fn init() {
    vfs::register_ext4();
    vfs::register_fat32();
    vfs::register_tangfs();  // Nowy filesystem
    
    // ... reszta inicjalizacji XD
}