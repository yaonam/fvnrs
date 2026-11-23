pub mod animation_player;
pub mod assets;
mod audio_playback;
mod cache_worker;
mod constants;
mod inputs;
mod persist_worker;
pub mod render;
pub mod screens;
pub mod ui;
mod widgets;

use eframe::egui;

pub fn default_native_options(viewport: egui::ViewportBuilder) -> eframe::NativeOptions {
    let mut options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut options.wgpu_options.wgpu_setup {
        #[cfg(target_os = "windows")]
        {
            // Default to DX12 first on Windows, then Vulkan
            if std::env::var("WGPU_BACKEND").is_err() {
                unsafe {
                    std::env::set_var("WGPU_BACKEND", "dx12,vulkan");
                }
            }
            setup.instance_descriptor.backends =
                eframe::wgpu::Backends::DX12 | eframe::wgpu::Backends::VULKAN;
            // setup.power_preference = eframe::wgpu::PowerPreference::HighPerformance;
        }

        #[cfg(target_os = "macos")]
        {
            setup.instance_descriptor.backends = eframe::wgpu::Backends::METAL;
        }

        #[cfg(all(unix, not(target_os = "macos"), not(target_os = "android")))]
        {
            setup.instance_descriptor.backends =
                eframe::wgpu::Backends::VULKAN | eframe::wgpu::Backends::GL;
        }
    }
    options
}
