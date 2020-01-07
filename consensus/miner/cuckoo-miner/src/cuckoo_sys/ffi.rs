// Copyright 2017 The Grin Developers

use plugin::*;
use std::sync::{Arc, Mutex};

use libloading;

use error::error::CuckooMinerError;

/// Struct to hold instances of loaded plugins

pub struct PluginLibrary {
    ///The full file path to the plugin loaded by this instance
    pub lib_full_path: String,

    loaded_library: Arc<Mutex<libloading::Library>>,
    cuckoo_create_solver_ctx: Arc<Mutex<CuckooCreateSolverCtx>>,
    cuckoo_destroy_solver_ctx: Arc<Mutex<CuckooDestroySolverCtx>>,
    cuckoo_run_solver: Arc<Mutex<CuckooRunSolver>>,
    cuckoo_stop_solver: Arc<Mutex<CuckooStopSolver>>,
    cuckoo_fill_default_params: Arc<Mutex<CuckooFillDefaultParams>>,
}

impl PluginLibrary {
    /// Loads the specified library

    pub fn new(lib_full_path: &str) -> Result<PluginLibrary, CuckooMinerError> {
        let result = libloading::Library::new(lib_full_path);

        if let Err(e) = result {
            return Err(CuckooMinerError::PluginNotFoundError(String::from(
                format!("{} - {:?}", lib_full_path, e),
            )));
        }

        let loaded_library = result.unwrap();
        PluginLibrary::load_symbols(loaded_library, lib_full_path)
    }

    fn load_symbols(
        loaded_library: libloading::Library,
        path: &str,
    ) -> Result<PluginLibrary, CuckooMinerError> {
        unsafe {
            let ret_val = PluginLibrary {
                lib_full_path: String::from(path),

                cuckoo_create_solver_ctx: {
                    let cuckoo_create_solver_ctx: libloading::Symbol<CuckooCreateSolverCtx> =
                        loaded_library.get(b"create_solver_ctx\0").unwrap();
                    Arc::new(Mutex::new(*cuckoo_create_solver_ctx.into_raw()))
                },

                cuckoo_destroy_solver_ctx: {
                    let cuckoo_destroy_solver_ctx: libloading::Symbol<CuckooDestroySolverCtx> =
                        loaded_library.get(b"destroy_solver_ctx\0").unwrap();
                    Arc::new(Mutex::new(*cuckoo_destroy_solver_ctx.into_raw()))
                },

                cuckoo_run_solver: {
                    let cuckoo_run_solver: libloading::Symbol<CuckooRunSolver> =
                        loaded_library.get(b"run_solver\0").unwrap();
                    Arc::new(Mutex::new(*cuckoo_run_solver.into_raw()))
                },

                cuckoo_stop_solver: {
                    let cuckoo_stop_solver: libloading::Symbol<CuckooStopSolver> =
                        loaded_library.get(b"stop_solver\0").unwrap();
                    Arc::new(Mutex::new(*cuckoo_stop_solver.into_raw()))
                },

                cuckoo_fill_default_params: {
                    let cuckoo_fill_default_params: libloading::Symbol<CuckooFillDefaultParams> =
                        loaded_library.get(b"fill_default_params\0").unwrap();
                    Arc::new(Mutex::new(*cuckoo_fill_default_params.into_raw()))
                },

                loaded_library: Arc::new(Mutex::new(loaded_library)),
            };

            return Ok(ret_val);
        }
    }

    /// #Description
    ///
    /// Unloads the currently loaded plugin and all symbols.
    ///
    /// #Arguments
    ///
    /// None
    ///
    /// #Returns
    ///
    /// Nothing
    ///

    pub fn unload(&self) {
        let cuckoo_create_solver_ref = self.cuckoo_create_solver_ctx.lock().unwrap();
        drop(cuckoo_create_solver_ref);

        let cuckoo_destroy_solver_ref = self.cuckoo_destroy_solver_ctx.lock().unwrap();
        drop(cuckoo_destroy_solver_ref);

        let cuckoo_run_solver_ref = self.cuckoo_run_solver.lock().unwrap();
        drop(cuckoo_run_solver_ref);

        let cuckoo_stop_solver_ref = self.cuckoo_stop_solver.lock().unwrap();
        drop(cuckoo_stop_solver_ref);

        let cuckoo_fill_default_params_ref = self.cuckoo_fill_default_params.lock().unwrap();
        drop(cuckoo_fill_default_params_ref);

        let loaded_library_ref = self.loaded_library.lock().unwrap();
        drop(loaded_library_ref);
    }

    /// Create a solver context
    pub fn create_solver_ctx(&self, params: &mut SolverParams) -> *mut SolverCtx {
        let call_ref = self.cuckoo_create_solver_ctx.lock().unwrap();
        unsafe { call_ref(params) }
    }

    /// Destroy solver context
    pub fn destroy_solver_ctx(&self, ctx: *mut SolverCtx) {
        let call_ref = self.cuckoo_destroy_solver_ctx.lock().unwrap();
        unsafe { call_ref(ctx) }
    }

    /// Run Solver
    pub fn run_solver(
        &self,
        ctx: *mut SolverCtx,
        header: Vec<u8>,
        nonce: u64,
        range: u32,
        solutions: &mut SolverSolutions,
        stats: &mut SolverStats,
    ) -> u32 {
        let call_ref = self.cuckoo_run_solver.lock().unwrap();
        unsafe {
            call_ref(
                ctx,
                header.as_ptr(),
                header.len() as u32,
                nonce,
                range,
                solutions,
                stats,
            )
        }
    }

    /// Stop solver
    pub fn stop_solver(&self, ctx: *mut SolverCtx) {
        let call_ref = self.cuckoo_stop_solver.lock().unwrap();
        unsafe { call_ref(ctx) }
    }

    /// Get default params
    pub fn get_default_params(&self) -> SolverParams {
        let mut ret_params = SolverParams::default();
        let call_ref = self.cuckoo_fill_default_params.lock().unwrap();
        unsafe {
            call_ref(&mut ret_params);
            ret_params
        }
    }

    /// Get an instance of the stop function, to allow it to run in another thread
    pub fn get_stop_solver_instance(&self) -> Arc<Mutex<CuckooStopSolver>> {
        self.cuckoo_stop_solver.clone()
    }

    /// Stop solver from a "detached" instance
    pub fn stop_solver_from_instance(inst: Arc<Mutex<CuckooStopSolver>>, ctx: *mut SolverCtx) {
        let call_ref = inst.lock().unwrap();
        unsafe { call_ref(ctx) }
    }
}
