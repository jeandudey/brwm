use super::{xcb, xcb_image};
use std::rc::Rc;

struct CursorData {
    pub width: u32,
    pub hot: [u16; 2],
    pub mask: [u8; 32],
    pub fore: [u8; 32],
}

static mut SWEEP0_DATA: CursorData = CursorData {
    width: 16,
    hot: [7, 7],
    mask: [0xC0, 0x03, 0xC0, 0x03, 0xC0, 0x03, 0xC0, 0x03, 0xC0, 0x03, 0xC0, 0x03, 0xFF, 0xFF,
           0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xC0, 0x03, 0xC0, 0x03, 0xC0, 0x03, 0xC0, 0x03,
           0xC0, 0x03, 0xC0, 0x03],
    fore: [0x00, 0x00, 0x80, 0x01, 0x80, 0x01, 0x80, 0x01, 0x80, 0x01, 0x80, 0x01, 0x80, 0x01,
           0xFE, 0x7F, 0xFE, 0x7F, 0x80, 0x01, 0x80, 0x01, 0x80, 0x01, 0x80, 0x01, 0x80, 0x01,
           0x80, 0x01, 0x00, 0x00],
};

static mut BOXCURS_DATA: CursorData = CursorData {
    width: 16,
    hot: [7, 7],
    mask: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x1F, 0xF8, 0x1F, 0xF8,
           0x1F, 0xF8, 0x1F, 0xF8, 0x1F, 0xF8, 0x1F, 0xF8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
           0xFF, 0xFF, 0xFF, 0xFF],
    fore: [0x00, 0x00, 0xFE, 0x7F, 0xFE, 0x7F, 0xFE, 0x7F, 0x0E, 0x70, 0x0E, 0x70, 0x0E, 0x70,
           0x0E, 0x70, 0x0E, 0x70, 0x0E, 0x70, 0x0E, 0x70, 0x0E, 0x70, 0xFE, 0x7F, 0xFE, 0x7F,
           0xFE, 0x7F, 0x00, 0x00],
};

static mut SIGHT_DATA: CursorData = CursorData {
    width: 16,
    hot: [7, 7],
    mask: [0xF8, 0x1F, 0xFC, 0x3F, 0xFE, 0x7F, 0xDF, 0xFB, 0xCF, 0xF3, 0xC7, 0xE3, 0xFF, 0xFF,
           0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xC7, 0xE3, 0xCF, 0xF3, 0xDF, 0x7B, 0xFE, 0x7F,
           0xFC, 0x3F, 0xF8, 0x1F],
    fore: [0x00, 0x00, 0xF0, 0x0F, 0x8C, 0x31, 0x84, 0x21, 0x82, 0x41, 0x82, 0x41, 0x82, 0x41,
           0xFE, 0x7F, 0xFE, 0x7F, 0x82, 0x41, 0x82, 0x41, 0x82, 0x41, 0x84, 0x21, 0x8C, 0x31,
           0xF0, 0x0F, 0x00, 0x00],
};

static mut ARROW_DATA: CursorData = CursorData {
    width: 16,
    hot: [1, 1],
    mask: [0xFF, 0x07, 0xFF, 0x07, 0xFF, 0x03, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x01, 0xFF, 0x03,
           0xFF, 0x07, 0xE7, 0x0F, 0xC7, 0x1F, 0x83, 0x3F, 0x00, 0x7F, 0x00, 0xFE, 0x00, 0x7C,
           0x00, 0x38, 0x00, 0x10],
    fore: [0x00, 0x00, 0xFE, 0x03, 0xFE, 0x00, 0x3E, 0x00, 0x7E, 0x00, 0xFE, 0x00, 0xF6, 0x01,
           0xE6, 0x03, 0xC2, 0x07, 0x82, 0x0F, 0x00, 0x1F, 0x00, 0x3E, 0x00, 0x7C, 0x00, 0x38,
           0x00, 0x10, 0x00, 0x00],
};

type ColorItem = xcb::ffi::xproto::xcb_coloritem_t;

pub struct CursorManager {
    pub arrow: xcb::Cursor,
    pub target: xcb::Cursor,
    pub sweep0: xcb::Cursor,
    pub boxcurs: xcb::Cursor,
    conn: Rc<xcb::Connection>,
}

impl CursorManager {
    pub fn new(conn: &Rc<xcb::Connection>, window: xcb::Window, colormap: xcb::Colormap) -> Self {
        let mut cm = CursorManager {
            arrow: 0,
            target: 0,
            sweep0: 0,
            boxcurs: 0,
            conn: conn.clone(),
        };

        let mut bl = ColorItem {
            pixel: 0,
            red: 0,
            green: 0,
            blue: 0,
            flags: 0,
            pad0: 0,
        };

        let mut wh = ColorItem {
            pixel: 0,
            red: 0,
            green: 0,
            blue: 0,
            flags: 0,
            pad0: 0,
        };

        debug!("Allocating named color: black");
        let black_color_cookie = xcb::alloc_named_color(&cm.conn, colormap, "black");
        if let Ok(reply) = black_color_cookie.get_reply() {
            bl.pixel = reply.pixel();
            bl.red = reply.exact_red();
            bl.green = reply.exact_green();
            bl.blue = reply.exact_blue();
        } else {
            warning!("Color not allocated (black), using fallback.");
            bl.pixel = 0;
            bl.red = 0;
            bl.green = 0;
            bl.blue = 0;
        }

        debug!("Allocating named color: white");
        let white_color_cookie = xcb::alloc_named_color(&cm.conn, colormap, "white");
        if let Ok(reply) = white_color_cookie.get_reply() {
            wh.pixel = reply.pixel();
            wh.red = reply.exact_red();
            wh.green = reply.exact_green();
            wh.blue = reply.exact_blue();
        } else {
            warning!("Color not allocated (white), using fallback.");
            wh.pixel = 0xFFFFFF;
            wh.red = 0xFFFF;
            wh.green = 0xFFFF;
            wh.blue = 0xFFFF;
        }

        unsafe {
            cm.arrow = Self::get_cursor(&cm.conn, window, &mut ARROW_DATA, &bl, &wh);
            cm.target = Self::get_cursor(&cm.conn, window, &mut SIGHT_DATA, &bl, &wh);
            cm.sweep0 = Self::get_cursor(&cm.conn, window, &mut SWEEP0_DATA, &bl, &wh);
            cm.boxcurs = Self::get_cursor(&cm.conn, window, &mut BOXCURS_DATA, &bl, &wh);
        }

        cm
    }

    fn get_cursor(conn: &xcb::Connection, window: xcb::Window, c: &mut CursorData, bl: &ColorItem, wh: &ColorItem) -> xcb::Cursor {
        use xcb_image::Size;

        let fore = xcb_image::create_pixmap_from_bitmap_data(conn, window, &mut c.fore, Size(c.width, c.width), 1, 1, 0, None).unwrap();
        let mask = xcb_image::create_pixmap_from_bitmap_data(conn, window, &mut c.mask, Size(c.width, c.width), 1, 1, 0, None).unwrap();

        let cursor = conn.generate_id();
        xcb::create_cursor(conn, cursor, fore, mask, bl.red, bl.green, bl.blue, wh.red, wh.green, wh.blue, c.hot[0], c.hot[1]);
        cursor
    }
}
