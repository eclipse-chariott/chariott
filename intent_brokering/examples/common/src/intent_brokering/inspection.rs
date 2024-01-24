// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{borrow::Borrow, collections::HashMap};

use intent_brokering_common::query::regex_from_query;
use intent_brokering_proto::common::{
    fulfillment::Fulfillment, inspect_fulfillment::Entry as EntryMessage, InspectFulfillment,
};

use super::value::Value;

pub struct Entry(Box<str>, HashMap<Box<str>, Value>);

impl Entry {
    pub fn new(
        path: impl Into<Box<str>>,
        items: impl IntoIterator<Item = (impl Into<Box<str>>, impl Into<Value>)>,
    ) -> Self {
        Self(path.into(), items.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }

    pub fn get(&self, key: impl Borrow<str>) -> Option<&Value> {
        self.1.get(key.borrow())
    }

    pub fn path(&self) -> &str {
        &self.0
    }
}

pub fn fulfill<'a>(
    query: impl AsRef<str>,
    entries: impl IntoIterator<Item = &'a Entry>,
) -> Fulfillment {
    let regex = regex_from_query(query.as_ref());
    Fulfillment::Inspect(InspectFulfillment {
        entries: entries
            .into_iter()
            .filter(|Entry(path, _)| regex.is_match(path))
            .map(|Entry(path, items)| EntryMessage {
                path: path.to_string(),
                items: items.iter().map(|(k, v)| (k.to_string(), v.clone().into())).collect(),
            })
            .collect(),
    })
}
