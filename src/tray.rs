use atom;
use xcb;

pub struct Tray<'a> {
    conn: &'a xcb::Connection,
    atoms: &'a atom::Atoms<'a>,
    screen: usize,
    icon_size: u16,
    window: xcb::Window,
    children: Vec<xcb::Window>
}

impl<'a> Tray<'a> {
    pub fn new<'b>(conn: &'b xcb::Connection, atoms: &'b atom::Atoms, screen: usize, icon_size: u16) -> Tray<'b> {
        Tray::<'b> {
            conn: conn,
            atoms: atoms,
            screen: screen,
            icon_size: icon_size,
            window: conn.generate_id(),
            children: vec![]
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
            self.icon_size, self.icon_size,
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

    pub fn adopt(&mut self, window: xcb::Window) {
        let offset = (self.children.len() as u16 * self.icon_size) as i16;
        xcb::change_window_attributes(self.conn, window, &[
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_STRUCTURE_NOTIFY)
        ]);
        xcb::reparent_window(self.conn, window, self.window, offset, 0);
        xcb::map_window(self.conn, window);
        self.conn.flush();
        self.children.push(window);
        self.resize();
    }

    pub fn forget(&mut self, window: xcb::Window) {
        self.children.retain(|child| *child != window);
        self.resize();
    }

    pub fn force_size(&self, window: xcb::Window) {
        xcb::configure_window(self.conn, window, &[
            (xcb::CONFIG_WINDOW_WIDTH as u16, self.icon_size as u32),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, self.icon_size as u32)
        ]);
        self.conn.flush();
    }

    pub fn resize(&self) {
        let len = self.children.len() as u16;
        if len > 0 {
            xcb::configure_window(self.conn, self.window, &[
                (xcb::CONFIG_WINDOW_WIDTH as u16, (len * self.icon_size) as u32)
            ]);
            xcb::map_window(self.conn, self.window);
        }
        else {
            xcb::unmap_window(self.conn, self.window);
        }
        self.conn.flush();
    }

    pub fn cleanup(&self) {
        let setup = self.conn.get_setup();
        let screen = setup.roots().nth(self.screen).unwrap();
        let root = screen.root();

        for child in self.children.iter() {
            xcb::unmap_window(self.conn, *child);
            xcb::reparent_window(self.conn, *child, root, 0, 0);
        }
        self.conn.flush();
    }
}
