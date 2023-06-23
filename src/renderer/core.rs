use ash::vk::DebugUtilsMessageSeverityFlagsEXT;
use ash::{vk, extensions::ext::DebugUtils, version::EntryV1_0};

use crate::window::Window;

use std::borrow::Cow;
use std::ffi::{CStr, CString};

unsafe extern "system" fn debug_callback_fn(
    msg_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _u_data: *mut std::os::raw::c_void) -> vk::Bool32 {
    let data = *p_data;
    let msg_id_num = data.message_id_number;

    let msg_id_name = if data.p_message_id_name.is_null() {
        Cow::from("[No ID found]")
    } else {
        CStr::from_ptr(data.p_message_id_name).to_string_lossy()
    };

    let msg = if data.p_message.is_null() {
        Cow::from("[No message found]")
    } else {
        CStr::from_ptr(data.p_message).to_string_lossy()
    };

    println! (
        "{:?}: {:?}, {} ({}):\n{}",
        msg_severity,
        msg_type,
        msg_id_name,
        &msg_id_num.to_string(),
        msg,
    );
    println!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");

    vk::FALSE
}

pub struct Core {
    pub entry: ash::Entry,
    pub instance: ash::Instance,

    debug_utils_init: DebugUtils,
    debug_callback: vk::DebugUtilsMessengerEXT,
}

impl Core {
    pub unsafe fn new(validation_enabled: bool, w: &Window) -> Core {
        let entry = ash::Entry::new().unwrap();

        let name = CString::new("Renderer").unwrap();

        let layer_names = if validation_enabled { vec![CString::new("VK_LAYER_KHRONOS_validation").unwrap()] } else { vec![] } ;
        let layer_names_raw: Vec<*const i8> = layer_names.iter().map(|layer| layer.as_ptr()).collect();

        let extension_names = ash_window::enumerate_required_extensions(&w.window).unwrap();
        let mut extension_names_raw: Vec<*const i8> = extension_names.iter().map(|extension| extension.as_ptr()).collect();

        extension_names_raw.push(DebugUtils::name().as_ptr());

        let app_i = vk::ApplicationInfo::builder()
            .application_name(&name);

        let instance_ci = vk::InstanceCreateInfo::builder()
            .application_info(&app_i)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extension_names_raw);

        let instance = entry.create_instance(&instance_ci, None).unwrap();

        let debug_ci = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(debug_callback_fn));

        let debug_utils_init = DebugUtils::new(&entry, &instance);
        let debug_callback = debug_utils_init.create_debug_utils_messenger(&debug_ci, None).unwrap();

        Core {
            entry,
            instance,

            debug_utils_init,
            debug_callback,
        }
    }
}