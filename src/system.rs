/// System specific workarounds for various issues.
pub fn initialize_workarounds() {
    #[cfg(target_os = "windows")]
    {
        unsafe { std::env::set_var("DISABLE_VULKAN_OBS_CAPTURE", "1") };
    }
}
