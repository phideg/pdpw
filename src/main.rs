#![windows_subsystem = "windows"]
mod editor;
mod galloc;
mod modal;
mod store;

use editor::Editor;
use galloc::SecureGlobalAlloc;

#[global_allocator]
static GA: galloc::SecureGlobalAlloc = SecureGlobalAlloc;

const DEFAULT_FILE_NAME: &str = "default.pdpw";

fn main() -> anyhow::Result<()> {
    // parse Arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        eprintln!("Wrong number of arguments");
        let prog_name = args.first().map(|n| n.as_str()).unwrap_or("pdpw");
        println!("Synopsis:\n  {prog_name} <path-to-pdpw-file>");
        std::process::exit(1);
    }

    let pdpw_file_param = args.get(1).cloned().unwrap_or_else(|| {
        dirs::home_dir()
            .and_then(|p| {
                let p = p.join(DEFAULT_FILE_NAME);
                p.to_str().map(|p| p.to_string())
            })
            .unwrap_or_else(|| "{DEFAULT_FILE_NAME}".to_string())
    });

    iced::application("pdpw - password store", Editor::update, Editor::view)
        .subscription(Editor::subscription)
        .default_font(iced::Font::MONOSPACE)
        .run_with(move || Editor::new(&pdpw_file_param))?;
    Ok(())
}
