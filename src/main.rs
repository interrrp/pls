use std::{
    env::{Args, args},
    ffi::CString,
    fs,
    iter::Skip,
    os::unix::process::CommandExt,
    process::{Command, exit},
};

use libc::{getpwuid, getuid, setuid};
use serde::Deserialize;

const ROOT: u32 = 0;
const SUCCESS: i32 = 0;
const FAILURE: i32 = 1;
const CFG_PATH: &'static str = "pls.toml";

#[derive(Deserialize, Debug)]
struct Config {
    superusers: Vec<String>,
}

fn main() {
    let cfg = read_cfg();

    check_if_superuser(&cfg.superusers);

    let mut args = get_args();
    setuid_root();
    exec(&mut args);
}

fn check_if_superuser(superusers: &[String]) {
    let user = get_user();
    if !superusers.contains(&user) {
        eprintln!(
            "You ({}) are not in the superusers list. Contact your system administrator if this is a mistake.",
            &user
        );
        exit(FAILURE);
    }
}

fn get_user() -> String {
    unsafe {
        let passwd = *getpwuid(getuid());
        let user = CString::from_raw(passwd.pw_name).into_string().unwrap();
        user
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

fn exec(args: &mut Skip<Args>) {
    let program = args.next().unwrap();
    let program_args: Vec<String> = args.collect();

    let _ = Command::new(program).args(program_args).exec();

    // If we've reached this point, it means the command failed
    eprintln!("Command failed");
    exit(FAILURE);
}

fn setuid_root() {
    unsafe {
        if setuid(ROOT) != SUCCESS {
            eprintln!("setuid failed. You need root privileges.");
            exit(FAILURE);
        }
    }
}
