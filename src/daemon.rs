//! The daemon that just keeps playing the sound.

use anyhow::Context;
use fd_lock::RwLock;
use nix::errno::Errno;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Write};

const PID_FILE: &str = "/tmp/noiseplayer.pid";

pub fn spawn() -> anyhow::Result<()> {
    let config = match crate::config::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load config: {}\nUsing default config.", e);
            crate::config::Config::default()
        }
    };
    let pidfile = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(PID_FILE)
        .context("can't open pidfile")?;
    let mut pidfile = RwLock::new(pidfile);
    let pidfile = pidfile.try_write();
    match pidfile {
        Ok(mut pidfile) => {
            // Now fork.
            match unsafe { nix::unistd::fork()? } {
                nix::unistd::ForkResult::Parent { child } => {
                    // This is the parent process.
                    //
                    // The fd lock is inherited by the child process, however, we still own it here.
                    // If we drop `pidfile` then the lock will be released. We don't want that to
                    // happen, so we forget the lock.
                    std::mem::forget(pidfile);
                    println!("Spawned daemon with pid {}", child);
                    Ok(())
                }
                nix::unistd::ForkResult::Child => {
                    // We are the child.
                    pidfile.set_len(0)?;
                    write!(pidfile, "{}", nix::unistd::getpid())?;
                    crate::player::loop_forever(config.volume)?;
                    Ok(())
                }
            }
        }
        Err(_) => {
            println!("Daemon already running.");
            std::process::exit(1);
        }
    }
}

/// Reads the pidfile and returns the pid if it exists.
///
/// This guarantees that the PID read is safe-ish. The returned PID will be greater than 0 and not
/// equal to the PID of the current process.
fn read_pid(mut file: &std::fs::File) -> anyhow::Result<nix::unistd::Pid> {
    let mut pid = String::new();
    file.read_to_string(&mut pid)?;
    let pid = pid.trim().parse::<i32>()?;
    if pid > 0 && pid != nix::unistd::getpid().as_raw() {
        Ok(nix::unistd::Pid::from_raw(pid))
    } else {
        Err(anyhow::anyhow!("Invalid PID"))
    }
}

pub fn kill() -> anyhow::Result<()> {
    let pidfile = match File::open(PID_FILE) {
        Ok(pidfile) => pidfile,
        Err(_) => {
            println!("Daemon not running.");
            std::process::exit(1);
        }
    };
    let pid = read_pid(&pidfile)?;
    match nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGTERM) {
        // ESRCH means the process does not exist, so already killed.
        Ok(()) | Err(Errno::ESRCH)=> (),
        Err(_) => {
            println!("Failed to kill daemon with pid {}", pid);
        }
    }
    Ok(())
}
