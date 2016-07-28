use super::xcb;
use xcb::{InternAtomCookie, Atom};

use std::collections::HashMap;
use std::rc::Rc;

static ATOMS_LIST: &'static [&'static str] = &["WM_STATE",
                                               "WM_CHANGE_STATE",
                                               "WM_PROTOCOLS",
                                               "WM_DELETE_WINDOW",
                                               "WM_TAKE_FOCUS",
                                               "WM_COLORMAP_WINDOWS",
                                               "COMPOUND_TEXT",
                                               "_MOZILLA_URL",
                                               "_MOTIF_WM_HINTS"];

/// Handles ICCCM and EWMH atoms.
pub struct AtomManager {
    /// List of atoms.
    pub atoms: HashMap<&'static str, Atom>,
    conn: Rc<xcb::Connection>,
}

impl AtomManager {
    pub fn new(conn: &Rc<xcb::Connection>) -> Result<Self, xcb::GenericError> {
        let mut am = AtomManager {
            atoms: HashMap::with_capacity(ATOMS_LIST.len()),
            conn: conn.clone(),
        };

        let mut cookies: Vec<InternAtomCookie> = Vec::with_capacity(ATOMS_LIST.len());

        for atom_name in ATOMS_LIST {
            let cookie = xcb::intern_atom(&*am.conn, false, atom_name);
            cookies.push(cookie);
        }

        for i in 0..ATOMS_LIST.len() {
            let reply = try!(cookies[i].get_reply());
            am.atoms.insert(ATOMS_LIST[i], reply.atom());
        }

        Ok(am)
    }
}
