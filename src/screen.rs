use super::x11::xlib;

pub struct Screen {
    screen: *mut xlib::Screen,
}

impl Screen {
    pub fn new(screen: *mut xlib::Screen) -> Self {
        Screen {
            screen: screen,
        }
    }

    pub fn root_window_of_screen(&self) -> xlib::Window {
        unsafe { xlib::XRootWindowOfScreen(self.screen) }
    }
}
