use rmm::*;

unsafe fn dump_tables<A: Arch>(table: PageTable<A>) {
    let level = table.level();
    for i in 0..PAGE_ENTRIES {
        if level == 0 {
            if let Some(entry) = table.entry(i) {
                if entry.present() {
                    let base = table.entry_base(i).unwrap();
                    println!("0x{:X}: 0x{:X}", base.data(), entry.address().data());
                }
            }
        } else {
            if let Some(next) = table.next(i) {
                dump_tables(next);
            }
        }
    }
}

unsafe fn inner<A: Arch>() {
    let areas = A::init();

    // Debug table
    dump_tables(PageTable::<A>::top());

    let megabyte = 0x100000;

    // Test read
    println!("0x{:X} = 0x{:X}", megabyte, A::read::<u8>(megabyte));

    // Test write
    A::write::<u8>(megabyte, 0x5A);

    // Test read
    println!("0x{:X} = 0x{:X}", megabyte, A::read::<u8>(megabyte));
}

fn main() {
    unsafe {
        inner::<EmulateArch>();
    }
}
