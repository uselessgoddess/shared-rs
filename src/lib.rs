#![feature(test)]
extern crate test;

use std::cell::RefCell;
use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use std::fmt;

type SharedData<T> = Rc<RefCell<T>>;

pub struct Shared<T: ?Sized> {
    data: SharedData<T>
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Shared { data: SharedData::new(RefCell::new(value)) }
    }

    pub fn use_count(&self) -> usize {
        Rc::strong_count(&self.data)
    }
}

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Shared::new(value)
    }
}

impl<T: ?Sized> From<SharedData<T>> for Shared<T> {
    fn from(data: SharedData<T>) -> Self {
        Shared { data }
    }
}

impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        From::from(self.data.clone())
    }
}

impl<T: Default> Default for Shared<T> {
    fn default() -> Self {
        Shared::new(Default::default())
    }
}

impl<T: ?Sized> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.as_ptr() }
    }
}

impl<T: ?Sized> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data.as_ptr() }
    }
}

impl<T: ?Sized> AsRef<T> for Shared<T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}

impl<T: ?Sized> AsMut<T> for Shared<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut **self
    }
}

impl<T: ?Sized> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
         self.as_ref() as *const T == other.as_ref() as *const T
    }
}

impl<T> Eq for Shared<T> { }

impl<T: ?Sized + fmt::Debug> fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(format!("Shared {{ data: {:?} }}", self.data).as_str())
    }
}

#[macro_export]
macro_rules! shared {
    ($value:expr) => {
        Shared::new($value)
    };
    ($($values:expr),+ $(,)?) => {
        Shared::<&[i32]>::new(&[$($values),+]);
    };
}

#[cfg(test)]
mod tests {
    use crate::Shared;
    use std::any::Any;
    use test::Bencher;

    #[test]
    fn it_works() {
        let shared = Shared::new(12);

        assert_eq!(*shared, 12);
    }

    #[test]
    fn macro_works() {
        let a = Shared::new(12);
        let b = shared!(12);

        assert_eq!(a.type_id(), b.type_id());
        assert_eq!(*a, *b);
    }

    #[test]
    fn use_count() {
        let a = shared!(12);
        let b = a.clone();

        assert_eq!(a.use_count(), b.use_count());
    }

    #[test]
    fn storage_arrays() {
        let x = Shared::<&[i32]>::new(&[1, 2, 3]);
        let y = shared!{1, 2, 3};
        assert_eq!(x.type_id(), y.type_id());
    }

    #[test]
    fn array_storage() {
        let x = Shared::<&[i32]>::new(&[1, 2, 3]);
        let y = shared!{1, 2, 3};
        assert_eq!(x.type_id(), y.type_id());
    }

    #[test]
    fn example() {
        let mut data = [
            shared!(228),
            shared!(1337),
            shared!(177013),
        ];

        let mut work_zone = vec![
            shared!(0), // false data
            shared!(0), // false data
            shared!(0), // false data
        ];

        for item in data.into_iter() {
            work_zone.push(item.clone());
        }

        for mut item in work_zone {
            *item.as_mut() += 1;
            *item += 1;
        }

        println!("{:?}", data)
    }

    #[bench]
    fn compare_shared_vec(b: &mut Bencher) {
        b.iter(|| {
            let _array: Vec<_> = (0..1000).into_iter()
                .map(|i| shared!(i))
                .collect();
            let _clone = _array.clone();
        });
    }

    #[bench]
    fn compare_data_vec(b: &mut Bencher) {
        b.iter(|| {
            let _array: Vec<_> = (0..1000).into_iter()
                .map(|i| i)
                .collect();
            let _clone = _array.clone();
        });
    }
}
