// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use examples_common::examples::detection::{DetectRequest, DetectResponse, DetectionObject};
use image::{io::Reader, DynamicImage, GenericImageView};
use intent_brokering_common::error::Error;
use ndarray::prelude::Array;
use serde::{Deserialize, Serialize};
use std::{
    env::var,
    fs::File,
    io::{BufReader, Cursor},
};
use tensorflow::{Graph, ImportGraphDefOptions, Session, SessionOptions, SessionRunArgs, Tensor};

const INVALID_IMAGE_FORMAT: &str = "Could not identify image format";
const INVALID_IMAGE: &str = "Could not decode image";
const NO_MATCHING_CATEGORY: &str = "NO_MATCHING_CATEGORY";

pub struct DetectionLogic {
    graph: Graph,
}

impl DetectionLogic {
    pub fn new() -> Self {
        let mut graph = Graph::new();
        let proto = include_bytes!("../models/ssd_mobilenet_v2_coco.pb");

        graph.import_graph_def(proto, &ImportGraphDefOptions::new()).unwrap();

        Self { graph }
    }

    pub fn detect_local(&self, body: DetectRequest) -> Result<DetectResponse, Error> {
        // Get image into DynamicImage type
        let image = Reader::new(Cursor::new(Vec::<u8>::from(body)))
            .with_guessed_format()
            .map_err(|_| Error::new(INVALID_IMAGE_FORMAT))?
            .decode()
            .map_err(|_| Error::new(INVALID_IMAGE))?;

        let result = detect_local_inner(&self.graph, image);
        return result.map_err(|e| Error::from_error(e.to_string(), e));

        fn detect_local_inner(
            graph: &Graph,
            image: DynamicImage,
        ) -> Result<DetectResponse, Box<dyn std::error::Error + Send + Sync>> {
            // Build ndarray
            let (width, height) = image.dimensions();
            let image_array_expanded =
                Array::from_shape_vec((height as usize, width as usize, 3), image.into_bytes())?
                    .insert_axis(ndarray::Axis(0));

            let image_tensor_op = graph.operation_by_name_required("image_tensor")?;
            let input_image_tensor = Tensor::new(&[1, height as u64, width as u64, 3])
                .with_values(image_array_expanded.as_slice().unwrap())?;
            let mut session_args = SessionRunArgs::new();
            session_args.add_feed(&image_tensor_op, 0, &input_image_tensor);

            let classes = graph.operation_by_name_required("detection_classes")?;
            let classes_token = session_args.request_fetch(&classes, 0);

            let scores = graph.operation_by_name_required("detection_scores")?;
            let scores_token = session_args.request_fetch(&scores, 0);

            // Run detection session
            let detection_session = Session::new(&SessionOptions::new(), graph)?;
            detection_session.run(&mut session_args)?;

            // Parse detection session results
            let classes_tensor = session_args.fetch::<f32>(classes_token)?;
            let scores_tensor = session_args.fetch::<f32>(scores_token)?;

            // Collect results and map to human readable categories
            let categories = get_categories()?;
            let classes_categories =
                classes_tensor.iter().map(|v| match categories.iter().find(|c| c.id.eq(v)) {
                    Some(c) => c.name.clone(),
                    None => NO_MATCHING_CATEGORY.to_owned(),
                });

            Ok(DetectResponse::new(
                scores_tensor
                    .iter()
                    .zip(classes_categories)
                    .filter(|(&score, _)| score > 0.0)
                    .map(|(&score, category)| DetectionObject::new(category, score.into()))
                    .collect(),
            ))
        }

        fn get_categories() -> Result<Vec<Category>, Box<dyn std::error::Error + Send + Sync>> {
            const DEFAULT_CATEGORIES_FILE_PATH: &str = "./models/categories.json";
            const CATEGORIES_FILE_PATH_ENV_NAME: &str = "CATEGORIES_FILE_PATH";

            let file = File::open(
                var(CATEGORIES_FILE_PATH_ENV_NAME)
                    .unwrap_or_else(|_| DEFAULT_CATEGORIES_FILE_PATH.to_owned()),
            )?;
            let reader = BufReader::new(file);
            let result = serde_json::from_reader(reader)?;
            Ok(result)
        }
    }
}

impl Default for DetectionLogic {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Category {
    supercategory: String,
    name: String,
    id: f32,
}
