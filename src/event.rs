use chan;
use xcb;

pub enum Event {
    Ready(xcb::Timestamp),
    ChildRequest(xcb::Window),
    ChildDestroyed(xcb::Window),
    ChildConfigured(xcb::Window)
}

const CLIENT_MESSAGE: u8 = xcb::CLIENT_MESSAGE | 0x80;

pub fn event_loop(conn: &xcb::Connection, tx: chan::Sender<Event>) {
    let mut ready = false;
    loop {
        match conn.wait_for_event() {
            Some(event) => match event.response_type() {
                xcb::PROPERTY_NOTIFY if !ready => {
                    ready = true;
                    let event: &xcb::PropertyNotifyEvent = xcb::cast_event(&event);
                    tx.send(Event::Ready(event.time()));
                },
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
                xcb::CONFIGURE_NOTIFY => {
                    let event: &xcb::ConfigureNotifyEvent = xcb::cast_event(&event);
                    tx.send(Event::ChildConfigured(event.window()));
                },
                _ => {}
            },
            None => { break; }
        }
    }
}
