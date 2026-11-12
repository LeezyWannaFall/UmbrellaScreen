mod canvas;
mod capture;
mod editor;
mod layer;
use gtk::prelude::*;

fn main() -> gtk::glib::ExitCode {
    if std::env::args().skip(1).any(|a| a == "--help" || a == "-h") {
        println!(
            "UmbrellaScreen: выделите область мышью; Enter — сохранить и скопировать; Esc — закрыть."
        );
        return gtk::glib::ExitCode::SUCCESS;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(capture::screens());
    });
    let receiver = std::rc::Rc::new(std::cell::RefCell::new(Some(rx)));
    let app = gtk::Application::builder()
        .application_id("io.github.umbrellascreen.UmbrellaScreen")
        .flags(gtk::gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(move |app| {
        let hold = app.hold();
        let Some(rx) = receiver.borrow_mut().take() else {
            return;
        };
        let app = app.clone();
        let mut hold = Some(hold);
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(5), move || {
            let frames = match rx.try_recv() {
                Ok(Ok(frames)) => frames,
                Ok(Err(error)) => {
                    eprintln!("{error}");
                    app.quit();
                    hold.take();
                    return gtk::glib::ControlFlow::Break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    return gtk::glib::ControlFlow::Continue;
                }
                Err(_) => {
                    app.quit();
                    hold.take();
                    return gtk::glib::ControlFlow::Break;
                }
            };
            let display = gtk::gdk::Display::default().expect("Wayland display");
            let monitors = display.monitors();
            for (name, png) in frames {
                let monitor = (0..monitors.n_items())
                    .filter_map(|i| monitors.item(i).and_downcast::<gtk::gdk::Monitor>())
                    .find(|m| m.connector().as_deref() == Some(name.as_str()));
                let Some(monitor) = monitor else {
                    continue;
                };
                let window = gtk::ApplicationWindow::builder()
                    .application(&app)
                    .title("UmbrellaScreen Overlay")
                    .decorated(false)
                    .build();
                layer::setup(&window, &monitor);
                let editor = editor::Editor::new(&window);
                if let Err(error) = editor.load(&png) {
                    eprintln!("{error}");
                    continue;
                }
                window.set_child(Some(&editor.root));
                let a = app.downgrade();
                window.connect_close_request(move |_| {
                    if let Some(app) = a.upgrade() {
                        app.quit();
                    }
                    gtk::glib::Propagation::Proceed
                });

                window.present();
            }
            hold.take();
            gtk::glib::ControlFlow::Break
        });
    });
    app.run_with_args::<&str>(&[])
}
