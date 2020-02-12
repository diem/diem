// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{aws::Aws, cluster::Cluster};
use anyhow::{bail, format_err, Result};
use rusoto_core::RusotoError;
use rusoto_ecr::{
    BatchGetImageRequest, DescribeImagesRequest, DescribeImagesResponse, Ecr, Image,
    ImageIdentifier, PutImageError, PutImageRequest,
};
use rusoto_ecs::{Ecs, UpdateServiceRequest};
use slog_scope::{info, warn};
use std::{env, thread, time::Duration};
use util::retry;

#[derive(Clone)]
pub struct DeploymentManager {
    aws: Aws,
    cluster: Cluster,
    running_tag: String,
}

const VALIDATOR_IMAGE_REPO: &str = "libra_validator";
const NIGHTLY_PREFIX: &str = "nightly";
const UPSTREAM_PREFIX: &str = "upstream_";
const MASTER_PREFIX: &str = "master_";

impl DeploymentManager {
    pub fn new(aws: Aws, cluster: Cluster) -> Self {
        let running_tag = env::var("TAG").unwrap_or_else(|_| aws.workspace().to_string());
        if running_tag.starts_with(NIGHTLY_PREFIX)
            || running_tag.starts_with(UPSTREAM_PREFIX)
            || running_tag.starts_with(MASTER_PREFIX)
        {
            panic!(
                "Cluster test can not deploy if workspace is configured to use {} tag.\
                 Use custom tag for your workspace to use --deploy",
                running_tag
            );
        }
        info!("Will use {} tag for deployment", running_tag);
        Self {
            aws,
            cluster,
            running_tag,
        }
    }

    pub fn redeploy(&mut self, hash: String) -> Result<()> {
        info!("Will deploy with digest {}", hash);
        self.tag_image(
            VALIDATOR_IMAGE_REPO,
            &ImageIdentifier {
                image_digest: Some(hash),
                image_tag: None,
            },
            &self.running_tag,
        )?;
        self.update_all_services()?;
        Ok(())
    }

    pub fn update_all_services(&self) -> Result<()> {
        for instance in self.cluster.all_instances() {
            let mut request = UpdateServiceRequest::default();
            request.cluster = Some(self.aws.workspace().clone());
            request.force_new_deployment = Some(true);
            request.service = instance.peer_name().to_string();

            self.aws
                .ecs()
                .update_service(request)
                .sync()
                .map_err(|e| format_err!("Failed to update {}: {:?}", instance, e))?;
            thread::sleep(Duration::from_millis(200));
        }
        Ok(())
    }

    fn image_digest_by_tag(&self, tag: &str) -> Result<String> {
        let result = self.describe_images(tag);
        let images = result
            .image_details
            .ok_or_else(|| format_err!("No image_details in ECR response"))?;
        if images.len() != 1 {
            bail!("Ecr returned {} images for libra_e2e:nightly", images.len());
        }
        let image = images.into_iter().next().unwrap();
        image
            .image_digest
            .ok_or_else(|| format_err!("No image_digest"))
    }

    fn describe_images(&self, tag: &str) -> DescribeImagesResponse {
        let mut retry = 0usize;
        loop {
            let mut request = DescribeImagesRequest::default();
            request.repository_name = VALIDATOR_IMAGE_REPO.into();
            let image_id = ImageIdentifier {
                image_digest: None,
                image_tag: Some(tag.to_string()),
            };
            request.image_ids = Some(vec![image_id]);
            match self.aws.ecr().describe_images(request).sync() {
                Ok(r) => return r,
                Err(e) => {
                    if retry > 10 {
                        panic!("Failed describe_images after 10 attempts");
                    } else {
                        warn!("Transient failure in describe_images: {}", e);
                        thread::sleep(Duration::from_secs(10));
                        retry += 1;
                    }
                }
            }
        }
    }

    fn get_images(&self, repository: &str, image_id: &ImageIdentifier) -> Result<Vec<Image>> {
        let mut get_request = BatchGetImageRequest::default();
        get_request.repository_name = repository.to_string();
        get_request.image_ids = vec![image_id.clone()];
        // Retry upto 10 times, waiting 10 sec between retries
        let response = retry::retry(retry::fixed_retry_strategy(10_000, 10), || {
            self.aws
                .ecr()
                .batch_get_image(get_request.clone())
                .sync()
                .map_err(|e| {
                    warn!(
                        "Failed to get image from repository: {:?}. Retrying...",
                        get_request.repository_name
                    );
                    e
                })
        })
        .map_err(|e| {
            format_err!(
                "Failed to get image from repository: {:?} after 10 tries: {:?}",
                get_request.repository_name,
                e
            )
        })?;
        response
            .images
            .ok_or_else(|| format_err!("No images in batch_get_image response"))
    }

    fn tag_image(&self, repository: &str, image_id: &ImageIdentifier, new_tag: &str) -> Result<()> {
        let images = self.get_images(repository, &image_id)?;
        let image = images
            .into_iter()
            .next()
            .ok_or_else(|| format_err!("get_images returned 0 images"))?;
        let manifest = image
            .image_manifest
            .ok_or_else(|| format_err!("no manifest in batch_get_image response"))?;
        let mut put_request = PutImageRequest::default();
        put_request.image_manifest = manifest;
        put_request.repository_name = repository.to_string();
        put_request.image_tag = Some(new_tag.to_string());
        let result = self.aws.ecr().put_image(put_request).sync();
        if let Err(e) = result {
            if let RusotoError::Service(PutImageError::ImageAlreadyExists(_)) = e {
                info!("Tag for Image already exists {}:{}", repository, new_tag);
                Ok(())
            } else {
                Err(format_err!("Failed to tag image: {:?}", e))
            }
        } else {
            Ok(())
        }
    }

    pub fn resolve(&self, hash_or_tag: &str) -> Result<String> {
        if hash_or_tag.starts_with("sha256:") {
            Ok(hash_or_tag.to_string())
        } else {
            self.image_digest_by_tag(hash_or_tag)
        }
    }
}
