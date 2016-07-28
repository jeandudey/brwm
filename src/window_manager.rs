use super::xcb;
use super::error::WMError;

use std::collections::HashMap;
use std::rc::Rc;

pub struct WindowManager<'a> {
    conn: Rc<xcb::Connection>,
    clients: HashMap<xcb::Window, xcb::Window>,
    screen: Rc<xcb::Screen<'a>>,
}

impl<'a> WindowManager<'a> {
    pub fn new(conn: &Rc<xcb::Connection>,
               preferred_screen: &Rc<xcb::Screen<'a>>)
               -> WindowManager<'a> {
        WindowManager {
            conn: conn.clone(),
            clients: HashMap::new(),
            screen: preferred_screen.clone(),
        }
    }

    pub fn run(&mut self) -> Result<(), WMError> {
        if let Err(e) = self.select_window_wm_events(self.screen.root()) {
            return Err(WMError::from(e));
        };
        debug!("WM events selected (wid: 0x{:X})", self.screen.root());

        let query_tree_cookie = xcb::query_tree(&self.conn, self.screen.root());
        let query_tree_reply = match query_tree_cookie.get_reply() {
            Ok(r) => r,
            Err(e) => return Err(WMError::from(e)),
        };

        let mut geometry_cookies: HashMap<xcb::Window, xcb::GetGeometryCookie> = HashMap::new();

        geometry_cookies.reserve(query_tree_reply.children_len() as usize);

        for child in query_tree_reply.children() {
            let cookie = xcb::get_geometry(&self.conn, *child);
            geometry_cookies.insert(*child, cookie);
        }

        for child in query_tree_reply.children() {
            if let Some(cookie) = geometry_cookies.get(child) {
                if let Ok(reply) = cookie.get_reply() {
                    if let Err(e) = self.frame(*child, reply) {
                        return Err(WMError::from(e));
                    }
                }
            } else {
                unreachable!();
            }
        }

        loop {
            if let Some(e) = self.conn.wait_for_event() {
                match e.response_type() as u8 {
                    xcb::MAP_REQUEST => {
                        let map_request: &xcb::MapRequestEvent = xcb::cast_event(&e);
                        if let Err(e) = self.on_map_request(map_request) {
                            return Err(WMError::from(e));
                        }
                    }
                    xcb::UNMAP_NOTIFY => {
                        let unmap_event: &xcb::UnmapNotifyEvent = xcb::cast_event(&e);
                        if let Err(e) = self.on_unmap_notify(unmap_event) {
                            return Err(WMError::from(e));
                        }
                    }
                    xcb::CONFIGURE_REQUEST => {
                        let configure_request: &xcb::ConfigureRequestEvent = xcb::cast_event(&e);

                        if let Err(e) = self.on_configure_request(configure_request) {
                            return Err(WMError::from(e));
                        }
                    }
                    _ => continue,
                }
            } else {
                return Ok(());
            }
        }
    }

    fn select_window_wm_events(&self, window: xcb::Window) -> Result<(), xcb::GenericError> {
        let event_mask = [(xcb::CW_EVENT_MASK,
                           xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY |
                           xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT)];

        xcb::change_window_attributes_checked(&self.conn, window, &event_mask).request_check()
    }

