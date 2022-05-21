use std::{path::PathBuf, env};

pub fn exe_folder() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|path| {
            path.parent()
                .map(|parent| parent.to_owned())
        })
        .unwrap()
}

pub fn vault_folder() -> PathBuf {
    let mut container = exe_folder();
    container.push("vaults");
    container
}
