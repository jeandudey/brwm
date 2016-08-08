use super::{xcb, xcb_cursor};
use std::rc::Rc;

pub struct CursorManager {
    pub arrow: xcb::Cursor,
    conn: Rc<xcb::Connection>,
}

impl CursorManager {
    pub fn new<'s>(conn: &'s Rc<xcb::Connection>, screen: &xcb::Screen<'s>) -> Option<CursorManager> {
        let mut cm = CursorManager {
            arrow: 0,
            conn: conn.clone(),
        };
        
        info!("Creating CursorContext.");
        let ctx = match xcb_cursor::CursorContext::new(conn, screen) {
            Some(ctx) => ctx,
            None => return None,
        };

        debug!("Loading cursor \"default\"");
        cm.arrow = match ctx.load_cursor("default") {
            Ok(c) => c,
            Err(_) => return None, // TODO: Improve this
        };

        Some(cm)
    }
}

impl Drop for CursorManager {
    fn drop(&mut self) {
        xcb::free_cursor(&self.conn, self.arrow);
    }
}
