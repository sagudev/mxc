use core::panic;
use std::fmt::Debug;

use thiserror::Error as SuperError;

pub type ExpectResult = Result<ExpectOk, ExpectError>;

#[derive(SuperError, Debug)]
pub enum ExpectError {
    #[error(
        r#"`(left == right)`
    left: `{0}`,
   right: `{1}`"#
    )]
    NotEq(String, String),
    #[error(
        r#"`(left != right)`
    left: `{0}`,
   right: `{1}`"#
    )]
    Eq(String, String),
}

impl ExpectError {
    pub fn generate<T, U>(c: char, left: &T, right: &U) -> Self
    where
        T: Debug + ?Sized,
        U: Debug + ?Sized,
    {
        match c {
            '=' => ExpectError::NotEq(format!("{:?}", left), format!("{:?}", right)),
            '!' => ExpectError::Eq(format!("{:?}", left), format!("{:?}", right)),
            _ => panic!("I do not know!"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ExpectOk {
    /// Exact result
    Exact,
    /// Result is exact with deviation of _
    WithDelta(f64),
}

impl ExpectOk {
    /// Is result exact
    pub const fn is_exact(&self) -> bool {
        matches!(self, Self::Exact)
    }
}

/// `assert_eq!` but returning result instead of panicking
#[macro_export]
macro_rules! expect_eq {
    ($left:expr, $right:expr $(,)?) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    Err($crate::expect::ExpectError::generate(
                        '=',
                        &*left_val,
                        &*right_val,
                    ))
                } else {
                    Ok(())
                }
            }
        }
    };
}

/// `assert_eq_delta!` but returning result instead of panicking
#[macro_export]
macro_rules! expect_eq_delta {
    ($left:expr, $right:expr, $d:expr) => {
        match (&$left, &$right, &$d) {
            (left_val, right_val, delta) => {
                if *left_val == *right_val {
                    Ok($crate::expect::ExpectOk::Exact)
                } else if !(*left_val - *right_val < *delta || *right_val - *left_val < *delta) {
                    // The reborrows below are intentional. Without them, the stack slot for the
                    // borrow is initialized even before the values are compared, leading to a
                    // noticeable slow down.
                    Err($crate::expect::ExpectError::generate(
                        '=',
                        &*left_val,
                        &*right_val,
                    ))
                } else {
                    Ok($crate::expect::ExpectOk::WithDelta(
                        *right_val - *left_val as f64,
                    ))
                }
            }
        }
    };
}

/// `assert_eq!` with delta
#[macro_export]
macro_rules! assert_eq_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if !($x - $y < $d || $y - $x < $d) {
            panic!(
                r#"assertion failed: `(left == right)`
      left: `{:?}`,
     right: `{:?}`"#,
                $x, $y,
            )
        }
    };
}
