use age::secrecy::Secret;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

const PDPW_EXTENSION: &str = "pdpw";

slint::slint! {
    import { TextEdit, VerticalBox, Button } from "std-widgets.slint";
export component Passwords inherits Window {
        in-out property<string> the_passwords: "";
        in-out property<string> the_title: "PD Password";
        callback save_passwords();
        callback passwords-edited(string);
        title: the_title;
        VerticalBox {
            Button {
                text: "Save";
                clicked => {
                    root.save_passwords();
                }
            }
            TextEdit {
                min-width: 200px;
                min-height: 100px;
                text: the_passwords;
                edited(text) => {
                    root.passwords-edited(text);
                }
            }
        }
    }
}

fn decrypt_pdpw_file(pdpw_file: &Path, pin: &str) -> anyhow::Result<String> {
    let passwords = if pdpw_file.extension().is_some_and(|e| e == PDPW_EXTENSION) {
        if pdpw_file.exists() {
            let encrypted = std::fs::read(pdpw_file)?;
            let decryptor = match age::Decryptor::new(&encrypted[..])? {
                age::Decryptor::Passphrase(d) => d,
                _ => unreachable!(),
            };
            let mut decrypted = vec![];
            let mut reader = decryptor.decrypt(&Secret::new(pin.to_owned()), None)?;
            reader.read_to_end(&mut decrypted)?;
            String::from_utf8(decrypted)?
        } else {
            String::new()
        }
    } else {
        eprintln!("{} is not a *.{PDPW_EXTENSION} file", pdpw_file.display());
        std::process::exit(2);
    };
    Ok(passwords)
}

fn main() -> anyhow::Result<()> {
    // parse Arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 2 {
        eprintln!("Wrong number of arguments");
        let prog_name = args.first().map(|n| n.as_str()).unwrap_or("pdpw");
        println!("Synopsis:\n  {prog_name} <path-to-pdpw-file>");
        std::process::exit(1);
    }
    let pdpw_file_param = args.get(1).map(String::as_str).unwrap_or("default.pdpw");
    let pdpw_file_path = Box::new(PathBuf::from(pdpw_file_param));
    let pin = rpassword::prompt_password("Please enter the PIN for your pdpw file: ")?;
    let passwords = decrypt_pdpw_file(&pdpw_file_path, pin.as_str())?;
    let shared_pwds = slint::SharedString::from(&passwords);
    let main_window = Passwords::new()?;
    let ui_handle = main_window.as_weak();
    main_window.on_passwords_edited(move |pdpw_decrypted| {
        if let Some(ui) = ui_handle.upgrade() {
            ui.set_the_passwords(pdpw_decrypted);
        }
    });
    let ui_handle = main_window.as_weak();
    main_window.on_save_passwords(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let passwords = ui.get_the_passwords();
            let encrypted = {
                let encryptor = age::Encryptor::with_user_passphrase(Secret::new(pin.clone()));
                let mut encrypted = vec![];
                let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
                writer.write_all(passwords.as_bytes()).unwrap();
                writer.finish().unwrap();
                encrypted
            };
            std::fs::write(pdpw_file_path.as_path(), encrypted).unwrap();
        }
    });
    main_window.set_the_passwords(shared_pwds);
    let title = slint::SharedString::from(format!("PD Password: {pdpw_file_param}"));
    main_window.set_the_title(title);
    main_window.window().set_maximized(true);
    main_window.run()?;
    Ok(())
}
