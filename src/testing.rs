//! Pest PHP-style fluent testing framework for OxideAdmin & Rust
//! Provides `expect(val).to_be(...)`, chainable `.and(...)`, and readable assertions.

use std::fmt::Debug;

pub struct Expectation<T> {
    pub value: T,
}

/// Create a new Pest-style expectation (Laravel/Pest `expect($value)`)
pub fn expect<T>(value: T) -> Expectation<T> {
    Expectation { value }
}

/// Pest-style test wrapper function (`pest_test('it does something', || { ... })`)
pub fn test<F: FnOnce()>(_description: &str, closure: F) {
    closure();
}

impl<T> Expectation<T> {
    /// Chain another expectation (Pest `->and($next)`)
    pub fn and<U>(&self, next_value: U) -> Expectation<U> {
        Expectation { value: next_value }
    }
}

impl<T: Debug> Expectation<T> {
    /// Assert equality (Pest `->toBe($expected)`)
    pub fn to_be<E: Debug>(self, expected: E) -> Self
    where
        T: PartialEq<E>,
    {
        assert!(
            self.value == expected,
            "Pest Expectation failed: expected `{:?}` to be `{:?}`",
            self.value, expected
        );
        self
    }

    /// Assert inequality (Pest `->toNotBe($unexpected)`)
    pub fn to_not_be<E: Debug>(self, unexpected: E) -> Self
    where
        T: PartialEq<E>,
    {
        assert!(
            self.value != unexpected,
            "Pest Expectation failed: expected `{:?}` to NOT be `{:?}`",
            self.value, unexpected
        );
        self
    }
}

impl<T: Debug> Expectation<T> {
    /// Assert greater than (Pest `->toBeGreaterThan($other)`)
    pub fn to_be_greater_than<E: Debug>(self, other: E) -> Self
    where
        T: PartialOrd<E>,
    {
        assert!(
            self.value > other,
            "Pest Expectation failed: expected `{:?}` to be greater than `{:?}`",
            self.value, other
        );
        self
    }

    /// Assert less than (Pest `->toBeLessThan($other)`)
    pub fn to_be_less_than<E: Debug>(self, other: E) -> Self
    where
        T: PartialOrd<E>,
    {
        assert!(
            self.value < other,
            "Pest Expectation failed: expected `{:?}` to be less than `{:?}`",
            self.value, other
        );
        self
    }
}

impl Expectation<bool> {
    /// Assert boolean true (Pest `->toBeTrue()`)
    pub fn to_be_true(self) -> Self {
        assert!(self.value, "Pest Expectation failed: expected true, got false");
        self
    }

    /// Assert boolean false (Pest `->toBeFalse()`)
    pub fn to_be_false(self) -> Self {
        assert!(!self.value, "Pest Expectation failed: expected false, got true");
        self
    }
}

impl<T: AsRef<str>> Expectation<T> {
    /// Assert string contains substring (Pest `->toContain($needle)`)
    pub fn to_contain(self, needle: &str) -> Self {
        let s = self.value.as_ref();
        assert!(
            s.contains(needle),
            "Pest Expectation failed: expected string `{}` to contain `{}`",
            s, needle
        );
        self
    }

    /// Assert string does not contain substring (Pest `->toNotContain($needle)`)
    pub fn to_not_contain(self, needle: &str) -> Self {
        let s = self.value.as_ref();
        assert!(
            !s.contains(needle),
            "Pest Expectation failed: expected string `{}` to NOT contain `{}`",
            s, needle
        );
        self
    }

    /// Assert string is empty (Pest `->toBeEmpty()`)
    pub fn to_be_empty(self) -> Self {
        let s = self.value.as_ref();
        assert!(
            s.is_empty(),
            "Pest Expectation failed: expected string to be empty, got `{}`",
            s
        );
        self
    }

    /// Assert string is not empty (Pest `->toNotBeEmpty()`)
    pub fn to_not_be_empty(self) -> Self {
        let s = self.value.as_ref();
        assert!(
            !s.is_empty(),
            "Pest Expectation failed: expected string to NOT be empty"
        );
        self
    }
}

impl<T: Debug> Expectation<Option<T>> {
    /// Assert option is Some (Pest `->toBeSome()`)
    pub fn to_be_some(self) -> Self {
        assert!(
            self.value.is_some(),
            "Pest Expectation failed: expected Option::Some, got Option::None"
        );
        self
    }

    /// Assert option is None (Pest `->toBeNone()`)
    pub fn to_be_none(self) -> Self {
        assert!(
            self.value.is_none(),
            "Pest Expectation failed: expected Option::None, got `{:?}`",
            self.value
        );
        self
    }
}

impl<T: Debug, E: Debug> Expectation<Result<T, E>> {
    /// Assert result is Ok (Pest `->toBeOk()`)
    pub fn to_be_ok(self) -> Self {
        assert!(
            self.value.is_ok(),
            "Pest Expectation failed: expected Result::Ok, got `{:?}`",
            self.value
        );
        self
    }

    /// Assert result is Err (Pest `->toBeErr()`)
    pub fn to_be_err(self) -> Self {
        assert!(
            self.value.is_err(),
            "Pest Expectation failed: expected Result::Err, got `{:?}`",
            self.value
        );
        self
    }
}
