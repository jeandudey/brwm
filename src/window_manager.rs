use super::x11::xlib;
use super::libc::{c_int, c_uint};
use super::display::Display;

use std::sync::Mutex;
use std::error;
use std::fmt;
use std::collections::HashMap;
use std::mem::zeroed;

lazy_static! {
    static ref WM_DETECTED: Mutex<bool> = Mutex::new(false);
}

#[derive(Debug)]
pub struct WindowManager {
    /// X Server display
    display: Display,
    root: xlib::Window,
    clients: HashMap<xlib::Window, xlib::Window>,
}

#[derive(Debug, Clone, Copy)]
pub enum WMError {
    /// Returned when another WM is running.
    AnotherWM,
}

impl fmt::Display for WMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            WMError::AnotherWM => write!(f, "Another Window Manager is running"),
        }
    }
}

impl error::Error for WMError {
    fn description(&self) -> &str {
        match *self {
            WMError::AnotherWM => "Another Window Manager is running",
        }
    }
}

const BORDER_WIDTH: u32 = 3;
const BORDER_COLOR: u32 = 0xff0000;
const BG_COLOR: u32 = 0x0000ff;


impl WindowManager {
    /// Creates an empty WindowManager structure.
    pub fn new() -> Option<Self>{
        if let Some(display) = Display::open_display(None) {
            let res = WindowManager {
                display: display,
                root: 0,
                clients: HashMap::new(),
            };

            return Some(res);
        } else {
            return None;
        }
    }

    pub fn run(&mut self) -> Result<(), WMError> {
        let screen = self.display.default_screen_of_display();
        self.root = screen.root_window_of_screen();

        unsafe { xlib::XSetErrorHandler(Some(on_wm_detected)) };

        self.display.select_input(self.root,
                                  xlib::SubstructureRedirectMask |
                                  xlib::SubstructureNotifyMask);

        self.display.sync(false);

        if *WM_DETECTED.lock().unwrap() {
            return Err(WMError::AnotherWM);
        }

        unsafe {
            xlib::XSetErrorHandler(Some(on_x_error));
        }

        self.display.grab_server();

        let query_result = self.display.query_tree(self.root);

        if let Some(children) = query_result.2 {
            for child in children {
                self.frame(child);
            }
        }

        self.display.ungrab_server();

        loop {
            let e = self.display.next_event();

            match e.get_type() {
                xlib::CreateNotify => {
                    let xcreatewindow = xlib::XCreateWindowEvent::from(e);
                    self.on_create_notify(&xcreatewindow);
                },
                xlib::DestroyNotify => {
                    let xdestroywindow = xlib::XDestroyWindowEvent::from(e);
                    self.on_destroy_notify(&xdestroywindow);
                },
                xlib::ReparentNotify => {
                    let xreparent = xlib::XReparentEvent::from(e);
                    self.on_reparent_notify(&xreparent);
                },
                xlib::MapNotify => {
                    let xmap = xlib::XMapEvent::from(e);
                    self.on_map_notify(&xmap);
                },
                xlib::UnmapNotify => {
                    let xunmap = xlib::XUnmapEvent::from(e);
                    self.on_unmap_notify(&xunmap);
                },
                xlib::ConfigureNotify => {
                    let xconfigure = xlib::XConfigureEvent::from(e);
                    self.on_configure_notify(&xconfigure);
                },
                xlib::MapRequest => {
                    let xmaprequest = xlib::XMapRequestEvent::from(e);
                    self.on_map_request(&xmaprequest);
                },
                xlib::ConfigureRequest => {
                    let xconfigurerequest = xlib::XConfigureRequestEvent::from(e);
                    self.on_configure_request(&xconfigurerequest);
                },
                _ => continue,
            }
        }

        Ok(())
    }

    fn frame(&mut self, window: xlib::Window) {
        assert!(!self.clients.contains_key(&window));

        let window_attrs = self.display.get_window_attributes(window);
        
        let frame = self.display.create_simple_window(self.root,
                                                      window_attrs.x,
                                                      window_attrs.y,
                                                      window_attrs.width as c_uint,
                                                      window_attrs.height as c_uint,
                                                      BORDER_WIDTH,
                                                      BORDER_COLOR,
                                                      BG_COLOR);
        self.display.select_input(frame,
                                  xlib::SubstructureRedirectMask |
                                  xlib::SubstructureNotifyMask);

        self.display.add_to_save_set(window);

        self.display.reparent_window(window, frame, 0, 0);

        self.display.map_window(frame);
    
        self.clients.insert(window, frame);

        self.display.grab_button(xlib::Button1,
                                 xlib::Mod1Mask,
                                  window,
                                 false,
                                 (xlib::ButtonPressMask |
                                 xlib::ButtonReleaseMask |
                                 xlib::ButtonMotionMask) as u32,
                                 xlib::GrabModeAsync,
                                 xlib::GrabModeAsync,
                                 0,
                                 0);

        self.display.grab_button(xlib::Button3,
                                 xlib::Mod1Mask,
                                 window,
                                 false,
                                 (xlib::ButtonPressMask |
                                 xlib::ButtonReleaseMask |
                                 xlib::ButtonMotionMask) as u32,
                                 xlib::GrabModeAsync,
                                 xlib::GrabModeAsync,
                                 0,
                                 0);
    }

    fn unframe(&mut self, window: xlib::Window) {
        if self.clients.contains_key(&window) {
            let frame = self.clients[&window];

            self.display.unmap_window(frame);
            self.display.reparent_window(window, self.root, 0, 0);
            self.display.remove_from_save_set(window);
            self.display.destroy_window(window);

            self.clients.remove(&window);
        }
    }
    
    fn on_create_notify(&self, _: &xlib::XCreateWindowEvent) {
    }

    fn on_destroy_notify(&self, _: &xlib::XDestroyWindowEvent) {
    }

    fn on_reparent_notify(&self, _: &xlib::XReparentEvent) {
    }

    fn on_map_notify(&self, _: &xlib::XMapEvent) {
    }

    fn on_unmap_notify(&mut self, e: &xlib::XUnmapEvent) {
        self.unframe(e.window);
    }

    fn on_configure_notify(&self, _: &xlib::XConfigureEvent) {
    }

    fn on_map_request(&mut self, e: &xlib::XMapRequestEvent) {
        self.frame(e.window);
        self.display.map_window(e.window);
    }


    fn on_configure_request(&self, e: &xlib::XConfigureRequestEvent) {
        let mut changes: xlib::XWindowChanges = unsafe { zeroed() };

        changes.x = e.x;
        changes.y = e.y;
        changes.width = e.width;
        changes.height = e.height;
        changes.border_width = e.border_width;
        changes.sibling = e.above;
        changes.stack_mode = e.detail;

        if self.clients.contains_key(&e.window) {
            let frame = self.clients[&e.window]; 
            self.display.configure_window(frame,
                                          e.value_mask,
                                          &mut changes);
        }

        self.display.configure_window(e.window,
                                      e.value_mask,
                                      &mut changes);
    }
}

unsafe extern "C" fn on_x_error(_: *mut xlib::Display,
                                _: *mut xlib::XErrorEvent) -> c_int {
    0
}


unsafe extern "C" fn on_wm_detected(_: *mut xlib::Display,
                                    e: *mut xlib::XErrorEvent) -> c_int {
    assert!((*e as xlib::XErrorEvent).error_code == xlib::BadAccess);
    *WM_DETECTED.lock().unwrap() = true;
    0
}
