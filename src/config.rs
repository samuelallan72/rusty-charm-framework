use serde_json;
use std::process::Command;

// TODO: drop the unwraps, figure out error handling/propogating
pub(crate) fn config<C>() -> C
where
    C: serde::de::DeserializeOwned,
{
    let output = Command::new("config-get")
        .args(["--format", "json", "--all"])
        .output()
        .expect("failed to execute config-get");
    serde_json::from_slice::<C>(&output.stdout).unwrap()
}
