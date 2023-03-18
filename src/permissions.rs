use nix::unistd::{Gid, Uid, User};
use std::env;

// Inspired by https://github.com/waycrate/swhkd/blob/main/swhkd/src/perms.rs

#[link(name = "c")]
extern "C" {
    fn geteuid() -> u32;
}

/// Get the UID of the caller.
pub fn get_caller_uid() -> Result<u32, Box<dyn std::error::Error>> {
    unsafe {
        let mut uid = geteuid();
        if uid == 0 {
            if let Ok(sudo_uid) = env::var("SUDO_UID") {
                uid = sudo_uid.parse::<u32>()?;
            } else if let Ok(pkexec_uid) = env::var("PKEXEC_UID") {
                uid = pkexec_uid.parse::<u32>()?;
            } else {
                return Err("UID is 0 but SUDO_UID and PKEXEC_UID are not set".into());
            }
        }
        Ok(uid)
    }
}

/// Drop privileges to the given user.
///
/// # Arguments
///
/// * `user_uid` - The user to drop privileges to.
pub fn drop_privileges(user_uid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let user_uid = Uid::from_raw(user_uid);
    if let Some(user) = User::from_uid(user_uid)? {
        set_initgroups(&user, user_uid.as_raw())?;
        set_egid(user_uid.as_raw())?;
        // set_euid(user_uid.as_raw())?;
        Ok(())
    } else {
        Err("Failed to get user".into())
    }
}

fn set_initgroups(user: &nix::unistd::User, gid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let gid = Gid::from_raw(gid);
    nix::unistd::initgroups(&user.gecos, gid)?;
    Ok(())
}

fn set_egid(gid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let gid = Gid::from_raw(gid);
    nix::unistd::setegid(gid)?;
    Ok(())
}

fn set_euid(uid: u32) -> Result<(), Box<dyn std::error::Error>> {
    let uid = Uid::from_raw(uid);
    nix::unistd::seteuid(uid)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_caller_uid() {
        assert!(get_caller_uid().is_ok());
    }
}
