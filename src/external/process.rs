use crate::external::module::{ Module };
use std::mem;
use std::ffi::{ CString, CStr };
use winapi::um::{
    tlhelp32::*,
    handleapi::{ CloseHandle, INVALID_HANDLE_VALUE },
    errhandlingapi::GetLastError,
    winnt::HANDLE,
};
use regex::bytes::Regex;
use winapi::shared::minwindef::DWORD;
use winapi::um::memoryapi::{ VirtualProtectEx, VirtualQueryEx };
use winapi::um::winnt::{ MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_NOACCESS, PAGE_READWRITE };
use winapi::{
    shared::{ basetsd::SIZE_T, minwindef::{ FALSE, LPCVOID, LPVOID } },
    um::{ memoryapi::{ ReadProcessMemory, WriteProcessMemory } },
};

/// Returns an Option which contains a Process ID if present.
pub fn get_process_id(process_name: &str) -> Option<u32> {
    unsafe {
        let h_process_snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        let process_name = CString::new(process_name).expect(
            "Couldn't convert process_name to CString!"
        );

        if h_process_snap == INVALID_HANDLE_VALUE {
            panic!("Invalid handle value! Last OS Error: {}", GetLastError());
        }

        let mut pe32: PROCESSENTRY32 = std::mem::zeroed();
        pe32.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;

        if Process32First(h_process_snap, &mut pe32) != 0 {
            let current_exe_name = CStr::from_ptr(&pe32.szExeFile as *const i8);
            if current_exe_name == process_name.as_c_str() {
                let id = pe32.th32ProcessID;
                CloseHandle(h_process_snap);
                return Some(id);
            }
        }

        while Process32Next(h_process_snap, &mut pe32) != 0 {
            let current_exe_name = CStr::from_ptr(&pe32.szExeFile as *const i8);
            if current_exe_name == process_name.as_c_str() {
                let id = pe32.th32ProcessID;
                CloseHandle(h_process_snap);
                return Some(id);
            }
        }
        CloseHandle(h_process_snap);
    }
    None
}

/// Returns an Option containing  the base address of a module (as usize).
pub fn get_module_base(process_id: u32, module_name: &str) -> Option<usize> {
    unsafe {
        let m_name = CString::new(module_name).expect("Couldn't convert module_name to CString!");
        let h_module_snap = CreateToolhelp32Snapshot(
            TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32,
            process_id
        );

        if h_module_snap == INVALID_HANDLE_VALUE {
            CloseHandle(h_module_snap);
            panic!("Invalid handle value! Last OS Error: {}", GetLastError());
        }

        let mut module_entry: MODULEENTRY32 = mem::zeroed();
        module_entry.dwSize = mem::size_of::<MODULEENTRY32>() as u32;

        if Module32First(h_module_snap, &mut module_entry) != 0 {
            let current_module_name = CStr::from_ptr(&module_entry.szModule as *const i8);
            if current_module_name == m_name.as_c_str() {
                CloseHandle(h_module_snap);
                return Some(module_entry.modBaseAddr as usize);
            }
        }
        while Module32Next(h_module_snap, &mut module_entry) != 0 {
            let current_module_name = CStr::from_ptr(&module_entry.szModule as *const i8);
            if current_module_name == m_name.as_c_str() {
                CloseHandle(h_module_snap);
                return Some(module_entry.modBaseAddr as usize);
            }
        }
        CloseHandle(h_module_snap);
    }
    None
}

// Returns an Option which contains a Handle to the Process if present.
pub unsafe fn open_process(desired_access: u32, process_id: u32) -> Option<HANDLE> {
    use winapi::um::processthreadsapi::OpenProcess;

    let process_handle: HANDLE = OpenProcess(desired_access, 0, process_id);

    if process_handle.is_null() {
        panic!("Invalid handle value! Last OS Error: {}", GetLastError());
    }

    Some(process_handle)
}

