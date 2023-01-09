use std::{task::Poll, ops::{RangeInclusive}};
use eframe::{egui::{Ui, RichText, Widget, Checkbox, DragValue, ComboBox, ScrollArea}, emath::Numeric};
use futures::{Stream, Future, StreamExt, FutureExt, TryStream, TryStreamExt, TryFuture, TryFutureExt};
use tokio::fs::{ReadDir, DirEntry};
pub mod list;

#[inline]
pub fn attribute_bool (ui: &mut Ui, key: impl Into<String>, value: &mut bool) {
    ui.horizontal(|ui| {
        Checkbox::new(value, RichText::new(key).strong()).ui(ui);
    });
}

#[inline]
pub fn attribute_num<Num: Numeric + ToString> (ui: &mut Ui, key: impl Into<String>, value: &mut Num, range: Option<RangeInclusive<Num>>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).strong());
        ui.horizontal(|ui| {
            ui.label(value.to_string());
            let mut drag = DragValue::new(value);
            if let Some(clamp_range) = range {
                drag = drag.clamp_range(clamp_range);
            }
            drag.ui(ui);
        });
    });
}

#[inline]
pub fn attribute (ui: &mut Ui, key: impl Into<String>, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).strong());
        ui.text_edit_singleline(value);
    });
}

#[inline]
pub fn attribute_combo<'a, I: IntoIterator<Item = &'a str>> (ui: &mut Ui, key: impl Into<String>, current: &mut &'a str, variants: I) {
    ComboBox::from_label(RichText::new(key).strong()).show_ui(ui, |ui| {
        for entry in variants.into_iter() {
            ui.selectable_value(current, entry, entry);
        }
    });
}

#[inline]
pub fn attribute_list<'a, I: IntoIterator<Item = &'a mut String>> (ui: &mut Ui, key: impl Into<String>, values: I) {    
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).strong());
        ui.vertical_centered(|ui| {
            for value in values.into_iter() {
                ui.text_edit_singleline(value);
            }
        });
    });
}

#[derive(Debug)]
#[repr(transparent)]
pub struct ReadDirStream {
    inner: ReadDir
}

impl ReadDirStream {
    #[inline]
    pub fn new (inner: ReadDir) -> Self {
        return Self { inner }
    }

    #[inline]
    pub fn into_inner (self) -> ReadDir {
        return self.inner
    }
}

impl Stream for ReadDirStream {
    type Item = std::io::Result<DirEntry>;

    #[inline]
    fn poll_next(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        self.inner.poll_next_entry(cx).map(std::io::Result::transpose)
    }
}

pin_project_lite::pin_project! {
    pub struct FlattenOkIter<St: TryStream> where St::Ok: IntoIterator {
        #[pin]
        stream: St,
        iter: Option<<St::Ok as IntoIterator>::IntoIter>
    }
}

impl<St: TryStream> FlattenOkIter<St> where St::Ok: IntoIterator {
    #[inline]
    pub fn new (stream: St) -> Self {
        return Self {
            stream,
            iter: None
        }
    }
}

impl<St: TryStream> Stream for FlattenOkIter<St> where St::Ok: IntoIterator {
    type Item = Result<<St::Ok as IntoIterator>::Item, St::Error>;

    #[inline]
    fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        let poll = match this.stream.try_poll_next(cx) {
            Poll::Ready(Some(Ok(iter))) => {
                let mut iter = iter.into_iter();
                if let Some(value) = iter.next() {
                    *this.iter = Some(iter);
                    return Poll::Ready(Some(Ok(value)))
                }
                Poll::Pending
            },
            Poll::Ready(Some(Err(e))) => return Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending
        };

        if let Some(iter) = this.iter {
            if let Some(value) = iter.next() {
                return Poll::Ready(Some(Ok(value)))
            }
        }

        return poll
    }
}

#[inline]
pub fn reduce<St, F, Fut> (st: St, mut f: F) -> impl Future<Output = Option<St::Item>> where
    St: Stream + Sized,
    F: FnMut(St::Item, St::Item) -> Fut,
    Fut: Future<Output = St::Item>
{
    return st.fold(None, move |lhs: Option<St::Item>, rhs: St::Item| {
        match lhs {
            Some(lhs) => futures::future::Either::Left(f(lhs, rhs).map(Some)),
            None => futures::future::Either::Right(futures::future::ready(Some(rhs)))
        }
    })
}

#[inline]
pub fn try_reduce<St, F, Fut> (st: St, mut f: F) -> impl TryFuture<Ok = Option<St::Ok>, Error = St::Error> where
    St: TryStream + Sized,
    F: FnMut(St::Ok, St::Ok) -> Fut,
    Fut: TryFuture<Ok = St::Ok, Error = St::Error>
{
    return st.try_fold(None, move |lhs: Option<St::Ok>, rhs: St::Ok| {
        match lhs {
            Some(lhs) => futures::future::Either::Left(f(lhs, rhs).map_ok(Some)),
            None => futures::future::Either::Right(futures::future::ready(Ok(Some(rhs))))
        }
    })
}

#[inline]
pub fn stream_and_then<St, F, T, E, U> (st: St, mut f: F) -> impl Stream<Item = Result<U, E>> where
    St: Stream<Item = Result<T, E>>,
    F: FnMut(T) -> Result<U, E>
{
    return st.map(move |x| match x {
        Ok(t) => f(t),
        Err(e) => Err(e),
    })
}