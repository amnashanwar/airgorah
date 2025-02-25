use super::*;
use crate::error::Error;
use crate::globals::*;

/// Check if the user is root, and if all the dependencies are installed
pub fn app_setup() -> Result<(), Error> {
    app_cleanup();

    ctrlc::set_handler(move || {
        app_cleanup();
        std::process::exit(1);
    })
    .expect("Error setting Ctrl-C handler");

    if sudo::check() != sudo::RunningAs::Root {
        return Err(Error::new("Airgorah need root privilege to run"));
    }

    load_settings();

    check_required_dependencies(&[
        "sh",
        "systemctl",
        "ip",
        "iw",
        "awk",
        "airmon-ng",
        "airodump-ng",
        "aireplay-ng",
        "aircrack-ng",
        "mergecap",
        "macchanger",
    ])
}

/// Stop the scan process, kill all the attack process, and remove all the files created by the app
pub fn app_cleanup() {
    stop_scan_process().ok();
    stop_all_deauth_attacks();

    if let Some(ref iface) = get_iface() {
        disable_monitor_mode(iface).ok();
        restore_network_manager().ok();
    }

    std::fs::remove_file(LIVE_SCAN_PATH.to_string() + "-01.csv").ok();
    std::fs::remove_file(LIVE_SCAN_PATH.to_string() + "-01.cap").ok();
    std::fs::remove_file(OLD_SCAN_PATH.to_string() + "-01.cap").ok();
}

/// Check if a dependency is installed
pub fn has_dependency(dep: &str) -> bool {
    which::which(dep).is_ok()
}

/// Check if all the required dependencies are installed
pub fn check_required_dependencies(deps: &[&str]) -> Result<(), Error> {
    for dep in deps {
        if !has_dependency(dep) {
            return Err(Error::new(&format!(
                "Missing required dependency: \"{}\"",
                dep,
            )));
        }
    }
    Ok(())
}

/// Check if a new version is available
pub fn check_update(current_version: &str) -> Option<String> {
    let url = "https://api.github.com/repos/martin-olivier/airgorah/releases/latest";

    if let Ok(response) = ureq::get(url).call() {
        if let Ok(json) = response.into_json::<serde_json::Value>() {
            if json["tag_name"] != current_version {
                let new_version = json["tag_name"].as_str().unwrap_or("unknown").to_owned();

                log::info!("a new version is available: \"{}\"", new_version);

                return Some(new_version);
            }
        }
    }

    log::info!("airgorah is up to date");

    None
}
