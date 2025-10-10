pub mod account;
pub mod directory;
pub mod switch;
pub mod webdav;
pub mod logs;

use comfy_table::{Table, presets::UTF8_FULL};

pub fn create_table() -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table
}
