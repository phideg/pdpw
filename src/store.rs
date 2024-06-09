use age::secrecy::Secret;
use std::{
    io::{Read, Write},
    path::Path,
};

const PDPW_EXTENSION: &str = "pdpw";

pub(crate) async fn load_pdpw_file(pdpw_file: &Path, pin: &str) -> anyhow::Result<String> {
    let passwords = if pdpw_file.extension().is_some_and(|e| e == PDPW_EXTENSION) {
        if pdpw_file.exists() {
            let encrypted = tokio::fs::read(pdpw_file).await?;
            let decryptor = match age::Decryptor::new_async_buffered(encrypted.as_slice()).await? {
                age::Decryptor::Passphrase(d) => d,
                _ => unreachable!(),
            };
            let mut decrypted = vec![];
            let mut reader = decryptor.decrypt_async(&Secret::new(pin.to_owned()), None)?;
            reader.read_to_end(&mut decrypted)?;
            String::from_utf8(decrypted)?
        } else {
            String::new()
        }
    } else {
        eprintln!("{pdpw_file:?} is not a *.{PDPW_EXTENSION} file");
        std::process::exit(2);
    };
    Ok(passwords)
}

pub(crate) async fn store_pdpw_file(
    pdpw_file: &Path,
    pin: &str,
    passwords: &str,
) -> anyhow::Result<()> {
    let encrypted = {
        let encryptor = age::Encryptor::with_user_passphrase(Secret::new(pin.to_string()));
        let mut encrypted = vec![];
        let mut writer = encryptor.wrap_async_output(&mut encrypted).await?;
        writer.write_all(passwords.as_bytes())?;
        writer.finish()?;
        encrypted
    };
    tokio::fs::write(pdpw_file, encrypted).await?;
    Ok(())
}
