// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{error::Error, fmt::Display};

use super::proto::common::{value::Value as ValueEnum, Blob, Value as ValueMessage};

#[derive(Debug)]
pub struct InvalidType;

impl Display for InvalidType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid type.")
    }
}

impl Error for InvalidType {}

#[derive(Debug)]
pub struct InvalidValueType(Value);

impl Display for InvalidValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid type.")
    }
}

impl Error for InvalidValueType {}

impl From<InvalidValueType> for Value {
    fn from(InvalidValueType(value): InvalidValueType) -> Self {
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value(ValueEnum);

impl Value {
    pub const TRUE: Self = Self(ValueEnum::Bool(true));
    pub const FALSE: Self = Self(ValueEnum::Bool(false));
    pub const NULL: Self = Self(ValueEnum::Null(0));

    pub fn new_any(type_url: String, value: Vec<u8>) -> Self {
        Self(ValueEnum::Any(prost_types::Any { type_url, value }))
    }

    pub fn new_blob(media_type: String, bytes: Vec<u8>) -> Self {
        Self(ValueEnum::Blob(Blob { media_type, bytes }))
    }

    pub fn to_i32(&self) -> Result<i32, InvalidType> {
        if let Self(ValueEnum::Int32(value)) = self {
            Ok(*value as _)
        } else {
            Err(InvalidType)
        }
    }

    pub fn to_i64(&self) -> Result<i64, InvalidType> {
        if let Self(ValueEnum::Int64(value)) = self {
            Ok(*value as _)
        } else {
            Err(InvalidType)
        }
    }

    pub fn to_bool(&self) -> Result<bool, InvalidType> {
        if let Self(ValueEnum::Bool(value)) = self {
            Ok(*value)
        } else {
            Err(InvalidType)
        }
    }

    pub fn as_str(&self) -> Result<&str, InvalidType> {
        if let Self(ValueEnum::String(value)) = self {
            Ok(value.as_str())
        } else {
            Err(InvalidType)
        }
    }

    pub fn into_string(self) -> Result<String, InvalidValueType> {
        if let Self(ValueEnum::String(value)) = self {
            Ok(value)
        } else {
            Err(InvalidValueType(self))
        }
    }

    pub fn into_any(self) -> Result<(String, Vec<u8>), InvalidValueType> {
        if let Self(ValueEnum::Any(any)) = self {
            Ok((any.type_url, any.value))
        } else {
            Err(InvalidValueType(self))
        }
    }

    pub fn into_blob(self) -> Result<(String, Vec<u8>), InvalidValueType> {
        if let Self(ValueEnum::Blob(blob)) = self {
            Ok((blob.media_type, blob.bytes))
        } else {
            Err(InvalidValueType(self))
        }
    }
}

impl TryFrom<ValueMessage> for Value {
    type Error = ();

    fn try_from(value: ValueMessage) -> Result<Self, Self::Error> {
        value.value.ok_or(()).map(|v| v.into())
    }
}

impl From<ValueEnum> for Value {
    fn from(value: ValueEnum) -> Self {
        Value(value)
    }
}

impl From<Value> for ValueMessage {
    fn from(value: Value) -> Self {
        ValueMessage { value: Some(value.into()) }
    }
}

impl From<Value> for ValueEnum {
    fn from(Value(value): Value) -> Self {
        value
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value(ValueEnum::String(value.into()))
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value(ValueEnum::Int32(value as _))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value(ValueEnum::Int64(value as _))
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value(ValueEnum::Bool(value))
    }
}
