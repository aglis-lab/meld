use serde_json::Value;
use std::{borrow::Cow, collections::HashMap, ops::Range};

pub(super) struct Stack<'a> {
    pub(super) scopes: Vec<Cow<'a, Value>>,
    path_parts: HashMap<String, Vec<String>>,
}

impl<'a> Stack<'a> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            path_parts: HashMap::new(),
        }
    }

    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            scopes: Vec::with_capacity(capacity),
            path_parts: HashMap::new(),
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
    pub fn get(&mut self, key: &str) -> Option<Cow<'a, Value>> {
        if !self.path_parts.contains_key(key) {
            self.path_parts
                .insert(key.to_owned(), key.split('.').map(str::to_owned).collect());
        }
        let parts = self.path_parts.get(key).expect("cached path parts");

        for scope in self.scopes.iter().rev() {
            match scope {
                Cow::Borrowed(root) => {
                    let (current_value, matched_count) = lookup_path(root, parts);
                    if matched_count == parts.len() {
                        return Some(Cow::Borrowed(current_value));
                    }
                    if matched_count > 0 {
                        break;
                    }
                }
                Cow::Owned(root) => {
                    let (current_value, matched_count) = lookup_path(root, parts);
                    if matched_count == parts.len() {
                        return Some(Cow::Owned(current_value.clone()));
                    }
                    if matched_count > 0 {
                        break;
                    }
                }
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
    pub fn drain_top(&mut self, count: usize) -> Option<Vec<Cow<'a, Value>>> {
        let start = self.scopes.len().checked_sub(count)?;
        Some(self.scopes.split_off(start))
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

#[inline(always)]
fn lookup_path<'b>(root: &'b Value, parts: &[String]) -> (&'b Value, usize) {
    let mut current_value = root;
    let mut matched_count = 0usize;

    for part in parts {
        if let Some(value) = current_value.get(part) {
            matched_count += 1;
            current_value = value;
        } else {
            break;
        }
    }

    (current_value, matched_count)
}
