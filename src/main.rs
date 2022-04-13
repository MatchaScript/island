#![windows_subsystem = "windows"]

use {
    window_vibrancy::*,
    windows::{
        core::{Interface, IntoParam, Result},
        Win32::{
            Foundation::HWND,
            System::WinRT::Xaml::IDesktopWindowXamlSourceNative,
            UI::WindowsAndMessaging::{SetWindowPos, SWP_SHOWWINDOW},
        },
        UI::Xaml::{
            Controls::{Button, Page, ScrollViewer, StackPanel},
            Hosting::DesktopWindowXamlSource,
            Markup::XamlReader,
            UIElement,
        },
    },
    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        platform::windows::WindowExtWindows,
        window::{Window, WindowBuilder},
    },
};

fn main() -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_transparent(true)
        .with_decorations(false)
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let island = XamlIsland::attached(&window)?;
    const MAIN_XAML: &'static str = include_str!("../res/main.xaml");
    let main_page: Page = XamlReader::Load(MAIN_XAML).unwrap().cast().unwrap();
    island.set_content(&main_page)?;
    apply_mica(&window).expect("Unsupported platform! 'apply_blur' is only supported on Windows");
    window.set_visible(true);
    window.set_decorations(true);
    window.request_redraw();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    island.resize(new_size.width as _, new_size.height as _)
                }
                _ => (),
            },
            _ => (),
        }
    });
}

pub struct XamlIsland {
    hwnd: HWND,
    source: DesktopWindowXamlSource,
}
impl XamlIsland {
    pub fn attached(window: &Window) -> Result<Self> {
        let source = DesktopWindowXamlSource::new()?;
        let interop: IDesktopWindowXamlSourceNative = source.cast()?;
        unsafe {
            interop.AttachToWindow(HWND(window.hwnd() as _))?;
        }
        let hwnd = unsafe { interop.WindowHandle() }?;
        let size = window.inner_size();

        let island = XamlIsland { hwnd, source };
        island.resize(size.width as _, size.height as _);

        Ok(island)
    }
    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            SetWindowPos(
                self.hwnd,
                HWND::default(),
                0,
                0,
                width,
                height,
                SWP_SHOWWINDOW,
            )
        };
    }
    pub fn set_content<'a>(&self, value: impl IntoParam<'a, UIElement>) -> Result<()> {
        self.source.SetContent(value)
    }
}
