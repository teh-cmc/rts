use raylib::prelude::*;
use specs::{prelude::*, WorldExt};
use std::{cell::UnsafeCell, sync::Arc};

// -----------------------------------------------------------------------------

// TODO(cmc): explain
pub struct Raylib(Arc<UnsafeCell<RaylibHandle>>);
unsafe impl Send for Raylib {}
unsafe impl Sync for Raylib {}

impl Raylib {
    pub fn new(rl: RaylibHandle) -> Self {
        Self(Arc::new(UnsafeCell::new(rl)))
    }

    pub fn read<T>(&self, mut f: impl FnMut(&RaylibHandle) -> T) -> T {
        unsafe { f(self.0.get().as_ref().unwrap()) }
    }

    pub fn write<T>(&mut self, mut f: impl FnMut(&mut RaylibHandle) -> T) -> T {
        unsafe { f(self.0.get().as_mut().unwrap()) }
    }

    pub fn draw(&mut self, tl: &RaylibThread, mut f: impl FnMut(&mut RaylibDrawHandle)) {
        let this = unsafe { self.0.get().as_mut().unwrap() };
        let mut d = this.begin_drawing(tl);
        f(&mut d)
    }
}

impl Clone for Raylib {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
