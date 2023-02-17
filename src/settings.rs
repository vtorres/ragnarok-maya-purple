use winapi::um::winnt::{ PROCESS_ALL_ACCESS, HANDLE };

use crate::external;

pub struct Settings {
    pub maya_purple_hack: bool,
}

impl Settings {
    pub fn new(maya_purple_hack: bool) -> Settings {
        Settings {
            maya_purple_hack: maya_purple_hack,
        }
    }
}

pub trait Toggleable {
    fn toggle_maya_purple_hack(&mut self, pid: u32);
}

impl Toggleable for Settings {
    fn toggle_maya_purple_hack(&mut self, pid: u32) {
        unsafe {
            // Toggle the maya_purple_hack state
            self.maya_purple_hack = !self.maya_purple_hack;

            // Open the process HANDLE
            let handle: HANDLE = external::process::open_process(PROCESS_ALL_ACCESS, pid).unwrap();

            // Main address of the effect - valid for Ragnarok hexed 2018 (most of the servers nowadays)
            const MAYA_PURPLE_OFFSET: usize = 0x00a9c3a7;

            // Original bytes
            let MAYA_PURPLE_ORIGINAL_BYTES: Vec<u8> = vec![0x84];

            // Modified bytes
            let MAYA_PURPLE_MODIFIED_BYTES: Vec<u8> = vec![0x85];

            // Patch it
            if self.maya_purple_hack {
                external::writemem::patch_ex(
                    handle,
                    MAYA_PURPLE_OFFSET,
                    MAYA_PURPLE_MODIFIED_BYTES
                );
            } else {
                external::writemem::patch_ex(
                    handle,
                    MAYA_PURPLE_OFFSET,
                    MAYA_PURPLE_ORIGINAL_BYTES
                );
            }

            // Close the Handle
            external::process::close_handle(handle);
        }
    }
}