#[derive(Debug)]
pub struct Bindable<T> {
    value: T,
    dirty: bool,
}

impl<T> Bindable<T> {
    pub fn new(value: T) -> Bindable<T> {
        Bindable { value, dirty: true }
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.dirty = true;
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

macro_rules! impl_binary_op {
    ($trait:ident, $method:ident, $op:tt, $receiver:ty) => {
        impl<T: Clone + std::ops::$trait<Output = T>> std::ops::$trait<T> for $receiver {
            type Output = T;
            fn $method(self, rhs: T) -> T {
                self.value.clone() $op rhs
            }
        }
    };
}

macro_rules! impl_assign_op {
    ($trait:ident, $assign_trait:ident, $method:ident, $op:tt, $receiver:ty) => {
        impl<T: Clone + std::ops::$trait<Output = T>> std::ops::$assign_trait<T> for $receiver {
            fn $method(&mut self, rhs: T) {
                self.set(self.value.clone() $op rhs);
            }
        }
    };
}

macro_rules! impl_operations {
    ($trait:ident, $assign_trait:ident, $method:ident, $assign_method:ident, $op:tt) => {
        impl_binary_op!($trait, $method, $op, Bindable<T>);
        impl_binary_op!($trait, $method, $op, &Bindable<T>);
        impl_binary_op!($trait, $method, $op, &mut Bindable<T>);

        impl_assign_op!($trait, $assign_trait, $assign_method, $op, Bindable<T>);
        impl_assign_op!($trait, $assign_trait, $assign_method, $op, &mut Bindable<T>);
    };
}

impl_operations!(Add, AddAssign, add, add_assign, +);
impl_operations!(Sub, SubAssign, sub, sub_assign, -);
impl_operations!(Mul, MulAssign, mul, mul_assign, *);
impl_operations!(Div, DivAssign, div, div_assign, /);
