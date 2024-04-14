use age::secrecy::Secret;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

slint::slint! {
    import { TextEdit, VerticalBox, Button } from "std-widgets.slint";
export component Passwords inherits Window {
        in-out property<string> the_passwords: "foo";
        callback save_passwords();
        callback passwords-edited(string);
        title: "PD Password";
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

fn decrypt_pdwd_file(pdwd_file: &Path) -> anyhow::Result<String> {
    let passwords = if pdwd_file.extension().is_some_and(|e| e == "pdwd") {
        if pdwd_file.exists() {
            let encrypted = std::fs::read(pdwd_file)?;
            let decryptor = match age::Decryptor::new(&encrypted[..])? {
                age::Decryptor::Passphrase(d) => d,
                _ => unreachable!(),
            };
            let mut decrypted = vec![];
            let mut reader = decryptor.decrypt(&Secret::new("foo".to_owned()), None)?;
            reader.read_to_end(&mut decrypted)?;
            String::from_utf8(decrypted)?
        } else {
            String::new()
        }
    } else {
        eprintln!("{} is not a *.pdwd file", pdwd_file.display());
        std::process::exit(2);
    };
    Ok(passwords)
}

fn main() -> anyhow::Result<()> {
    // parse Arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Wrong number of arguments");
        let prog_name = args.first().map(|n| n.as_str()).unwrap_or("pdpw");
        println!("Synopsis:\n  {prog_name} <path-to-pdpw-file>");
        std::process::exit(1);
    }
    let pdwd_file = Box::new(PathBuf::from(&args[1]));
    let passwords = decrypt_pdwd_file(&pdwd_file)?;
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
                let encryptor = age::Encryptor::with_user_passphrase(Secret::new("foo".to_owned()));
                let mut encrypted = vec![];
                let mut writer = encryptor.wrap_output(&mut encrypted).unwrap();
                writer.write_all(passwords.as_bytes()).unwrap();
                writer.finish().unwrap();
                encrypted
            };
            std::fs::write(pdwd_file.as_path(), encrypted).unwrap();
        }
    });
    main_window.set_the_passwords(shared_pwds);
    main_window.window().set_maximized(true);
    main_window.run()?;
    Ok(())
}
