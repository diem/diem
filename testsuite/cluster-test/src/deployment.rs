// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{aws::Aws, cluster::Cluster};
use failure::prelude::format_err;
use retry::{delay::Fixed, retry};
use rusoto_core::RusotoError;
use rusoto_ecr::{
    BatchGetImageRequest, DescribeImagesRequest, Ecr, Image, ImageIdentifier, PutImageError,
    PutImageRequest,
};
use rusoto_ecs::{Ecs, UpdateServiceRequest};
use slog_scope::{info, warn};
use std::{fs, io::ErrorKind, thread, time::Duration};

#[derive(Clone)]
pub struct DeploymentManager {
    aws: Aws,
    cluster: Cluster,

    last_deployed_digest: Option<String>,
}

const LAST_DEPLOYED_FILE: &str = ".last_deployed_digest";
const VALIDATOR_IMAGE_REPO: &str = "libra_e2e";
const CLIENT_IMAGE_REPO: &str = "libra_client";
const FAUCET_IMAGE_REPO: &str = "libra_faucet";
pub const SOURCE_TAG: &str = "nightly";
pub const RUNNING_TAG: &str = "cluster_test";
pub const TESTED_TAG: &str = "nightly_tested";
const UPSTREAM_PREFIX: &str = "upstream_";

impl DeploymentManager {
    pub fn new(aws: Aws, cluster: Cluster) -> Self {
        let last_deployed_digest = match fs::read_to_string(LAST_DEPLOYED_FILE) {
            Ok(v) => {
                info!("Read last deployed digest: {}", v);
                Some(v)
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    None
                } else {
                    panic!("Failed to read .last_deployed_digest: {:?}", e);
                }
            }
        };

        Self {
            aws,
            cluster,
            last_deployed_digest,
        }
    }

    pub fn latest_hash_changed(&self) -> Option<String> {
        let hash = self.image_digest_by_tag(SOURCE_TAG);
        if let Some(last) = &self.last_deployed_digest {
            if last == &hash {
                info!("Last deployed digest matches latest digest we expect, not doing redeploy");
                return None;
            }
        } else {
            info!("Last deployed digest unknown, re-deploying anyway");
        }
        Some(hash)
    }

    pub fn redeploy(&mut self, hash: String) -> failure::Result<()> {
        info!("Will deploy with digest {}", hash);
        self.tag_image(
            VALIDATOR_IMAGE_REPO,
            &ImageIdentifier {
                image_digest: Some(hash),
                image_tag: None,
            },
            RUNNING_TAG,
        )?;
        let _ignore = fs::remove_file(LAST_DEPLOYED_FILE);
        self.update_all_services()?;
        Ok(())
    }

    pub fn update_all_services(&self) -> failure::Result<()> {
        for instance in self.cluster.instances() {
            let mut request = UpdateServiceRequest::default();
            request.cluster = Some(self.aws.workplace().clone());
            request.force_new_deployment = Some(true);
            request.service = format!(
                "{w}/{w}-validator-{hash}",
                w = self.aws.workplace(),
                hash = instance.short_hash()
            );

            self.aws
                .ecs()
                .update_service(request)
                .sync()
                .map_err(|e| format_err!("Failed to update {}: {:?}", instance, e))?;
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    fn image_digest_by_tag(&self, tag: &str) -> String {
        let mut request = DescribeImagesRequest::default();
        request.repository_name = VALIDATOR_IMAGE_REPO.into();
        let image_id = ImageIdentifier {
            image_digest: None,
            image_tag: Some(tag.to_string()),
        };
        request.image_ids = Some(vec![image_id]);
        let result = self
            .aws
            .ecr()
            .describe_images(request)
            .sync()
            .expect("Failed to find latest nightly image");
        let images = result
            .image_details
            .expect("No image_details in ECR response");
        if images.len() != 1 {
            panic!("Ecr returned {} images for libra_e2e:nightly", images.len());
        }
        let image = images.into_iter().next().unwrap();
        image.image_digest.expect("No image_digest")
    }

    pub fn get_tested_upstream_commit(&self) -> failure::Result<String> {
        let digest = self.image_digest_by_tag(TESTED_TAG);
        let prev_upstream_tag = self.get_upstream_tag(&digest)?;
        Ok(prev_upstream_tag[UPSTREAM_PREFIX.len()..].to_string())
    }

    pub fn tag_tested_image(&mut self, hash: String) -> failure::Result<String> {
        let image_id = ImageIdentifier {
            image_digest: Some(hash.clone()),
            image_tag: None,
        };
        self.tag_image(VALIDATOR_IMAGE_REPO, &image_id, TESTED_TAG)?;
        let upstream_tag = self.get_upstream_tag(&hash)?;
        self.tag_image(
            CLIENT_IMAGE_REPO,
            &ImageIdentifier {
                image_digest: None,
                image_tag: Some(upstream_tag.clone()),
            },
            TESTED_TAG,
        )?;
        self.tag_image(
            FAUCET_IMAGE_REPO,
            &ImageIdentifier {
                image_digest: None,
                image_tag: Some(upstream_tag.clone()),
            },
            TESTED_TAG,
        )?;

        fs::write(LAST_DEPLOYED_FILE, &hash).expect("Failed to write .last_deployed_digest");
        self.last_deployed_digest = Some(hash);
        let upstream_commit = upstream_tag[UPSTREAM_PREFIX.len()..].to_string();
        Ok(upstream_commit)
    }

    pub fn get_upstream_tag(&self, digest: &str) -> failure::Result<String> {
        let image_id = ImageIdentifier {
            image_digest: Some(digest.to_string()),
            image_tag: None,
        };
        let images = self.get_images(VALIDATOR_IMAGE_REPO, &image_id)?;
        for image in images {
            let image_id = match image.image_id {
                Some(image_id) => image_id,
                None => continue,
            };
            let tag = match image_id.image_tag {
                Some(tag) => tag,
                None => continue,
            };
            if tag.starts_with(UPSTREAM_PREFIX) {
                return Ok(tag);
            }
        }
        Err(format_err!("Failed to find upstream tag"))
    }

    fn get_images(
        &self,
        repository: &str,
        image_id: &ImageIdentifier,
    ) -> failure::Result<Vec<Image>> {
        let mut get_request = BatchGetImageRequest::default();
        get_request.repository_name = repository.to_string();
        get_request.image_ids = vec![image_id.clone()];
        // Retry upto 10 times, waiting 10 sec between retries
        let response = retry(Fixed::from_millis(10_000).take(10), || {
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

    fn tag_image(
        &self,
        repository: &str,
        image_id: &ImageIdentifier,
        new_tag: &str,
    ) -> failure::Result<()> {
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
}
