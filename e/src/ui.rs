use tao::event_loop::EventLoop;
use tao::window::WindowBuilder;
use tao::platform::unix::WindowExtUnix;
use wry::WebViewBuilderExtUnix;
use wry::WebViewBuilder;

pub struct Ui;

impl Ui {
    pub fn new() -> Self { Ui }

    pub fn open_window(html: &str) -> Result<(), String> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("E — UI")
            .build(&event_loop)
            .map_err(|e| format!("window: {}", e))?;

        // Linux: use GTK-specific API for Wayland/X11 support
        // macOS/Windows: use raw window handle
        #[cfg(target_os = "linux")]
        let builder = WebViewBuilder::new_gtk(window.gtk_window());
        #[cfg(not(target_os = "linux"))]
        let builder = WebViewBuilder::new(&window);

        let _webview = builder
            .with_html(html.to_string())
            .build()
            .map_err(|e| format!("webview: {:?}", e))?;

        println!("  ✅ WebView created (close window to continue)");

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
