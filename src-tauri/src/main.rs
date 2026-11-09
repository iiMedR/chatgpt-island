// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if !chatgpt_island_lib::run_capture_helper_if_requested() {
        chatgpt_island_lib::run()
    }
}
