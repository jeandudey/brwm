use super::xcb;

use std::collections::HashMap;
use std::rc::Rc;

pub struct WindowManager<'a> {
    conn: Rc<xcb::Connection>,
    clients: HashMap<xcb::Window, xcb::Window>,
    screen: Rc<xcb::Screen<'a>>
}

impl<'a> WindowManager<'a> {
    pub fn new(conn: &Rc<xcb::Connection>,
               preferred_screen: &Rc<xcb::Screen<'a>>) -> WindowManager<'a> {
        WindowManager {
            conn: conn.clone(),
            clients: HashMap::new(),
            screen: preferred_screen.clone(),
        }
    }

    pub fn run(&mut self) {
        let event_mask =
            [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY |
                                  xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT)];
        let change_cookie = 
            xcb::change_window_attributes_checked(&self.conn,
                                                  self.screen.root(),
                                                  &event_mask);
        
        if change_cookie.request_check().is_err() {
            panic!("Oh no!");
        }

        let query_tree_cookie = xcb::query_tree(&self.conn, self.screen.root());
        let query_tree_reply = query_tree_cookie.get_reply().unwrap();
        
        let mut geometry_cookies: HashMap<xcb::Window, xcb::GetGeometryCookie> =
            HashMap::new();

        geometry_cookies.reserve(query_tree_reply.children_len() as usize);

        for child in query_tree_reply.children() {
            let cookie = xcb::get_geometry(&self.conn, *child);
            geometry_cookies.insert(*child, cookie);
        }

        for child in query_tree_reply.children() {
            if let Some(cookie) = geometry_cookies.get(child) {
                if let Ok(reply) = cookie.get_reply() {
                    self.frame(*child, reply);
                }
            } else {
                unreachable!();
            }
        }

        loop {
            if let Some(e) = self.conn.wait_for_event() {
                match e.response_type() as u8 {
                    xcb::MAP_REQUEST => {
                        let map_request: &xcb::MapRequestEvent =
                            xcb::cast_event(&e);
                        self.on_map_request(map_request);
                    },
                    xcb::UNMAP_NOTIFY => {
                        let unmap_event: &xcb::UnmapNotifyEvent =
                            xcb::cast_event(&e);
                        self.on_unmap_notify(unmap_event);
                    },
                    xcb::CONFIGURE_REQUEST => {
                        let configure_request: &xcb::ConfigureRequestEvent =
                            xcb::cast_event(&e);
                        self.on_configure_request(configure_request);
                    },
                    _ => continue,
                }
            } else {
                return;
            }
        }
    }

    fn frame(&mut self, window: xcb::Window, window_geometry: xcb::GetGeometryReply) {
        assert!(!self.clients.contains_key(&window));

        let frame = self.conn.generate_id();

        xcb::create_window(&self.conn,
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
                           &[(0, 0)]);

        let event_mask =
            [(xcb::CW_EVENT_MASK, xcb::EVENT_MASK_SUBSTRUCTURE_NOTIFY |
                                  xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT)];
        let change_cookie = 
            xcb::change_window_attributes_checked(&self.conn, frame, &event_mask); 

        if change_cookie.request_check().is_err() {
            panic!("Another WM is running!");
        }

        xcb::change_save_set(&self.conn,
                             xcb::xproto::SET_MODE_INSERT as u8,
                             frame);

        xcb::reparent_window(&self.conn, window, frame, 0, 0);

        xcb::map_window(&self.conn, window);
 
        self.clients.insert(window, frame);
    }

    fn unframe(&mut self, window: xcb::Window) {
        if self.clients.contains_key(&window) {
            let frame = self.clients[&window];

            xcb::unmap_window(&self.conn, frame);
            xcb::reparent_window(&self.conn, window, self.screen.root(), 0, 0);

            xcb::change_save_set(&self.conn,
                                 xcb::xproto::SET_MODE_DELETE as u8,
                                 frame);

            self.clients.remove(&window);
        }
    }
    
    fn on_unmap_notify(&mut self, e: &xcb::UnmapNotifyEvent) {
        self.unframe(e.window());
    }

    fn on_map_request(&mut self, e: &xcb::MapRequestEvent) {
        let cookie = xcb::get_geometry(&self.conn, e.window());

        if let Ok(reply) = cookie.get_reply() {
            self.frame(e.window(), reply);
            xcb::map_window(&self.conn, e.window());
        }
    }

    fn on_configure_request(&self, e: &xcb::ConfigureRequestEvent) {
        use xcb::ffi::xproto;

        let values_list: [(u16, u32); 7] = [
            (xproto::XCB_CONFIG_WINDOW_X as u16, e.x() as u32),
            (xproto::XCB_CONFIG_WINDOW_Y as u16, e.y() as u32),
            (xproto::XCB_CONFIG_WINDOW_WIDTH as u16, e.width() as u32),
            (xproto::XCB_CONFIG_WINDOW_HEIGHT as u16, e.height() as u32),
            (xproto::XCB_CONFIG_WINDOW_BORDER_WIDTH as u16, e.border_width() as u32),
            (xproto::XCB_CONFIG_WINDOW_SIBLING as u16, e.sibling() as u32),
            (xproto::XCB_CONFIG_WINDOW_STACK_MODE as u16, e.stack_mode() as u32)
        ];

        if self.clients.contains_key(&e.window()) {
            let frame = self.clients[&e.window()]; 

            xcb::configure_window(&self.conn, frame, &values_list);
        }

        xcb::configure_window(&self.conn, e.window(), &values_list);
    }
}

const BORDER_WIDTH: u16 = 3;
