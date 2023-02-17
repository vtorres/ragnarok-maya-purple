use std::mem::{size_of, zeroed};
use std::ptr::copy_nonoverlapping;
use std::str::Utf8Error;
use winapi::shared::minwindef::{DWORD, HMODULE, MAX_PATH};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::psapi::{GetModuleBaseNameA, GetModuleInformation, MODULEINFO};
use winapi::um::winnt::{CHAR, LPSTR};

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub handle: HMODULE,
    pub size: u32,
    pub base_address: usize,
    pub data: Vec<u8>,
}

pub fn from_handle(handle: HMODULE) -> Option<Module> {
    unsafe {
        let mut module_info: MODULEINFO = zeroed::<MODULEINFO>();
        let process_handle = GetCurrentProcess();

        GetModuleInformation(
            process_handle,
            handle,
            &mut module_info,
            size_of::<MODULEINFO>() as DWORD,
        );

        let mut name_buffer: [CHAR; MAX_PATH] = [0; MAX_PATH];

        GetModuleBaseNameA(
            GetCurrentProcess(),
            handle,
            &mut name_buffer as LPSTR,
            std::mem::size_of_val(&name_buffer) as u32,
        );

        let module_name =
            read_null_terminated_string(&mut name_buffer as *mut i8 as usize).unwrap();

        let mut data: Vec<u8> = Vec::with_capacity(module_info.SizeOfImage as usize);
        let data_ptr = data.as_mut_ptr();
        data.set_len(0);

        copy_nonoverlapping(
            module_info.lpBaseOfDll as *const u8,
            data_ptr,
            module_info.SizeOfImage as usize,
        );

        data.set_len(module_info.SizeOfImage as usize);

        let module = Module {
            name: String::from(module_name),
            handle,
            base_address: module_info.lpBaseOfDll as usize,
            size: module_info.SizeOfImage,
            data,
        };

        Some(module)
    }
}

pub unsafe fn read_null_terminated_string(base_address: usize) -> Result<String, Utf8Error> {
    let len = (0..500).take_while(|&i| *(base_address as *const u8).offset(i) != 0 ).count();
    let slice = std::slice::from_raw_parts(base_address as *const u8, len);

    match String::from_utf8(slice.to_vec()) {
        Ok(val) => Ok(val),
        Err(e) => return Err(e.utf8_error())
    }
}