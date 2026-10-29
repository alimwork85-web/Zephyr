//! A small native window for picking a city. It runs on its own thread with its
//! own message loop so it never blocks the tray, and reports the choice through
//! the callback given to `open`.

use crate::config::City;
use crate::weather;
use native_windows_gui as nwg;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

static OPEN: AtomicBool = AtomicBool::new(false);

pub fn open(
    rt: tokio::runtime::Handle,
    client: reqwest::Client,
    on_select: impl Fn(City) + Send + 'static,
) {
    if OPEN.swap(true, Ordering::SeqCst) {
        return; // already showing
    }
    std::thread::spawn(move || {
        let _ = run(rt, client, Box::new(on_select));
        OPEN.store(false, Ordering::SeqCst);
    });
}

struct App {
    window: nwg::Window,
    input: nwg::TextInput,
    search: nwg::Button,
    list: nwg::ListBox<String>,
    status: nwg::Label,
    choose: nwg::Button,
    notice: nwg::Notice,
    found: Arc<Mutex<Option<Result<Vec<City>, String>>>>,
    results: RefCell<Vec<City>>,
    rt: tokio::runtime::Handle,
    client: reqwest::Client,
    on_select: Box<dyn Fn(City)>,
}

impl App {
    fn do_search(&self) {
        let query = self.input.text();
        let query = query.trim().to_string();
        if query.chars().count() < 2 {
            self.status.set_text("Type at least 2 letters.");
            return;
        }
        self.search.set_enabled(false);
        self.status.set_text("Searching...");
        // Search off the UI thread so the window keeps responding; the notice
        // wakes the message loop when the result lands.
        let (slot, sender, client) = (
            Arc::clone(&self.found),
            self.notice.sender(),
            self.client.clone(),
        );
        self.rt.spawn(async move {
            let cities = weather::search_cities(&client, &query).await;
            *slot.lock().unwrap_or_else(|e| e.into_inner()) = Some(cities);
            sender.notice();
        });
    }

    fn show_found(&self) {
        let Some(found) = self.found.lock().unwrap_or_else(|e| e.into_inner()).take() else {
            return;
        };
        self.search.set_enabled(true);
        match found {
            Ok(cities) if cities.is_empty() => {
                self.results.borrow_mut().clear();
                self.list.set_collection(vec![]);
                self.status.set_text("No city found. Try another spelling.");
            }
            Ok(cities) => {
                self.list.set_collection(cities.iter().map(City::label).collect());
                self.list.set_selection(Some(0));
                self.status.set_text("Select a city, then press Use this city.");
                *self.results.borrow_mut() = cities;
            }
            Err(_) => self.status.set_text("Search failed. Check your internet connection."),
        }
    }

    fn do_choose(&self) {
        let picked = self
            .list
            .selection()
            .and_then(|i| self.results.borrow().get(i).cloned());
        match picked {
            Some(city) => {
                (self.on_select)(city);
                nwg::stop_thread_dispatch();
            }
            None => self.status.set_text("Search and select a city first."),
        }
    }
}

fn run(
    rt: tokio::runtime::Handle,
    client: reqwest::Client,
    on_select: Box<dyn Fn(City)>,
) -> Result<(), nwg::NwgError> {
    nwg::init()?;

    let mut window = nwg::Window::default();
    let mut input = nwg::TextInput::default();
    let mut search = nwg::Button::default();
    let mut list = nwg::ListBox::<String>::default();
    let mut status = nwg::Label::default();
    let mut choose = nwg::Button::default();
    let mut notice = nwg::Notice::default();

    nwg::Window::builder()
        .size((370, 310))
        .position((400, 250))
        .title("Zephyr - Change city")
        .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
        .build(&mut window)?;
    nwg::TextInput::builder()
        .parent(&window)
        .position((10, 10))
        .size((270, 25))
        .placeholder_text(Some("City name, e.g. London"))
        .focus(true)
        .build(&mut input)?;
    nwg::Button::builder()
        .parent(&window)
        .text("Search")
        .position((290, 10))
        .size((70, 25))
        .build(&mut search)?;
    nwg::ListBox::builder()
        .parent(&window)
        .position((10, 45))
        .size((350, 170))
        .build(&mut list)?;
    nwg::Label::builder()
        .parent(&window)
        .text("Type a city name and press Search.")
        .position((10, 225))
        .size((350, 20))
        .build(&mut status)?;
    nwg::Button::builder()
        .parent(&window)
        .text("Use this city")
        .position((10, 255))
        .size((350, 32))
        .build(&mut choose)?;
    nwg::Notice::builder().parent(&window).build(&mut notice)?;

    let app = Rc::new(App {
        window,
        input,
        search,
        list,
        status,
        choose,
        notice,
        found: Arc::new(Mutex::new(None)),
        results: RefCell::new(Vec::new()),
        rt,
        client,
        on_select,
    });

    let handler_app = Rc::clone(&app);
    let handler = nwg::full_bind_event_handler(&app.window.handle, move |evt, _data, handle| {
        use nwg::Event as E;
        let a = &handler_app;
        match evt {
            E::OnWindowClose if handle == a.window.handle => nwg::stop_thread_dispatch(),
            E::OnButtonClick if handle == a.search.handle => a.do_search(),
            E::OnNotice if handle == a.notice.handle => a.show_found(),
            E::OnButtonClick if handle == a.choose.handle => a.do_choose(),
            E::OnListBoxDoubleClick if handle == a.list.handle => a.do_choose(),
            _ => {}
        }
    });

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
    Ok(())
}
