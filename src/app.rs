use iced::window;
use iced::{Element, Event, Subscription, Task, Theme};
use porter_threads::initialize_thread_pool;

use crate::Controller;
use crate::MainWindow;
use crate::Message;
use crate::windows::MainMessage;
use crate::{AppState, system};

/// Entry point for the iced application.
pub struct App {
    state: AppState,
    main_window: MainWindow,
}

impl App {
    /// Constructs a new app entry point.
    pub fn new(state: AppState) -> (Self, Task<Message>) {
        let (main_window, main_window_task) = MainWindow::create();

        let task = Task::batch([
            main_window_task.discard(),
            crate::fonts::load().map(|result| Message::Main(MainMessage::FontLoaded(result))),
            Task::perform(load_settings(), Message::SettingsLoaded),
        ]);

        let ui = Self { state, main_window };

        (ui, task)
    }

    /// Provides the title for the given window.
    pub fn title(&self, id: window::Id) -> String {
        if id == self.main_window.id {
            self.main_window.title(&self.state)
        } else {
            String::from("<unset>")
        }
    }

    /// Handles updating the app state.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        use Message::*;

        match message {
            Noop => self.on_noop(),
            UI(event, id) => self.on_ui(event, id),
            WindowOpened(id) => self.on_window_opened(id),
            Controller(controller) => self.on_controller(controller),
            Main(message) => self.main_window.update(&mut self.state, message),
            ThemeChanged(theme) => self.on_theme_changed(theme),
            SettingsLoaded(settings) => self.on_settings_loaded(settings),
            SettingsSaved => self.on_settings_saved(),
        }
    }

    /// Custom theme.
    pub fn theme(&self, _: window::Id) -> Theme {
        self.state.settings.theme.to_iced_theme()
    }

    /// Handles global and controller events.
    pub fn subscription(&self) -> Subscription<Message> {
        use iced::event;
        use iced::event::Status;
        use iced::stream;

        use iced::futures::SinkExt;
        use iced::futures::StreamExt;
        use iced::futures::channel::mpsc;

        /// Filters out events that aren't necessary for the global listener.
        #[inline(always)]
        fn filter_event(event: Event, id: window::Id) -> Option<Message> {
            if matches!(event, Event::Keyboard(_))
                || matches!(event, Event::Window(window::Event::Closed))
                || matches!(event, Event::Window(window::Event::Opened { .. }))
                || matches!(event, Event::Window(window::Event::FileDropped(_)))
                || matches!(event, Event::Window(window::Event::FileHovered(_)))
                || matches!(event, Event::Window(window::Event::FilesHoveredLeft))
            {
                return Some(Message::UI(event, id));
            }

            None
        }

        let events = event::listen_with(|event, status, id| match status {
            Status::Ignored => filter_event(event, id),
            Status::Captured => None,
        });

        let controller = Subscription::run(|| {
            stream::channel(100, |mut output: mpsc::Sender<Message>| async move {
                let (tx, mut rx) = mpsc::unbounded::<Message>();

                output
                    .send(Message::Controller(Controller::with_channel(tx)))
                    .await
                    .expect("Failed to initialize controller!");

                loop {
                    while let Some(message) = rx.next().await {
                        let result = output.send(message).await;
                        debug_assert!(result.is_ok());
                    }
                }
            })
        });

        Subscription::batch([events, controller])
    }

    /// Handles rendering a given window.
    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        if id == self.main_window.id {
            self.main_window.view(&self.state)
        } else {
            iced::widget::text("<unset>").into()
        }
    }

    /// Occurs when nothing should happen.
    fn on_noop(&mut self) -> Task<Message> {
        Task::none()
    }

    /// Occurs when a ui event has triggered for a given window.
    fn on_ui(&mut self, event: Event, id: window::Id) -> Task<Message> {
        if id == self.main_window.id {
            self.main_window
                .update(&mut self.state, MainMessage::UI(event))
        } else {
            Task::none()
        }
    }

    /// Occurs when a window opens.
    fn on_window_opened(&mut self, id: window::Id) -> Task<Message> {
        if id == self.main_window.id {
            Task::done(Message::Main(MainMessage::Show))
        } else {
            Task::none()
        }
    }

    /// Occurs when the global controller is initialized.
    fn on_controller(&mut self, controller: Controller) -> Task<Message> {
        self.state.controller = controller;
        Task::none()
    }

    /// Occurs when the theme is changed.
    fn on_theme_changed(&mut self, theme: crate::theme::AppTheme) -> Task<Message> {
        self.state.settings.theme = theme;
        let settings = self.state.settings.clone();
        Task::perform(save_settings(settings), |_| Message::SettingsSaved)
    }

    /// Occurs when settings have been loaded.
    fn on_settings_loaded(&mut self, settings: crate::Settings) -> Task<Message> {
        self.state.settings = settings;
        tracing::info!("Settings loaded successfully");
        Task::none()
    }

    /// Occurs when settings have been saved.
    fn on_settings_saved(&mut self) -> Task<Message> {
        tracing::debug!("Settings saved");
        Task::none()
    }

    /// Launches the application.
    pub fn launch() -> iced::Result {
        crate::panic_hook::install("image_baker", env!("CARGO_PKG_VERSION"));
        crate::logger::init_logging();

        // Initialize global rayon thread pool.
        initialize_thread_pool();

        // Initialize system specific workarounds.
        system::initialize_workarounds();

        let state = AppState::new();

        iced::daemon(App::title, App::update, App::view)
            .subscription(App::subscription)
            .theme(App::theme)
            .font(include_bytes!("./fonts/JetBrainsMonoNL-Regular.ttf"))
            .default_font(crate::fonts::JETBRAINS_MONO)
            .run_with(move || App::new(state))
    }
}

async fn load_settings() -> crate::Settings {
    crate::Settings::load()
}

async fn save_settings(settings: crate::Settings) {
    settings.save();
}
