#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod gui;
mod settings;
mod external;

fn main() {
    gui::draw_window();
}