use nexus::{AddonFlags, UpdateProvider};

pub mod init;

// ======= Nexus export - only compiled when building for nexus =============
#[cfg(feature = "nexus")]
nexus::export! {
    name: "External DX11 overlay runner",
    signature: -0x7A8B9C2D,
    load: init::nexus_load,
    unload: init::nexus_unload,
    flags: AddonFlags::None,
    provider: UpdateProvider::GitHub,
    update_link: "https://github.com/SorryQuick/external-dx11-overlay",
    log_filter: "trace"
}