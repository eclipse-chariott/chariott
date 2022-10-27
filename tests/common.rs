// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

pub fn get_uuid() -> Box<str> {
    uuid::Uuid::new_v4().to_string().into()
}
