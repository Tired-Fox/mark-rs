use std::fmt::Display;

pub trait Argument {}

impl<const N: usize> Argument for [Arg; N] {}

pub enum Arg {
    Positional(Box<dyn Argument>),
    Named(String, Box<dyn Argument>),
}

pub trait IntoArgument {
    fn into_argument(self) -> Arg;
}

impl<A: Argument + 'static> IntoArgument for A {
    fn into_argument(self) -> Arg {
        Arg::Positional(Box::new(self))
    }
}
impl<S: ToString, A: Argument + 'static> IntoArgument for (S, A) {
    fn into_argument(self) -> Arg {
        Arg::Named(self.0.to_string(), Box::new(self.1))
    }
}

impl Argument for &str {}
impl<A: Argument> Argument for Option<A> {}
impl Argument for () {}

pub fn formatter<S: Display, A: IntoArgument>(fmt: &str, args: A) {
    println!("Hello, world!");
}

#[macro_export]
macro_rules! format {
    ($fmt: literal) => {
        $crate::formatter($fmt, ());
    };
    ($fmt: literal, $($args: tt)*) => {
        $crate::format!(@ $fmt, [], $($args)*)
    };
    (@ $fmt: literal, [$($args: tt)*], $key: ident = $value: expr, $($arg: tt)*) => {
        $crate::format!(@ $fmt, [$($args)* ($key, $value).into_argument(),], $($args)*)
    };
    (@ $fmt: literal, [$($args: tt)*], $value: expr, $($arg: tt)*) => {
        $crate::format!(@ $fmt, [$($args)* ($value).into_argument(),], $($args)*)
    };
    (@ $fmt: literal, [$($args: tt)*], $key: ident = $value: expr) => {
        {
            use $crate::IntoArgument;
            $crate::formatter($fmt, ($($args)*, ($key, $value).into_argument(),))
        }
    };
    (@ $fmt: literal, [$($args: tt)*], $value: expr) => {
        {
            use $crate::IntoArgument;
            $crate::formatter($fmt, ($($args)*, ($value).into_argument(),))
        }
    };
    (@ $fmt: literal, [$($args: tt)*], $(,)?) => {
        {
            use $crate::IntoArgument;
            $crate::formatter($fmt, ($($args)*))
        }
    };
}