/// API extracted from https://docs.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle
/// This function is unsafe because closing a handle twice will trigger an exception when debugged
pub unsafe fn close_handle(handle: HANDLE) {
    CloseHandle(handle);
}

// Find patterns

pub fn find_pattern(module: &Module, pattern: &str) -> Option<usize> {
    let mut regex = pattern
        .split_whitespace()
        .map(|val| if val == "?" { ".".to_string() } else { format!("\\x{}", val) })
        .collect::<Vec<_>>()
        .join("");

    regex.insert_str(0, "(?s-u)");

    Regex::new(&regex)
        .ok()
        .and_then(|f| f.find(&module.data))
        .and_then(|f| Some(f.start()))
}

// pattern scan basically be for calculating offset of some value. It adds the offset to the pattern-matched address, dereferences, and add the `extra`.
// * `pattern` - pattern string you're looking for. format: "8D 34 85 ? ? ? ? 89 15 ? ? ? ? 8B 41 08 8B 48 04 83 F9 FF"
// * `offset` - offset of the address from pattern's base.
// * `extra` - offset of the address from dereferenced address.

pub fn pattern_scan<T>(
    process_handle: &HANDLE,
    module: Module,
    pattern: &str,
    offset: usize,
    extra: usize
)
    -> Option<T>
    where
        T: std::ops::Add<Output = T>,
        T: std::ops::Sub<Output = T>,
        T: std::convert::TryFrom<usize>,
        <T as std::convert::TryFrom<usize>>::Error: std::fmt::Debug
{
    let address = find_pattern(&module, pattern).unwrap();
    let address = address + offset;
    let mut target_buffer: T = unsafe { std::mem::zeroed::<T>() };

    read::<T>(
        &process_handle,
        module.base_address + address,
        std::mem::size_of::<T>(),
        &mut target_buffer as *mut T
    ).expect("READ FAILED IN PATTERN SCAN");

    Some(target_buffer - module.base_address.try_into().unwrap() + extra.try_into().unwrap())
}

/// read fetches the value that given address is holding.
/// * `process_handle` - handle of the process that module belongs to.
/// * `base_address` - the address that is supposed to have the value you want
/// * `buffer` - the buffer to be filled with read value. must have identical type as T.
pub fn read<T>(
    process_handle: &HANDLE,
    base_address: usize,
    size: usize,
    buffer: *mut T
) -> Result<(), std::fmt::Error> {
    unsafe {
        let mut memory_info: MEMORY_BASIC_INFORMATION =
            std::mem::zeroed::<MEMORY_BASIC_INFORMATION>();
        VirtualQueryEx(
            *process_handle,
            base_address as LPCVOID,
            &mut memory_info,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>()
        );
        let is_readable = is_page_readable(&memory_info);
        let mut old_protect = PAGE_READWRITE;
        let mut new_protect = PAGE_READWRITE;
        if !is_readable {
            VirtualProtectEx(
                *process_handle,
                base_address as LPVOID,
                std::mem::size_of::<LPVOID>(),
                new_protect,
                &mut old_protect as *mut DWORD
            );
        }

        let ok = ReadProcessMemory(
            *process_handle,
            base_address as LPCVOID,
            buffer as *mut T as LPVOID,
            size as SIZE_T,
            std::ptr::null_mut::<SIZE_T>()
        );

        if !is_readable {
            VirtualProtectEx(
                *process_handle,
                base_address as LPVOID,
                std::mem::size_of::<LPVOID>(),
                old_protect,
                &mut new_protect as *mut DWORD
            );
        }

        if ok == FALSE {
            let error_code = GetLastError();
            panic!("{}", "a");
            // return match error_code {
            // println!("{} {:?}", "Error with code", error_code);
            // };
        }
        Ok(())
    }
}

pub fn is_page_readable(memory_info: &MEMORY_BASIC_INFORMATION) -> bool {
    if
        memory_info.State != MEM_COMMIT ||
        memory_info.Protect == 0x0 ||
        memory_info.Protect == PAGE_NOACCESS
    {
        return false;
    }
    true
}