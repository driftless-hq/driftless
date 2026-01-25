use sysinfo::{System, Disks};

fn main() {
    let mut system = System::new();
    system.refresh_disks_list();
    
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        println!("Disk: {:?}", disk.name());
        // Try to see what methods are available
        println!("Available methods:");
        println!("  kind: {:?}", disk.kind());
        println!("  file_system: {:?}", disk.file_system());
        println!("  mount_point: {:?}", disk.mount_point());
        println!("  total_space: {}", disk.total_space());
        println!("  available_space: {}", disk.available_space());
        println!("  is_removable: {}", disk.is_removable());
        
        // Check for I/O methods - these might exist in newer versions
        // println!("  read_bytes: {}", disk.read_bytes());
        // println!("  written_bytes: {}", disk.written_bytes());
        // println!("  read_ops: {}", disk.read_ops());
        // println!("  write_ops: {}", disk.write_ops());
    }
}
