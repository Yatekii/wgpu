use crate::{
    binding_model::MAX_BIND_GROUPS,
    hub::HUB,
    AdapterHandle,
    AdapterId,
    DeviceHandle,
    InstanceId,
    SurfaceHandle,
};
#[cfg(feature = "local")]
use crate::{device::BIND_BUFFER_ALIGNMENT, DeviceId, SurfaceId};

#[cfg(feature = "local")]
use log::info;
#[cfg(feature = "remote")]
use serde::{Deserialize, Serialize};

use hal::{self, Instance as _, PhysicalDevice as _};

#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[cfg_attr(feature = "remote", derive(Serialize, Deserialize))]
pub enum PowerPreference {
    Default = 0,
    LowPower = 1,
    HighPerformance = 2,
}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "remote", derive(Clone, Serialize, Deserialize))]
pub struct AdapterDescriptor {
    pub power_preference: PowerPreference,
}

#[repr(C)]
#[derive(Debug, Default)]
#[cfg_attr(feature = "remote", derive(Clone, Serialize, Deserialize))]
pub struct Extensions {
    pub anisotropic_filtering: bool,
}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "remote", derive(Clone, Serialize, Deserialize))]
pub struct Limits {
    pub max_bind_groups: u32,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            max_bind_groups: MAX_BIND_GROUPS as u32,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "remote", derive(Clone, Serialize, Deserialize))]
pub struct DeviceDescriptor {
    pub extensions: Extensions,
    pub limits: Limits,
}

#[cfg(not(feature = "gfx-backend-gl"))]
pub fn create_instance() -> ::back::Instance {
    ::back::Instance::create("wgpu", 1)
}

#[cfg(all(feature = "local", not(feature = "gfx-backend-gl")))]
#[no_mangle]
pub extern "C" fn wgpu_create_instance() -> InstanceId {
    let inst = create_instance();
    HUB.instances.register_local(inst)
}

#[cfg(all(feature = "local", feature = "gfx-backend-gl"))]
pub fn wgpu_create_gl_instance(windowed_context: back::glutin::WindowedContext) -> InstanceId {
    let raw = back::Surface::from_window(windowed_context);
    let surface = SurfaceHandle::new(raw);
    HUB.surfaces.register_local(surface)
}

#[cfg(all(feature = "window-winit", not(feature = "gfx-backend-gl")))]
#[no_mangle]
pub extern "C" fn wgpu_instance_create_surface_from_winit(
    instance_id: InstanceId,
    window: &winit::Window,
) -> SurfaceId {
    let raw = HUB.instances.read()[instance_id].create_surface(window);
    let surface = SurfaceHandle::new(raw);
    HUB.surfaces.register_local(surface)
}

#[cfg(not(feature = "gfx-backend-gl"))]
#[allow(unused_variables)]
pub fn instance_create_surface_from_xlib(
    instance_id: InstanceId,
    display: *mut *const std::ffi::c_void,
    window: u64,
) -> SurfaceHandle {
    #[cfg(not(all(unix, feature = "gfx-backend-vulkan")))]
    unimplemented!();

    #[cfg(all(unix, feature = "gfx-backend-vulkan"))]
    SurfaceHandle::new(HUB.instances.read()[instance_id].create_surface_from_xlib(display, window))
}

#[cfg(all(feature = "local", not(feature = "gfx-backend-gl")))]
#[no_mangle]
pub extern "C" fn wgpu_instance_create_surface_from_xlib(
    instance_id: InstanceId,
    display: *mut *const std::ffi::c_void,
    window: u64,
) -> SurfaceId {
    let surface = instance_create_surface_from_xlib(instance_id, display, window);
    HUB.surfaces.register_local(surface)
}

#[cfg(not(feature = "gfx-backend-gl"))]
#[allow(unused_variables)]
pub fn instance_create_surface_from_macos_layer(
    instance_id: InstanceId,
    layer: *mut std::ffi::c_void,
) -> SurfaceHandle {
    #[cfg(not(feature = "gfx-backend-metal"))]
    unimplemented!();

    #[cfg(feature = "gfx-backend-metal")]
    SurfaceHandle::new(
        HUB.instances.read()[instance_id]
            .create_surface_from_layer(layer as *mut _, cfg!(debug_assertions)),
    )
}

#[cfg(not(feature = "gfx-backend-gl"))]
#[cfg(feature = "local")]
#[no_mangle]
pub extern "C" fn wgpu_instance_create_surface_from_macos_layer(
    instance_id: InstanceId,
    layer: *mut std::ffi::c_void,
) -> SurfaceId {
    let surface = instance_create_surface_from_macos_layer(instance_id, layer);
    HUB.surfaces.register_local(surface)
}

