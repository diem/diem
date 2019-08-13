use crate::{aws::Aws, cluster::Cluster};
use rusoto_ecr::{DescribeImagesRequest, Ecr, ImageIdentifier};
use rusoto_ecs::{Ecs, UpdateServiceRequest};
use std::{env, fs, io::ErrorKind, thread, time::Duration};

#[derive(Clone)]
pub struct DeploymentManager {
    aws: Aws,
    cluster: Cluster,

    last_deployed_digest: Option<String>,
}

const LAST_DEPLOYED_FILE: &str = ".last_deployed_digest";

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

    pub fn redeploy_if_needed(&mut self) -> bool {
        let hash = self.latest_nightly_image_digest();
        if let Some(last) = &self.last_deployed_digest {
            if last == &hash {
                println!(
                    "Last deployed digest matches latest digest we expect, not doing redeploy"
                );
                return false;
            }
        } else {
            println!("Last deployed digest unknown, re-deploying anyway");
        }
        println!("Will deploy with digest {}", hash);
        let _ignore = fs::remove_file(LAST_DEPLOYED_FILE);
        if env::var("ALLOW_DEPLOY") == Ok("yes".to_string()) {
            self.update_all_services();
        } else {
            println!(
                "ALLOW_DEPLOY var is not set, not doing deploy and updating last_deployed_digest"
            );
        }
        fs::write(LAST_DEPLOYED_FILE, &hash).expect("Failed to write .last_deployed_digest");
        self.last_deployed_digest = Some(hash);
        true
    }

    fn update_all_services(&self) {
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
                .expect("Failed to update service");
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn latest_nightly_image_digest(&self) -> String {
        let mut request = DescribeImagesRequest::default();
        request.repository_name = "libra_e2e".into();
        request.image_ids = Some(vec![ImageIdentifier {
            image_digest: None,
            image_tag: Some("cluster_test".into()),
        }]);
        let result = self
            .aws
            .ecr()
            .describe_images(request)
            .sync()
            .expect("Failed to find latest nightly image");
        let mut images = result
            .image_details
            .expect("No image_details in ECR response");
        if images.len() != 1 {
            panic!("Ecr returned {} images for libra_e2e:nightly", images.len());
        }
        let image = images.remove(0);
        image.image_digest.expect("No image_digest")
    }
}
