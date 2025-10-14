pub mod account;
pub mod base_url;
pub mod directory;
pub mod logs;
pub mod settings;
pub mod switch;
pub mod webdav;

use comfy_table::{presets::UTF8_FULL, Table};

pub fn create_table() -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table
}
