// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub fn get_uuid() -> Box<str> {
    uuid::Uuid::new_v4().to_string().into()
}
