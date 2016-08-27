use std::collections::HashMap;
use std::cell::RefCell;
use xcb;

macro_rules! atoms {
    ( $( $x:ident ),* ) => {
        #[allow(non_snake_case)]
        $(pub const $x: &'static str = stringify!($x);)*
    }
}

atoms!(
    _NET_SYSTEM_TRAY_S0,
    _NET_SYSTEM_TRAY_ORIENTATION,
    _NET_WM_WINDOW_TYPE,
    _NET_WM_WINDOW_TYPE_DOCK,
    MANAGER
);

pub struct Atoms<'a> {
    conn: &'a xcb::Connection,
    cache: RefCell<HashMap<String, xcb::Atom>>
}

impl<'a> Atoms<'a> {
    pub fn new(conn: &xcb::Connection) -> Atoms {
        Atoms {
            conn: conn,
            cache: RefCell::new(HashMap::new())
        }
    }

    pub fn get(&self, name: &str) -> xcb::Atom {
        let mut cache = self.cache.borrow_mut();
        if cache.contains_key(name) {
            *cache.get(name).unwrap()
        }
        else {
            let atom = xcb::intern_atom(self.conn, false, name).get_reply().unwrap().atom();
            cache.insert(name.to_string(), atom);
            atom
        }
    }
}
