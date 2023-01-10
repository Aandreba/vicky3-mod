use std::{cell::{UnsafeCell, Cell}, ops::{Deref, DerefMut}, fmt::Debug};

const CLEAR: u8 = 0;
const READ: u8 = 1;
const WRITE: u8 = 2;

pub struct RefCell<T> {
    #[cfg(debug_assertions)]
    state: Cell<u8>,
    inner: UnsafeCell<T>
}

impl<T> RefCell<T> {
    #[inline]
    pub const fn new (t: T) -> Self {
        return Self {
            #[cfg(debug_assertions)]
            state: Cell::new(CLEAR),
            inner: UnsafeCell::new(t)
        }
    }

    #[inline]
    pub fn borrow (&self) -> Ref<'_, T> {
        #[cfg(debug_assertions)]
        match self.state.get() {
            WRITE => panic!("The value is currently mutably borrowed"),
            _ => self.state.set(READ)
        }
        return Ref { inner: self }
    }

    #[inline]
    pub fn borrow_mut (&self) -> RefMut<'_, T> {
        #[cfg(debug_assertions)]
        match self.state.get() {
            READ => self.state.set(WRITE),
            _ => panic!("The value cannot be mutably borrowed currently")
        }
        return RefMut { inner: self }
    }
}

impl<T: Debug> Debug for RefCell<T> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(debug_assertions)]
        if self.state.get() & WRITE != 0 {
            return write!(f, "RefCell {{ BorrowedMut }}");
        }
        return f.debug_struct("RefCell")
                .field("inner", unsafe { &*self.inner.get() })
                .finish()
    }
}

unsafe impl<T: Send> Send for RefCell<T> {}

#[repr(transparent)]
pub struct Ref<'a, T> {
    inner: &'a RefCell<T>
}

impl<'a, T> Ref<'a, T> {
    #[inline]
    pub fn parent (&self) -> &RefCell<T> {
        return self.inner
    }
}

impl<'a, T> Deref for Ref<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.inner.get() }
    }
}

#[repr(transparent)]
pub struct RefMut<'a, T> {
    inner: &'a RefCell<T>
}

impl<'a, T> RefMut<'a, T> {
    #[inline]
    pub fn parent (&self) -> &RefCell<T> {
        return self.inner
    }
}

impl<'a, T> Deref for RefMut<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.inner.get() }
    }
}

impl<'a, T> DerefMut for RefMut<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner.inner.get() }
    }
}

impl<'a, T> Drop for RefMut<'a, T> {
    #[inline]
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        self.inner.state.set(CLEAR);
    }
}