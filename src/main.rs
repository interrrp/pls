#![deny(clippy::all, clippy::pedantic)]

use std::{
    env::{Args, args},
    ffi::CString,
    fs::{self, OpenOptions},
    io::Write,
    iter::Skip,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, exit},
};

use libc::{getpwuid, getuid, setuid};
use serde::Deserialize;

const ROOT: u32 = 0;
const SUCCESS: i32 = 0;
const FAILURE: i32 = 1;
const CFG_PATH: &str = "pls.toml";

#[derive(Deserialize, Debug)]
struct Config {
    superusers: Vec<String>,
    log_file: PathBuf,
}

fn main() {
    let cfg = read_cfg();

    let user = get_user();
    if !cfg.superusers.contains(&user) {
        print_unprivileged_notice(&user);
    }

    let mut args = get_args();
    let (program, program_args) = get_program_parts(&mut args);

    setuid_root();
    write_log(&user, &cfg.log_file, &program, &program_args);
    exec(&program, &program_args);
}

fn print_unprivileged_notice(user: &str) {
    eprintln!(
        "You ({user}) are not in the superusers list. Contact your system administrator if this is a mistake."
    );
    exit(FAILURE);
}

fn get_user() -> String {
    unsafe {
        let passwd = *getpwuid(getuid());
        CString::from_raw(passwd.pw_name).into_string().unwrap()
    }
}

fn read_cfg() -> Config {
    let cfg = fs::read_to_string(CFG_PATH).unwrap();
    let cfg: Config = toml::from_str(&cfg).unwrap();
    cfg
}

fn get_args() -> Skip<Args> {
    let args = args().skip(1);
    if args.len() == 0 {
        print_usage();
    }
    args
}

fn print_usage() {
    eprintln!("Usage: pls <command>");
    exit(FAILURE);
}

fn get_program_parts(args: &mut Skip<Args>) -> (String, Vec<String>) {
    (args.next().unwrap(), args.collect())
}

fn exec(program: &str, program_args: &[String]) {
    let _ = Command::new(program).args(program_args).exec();

    // If we've reached this point, it means the command failed
    eprintln!("Command failed");
    exit(FAILURE);
}

fn write_log(user: &str, log_file: &Path, program: &str, program_args: &[String]) {
    let log_line = format!("{user}: {program} {}", program_args.join(" "));

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .unwrap();

    writeln!(file, "{log_line}").unwrap();
}

fn setuid_root() {
    unsafe {
        if setuid(ROOT) != SUCCESS {
            eprintln!("setuid failed. You need root privileges.");
            exit(FAILURE);
        }
    }
}
