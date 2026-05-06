pub struct Ui;

impl Ui {
    pub fn new() -> Self {
        Ui
    }

    pub fn open_window(html: &str) -> Result<(), String> {
        use tao::event_loop::EventLoop;
        use tao::window::WindowBuilder;

        let event_loop = EventLoop::new();
        let _window = match WindowBuilder::new()
            .with_title("E — UI")
            .build(&event_loop)
        {
            Ok(w) => w,
            Err(e) => return Err(format!("window: {}", e)),
        };

        // For now, just print that the UI was prepared
        // Full webview integration needs platform-specific setup
        println!("  🖥️ UI window ready (close to continue)");

        event_loop.run(move |_event, _, control_flow| {
            *control_flow = tao::event_loop::ControlFlow::Wait;
            if let tao::event::Event::WindowEvent { event, .. } = _event {
                use tao::event::WindowEvent;
                if let WindowEvent::CloseRequested = event {
                    *control_flow = tao::event_loop::ControlFlow::Exit;
                }
            }
        });
        Ok(())
    }
}
