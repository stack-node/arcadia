//! Resolve another process's current working directory (for shell header preview).

#[cfg(all(unix, target_os = "linux"))]
pub fn cwd_for_pid(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    let path = format!("/proc/{pid}/cwd");
    std::fs::read_link(path)
        .ok()?
        .into_os_string()
        .into_string()
        .ok()
}

#[cfg(all(unix, target_os = "macos"))]
pub fn cwd_for_pid(pid: u32) -> Option<String> {
    use libc::{c_void, proc_pidinfo, proc_vnodepathinfo};

    if pid == 0 {
        return None;
    }
    let mut vnodepathinfo = std::mem::MaybeUninit::<proc_vnodepathinfo>::uninit();
    let rc = unsafe {
        proc_pidinfo(
            pid as libc::c_int,
            libc::PROC_PIDVNODEPATHINFO,
            0,
            vnodepathinfo.as_mut_ptr() as *mut c_void,
            std::mem::size_of::<proc_vnodepathinfo>() as libc::c_int,
        )
    };
    if rc <= 0 {
        return None;
    }
    let vnodepathinfo = unsafe { vnodepathinfo.assume_init() };
    let stat = vnodepathinfo.pvi_cdir.vip_vi.vi_stat;
    if stat.vst_dev == 0 {
        return None;
    }
    let path = &vnodepathinfo.pvi_cdir.vip_path;
    let flat = unsafe {
        std::slice::from_raw_parts(path.as_ptr() as *const u8, std::mem::size_of_val(path))
    };
    let end = flat.iter().position(|&b| b == 0).unwrap_or(flat.len());
    std::str::from_utf8(&flat[..end])
        .ok()
        .map(|s| s.to_string())
}

#[cfg(not(any(all(unix, target_os = "linux"), all(unix, target_os = "macos"))))]
pub fn cwd_for_pid(_pid: u32) -> Option<String> {
    None
}
