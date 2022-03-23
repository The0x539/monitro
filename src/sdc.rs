use windows::Win32::{Devices::Display::*, Graphics::Gdi::*};

use crate::{win_result, WinResult};

#[derive(Copy, Clone)]
pub enum SdcAction {
    Apply { no_optimization: bool },
    Validate,
}

impl Default for SdcAction {
    fn default() -> Self {
        Self::Apply {
            no_optimization: false,
        }
    }
}

#[derive(Copy, Clone)]
pub struct SdcDatabaseTopologies {
    pub internal: bool,
    pub clone: bool,
    pub extend: bool,
    pub external: bool,
}

impl SdcDatabaseTopologies {
    pub const CURRENT: Self = Self {
        internal: true,
        clone: true,
        extend: true,
        external: true,
    };
}

impl Default for SdcDatabaseTopologies {
    fn default() -> Self {
        Self::CURRENT
    }
}

#[derive(Copy, Clone)]
pub enum SdcConfig<'a> {
    SuppliedConfig {
        force_mode_enumeration: bool,
        save_to_database: bool,
        paths: &'a [DISPLAYCONFIG_PATH_INFO],
        modes: &'a [DISPLAYCONFIG_MODE_INFO],
    },
    SuppliedTopology {
        allow_path_order_changes: bool,
        paths: &'a [DISPLAYCONFIG_PATH_INFO],
    },
    Database {
        topologies: SdcDatabaseTopologies,
        path_persist_if_required: bool,
    },
}

impl Default for SdcConfig<'_> {
    fn default() -> Self {
        Self::Database {
            topologies: Default::default(),
            path_persist_if_required: false,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Sdc<'a> {
    pub action: SdcAction,
    pub config: SdcConfig<'a>,
    pub allow_changes: bool,
}

impl Sdc<'_> {
    fn flags(&self) -> u32 {
        let mut flags = 0;

        macro_rules! flag {
            ($f:expr) => {
                flags |= $f;
            };
            ($b:expr, $f:expr) => {
                if $b {
                    flags |= $f;
                }
            };
        }

        match self.action {
            SdcAction::Apply { no_optimization } => {
                flag!(SDC_APPLY);
                flag!(no_optimization, SDC_NO_OPTIMIZATION);
            }
            SdcAction::Validate => flags |= SDC_VALIDATE,
        }
        match self.config {
            SdcConfig::SuppliedConfig {
                force_mode_enumeration,
                save_to_database,
                ..
            } => {
                // TODO: this is a hack
                if force_mode_enumeration {
                    assert!(flags & SDC_APPLY != 0);
                }

                flag!(SDC_USE_SUPPLIED_DISPLAY_CONFIG);
                flag!(force_mode_enumeration, SDC_FORCE_MODE_ENUMERATION);
                flag!(save_to_database, SDC_SAVE_TO_DATABASE);
            }
            SdcConfig::SuppliedTopology {
                allow_path_order_changes,
                ..
            } => {
                flag!(SDC_TOPOLOGY_SUPPLIED);
                flag!(allow_path_order_changes, SDC_ALLOW_PATH_ORDER_CHANGES);
            }
            SdcConfig::Database {
                topologies,
                path_persist_if_required,
            } => {
                flag!(topologies.internal, SDC_TOPOLOGY_INTERNAL);
                flag!(topologies.clone, SDC_TOPOLOGY_CLONE);
                flag!(topologies.extend, SDC_TOPOLOGY_EXTEND);
                flag!(topologies.external, SDC_TOPOLOGY_EXTERNAL);
                flag!(path_persist_if_required, SDC_PATH_PERSIST_IF_REQUIRED);
            }
        }
        flag!(self.allow_changes, SDC_ALLOW_CHANGES);
        flags
    }

    fn paths(&self) -> &[DISPLAYCONFIG_PATH_INFO] {
        if let SdcConfig::SuppliedConfig { paths, .. } | SdcConfig::SuppliedTopology { paths, .. } =
            self.config
        {
            paths
        } else {
            &[]
        }
    }

    fn modes(&self) -> &[DISPLAYCONFIG_MODE_INFO] {
        if let SdcConfig::SuppliedConfig { modes, .. } = self.config {
            modes
        } else {
            &[]
        }
    }
}

pub fn set_display_config(sdc: Sdc) -> WinResult<()> {
    unsafe {
        let ret = SetDisplayConfig(sdc.paths(), sdc.modes(), sdc.flags());
        win_result(ret)
    }
}
