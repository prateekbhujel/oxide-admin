//! Fluent testing assertions framework for Terminus & Rust
//! Provides `expect(val).to_be(...)`, chainable `.and(...)`, and readable assertions.

use std::fmt::Debug;

pub struct Expectation<T> {
    pub value: T,
}

/// Create a new fluent expectation
pub fn expect<T>(value: T) -> Expectation<T> {
    Expectation { value }
}

/// Fluent test wrapper function
pub fn test<F: FnOnce()>(_description: &str, closure: F) {
    closure();
}

impl<T> Expectation<T> {
    /// Chain another expectation
    pub fn and<U>(&self, next_value: U) -> Expectation<U> {
        Expectation { value: next_value }
    }
}

impl<T: Debug> Expectation<T> {
    /// Assert equality
    pub fn to_be<E: Debug>(self, expected: E) -> Self
    where
        T: PartialEq<E>,
    {
        assert!(
            self.value == expected,
            "Expectation failed: expected `{:?}` to be `{:?}`",
            self.value, expected
        );
        self
    }

    /// Assert equality (alias for to_be)
    pub fn to_equal<E: Debug>(self, expected: E) -> Self
    where
        T: PartialEq<E>,
    {
        self.to_be(expected)
    }

    /// Assert inequality
    pub fn to_not_be<E: Debug>(self, unexpected: E) -> Self
    where
        T: PartialEq<E>,
    {
        assert!(
            self.value != unexpected,
            "Expectation failed: expected `{:?}` to NOT be `{:?}`",
            self.value, unexpected
        );
        self
    }

    /// Assert inequality (alias for to_not_be)
    pub fn to_not_equal<E: Debug>(self, unexpected: E) -> Self
    where
        T: PartialEq<E>,
    {
        self.to_not_be(unexpected)
    }
}

impl<T> Expectation<Vec<T>> {
    /// Assert vector has expected length
    pub fn to_have_length(self, len: usize) -> Self {
        assert_eq!(
            self.value.len(),
            len,
            "Expectation failed: expected vector to have length {}, got {}",
            len,
            self.value.len()
        );
        self
    }
}

impl<T: Debug> Expectation<T> {
    /// Assert greater than
    pub fn to_be_greater_than<E: Debug>(self, other: E) -> Self
    where
        T: PartialOrd<E>,
    {
        assert!(
            self.value > other,
            "Expectation failed: expected `{:?}` to be greater than `{:?}`",
            self.value, other
        );
        self
    }

    /// Assert less than
    pub fn to_be_less_than<E: Debug>(self, other: E) -> Self
    where
        T: PartialOrd<E>,
    {
        assert!(
            self.value < other,
            "Expectation failed: expected `{:?}` to be less than `{:?}`",
            self.value, other
        );
        self
    }
}

impl Expectation<bool> {
    /// Assert boolean true
    pub fn to_be_true(self) -> Self {
        assert!(self.value, "Expectation failed: expected true, got false");
        self
    }

    /// Assert boolean false
    pub fn to_be_false(self) -> Self {
        assert!(!self.value, "Expectation failed: expected false, got true");
        self
    }
}

impl<T: AsRef<str>> Expectation<T> {
    /// Assert string contains substring
    pub fn to_contain(self, needle: &str) -> Self {
        let s = self.value.as_ref();
        assert!(
            s.contains(needle),
            "Expectation failed: expected string `{}` to contain `{}`",
            s, needle
        );
        self
    }

    /// Assert string does not contain substring
    pub fn to_not_contain(self, needle: &str) -> Self {
        let s = self.value.as_ref();
        assert!(
            !s.contains(needle),
            "Expectation failed: expected string `{}` to NOT contain `{}`",
            s, needle
        );
        self
    }

    /// Assert string is empty
    pub fn to_be_empty(self) -> Self {
        let s = self.value.as_ref();
        assert!(
            s.is_empty(),
            "Expectation failed: expected string to be empty, got `{}`",
            s
        );
        self
    }

    /// Assert string is not empty
    pub fn to_not_be_empty(self) -> Self {
        let s = self.value.as_ref();
        assert!(
            !s.is_empty(),
            "Expectation failed: expected string to NOT be empty"
        );
        self
    }
}

impl<T: Debug> Expectation<Option<T>> {
    /// Assert option is Some
    pub fn to_be_some(self) -> Self {
        assert!(
            self.value.is_some(),
            "Expectation failed: expected Option::Some, got Option::None"
        );
        self
    }

    /// Assert option is None
    pub fn to_be_none(self) -> Self {
        assert!(
            self.value.is_none(),
            "Expectation failed: expected Option::None, got `{:?}`",
            self.value
        );
        self
    }
}

impl<T: Debug, E: Debug> Expectation<Result<T, E>> {
    /// Assert result is Ok
    pub fn to_be_ok(self) -> Self {
        assert!(
            self.value.is_ok(),
            "Expectation failed: expected Result::Ok, got `{:?}`",
            self.value
        );
        self
    }

    /// Assert result is Err
    pub fn to_be_err(self) -> Self {
        assert!(
            self.value.is_err(),
            "Expectation failed: expected Result::Err, got `{:?}`",
            self.value
        );
        self
    }
}