#[cfg(not(feature = "gfx-backend-gl"))]
#[allow(unused_variables)]
pub fn instance_create_surface_from_windows_hwnd(
    instance_id: InstanceId,
    hinstance: *mut std::ffi::c_void,
    hwnd: *mut std::ffi::c_void,
) -> SurfaceHandle {
    #[cfg(not(any(
        feature = "gfx-backend-dx11",
        feature = "gfx-backend-dx12",
        all(target_os = "windows", feature = "gfx-backend-vulkan"),
    )))]
    let raw = unimplemented!();

    #[cfg(any(feature = "gfx-backend-dx11", feature = "gfx-backend-dx12"))]
    let raw = HUB.instances.read()[instance_id].create_surface_from_hwnd(hwnd);

    #[cfg(all(target_os = "windows", feature = "gfx-backend-vulkan"))]
    let raw = HUB.instances.read()[instance_id].create_surface_from_hwnd(hinstance, hwnd);

    #[allow(unreachable_code)]
    SurfaceHandle::new(raw)
}

#[cfg(not(feature = "gfx-backend-gl"))]
#[cfg(feature = "local")]
#[no_mangle]
pub extern "C" fn wgpu_instance_create_surface_from_windows_hwnd(
    instance_id: InstanceId,
    hinstance: *mut std::ffi::c_void,
    hwnd: *mut std::ffi::c_void,
) -> SurfaceId {
    let surface = instance_create_surface_from_windows_hwnd(instance_id, hinstance, hwnd);
    HUB.surfaces.register_local(surface)
}

#[cfg(all(feature = "local", feature = "gfx-backend-gl"))]
pub fn wgpu_instance_get_gl_surface(instance_id: InstanceId) -> SurfaceId {
    instance_id
}

pub fn instance_get_adapter(instance_id: InstanceId, desc: &AdapterDescriptor) -> AdapterHandle {
    #[cfg(not(feature = "gfx-backend-gl"))]
    let adapters = {
        let instance_guard = HUB.instances.read();
        let instance = &instance_guard[instance_id];
        instance.enumerate_adapters()
    };
    #[cfg(feature = "gfx-backend-gl")]
    let adapters = {
        let surface_guard = HUB.surfaces.read();
        let surface = &surface_guard[instance_id];
        surface.raw.enumerate_adapters()
    };

    let (mut integrated_first, mut discrete_first, mut discrete_last, mut alternative) =
        (None, None, None, None);
        
    // On Windows > 1803, dx12 enumerate_adapters returns the adapters in order from highest to
    // lowest performance. Therefore, the first found adapter in each category is selected.
    //
    // TODO: move power/performance policy querying into gfx, which has more context into
    // performance policy than wgpu
    for (i, adapter) in adapters.iter().enumerate() {
        match adapter.info.device_type {
            hal::adapter::DeviceType::IntegratedGpu => {
                integrated_first = integrated_first.or(Some(i));
            }
            hal::adapter::DeviceType::DiscreteGpu => {
                discrete_first = discrete_first.or(Some(i));
                discrete_last = Some(i);
            }
            _ => alternative = Some(i),
        }
    }

    let preferred_gpu = match desc.power_preference {
        // If `LowPower`, prefer lowest power `DiscreteGPU`
        PowerPreference::LowPower => integrated_first.or(discrete_last),
        PowerPreference::HighPerformance | PowerPreference::Default => {
            discrete_first.or(integrated_first)
        }
    };

    let selected = preferred_gpu
        .or(alternative)
        .expect("No adapters found. Please enable the feature for one of the graphics backends: vulkan, metal, dx12, dx11, gl");

    adapters.into_iter().nth(selected).unwrap()
}

#[cfg(feature = "local")]
#[no_mangle]
pub extern "C" fn wgpu_instance_get_adapter(
    instance_id: InstanceId,
    desc: &AdapterDescriptor,
) -> AdapterId {
    let adapter = instance_get_adapter(instance_id, desc);
    let limits = adapter.physical_device.limits();

    info!("Adapter {:?}", adapter.info);

    assert!(
        BIND_BUFFER_ALIGNMENT % limits.min_storage_buffer_offset_alignment == 0,
        "Adapter storage buffer offset alignment not compatible with WGPU"
    );
    assert!(
        BIND_BUFFER_ALIGNMENT % limits.min_uniform_buffer_offset_alignment == 0,
        "Adapter uniform buffer offset alignment not compatible with WGPU"
    );

    HUB.adapters.register_local(adapter)
}

pub fn adapter_create_device(adapter_id: AdapterId, _desc: &DeviceDescriptor) -> DeviceHandle {
    let adapter_guard = HUB.adapters.read();
    let adapter = &adapter_guard[adapter_id];
    let (raw, queue_group) = adapter.open_with::<_, hal::General>(1, |_qf| true).unwrap();
    let mem_props = adapter.physical_device.memory_properties();

    DeviceHandle::new(raw, adapter_id, queue_group, mem_props)
}

#[cfg(feature = "local")]
#[no_mangle]
pub extern "C" fn wgpu_adapter_request_device(
    adapter_id: AdapterId,
    desc: &DeviceDescriptor,
) -> DeviceId {
    let device = adapter_create_device(adapter_id, desc);
    HUB.devices.register_local(device)
}
