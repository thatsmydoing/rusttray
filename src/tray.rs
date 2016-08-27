use atom;
use xcb;

pub enum HorizontalAlign {
    Left,
    Right
}

pub enum VerticalAlign {
    Top,
    Bottom
}

pub type Position = (VerticalAlign, HorizontalAlign);

pub const TOP_LEFT: Position = (VerticalAlign::Top, HorizontalAlign::Left);
pub const TOP_RIGHT: Position = (VerticalAlign::Top, HorizontalAlign::Right);
pub const BOTTOM_LEFT: Position = (VerticalAlign::Bottom, HorizontalAlign::Left);
pub const BOTTOM_RIGHT: Position = (VerticalAlign::Bottom, HorizontalAlign::Right);

pub struct Tray<'a> {
    conn: &'a xcb::Connection,
    atoms: &'a atom::Atoms<'a>,
    screen: usize,
    icon_size: u16,
    position: Position,
    window: xcb::Window,
    children: Vec<xcb::Window>,
    timestamp: xcb::Timestamp
}

impl<'a> Tray<'a> {
    pub fn new<'b>(
        conn: &'b xcb::Connection,
        atoms: &'b atom::Atoms,
        screen: usize,
        icon_size: u16,
        position: Position
    ) -> Tray<'b> {
        Tray::<'b> {
            conn: conn,
            atoms: atoms,
            screen: screen,
            icon_size: icon_size,
            position: position,
            window: conn.generate_id(),
            children: vec![],
            timestamp: 0
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
                (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_PROPERTY_CHANGE)
            ]
        );
        xcb::change_property(
            self.conn,
            xcb::PROP_MODE_REPLACE as u8,
            self.window,
            self.atoms.get(atom::_NET_WM_WINDOW_TYPE),
            xcb::ATOM_ATOM,
            32,
            &[self.atoms.get(atom::_NET_WM_WINDOW_TYPE_DOCK)]
        );
        self.conn.flush();
    }

    pub fn take_selection(&mut self, timestamp: xcb::Timestamp) -> bool {
        let selection = self.atoms.get(atom::_NET_SYSTEM_TRAY_S0);
        xcb::set_selection_owner(self.conn, self.window, selection, timestamp);
        let owner = xcb::get_selection_owner(self.conn, selection).get_reply().unwrap().owner();
        let ok = owner == self.window;
        if ok {
            self.timestamp = timestamp;
            let setup = self.conn.get_setup();
            let screen = setup.roots().nth(self.screen).unwrap();

            let client_event = xcb::ClientMessageEvent::new(
                32, // 32 bits (refers to data)
                screen.root(),
                self.atoms.get(atom::MANAGER),
                xcb::ClientMessageData::from_data32([timestamp, selection, self.window, 0, 0])
            );
            xcb::send_event(self.conn, false, screen.root(), xcb::EVENT_MASK_STRUCTURE_NOTIFY, &client_event);
            self.conn.flush();
        }
        ok
    }

    pub fn adopt(&mut self, window: xcb::Window) {
        let offset = (self.children.len() as u16 * self.icon_size) as i16;
        xcb::change_window_attributes(self.conn, window, &[
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_STRUCTURE_NOTIFY)
        ]);
        xcb::reparent_window(self.conn, window, self.window, offset, 0);
        xcb::map_window(self.conn, window);
        self.force_size(window);
        self.conn.flush();
        self.children.push(window);
        self.reposition();
    }

    pub fn forget(&mut self, window: xcb::Window) {
        self.children.retain(|child| *child != window);
        self.reposition();
    }

    pub fn force_size(&self, window: xcb::Window) {
        xcb::configure_window(self.conn, window, &[
            (xcb::CONFIG_WINDOW_WIDTH as u16, self.icon_size as u32),
            (xcb::CONFIG_WINDOW_HEIGHT as u16, self.icon_size as u32)
        ]);
        self.conn.flush();
    }

    pub fn reposition(&self) {
        let width = self.children.len() as u16 * self.icon_size;
        if width > 0 {
            let setup = self.conn.get_setup();
            let screen = setup.roots().nth(self.screen).unwrap();

            let (ref valign, ref halign) = self.position;
            let y = match valign {
                &VerticalAlign::Top => 0,
                &VerticalAlign::Bottom => screen.height_in_pixels() - self.icon_size
            };
            let x = match halign {
                &HorizontalAlign::Left => 0,
                &HorizontalAlign::Right => screen.width_in_pixels() - width
            };
            xcb::configure_window(self.conn, self.window, &[
                (xcb::CONFIG_WINDOW_X as u16, x as u32),
                (xcb::CONFIG_WINDOW_Y as u16, y as u32),
                (xcb::CONFIG_WINDOW_WIDTH as u16, width as u32)
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

        let selection = self.atoms.get(atom::_NET_SYSTEM_TRAY_S0);
        xcb::set_selection_owner(self.conn, xcb::NONE, selection, self.timestamp);
        self.conn.flush();
    }
}
