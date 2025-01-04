#![windows_subsystem = "windows"]
mod about;
mod editor;
mod galloc;
mod modal;
mod store;

use std::io::IsTerminal;

use about::MsgPopup;
use anyhow::{anyhow, Context};
use editor::Editor;
use galloc::SecureGlobalAlloc;

#[global_allocator]
static GA: galloc::SecureGlobalAlloc = SecureGlobalAlloc;

const DEFAULT_FILE_NAME: &str = "default.pdpw";
const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Cli {
    pdpw_file: String,
    skip_cleanup: bool,
}

impl Cli {
    fn print_help(prog_name: &str, err: Option<&str>) {
        let mut help_message = format!(
            r#"
Simple passvault application [v{}]

Synopsis:
    {} [OPTIONS] <path-to-pdpw-file>

Options:
    --skip-clipboard-cleanup      Do not cleanup OS clipboard on program exit
    --help                        Print this message

"#,
            VERSION, prog_name
        );
        if let Some(err_msg) = err.as_ref() {
            help_message = format!("{err_msg}\n\n{help_message}");
        }
        if std::io::stdin().is_terminal() {
            println!("{help_message}");
        } else {
            iced::application("pdpw - About", MsgPopup::update, MsgPopup::view)
                .run_with(move || MsgPopup::new(help_message))
                .unwrap();
        }
        if err.is_some() {
            std::process::exit(1);
        } else {
            std::process::exit(0);
        }
    }

    fn parse_arguments() -> anyhow::Result<Self> {
        let args: Vec<String> = std::env::args().collect();
        let prog_name = args.first().map(|n| n.as_str()).unwrap_or("pdpw");
        if args.len() > 3 {
            Cli::print_help(prog_name, Some("Error: Wrong number of arguments!"));
        }
        if args.iter().any(|v| v.contains("--help")) {
            Cli::print_help(prog_name, None);
        }
        let skip_cleanup = args.iter().any(|v| v.contains("--skip-clipboard-cleanup"));
        if !skip_cleanup && args.len() > 2 {
            Cli::print_help(
                prog_name,
                Some(&format!("Error: Unexpected option {}", args[1])),
            );
        }
        let pdpw_file = if args.len() == 1 || skip_cleanup && args.len() == 2 {
            // use default pdpw file path
            dirs::home_dir()
                .and_then(|p| {
                    let p = p.join(DEFAULT_FILE_NAME);
                    p.to_str().map(|p| p.to_string())
                })
                .unwrap_or_else(|| "{DEFAULT_FILE_NAME}".to_string())
        } else {
            args.last()
                .map(|path| path.to_string())
                .ok_or(anyhow!("Invalid pdpw file"))?
        };
        if !pdpw_file.ends_with(".pdpw") {
            Cli::print_help(prog_name, Some("Error: Expected *.pdpw file!"))
        }
        Ok(Self {
            pdpw_file,
            skip_cleanup,
        })
    }
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse_arguments()?;

    iced::application("pdpw - password store", Editor::update, Editor::view)
        .subscription(Editor::subscription)
        .default_font(iced::Font::MONOSPACE)
        .run_with(move || Editor::new(&args.pdpw_file))?;

    if !args.skip_cleanup {
        // clear clipboard at the end
        let mut clipboard =
            arboard::Clipboard::new().with_context(|| "Couldn't access the clipboard!")?;
        clipboard
            .clear()
            .with_context(|| "Something went wrong while trying to clean the clipboard")?;
    }

    Ok(())
}
