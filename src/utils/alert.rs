use std::{pin::Pin};
use eframe::egui::{Window, Context, InnerResponse};
use sis::self_referencing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlertType {
    Information,
    Warning,
    Error
}

impl AlertType {
    #[inline]
    pub fn as_str (&self) -> &'static str {
        match self {
            Self::Information => "Information",
            Self::Warning => "Warning",
            Self::Error => "Error"
        }
    }
}

#[self_referencing(extern)]
pub struct Alert {
    is_open: bool,
    message: String,
    #[borrows(mut is_open)]
    window: Window<'this>
}

#[macro_export]
macro_rules! show_alert {
    ($ctx:expr, $ty:expr, $title:expr, $message:expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            new_alert! {
                { true, into_string::IntoString::into_string($message) },
                { |open| eframe::egui::containers::Window::new(format!("{}: {}", $crate::utils::alert::AlertType::as_str($ty), $title)).open(core::pin::Pin::get_mut(open)) },
                __alert__
            }
            __alert__.show($ctx)
        }
    };
}

impl<'this> Alert<'this> {
    #[inline]
    pub unsafe fn show (self: Pin<&mut Self>, ctx: &Context) -> InnerResponse<Option<()>> {
        let message = core::ptr::read(&self.message);
        let window = core::ptr::read(self.window());

        return window.show(ctx, move |ui| {
            ui.label(message);
        }).unwrap();
    }
}