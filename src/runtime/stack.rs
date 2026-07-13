use serde_json::Value;
use std::{borrow::Cow, ops::Range};

pub(super) struct Stack<'a> {
    pub(super) scopes: Vec<Cow<'a, Value>>,
}

impl<'a> Stack<'a> {
    #[inline(always)]
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            scopes: Vec::with_capacity(capacity),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, value: Cow<'a, Value>) {
        self.scopes.push(value);
    }

    #[inline(always)]
    pub fn pop(&mut self) -> Option<Cow<'a, Value>> {
        self.scopes.pop()
    }

    #[inline(always)]
    pub fn peek(&self) -> Option<&Cow<'a, Value>> {
        self.scopes.last()
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    #[inline(always)]
    pub fn get(&self, key: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            let mut current_value = scope.as_ref();
            let mut matched_count = 0usize;
            let mut key_count = 0usize;

            for part in key.split('.') {
                key_count += 1;
                if let Some(value) = current_value.get(part) {
                    matched_count += 1;
                    current_value = value;
                } else {
                    break;
                }
            }

            // There is a match in the current scope
            if matched_count == key_count {
                return Some(current_value);
            }

            // No need to check the outer scopes if we found a partial match in the current scope
            if matched_count > 0 {
                break;
            }
        }

        None
    }

    #[inline(always)]
    pub fn get_range(&mut self, range: Range<usize>) -> Option<&[Cow<'_, Value>]> {
        if range.end > self.scopes.len() {
            return None;
        }

        Some(&self.scopes[range])
    }

    #[inline(always)]
    pub fn get_drain_range(&mut self, range: Range<usize>) -> Option<Vec<Cow<'_, Value>>> {
        if range.end > self.scopes.len() {
            return None;
        }

        Some(self.scopes.drain(range).collect())
    }

    #[inline(always)]
    pub fn drain_range(&mut self, range: Range<usize>) -> bool {
        if range.end > self.scopes.len() {
            return false;
        }

        self.scopes.drain(range);
        true
    }
}
