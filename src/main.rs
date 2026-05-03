#![windows_subsystem = "windows"]
mod about;
mod editor;
mod galloc;
mod modal;
mod store;

use std::io::IsTerminal;

use about::MsgPopup;
use anyhow::{Context, anyhow};
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
            r"
Simple passvault application [v{VERSION}]

Synopsis:
    {prog_name} [OPTIONS] <path-to-pdpw-file>

Options:
    --skip-clipboard-cleanup      Do not cleanup OS clipboard on program exit
    --help                        Print this message

"
        );
        if let Some(err_msg) = err.as_ref() {
            help_message = format!("{err_msg}\n\n{help_message}");
        }
        if std::io::stdin().is_terminal() {
            println!("{help_message}");
        } else {
            let help = std::sync::Arc::new(help_message);
            iced::application(
                move || MsgPopup::new((*help).clone()),
                MsgPopup::update,
                MsgPopup::view,
            )
            .run()
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
        let prog_name = args.first().map_or("pdpw", std::string::String::as_str);
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
                    p.to_str().map(std::string::ToString::to_string)
                })
                .unwrap_or_else(|| "{DEFAULT_FILE_NAME}".to_string())
        } else {
            args.last().cloned()
                .ok_or(anyhow!("Invalid pdpw file"))?
        };
        if !pdpw_file.ends_with(".pdpw") {
            Cli::print_help(prog_name, Some("Error: Expected *.pdpw file!"));
        }
        Ok(Self {
            pdpw_file,
            skip_cleanup,
        })
    }
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse_arguments()?;

    let pdpw_file = std::sync::Arc::new(args.pdpw_file);
    iced::application(
        move || Editor::new(pdpw_file.clone()),
        Editor::update,
        Editor::view,
    )
    .title("PdPw - Your Personal Passvault")
    .subscription(Editor::subscription)
    .default_font(iced::Font::MONOSPACE)
    .run()?;

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
