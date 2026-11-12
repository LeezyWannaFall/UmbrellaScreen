use gtk::{glib::translate::ToGlibPtr, prelude::*};
#[link(name = "gtk4-layer-shell")]
unsafe extern "C" {
    fn gtk_layer_init_for_window(window: *mut gtk::ffi::GtkWindow);
    fn gtk_layer_set_namespace(window: *mut gtk::ffi::GtkWindow, name: *const std::ffi::c_char);
    fn gtk_layer_set_layer(window: *mut gtk::ffi::GtkWindow, layer: i32);
    fn gtk_layer_set_monitor(
        window: *mut gtk::ffi::GtkWindow,
        monitor: *mut gtk::gdk::ffi::GdkMonitor,
    );
    fn gtk_layer_set_anchor(window: *mut gtk::ffi::GtkWindow, edge: i32, anchor: i32);
    fn gtk_layer_set_exclusive_zone(window: *mut gtk::ffi::GtkWindow, zone: i32);
    fn gtk_layer_set_keyboard_mode(window: *mut gtk::ffi::GtkWindow, mode: i32);
}
pub fn setup(window: &gtk::ApplicationWindow, monitor: &gtk::gdk::Monitor) {
    // Called on GTK's main thread before realization; both objects remain alive.
    unsafe {
        let ptr = window.upcast_ref::<gtk::Window>().to_glib_none().0;
        gtk_layer_init_for_window(ptr);
        gtk_layer_set_namespace(ptr, c"umbrellascreen".as_ptr());
        gtk_layer_set_layer(ptr, 3);
        gtk_layer_set_monitor(ptr, monitor.to_glib_none().0);
        for edge in 0..4 {
            gtk_layer_set_anchor(ptr, edge, 1);
        }
        gtk_layer_set_exclusive_zone(ptr, -1);
        gtk_layer_set_keyboard_mode(ptr, 1);
    }
}
