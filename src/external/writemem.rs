use winapi::um::memoryapi::{ WriteProcessMemory, VirtualProtectEx };
use winapi::shared::minwindef::{ LPCVOID, LPVOID };
use winapi::um::winnt::{ HANDLE, PAGE_EXECUTE_READWRITE };
use winapi::um::errhandlingapi::GetLastError;

/// Writes bytes in Vec<u8> to memory location.
pub fn write_bytes(handle: HANDLE, address: usize, bytes: Vec<u8>) {
    unsafe {
        if
            WriteProcessMemory(
                handle,
                address as LPVOID,
                bytes.as_ptr() as LPVOID,
                bytes.len(),
                std::ptr::null_mut()
            ) == 0
        {
            panic!("Last OS Error: {}", GetLastError());
        }
    }
}

/// Writes an i64 to memory.
pub fn write_i64(handle: HANDLE, address: usize, to_write: i64) {
    write_primitive(handle, address, to_write);
}

/// Writes an i32 to memory.
pub fn write_i32(handle: HANDLE, address: usize, to_write: i32) {
    write_primitive(handle, address, to_write);
}

/// Writes an i16 to memory.
pub fn write_i16(handle: HANDLE, address: usize, to_write: i16) {
    write_primitive(handle, address, to_write);
}

/// Writes an i8 to memory.
pub fn write_i8(handle: HANDLE, address: usize, to_write: i8) {
    write_primitive(handle, address, to_write);
}

/// Writes an u64 to memory.
pub fn write_u64(handle: HANDLE, address: usize, to_write: u64) {
    write_primitive(handle, address, to_write);
}

/// Writes an u32 to memory.
pub fn write_u32(handle: HANDLE, address: usize, to_write: u32) {
    write_primitive(handle, address, to_write);
}

/// Writes an u16 to memory.
pub fn write_u16(handle: HANDLE, address: usize, to_write: u16) {
    write_primitive(handle, address, to_write);
}

/// Writes an u8 to memory.
pub fn write_u8(handle: HANDLE, address: usize, to_write: u8) {
    write_primitive(handle, address, to_write);
}

/// Writes an f32 to memory.
pub fn write_f32(handle: HANDLE, address: usize, to_write: f32) {
    write_primitive(handle, address, to_write);
}

/// Writes an f64 to memory.
pub fn write_f64(handle: HANDLE, address: usize, to_write: f64) {
    write_primitive(handle, address, to_write);
}

fn write_primitive<T>(handle: HANDLE, address: usize, to_write: T) {
    unsafe {
        if
            WriteProcessMemory(
                handle,
                address as LPVOID,
                &to_write as *const _ as LPCVOID,
                std::mem::size_of::<T>(),
                std::ptr::null_mut()
            ) == 0
        {
            panic!("Last OS Error: {}", GetLastError());
        }
    }
}

// Nop instructions
pub fn nop_ex(handle: HANDLE, address: usize, size: usize) {
    let nop_array: Vec<u8> = vec![0; size];

    unsafe {
        std::ptr::write_bytes(nop_array.as_ptr() as *mut u8, 0x90, size);
    }

    patch_ex(handle, address, nop_array);
}

/// Changes memory protection to writeable and overrides memory. Then sets memory protection to original state.
pub fn patch_ex(handle: HANDLE, address: usize, bytes: Vec<u8>) {
    let mut last_protection: u32 = 0;
    let byte_amount = bytes.len();

    unsafe {
        if
            VirtualProtectEx(
                handle,
                address as LPVOID,
                byte_amount,
                PAGE_EXECUTE_READWRITE,
                &mut last_protection as *mut _
            ) == 0
        {
            panic!("Last OS Error: {}", GetLastError());
        }

        write_bytes(handle, address, bytes);

        if
            VirtualProtectEx(
                handle,
                address as LPVOID,
                byte_amount,
                last_protection,
                &mut last_protection as *mut _
            ) == 0
        {
            panic!("Last OS Error: {}", GetLastError());
        }
    }
}