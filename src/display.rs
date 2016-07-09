use super::x11::xlib;
use super::libc::{c_int, c_uint, c_long, c_ulong};
use super::screen::Screen;

use std::ffi::CString;
use std::ptr::{null, null_mut};
use std::os::raw::c_void;
use std::mem::zeroed;

#[derive(Debug)]
pub struct Display {
    display: *mut xlib::Display,
}

impl Display {
    pub fn open_display(display_name: Option<CString>) -> Option<Display> {
        let mut result = Display {
            display: null_mut(),
        };

        if let Some(name) = display_name {
            result.display = unsafe { xlib::XOpenDisplay(name.as_ptr()) };
        } else {
            result.display = unsafe { xlib::XOpenDisplay(null()) };
        }

        if result.display == null_mut() {
            return None;
        }

        Some(result)
    }

    pub fn default_screen_of_display(&self) -> Screen {
        let screen = unsafe { xlib::XDefaultScreenOfDisplay(self.display) };

        Screen::new(screen)
    }

    pub fn select_input(&self, root: xlib::Window, flags: c_long) {
        unsafe { xlib::XSelectInput(self.display, root, flags) };
    }

    pub fn sync(&self, discard: bool) {
        unsafe {
            xlib::XSync(self.display, if discard { xlib::True } else { xlib::False })
        };
    }

    pub fn grab_server(&self) {
        unsafe { xlib::XGrabServer(self.display) };
    }

    pub fn ungrab_server(&self) {
        unsafe { xlib::XUngrabServer(self.display) };
    }

    pub fn query_tree(&self, w: xlib::Window) -> (xlib::Window,
                                                  xlib::Window,
                                                  Option<Vec<xlib::Window>>) {
        let mut root_return: xlib::Window = 0;
        let mut parent_return: xlib::Window = 0;
        let mut children_return: *mut xlib::Window = null_mut();
        let mut nchildren_return: c_uint = 0;

        unsafe {
            xlib::XQueryTree(self.display,
                             w,
                             &mut root_return,
                             &mut parent_return,
                             &mut children_return,
                             &mut nchildren_return);
        }

        assert_eq!(w, root_return);

        if children_return != null_mut() {
            let mut children: Vec<xlib::Window> = Vec::with_capacity(nchildren_return as usize);
            for i in 0..nchildren_return {
                let child = unsafe {
                    *children_return.offset(i as isize) as xlib::Window
                };

                children.push(child);
            }

            unsafe { xlib::XFree(children_return as *mut c_void); }
            
            return (root_return, parent_return, Some(children));
        } else {
            return (root_return, parent_return, None);
        }
    }

    pub fn next_event(&self) -> xlib::XEvent {
        unsafe {
            let mut event_return: xlib::XEvent = zeroed();
            xlib::XNextEvent(self.display, &mut event_return);
            return event_return;
        }
    }

    pub fn get_window_attributes(&self, w: xlib::Window) -> xlib::XWindowAttributes {
        unsafe {
            let mut window_attributes_return: xlib::XWindowAttributes = zeroed();
            xlib::XGetWindowAttributes(self.display, w, &mut window_attributes_return);
            return window_attributes_return;
        }
    }

    pub fn create_simple_window(&self,
                                parent: xlib::Window,
                                x: c_int,
                                y: c_int,
                                width: c_uint,
                                height: c_uint,
                                border_width: c_uint,
                                border: c_ulong,
                                border_background: c_ulong) -> xlib::Window {
        unsafe {
            xlib::XCreateSimpleWindow(self.display,
                                      parent,
                                      x,
                                      y,
                                      width,
                                      height,
                                      border_width,
                                      border,
                                      border_background)
        }
    }

    pub fn destroy_window(&self, w: xlib::Window) {
        unsafe {
            xlib::XDestroyWindow(self.display, w);
        }
    }

    pub fn add_to_save_set(&self, w: xlib::Window) {
        unsafe {
            xlib::XAddToSaveSet(self.display, w);
        }
    }

    pub fn remove_from_save_set(&self, w: xlib::Window) {
        unsafe {
            xlib::XRemoveFromSaveSet(self.display, w);
        }
    }

    pub fn reparent_window(&self,
                           w: xlib::Window,
                           parent: xlib::Window,
                           x: c_int,
                           y: c_int) {
        unsafe {
            xlib::XReparentWindow(self.display, w, parent, x, y);
        }
    }

    pub fn map_window(&self, w: xlib::Window) {
        unsafe {
            xlib::XMapWindow(self.display, w);
        }
    }

    pub fn unmap_window(&self, w: xlib::Window) {
        unsafe {
            xlib::XUnmapWindow(self.display, w);
        }
    }

    pub fn grab_button(&self,
                       button: c_uint,
                       modifiers: c_uint,
                       grab_window: xlib::Window,
                       owner_events: bool,
                       event_mask: c_uint,
                       pointer_mode: c_int,
                       keyboard_mode: c_int,
                       confine_to: xlib::Window,
                       cursor: xlib::Cursor) {
        let owner_events_raw = if owner_events { xlib::True } else { xlib::False };

        unsafe {
            xlib::XGrabButton(self.display,
                              button,
                              modifiers,
                              grab_window,
                              owner_events_raw,
                              event_mask,
                              pointer_mode,
                              keyboard_mode,
                              confine_to,
                              cursor);
        }
    }

    pub fn configure_window(&self,
                            w: xlib::Window,
                            value_mask: c_uint,
                            values: &mut xlib::XWindowChanges) {
        unsafe {
            xlib::XConfigureWindow(self.display, w, value_mask, values);
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        if self.display != null_mut() {
            unsafe { xlib::XCloseDisplay(self.display) };
        }
    }
}
