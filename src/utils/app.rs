use eframe::App;
use futures::Future;

pub trait AsyncApp {
    type Fut: Future<>;
    fn update (&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) -> Fut;
}

impl<T: AsyncApp> App for T {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        todo!()
    }
}