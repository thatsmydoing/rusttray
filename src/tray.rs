use atom;
use xcb;

pub struct Tray<'a> {
    conn: &'a xcb::Connection,
    atoms: &'a atom::Atoms<'a>,
    screen: usize,
    window: xcb::Window
}

impl<'a> Tray<'a> {
    pub fn new<'b>(conn: &'b xcb::Connection, atoms: &'b atom::Atoms, screen: usize) -> Tray<'b> {
        Tray::<'b> {
            conn: conn,
            atoms: atoms,
            screen: screen,
            window: conn.generate_id()
        }
    }

    pub fn create(&self) {
        let setup = self.conn.get_setup();
        let screen = setup.roots().nth(self.screen).unwrap();

        xcb::create_window(
            &self.conn,
            xcb::COPY_FROM_PARENT as u8,
            self.window,
            screen.root(),
            0, 0,
            20, 20,
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            screen.root_visual(),
            &[
                (xcb::CW_BACK_PIXEL, screen.black_pixel()),
                (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE)
            ]
        );
    }

    pub fn take_selection(&self) -> bool {
        let selection = self.atoms.get(atom::_NET_SYSTEM_TRAY_S0);
        xcb::set_selection_owner(self.conn, self.window, selection, xcb::CURRENT_TIME);
        let owner = xcb::get_selection_owner(self.conn, selection).get_reply().unwrap().owner();
        owner == self.window
    }
}
