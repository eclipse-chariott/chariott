// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{borrow::Borrow, collections::HashMap};

use chariott_common::{
    proto::common::{
        fulfillment::Fulfillment, inspect_fulfillment::Entry as ProtoEntry, InspectFulfillment,
    },
    query::regex_from_query,
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
            .map(|Entry(path, items)| ProtoEntry {
                path: path.to_string(),
                items: items.iter().map(|(k, v)| (k.to_string(), v.clone().into())).collect(),
            })
            .collect(),
    })
}
