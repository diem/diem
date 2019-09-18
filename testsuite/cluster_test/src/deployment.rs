use crate::{aws::Aws, cluster::Cluster};
use failure::prelude::{bail, format_err};
use rusoto_core::RusotoError;
use rusoto_ecr::{
    BatchGetImageRequest, DescribeImagesRequest, Ecr, ImageIdentifier, PutImageError,
    PutImageRequest,
};
use rusoto_ecs::{Ecs, UpdateServiceRequest};
use std::{fs, io::ErrorKind, thread, time::Duration};

#[derive(Clone)]
pub struct DeploymentManager {
    aws: Aws,
    cluster: Cluster,

    last_deployed_digest: Option<String>,
}

const LAST_DEPLOYED_FILE: &str = ".last_deployed_digest";
const REPOSITORY_NAME: &str = "libra_e2e";
pub const SOURCE_TAG: &str = "nightly";
pub const RUNNING_TAG: &str = "cluster_test";
pub const TESTED_TAG: &str = "nightly_tested";

impl DeploymentManager {
    pub fn new(aws: Aws, cluster: Cluster) -> Self {
        let last_deployed_digest = match fs::read_to_string(LAST_DEPLOYED_FILE) {
            Ok(v) => {
                println!("Read last deployed digest: {}", v);
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
        let hash = self.latest_nightly_image_digest();
        if let Some(last) = &self.last_deployed_digest {
            if last == &hash {
                println!(
                    "Last deployed digest matches latest digest we expect, not doing redeploy"
                );
                return None;
            }
        } else {
            println!("Last deployed digest unknown, re-deploying anyway");
        }
        Some(hash)
    }

    pub fn redeploy(&mut self, hash: String) -> failure::Result<()> {
        println!("Will deploy with digest {}", hash);
        self.tag_image(RUNNING_TAG.to_string(), hash)?;
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

    fn latest_nightly_image_digest(&self) -> String {
        let mut request = DescribeImagesRequest::default();
        request.repository_name = REPOSITORY_NAME.into();
        request.image_ids = Some(vec![Self::nightly_image_id()]);
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

    fn nightly_image_id() -> ImageIdentifier {
        ImageIdentifier {
            image_digest: None,
            image_tag: Some(SOURCE_TAG.to_string()),
        }
    }

    pub fn tag_tested_image(&mut self, hash: String) -> failure::Result<()> {
        self.tag_image(TESTED_TAG.to_string(), hash.clone())?;
        fs::write(LAST_DEPLOYED_FILE, &hash).expect("Failed to write .last_deployed_digest");
        self.last_deployed_digest = Some(hash);
        Ok(())
    }

    fn tag_image(&self, tag: String, hash: String) -> failure::Result<()> {
        let mut get_request = BatchGetImageRequest::default();
        get_request.repository_name = REPOSITORY_NAME.to_string();
        get_request.image_ids = vec![ImageIdentifier {
            image_digest: Some(hash.clone()),
            image_tag: None,
        }];
        let response = self
            .aws
            .ecr()
            .batch_get_image(get_request)
            .sync()
            .map_err(|e| format_err!("Failed to get image {}: {:?}", hash, e))?;
        let images = response
            .images
            .expect("No images in batch_get_image response");
        if images.is_empty() {
            bail!("batch_get_image returned {} images", images.len());
        }
        let image = images.into_iter().next().unwrap();
        let manifest = image
            .image_manifest
            .expect("no manifest in batch_get_image response");
        let mut put_request = PutImageRequest::default();
        put_request.image_manifest = manifest;
        put_request.repository_name = REPOSITORY_NAME.to_string();
        put_request.image_tag = Some(tag.clone());
        let result = self.aws.ecr().put_image(put_request).sync();
        if let Err(e) = result {
            if let RusotoError::Service(PutImageError::ImageAlreadyExists(_)) = e {
                println!("Tagging {} with {}: Image already exist", hash, tag);
                Ok(())
            } else {
                Err(format_err!("Failed to tag image: {:?}", e))
            }
        } else {
            Ok(())
        }
    }
}
