use std::{ops::{Deref, DerefMut}, pin::Pin};
use eframe::egui::WidgetText;
use sis::self_referencing;

#[self_referencing(extern)]
pub struct Window {
    open: bool,
    #[borrows(mut open)]
    inner: eframe::egui::containers::Window<'this>
}

#[macro_export]
macro_rules! create_window {
    ($title:expr, $open:expr => $($t:tt)*) => {
        new_window! {
            { $open },
            {
                |open| eframe::egui::containers::Window::new($title).open(std::pin::Pin::into_inner(open))
            },
            $($t)*
        }
    };
}

impl<'this> Window<'this> {
    #[inline]
    pub unsafe fn initialize (self: Pin<&'this mut Self>, title: impl Into<WidgetText>) {
        self._initialize(|open| {
            return eframe::egui::containers::Window::new(title)
                .open(Pin::into_inner(open))
        });
    }

    #[inline]
    pub fn is_open (&self) -> bool {
        return self.open
    }

    #[inline]
    pub fn open (&mut self) {
        self.open = true
    }

    #[inline]
    pub fn close (&mut self) {
        self.open = false
    }
}

impl<'this> Deref for Window<'this> {
    type Target = eframe::egui::containers::Window<'this>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        return self.inner()
    }
}

impl<'this> DerefMut for Window<'this> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        return self.inner_mut()
    }
}