    fn frame(&mut self,
             window: xcb::Window,
             window_geometry: xcb::GetGeometryReply)
             -> Result<(), xcb::GenericError> {
        info!("Creating frame for window (wid: 0x{:X}).", window);

        assert!(!self.clients.contains_key(&window));

        let frame = self.conn.generate_id();
        debug!("Window ID generated: 0x{:X}.", frame);

        try!(xcb::create_window_checked(&self.conn,
                                        xcb::ffi::base::XCB_COPY_FROM_PARENT as u8,
                                        frame,
                                        self.screen.root(),
                                        window_geometry.x(),
                                        window_geometry.y(),
                                        window_geometry.width(),
                                        window_geometry.height(),
                                        BORDER_WIDTH,
                                        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                                        self.screen.root_visual(),
                                        &[(0, 0)])
            .request_check());

        debug!("Window created (wid: 0x{:X})", frame);

        try!(self.select_window_wm_events(frame));
        debug!("WM events selected (wid: 0x{:X}).", frame);

        try!(xcb::change_save_set_checked(&self.conn, xcb::xproto::SET_MODE_INSERT as u8, window)
            .request_check());
        debug!("Save set changed for (wid: 0x{:X}, mode: {})",
               frame,
               stringify!(xcb::xproto::SET_MODE_INSERT));

        try!(xcb::reparent_window_checked(&self.conn, window, frame, 0, 0).request_check());
        debug!("Window reparented (window: 0x{:X}, parent: 0x{:X})",
               window,
               frame);

        try!(xcb::map_window_checked(&self.conn, frame).request_check());
        debug!("Window mapped (wid: 0x{:X})", frame);

        self.clients.insert(window, frame);

        return Ok(());
    }

    fn unframe(&mut self, window: xcb::Window) -> Result<(), xcb::GenericError> {
        if self.clients.contains_key(&window) {
            let frame = self.clients[&window];

            try!(xcb::unmap_window_checked(&self.conn, frame).request_check());
            debug!("Window unmapped (wid: 0x{:X}).", frame);

            try!(xcb::reparent_window_checked(&self.conn, window, self.screen.root(), 0, 0)
                .request_check());
            debug!("Window reparented (wid: 0x{:X}, parent: 0x{:X}).",
                   window,
                   self.screen.root());

            try!(xcb::change_save_set_checked(&self.conn,
                                              xcb::xproto::SET_MODE_DELETE as u8,
                                              window)
                .request_check());
            debug!("Save set changed (wid: 0x{:X}, mode: {}).",
                   window,
                   stringify!(xcb::xproto::SET_MODE_DELETE));

            self.clients.remove(&window);
        }

        Ok(())
    }

    fn on_unmap_notify(&mut self, e: &xcb::UnmapNotifyEvent) -> Result<(), xcb::GenericError> {
        debug!("UnmapNotifyEvent received.");
        try!(self.unframe(e.window()));
        debug!("Window unframed (wid: 0x{:X})", e.window());

        Ok(())
    }

    fn on_map_request(&mut self, e: &xcb::MapRequestEvent) -> Result<(), xcb::GenericError> {
        debug!("MapRequestEvent received.");
        let cookie = xcb::get_geometry(&self.conn, e.window());
        let reply = try!(cookie.get_reply());

        try!(self.frame(e.window(), reply));
        debug!("Window framed (wid: 0x{:X}).", e.window());

        try!(xcb::map_window(&self.conn, e.window()).request_check());
        debug!("Window mapped (wid: 0x{:X})", e.window());

        Ok(())
    }

    fn on_configure_request(&self,
                            e: &xcb::ConfigureRequestEvent)
                            -> Result<(), xcb::GenericError> {
        let values_list: [(u16, u32); 7] =
            [(xcb::CONFIG_WINDOW_X as u16, e.x() as u32),
             (xcb::CONFIG_WINDOW_Y as u16, e.y() as u32),
             (xcb::CONFIG_WINDOW_WIDTH as u16, e.width() as u32),
             (xcb::CONFIG_WINDOW_HEIGHT as u16, e.height() as u32),
             (xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, e.border_width() as u32),
             (xcb::CONFIG_WINDOW_SIBLING as u16, e.sibling() as u32),
             (xcb::CONFIG_WINDOW_STACK_MODE as u16, e.stack_mode() as u32)];

        if self.clients.contains_key(&e.window()) {
            let frame = self.clients[&e.window()];

            try!(xcb::configure_window_checked(&self.conn, frame, &values_list).request_check());
            debug!("Window configured (wid: 0x{:X})", frame);
        }

        try!(xcb::configure_window_checked(&self.conn, e.window(), &values_list).request_check());
        debug!("Window configured (wid: 0x{:X})", e.window());

        Ok(())
    }
}

const BORDER_WIDTH: u16 = 3;
