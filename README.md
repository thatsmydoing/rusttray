# rustray

A rudimentary system tray implemented in Rust.

This is more of an experiment than something you should use on a daily basis. I
primarily wrote it to understand how the system tray mechanics work in X. This
doesn't actually implement all of the system tray specification but it works for
most of the programs I use.

rustray only implements XEMBED style icons. The tray icons themselves perform
the drawing and the tray only manages their sizes and positions. It does not
draw icons by itself. In addition, balloon messages are not handled as well.

## How it works

When starting,

1. Create a window
2. [Acquire a selection][1] to [`_NET_SYSTEM_TRAY_S0`][2]
3. Announce arrival as [manager][3]
4. Receive client messages to request docking
5. Reparent tray icon windows into our window
6. Map the tray icon windows

When exiting,

1. Unmap tray icon windows
2. Reparent tray icon windows back to screen root
3. [Release the selection][4]

It is important that the tray waits for the reparenting back to root to actually
finish. If the tray exits before the tray windows are reparented, it will cause
those applications to crash.

[1]: https://tronche.com/gui/x/icccm/sec-2.html#s-2.1
[2]: https://specifications.freedesktop.org/systemtray-spec/systemtray-spec-latest.html#locating
[3]: https://tronche.com/gui/x/icccm/sec-2.html#s-2.8
[4]: https://tronche.com/gui/x/icccm/sec-2.html#s-2.3
