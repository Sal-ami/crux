pub mod adapter;
pub mod blame;
pub mod cli;
pub mod diff;
pub mod doctor;
pub mod error;
pub mod fingerprint;
pub mod git;
pub mod guardian;
pub mod init;
pub mod min;
pub mod report;
pub mod sandbox;
pub mod sig;
pub mod slice;
pub mod store;

pub(crate) fn shell_cmd() -> (&'static str, &'static [&'static str]) {
    if cfg!(windows) {
        ("cmd", &["/C"])
    } else {
        ("sh", &["-c"])
    }
}
