use ash::vk;

#[cfg(windows)]
use ash::extensions::khr::Win32Surface;

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;

#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;

#[cfg(windows)]
pub fn required_extension_names() -> Vec<*const std::ffi::c_char>
{
    vec!
    [
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn required_extension_names() -> Vec<*const i8>
{
    vec!
    [
        Surface::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

#[cfg(target_os = "macos")]
pub fn required_extension_names() -> Vec<*const i8>
{
    vec!
    [
        Surface::name().as_ptr(),
        MacOSSurface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

#[cfg(windows)]
pub unsafe fn create_surface
(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window
) -> Result<vk::SurfaceKHR, vk::Result>
{
    use std::os::raw::c_void;
    use std::ptr;
    use winapi::shared::windef::HWND;
    use winapi::um::libloaderapi::GetModuleHandleW;
    use winit::platform::windows::WindowExtWindows;

    let hwnd = window.hwnd() as HWND;
    let hinstance = GetModuleHandleW(ptr::null()) as *const c_void;
    let win32_create_info = vk::Win32SurfaceCreateInfoKHR::builder()
        .flags(Default::default())
        .hinstance(hinstance)
        .hwnd(hwnd as *const c_void);
    
    let win32_surface_loader = Win32Surface::new(entry, instance);
    win32_surface_loader.create_win32_surface(&win32_create_info, None)
}