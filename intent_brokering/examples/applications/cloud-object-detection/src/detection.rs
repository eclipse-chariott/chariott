// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use examples_common::examples::detection::{DetectRequest, DetectResponse, DetectionObject};
use intent_brokering_common::error::{Error, ResultExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{env, mem};

pub struct DetectionLogic {
    http_client: Client,
    cognitive_endpoint: String,
    cognitive_key: String,
}

impl DetectionLogic {
    pub fn new() -> Self {
        let http_client = Client::new();

        let cognitive_endpoint = match env::var("COGNITIVE_ENDPOINT") {
            Ok(e) => e.replace("https://", ""),
            Err(_) => panic!("Missing COGNITIVE_ENDPOINT environment variable"),
        };
        let cognitive_key = match env::var("COGNITIVE_KEY") {
            Ok(e) => e,
            Err(_) => panic!("Missing COGNITIVE_KEY environment variable"),
        };

        Self { http_client, cognitive_endpoint, cognitive_key }
    }

    pub async fn detect_cloud(&self, body: DetectRequest) -> Result<DetectResponse, Error> {
        let response = self
            .http_client
            .post(format!(
                "https://{}/vision/v3.2/detect?model-version=2021-04-01",
                self.cognitive_endpoint
            ))
            .header("Ocp-Apim-Subscription-Key", self.cognitive_key.to_owned())
            .header("Content-Type", "application/octet-stream")
            .body(Vec::<u8>::from(body))
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err_with("Request to Cognitive Services failed.")?;

        let deserialized_response =
            response.json::<DetectionResponse>().await.map_err_with("Deserialization failed")?;

        Ok(DetectResponse::new(
            deserialized_response
                .objects
                .iter()
                .flat_map(|o| o.ascendants_and_self())
                .map(|o| DetectionObject::new(o.object.clone(), o.confidence))
                .collect(),
        ))
    }
}

impl Default for DetectionLogic {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
pub struct DetectionResponse {
    objects: Vec<Object>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Object {
    object: String,
    confidence: f64,
    parent: Option<Box<Object>>,
}

impl Object {
    pub fn ascendants_and_self(&self) -> ObjectAscendantsAndSelfIterator<'_> {
        ObjectAscendantsAndSelfIterator { next: Some(self) }
    }
}

pub struct ObjectAscendantsAndSelfIterator<'a> {
    next: Option<&'a Object>,
}

impl<'a> Iterator for ObjectAscendantsAndSelfIterator<'a> {
    type Item = &'a Object;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next = self.next.and_then(|n| n.parent.as_ref()).map(|p| p.as_ref());
        mem::swap(&mut self.next, &mut next);
        next
    }
}
