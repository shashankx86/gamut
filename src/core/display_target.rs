#[cfg(target_os = "linux")]
mod imp {
    use gtk::gdk;
    use gtk::glib;
    use gtk::glib::translate::ToGlibPtr;
    use gtk::prelude::*;
    use std::ffi::CStr;

    pub(crate) fn active_output_name() -> Option<String> {
        if !gtk::is_initialized() {
            gtk::init().ok()?;
        }

        let display = gdk::Display::default()?;
        let seat = display.default_seat()?;
        let pointer = seat.pointer()?;
        let (screen, x, y) = pointer.position();

        let monitor_index =
            unsafe { gdk::ffi::gdk_screen_get_monitor_at_point(screen.to_glib_none().0, x, y) };

        if monitor_index < 0 {
            return None;
        }

        let plug_name = unsafe {
            gdk::ffi::gdk_screen_get_monitor_plug_name(screen.to_glib_none().0, monitor_index)
        };

        if plug_name.is_null() {
            return None;
        }

        let output_name = unsafe { CStr::from_ptr(plug_name) }
            .to_string_lossy()
            .into_owned();
        unsafe { glib::ffi::g_free(plug_name.cast()) };

        Some(output_name)
    }
}

#[cfg(not(target_os = "linux"))]
mod imp {
    pub(crate) fn active_output_name() -> Option<String> {
        None
    }
}

pub(crate) use imp::active_output_name;
