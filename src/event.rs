use chan;
use xcb;

pub enum Event {
    ChildRequest(xcb::Window),
    ChildDestroyed(xcb::Window)
}

const CLIENT_MESSAGE: u8 = xcb::CLIENT_MESSAGE | 0x80;

pub fn event_loop(conn: &xcb::Connection, tx: chan::Sender<Event>) {
    loop {
        match conn.wait_for_event() {
            Some(event) => match event.response_type() {
                xcb::EXPOSE => { println!("expose") },
                CLIENT_MESSAGE => {
                    let event: &xcb::ClientMessageEvent = xcb::cast_event(&event);
                    let data = event.data().data32();
                    let window = data[2];
                    tx.send(Event::ChildRequest(window));
                },
                xcb::DESTROY_NOTIFY => {
                    let event: &xcb::DestroyNotifyEvent = xcb::cast_event(&event);
                    tx.send(Event::ChildDestroyed(event.window()));
                },
                _ => {}
            },
            None => { break; }
        }
    }
}
