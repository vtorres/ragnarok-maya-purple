use winapi::shared::minwindef::{ LPCVOID, LPVOID };
use winapi::um::winnt::{ HANDLE };
use winapi::um::memoryapi::{ ReadProcessMemory };
use winapi::um::wow64apiset::IsWow64Process;
use winapi::um::errhandlingapi::GetLastError;

/// Reads bytes in memory to a Vec<u8>.
pub fn read_bytes(handle: HANDLE, address: usize, amount: usize) -> Vec<u8> {
    let bytes: Vec<u8> = vec![0; amount];
    unsafe {
        if
            ReadProcessMemory(
                handle,
                address as LPCVOID,
                bytes.as_ptr() as LPVOID,
                amount,
                std::ptr::null_mut()
            ) == 0
        {
            panic!("ReadProcessMemory failed! Last OS Error: {}", GetLastError());
        }
    }
    bytes
}

/// Reads bytes in memory to an i64.
pub fn read_i64(handle: HANDLE, address: usize) -> i64 {
    read_primitive(handle, address)
}

/// Reads bytes in memory to an i32.
pub fn read_i32(handle: HANDLE, address: usize) -> i32 {
    read_primitive(handle, address)
}

/// Reads bytes in memory to an i16.
pub fn read_i16(handle: HANDLE, address: usize) -> i16 {
    read_primitive(handle, address)
}

/// Reads byte in memory to an i8.
pub fn read_i8(handle: HANDLE, address: usize) -> i8 {
    read_primitive(handle, address)
}

/// Reads bytes in memory to an u64.
pub fn read_u64(handle: HANDLE, address: usize) -> u64 {
    read_primitive(handle, address)
}

/// Reads bytes in memory to an u32.
pub fn read_u32(handle: HANDLE, address: usize) -> u32 {
    read_primitive(handle, address)
}

/// Reads bytes in memory to an u16.
pub fn read_u16(handle: HANDLE, address: usize) -> u16 {
    read_primitive(handle, address)
}

/// Reads byte in memory to an u8.
pub fn read_u8(handle: HANDLE, address: usize) -> u8 {
    read_primitive(handle, address)
}

/// Reads bytes in memory to a f64.
pub fn read_f64(handle: HANDLE, address: usize) -> f64 {
    read_primitive(handle, address)
}

/// Reads bytes in memory to a f32.
pub fn read_f32(handle: HANDLE, address: usize) -> f32 {
    read_primitive(handle, address)
}

fn read_primitive<T: Default>(handle: HANDLE, address: usize) -> T {
    let mut read_result = T::default();
    unsafe {
        if
            ReadProcessMemory(
                handle,
                address as LPCVOID,
                &mut read_result as *mut _ as LPVOID,
                std::mem::size_of::<T>(),
                std::ptr::null_mut()
            ) == 0
        {
            panic!("ReadProcessMemory failed! Last OS Error: {}", GetLastError());
        }
    }
    read_result
}

// Resolves multilevel pointer -> returns last address of a multi level pointer.
pub fn resolve_multi_level_pointer(handle: HANDLE, base_ptr: usize, offsets: Vec<usize>) -> usize {
    unsafe {
        // Default at 64 bit
        let mut pointer_size = 8;
        let mut is_32_bit: i32 = 0;

        if IsWow64Process(handle, &mut is_32_bit as *mut i32) == 0 {
            panic!("IsWow64Process failed! Last OS Error: {}", GetLastError());
        }

        // If 32 bit set pointer_size to 4 bytes.
        if is_32_bit > 0 {
            pointer_size = 4;
        }

        let mut address = base_ptr;
        let mut buffer: usize;

        for i in 0..offsets.len() {
            buffer = 0;
            if
                ReadProcessMemory(
                    handle,
                    address as LPCVOID,
                    &mut buffer as *mut _ as LPVOID,
                    pointer_size,
                    std::ptr::null_mut()
                ) == 0
            {
                panic!("ReadProcessMemory failed! Last OS Error: {}", GetLastError());
            }
            address = buffer + offsets[i];
        }

        address
    }
}