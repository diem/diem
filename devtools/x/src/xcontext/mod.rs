// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//remove pub once conversion is complete.
pub mod execution_location;
//pub will still be needed here for config.
pub mod project_metadata;

use coi::{self, ContainerBuilder, Inject, Registration, RegistrationKind};
use project_metadata::Config;
use std::{path::Path, sync::Arc};

/// Reference to the root of the project (location of root Cargo.toml)
///
/// This is a non-ideal implementation as it's generally better to have no
/// sensing of information for the construction of components in the container
/// done by the components themselves.   Some exceptions can be made for "top level"
/// components that exist in the main method as a specialized configuration of that
/// container.
pub trait ProjectRoot: Inject {
    fn root(&self) -> &Path;
}

#[derive(Inject)]
#[coi(provides dyn ProjectRoot with ProjectRootImpl::new())]
struct ProjectRootImpl {
    root: Box<Path>,
}

impl ProjectRoot for ProjectRootImpl {
    fn root(&self) -> &Path {
        self.root.as_ref()
    }
}

impl ProjectRootImpl {
    fn new() -> Self {
        //TODO remove once people see it in action.
        println!("constructing root");
        Self {
            root: Box::from(execution_location::project_root()),
        }
    }
}

/// Reference to the path of this execution, the working dir.
pub trait ExecutionPath: Inject {
    fn path(&self) -> &Path;
}
#[derive(Inject)]
#[coi(provides dyn ExecutionPath with ExecutionPathImpl::new())]
struct ExecutionPathImpl {
    path: Box<Path>,
}

impl ExecutionPath for ExecutionPathImpl {
    fn path(&self) -> &Path {
        self.path.as_ref()
    }
}

impl ExecutionPathImpl {
    fn new() -> Self {
        // TODO: remove once people see it in action.
        println!("constructing path");
        Self {
            path: Box::from(execution_location::locate_project().unwrap().as_path()),
        }
    }
}

/// Is the root of the project the current execution path.
pub trait RootExecutionPath: Inject {
    fn the_same(&self) -> bool;
}

/// This struct can be thought of as a representation of a factory
/// project_root == execution_path -> is_project_root_execution_path;
#[derive(Inject)]
#[coi(provides dyn RootExecutionPath with RootExecutionPathImpl::new(project_root, execution_path))]
pub struct RootExecutionPathImpl {
    // these fields are unused but exist as tracking informantion, coi as it stands now
    // can't define a Provider without making it context aware -- allowing the provider
    // access to the entire context and breaking the ability to track a dependency graph
    // through that provider.  Not good.  So this approach allows us to track the direct deps.
    #[coi(inject)]
    #[allow(dead_code)]
    project_root: Arc<dyn ProjectRoot>,
    #[coi(inject)]
    #[allow(dead_code)]
    execution_path: Arc<dyn ExecutionPath>,
    // this is the relevant field calculated and cached from the prior fields.
    the_same: bool,
}

impl RootExecutionPath for RootExecutionPathImpl {
    fn the_same(&self) -> bool {
        self.the_same
    }
}

impl RootExecutionPathImpl {
    fn new(project_root: Arc<dyn ProjectRoot>, execution_path: Arc<dyn ExecutionPath>) -> Self {
        Self {
            the_same: project_root.root() == execution_path.path(),
            project_root,
            execution_path,
        }
    }
}

/// Is the root of the project the current execution path.
pub trait ProjectConfig: Inject {
    fn config(&self) -> &Config;
}

#[derive(Inject)]
#[coi(provides dyn ProjectConfig with ProjectConfigImpl::new(project_root))]
pub struct ProjectConfigImpl {
    // these fields are unused but exist as tracking informantion, coi as it stands now
    // can't define a Provider without making it context aware -- allowing the provider
    // access to the entire context and breaking the ability to track a dependency graph
    // through that provider.  Not good.  So this approach allows us to track the direct deps.
    #[coi(inject)]
    #[allow(dead_code)]
    project_root: Arc<dyn ProjectRoot>,
    // this is the relevant field calculated and cached from the prior fields.
    config: Config,
}

impl ProjectConfig for ProjectConfigImpl {
    fn config(&self) -> &Config {
        &self.config
    }
}

impl ProjectConfigImpl {
    fn new(project_root: Arc<dyn ProjectRoot>) -> Self {
        Self {
            config: project_metadata::Config::from_file(project_root.root().join("x.toml"))
                .expect("expecting x.toml in the project root"),
            project_root,
        }
    }
}

/// Register the default components from xcontext with the coi Container.
/// Todo: Will want an easy mechanism to override the existing providers for testing.
/// Perhaps simply providing an ignore list is enough?  Specialized Builder Patter?
pub fn register_default_components(builder: ContainerBuilder) -> ContainerBuilder {
    builder
        .register_as(
            "execution_path",
            Registration::new(RegistrationKind::Singleton, ExecutionPathImplProvider),
        )
        .register_as(
            "project_root",
            Registration::new(RegistrationKind::Singleton, ProjectRootImplProvider),
        )
        .register_as(
            "root_execution_path",
            Registration::new(RegistrationKind::Singleton, RootExecutionPathImplProvider),
        )
        .register_as(
            "project_config",
            Registration::new(RegistrationKind::Singleton, ProjectConfigImplProvider),
        )
}

#[test]
fn xcontext_component_test() {
    println!("before container");

    let container = register_default_components(ContainerBuilder::new()).build();

    // cool get a managed instance via it's dependencies getting construct
    let root_path_is_execution_path = container
        .resolve::<dyn RootExecutionPath>("root_execution_path")
        .expect("root_execution_path should exist");

    println!(
        "Root path is execution path {:?}",
        root_path_is_execution_path.the_same()
    );

    println!("before reference");

    // get one of the paths...
    let expath = container
        .resolve::<dyn ExecutionPath>("execution_path")
        .expect("execution_path should exist");

    println!("before use");
    println!("{:?}", expath.path());

    // get the other path...
    let propath = container
        .resolve::<dyn ProjectRoot>("project_root")
        .expect("project_root should exist");

    println!("{:?}", propath.root());

    // get the config...
    let config = container
        .resolve::<dyn ProjectConfig>("project_config")
        .expect("project_root should exist");

    println!("{:?}", config.config());
}
