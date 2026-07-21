use std::collections::HashMap;

use crate::{error::HasExpectedErrors, planner::{BuiltinPlanner, Planner, PlannerError}, settings::CommonSettings};


pub struct Windows {
    settings: CommonSettings,
};


impl Planner for Windows  {
    fn plan(&self) -> Result<Vec<crate::action::StatefulAction<Box<dyn crate::action::Action> > > ,super::PlannerError> {

    }

    fn settings(&self) -> Result<HashMap<String, serde_json::Value>, InstallSettingsError> {
        let Self {
            settings,
            // encrypt,
            // volume_label,
            // case_sensitive,
            // root_disk,
            // use_ec2_instance_store,
        } = self;
        let mut map = HashMap::default();

        map.extend(settings.settings()?);
        // map.insert("volume_encrypt".into(), serde_json::to_value(encrypt)?);
        // map.insert("volume_label".into(), serde_json::to_value(volume_label)?);
        // map.insert("root_disk".into(), serde_json::to_value(root_disk)?);
        // map.insert(
        //     "use_ec2_instance_store".into(),
        //     serde_json::to_value(use_ec2_instance_store)?,
        // );
        // map.insert(
        //     "case_sensitive".into(),
        //     serde_json::to_value(case_sensitive)?,
        // );

        Ok(map)
    }
   fn configured_settings(&self) -> Result<HashMap<String, serde_json::Value>, PlannerError> {
        let default = Self::try_default()?.settings()?;
        let configured = self.settings()?;

        let mut settings: HashMap<String, serde_json::Value> = HashMap::new();
        for (key, value) in configured.iter() {
            if default.get(key) != Some(value) {
                settings.insert(key.clone(), value.clone());
            }
        }

        Ok(settings)
    }

    fn platform_check(&self) -> Result<(),PlannerError> {
        use target_lexicon::OperatingSystem;
        match target_lexicon::OperatingSystem::host() {
            OperatingSystem::Windows => Ok(()),
            host_os => Err(PlannerError::IncompatibleOperatingSystem {
                planner: self.typetag_name(),
                host_os,
            }),
        }
    }

    fn pre_install_check(&self) -> Result<(),PlannerError> {
      check_windows_version()?;
      Ok(())
    }
}

impl From<Windows> for BuiltinPlanner {
    fn from(val: Windows) -> Self {
        BuiltinPlanner::Windows(val)
    }
}

//TODO(eveeifyeve): explain why and use https://en.wikipedia.org/wiki/Microsoft_Windows#Timeline_of_releases as a resource.
/// The minimum Windows version required to run the bundled Nix.
///
/// TODO.
const MIN_WINDOWS_NT_VERSION: (u16, u16, u16) = (10, 0, 14393);

fn check_windows_version() -> Result<(), WindowsError> {
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let nt_key = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")?;
    let version_string: String = nt_key.get_value("CurrentVersion")?;

    let parts: Vec<u64> = version_string
        .trim()
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    let (major, minor, patch) = (
        parts.first().copied().unwrap_or(0),
        parts.get(1).copied().unwrap_or(0),
        parts.get(2).copied().unwrap_or(0),
    );

    if (major, minor, patch) < MIN_WINDOWS_NT_VERSION {
        return Err(WindowsError::UnsupportedWindowsVersion {
            got: version_string.trim().to_owned(),
            min: format!("{}.{}.{}", MIN_WINDOWS_NT_VERSION.0, MIN_WINDOWS_NT_VERSION.1, MIN_WINDOWS_NT_VERSION.2),
        })
        .map_err(|e| PlannerError::Custom(Box::new(e)));
    }
    Ok(())
}


#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum WindowsError {
    // TODO(eveeifyeve): Add this back when nix-windows becomes a realiliy.
    // #[error(
    //     "`nix-windows` installation detected, it must be removed before uninstalling Nix. Please refer to ... for instructions how to uninstall `nix-windows`."
    // )]
    // UninstallNixWindows,
    #[error(
        "Windows NT Version {got} is too old — Nix requires Windows NT Version {min} or later."
    )]
    UnsupportedWindowsVersion { got: String, min: String },
}


impl HasExpectedErrors for WindowsError {
    fn expected<'a>(&'a self) -> Option<Box<dyn std::error::Error + 'a>> {
        match self {
            // this @ WindowsError::UninstallNixWindows => Some(Box::new(this)),
            this @ WindowsError::UnsupportedWindowsVersion { .. } => Some(Box::new(this)),
        }
    }
}
