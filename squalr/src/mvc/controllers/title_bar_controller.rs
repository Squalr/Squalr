use crate::callback::Callback;
use slint::{PhysicalSize, Window};
use std::rc::Rc;

#[derive(Clone)]
pub struct TitleBarController {
    minimize_callback: Rc<Callback<(), ()>>,
    maximize_callback: Rc<Callback<(), ()>>,
    close_callback: Rc<Callback<(), ()>>,
}

impl TitleBarController {
    pub fn new() -> Self {
        Self {
            minimize_callback: Rc::new(Callback::default()),
            maximize_callback: Rc::new(Callback::default()),
            close_callback: Rc::new(Callback::default()),
        }
    }

    pub fn minimize(&self) {
        self.minimize_callback.invoke(&());
    }

    pub fn on_minimize(
        &self,
        mut callback: impl FnMut() + 'static,
    ) {
        self.minimize_callback.on(move |()| {
            callback();
        });
    }

    pub fn maximize(&self) {
        self.maximize_callback.invoke(&());
    }

    pub fn on_maximize(
        &self,
        mut callback: impl FnMut() + 'static,
    ) {
        self.maximize_callback.on(move |()| {
            callback();
        });
    }

    pub fn close(&self) {
        self.close_callback.invoke(&());
    }

    pub fn on_close(
        &self,
        mut callback: impl FnMut() + 'static,
    ) {
        self.close_callback.on(move |()| {
            callback();
        });
    }
}
