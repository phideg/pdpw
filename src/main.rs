mod editor;
mod modal;
mod store;

use editor::Editor;

fn main() -> anyhow::Result<()> {
    // parse Arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        eprintln!("Wrong number of arguments");
        let prog_name = args.first().map(|n| n.as_str()).unwrap_or("pdpw");
        println!("Synopsis:\n  {prog_name} <path-to-pdpw-file>");
        std::process::exit(1);
    }

    let pdpw_file_param = args
        .get(1)
        .map(String::clone)
        .unwrap_or("default.pdpw".to_string());

    iced::program("pdpw - password store", Editor::update, Editor::view)
        .load(move || Editor::load(&pdpw_file_param))
        .subscription(Editor::subscription)
        .default_font(iced::Font::MONOSPACE)
        .run()?;
    Ok(())
}
