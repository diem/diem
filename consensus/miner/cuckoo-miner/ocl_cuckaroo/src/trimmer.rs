use ocl;
use ocl::enums::{ArgVal, DeviceInfo, DeviceInfoResult};
use ocl::flags::{CommandQueueProperties, MemFlags};
use ocl::prm::{Uint2, Ulong4};
use ocl::{
    Buffer, Context, Device, Event, EventList, Kernel, Platform, Program, Queue, SpatialDims,
};
use std::collections::HashMap;
use std::env;

const DUCK_SIZE_A: usize = 129; // AMD 126 + 3
const DUCK_SIZE_B: usize = 83;
const BUFFER_SIZE_A1: usize = DUCK_SIZE_A * 1024 * (4096 - 128) * 2;
const BUFFER_SIZE_A2: usize = DUCK_SIZE_A * 1024 * 256 * 2;
const BUFFER_SIZE_B: usize = DUCK_SIZE_B * 1024 * 4096 * 2;
const INDEX_SIZE: usize = 256 * 256 * 4;

pub struct Trimmer {
    q: Queue,
    program: Program,
    buffer_a1: Buffer<u32>,
    buffer_a2: Buffer<u32>,
    buffer_b: Buffer<u32>,
    buffer_i1: Buffer<u32>,
    buffer_i2: Buffer<u32>,
    buffer_r: Buffer<u32>,
    buffer_nonces: Buffer<u32>,
    pub device_name: String,
    pub device_id: usize,
    is_nvidia: bool,
}

struct ClBufferParams {
    size: usize,
    flags: MemFlags,
}

macro_rules! clear_buffer (
	($buf:expr) => (
		$buf.cmd().fill(0, None).enq()?;
	));

macro_rules! kernel_enq(
	($kernel:expr, $event_list:expr, $names:expr, $msg:expr) => (
		#[cfg(feature = "profile")]
		{
		$kernel.cmd().enew(&mut $event_list).enq()?;
		$names.push($msg);
		}
		#[cfg(not(feature = "profile"))]
		{
		$kernel.cmd().enq()?;
		}
	));

macro_rules! get_device_info {
    ($dev:ident, $name:ident) => {{
        match $dev.info(DeviceInfo::$name) {
            Ok(DeviceInfoResult::$name(value)) => value,
            _ => panic!("Failed to retrieve device {}", stringify!($name)),
        }
    }};
}

#[cfg(feature = "profile")]
fn queue_props() -> Option<CommandQueueProperties> {
    Some(CommandQueueProperties::PROFILING_ENABLE)
}

#[cfg(not(feature = "profile"))]
fn queue_props() -> Option<CommandQueueProperties> {
    None
}

macro_rules! kernel_builder(
	($obj: expr, $kernel: expr, $global_works_size: expr) => (
			 Kernel::builder()
			.name($kernel)
			.program(&$obj.program)
			.queue($obj.q.clone())
			.global_work_size($global_works_size)
));

impl Trimmer {
    pub fn build(platform_name: Option<&str>, device_id: Option<usize>) -> ocl::Result<Trimmer> {
        env::set_var("GPU_MAX_HEAP_SIZE", "100");
        env::set_var("GPU_USE_SYNC_OBJECTS", "1");
        env::set_var("GPU_MAX_ALLOC_PERCENT", "100");
        env::set_var("GPU_SINGLE_ALLOC_PERCENT", "100");
        env::set_var("GPU_64BIT_ATOMICS", "1");
        env::set_var("GPU_MAX_WORKGROUP_SIZE", "1024");
        let platform = find_platform(platform_name)
            .ok_or::<ocl::Error>("Can't find OpenCL platform".into())?;
        let p_name = platform.name()?;
        let device = find_device(&platform, device_id)?;
        let mut buffers = HashMap::new();
        buffers.insert(
            "A1".to_string(),
            ClBufferParams {
                size: BUFFER_SIZE_A1,
                flags: MemFlags::empty(),
            },
        );
        buffers.insert(
            "A2".to_string(),
            ClBufferParams {
                size: BUFFER_SIZE_A2,
                flags: MemFlags::empty(),
            },
        );
        buffers.insert(
            "B".to_string(),
            ClBufferParams {
                size: BUFFER_SIZE_B,
                flags: MemFlags::empty(),
            },
        );
        buffers.insert(
            "I1".to_string(),
            ClBufferParams {
                size: INDEX_SIZE,
                flags: MemFlags::empty(),
            },
        );
        buffers.insert(
            "I2".to_string(),
            ClBufferParams {
                size: INDEX_SIZE,
                flags: MemFlags::empty(),
            },
        );
        buffers.insert(
            "R".to_string(),
            ClBufferParams {
                size: 42 * 2,
                flags: MemFlags::READ_ONLY,
            },
        );
        buffers.insert(
            "NONCES".to_string(),
            ClBufferParams {
                size: INDEX_SIZE,
                flags: MemFlags::empty(),
            },
        );

        check_device_compatibility(&device, &buffers)?;

        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()?;

        let q = Queue::new(&context, device, queue_props())?;

        let program = Program::builder()
            .devices(device)
            .src(SRC)
            .build(&context)?;

        let buffer_a1 = build_buffer(buffers.get("A1"), &q)?;
        let buffer_a2 = build_buffer(buffers.get("A2"), &q)?;
        let buffer_b = build_buffer(buffers.get("B"), &q)?;
        let buffer_i1 = build_buffer(buffers.get("I1"), &q)?;
        let buffer_i2 = build_buffer(buffers.get("I2"), &q)?;
        let buffer_r = build_buffer(buffers.get("R"), &q)?;
        let buffer_nonces = build_buffer(buffers.get("NONCES"), &q)?;

        Ok(Trimmer {
            q,
            program,
            buffer_a1,
            buffer_a2,
            buffer_b,
            buffer_i1,
            buffer_i2,
            buffer_r,
            buffer_nonces,
            device_name: device.name()?,
            device_id: device_id.unwrap_or(0),
            is_nvidia: p_name.to_lowercase().contains("nvidia"),
        })
    }

    pub unsafe fn recover(
        &self,
        mut nodes: Vec<u32>,
        k: &[u64; 4],
    ) -> ocl::Result<(Vec<u32>, bool)> {
        let event_list = EventList::new();
        let names = vec![];

        let mut kernel_recovery = kernel_builder!(self, "FluffyRecovery", 2048 * 256)
            .arg(k[0])
            .arg(k[1])
            .arg(k[2])
            .arg(k[3])
            .arg(None::<&Buffer<u64>>)
            .arg(None::<&Buffer<i32>>)
            .build()?;

        if self.is_nvidia {
            kernel_recovery.set_default_local_work_size(SpatialDims::One(256));
        }

        kernel_recovery.set_arg_unchecked(4, ArgVal::mem(&self.buffer_r))?;
        kernel_recovery.set_arg_unchecked(5, ArgVal::mem(&self.buffer_nonces))?;

        nodes.push(nodes[0]);

        let edges = nodes.windows(2).flatten().map(|v| *v).collect::<Vec<u32>>();
        self.buffer_r.cmd().write(edges.as_slice()).enq()?;
        self.buffer_nonces.cmd().fill(0, None).enq()?;
        kernel_enq!(kernel_recovery, event_list, names, "recovery");
        let mut nonces: Vec<u32> = vec![0; 42];

        self.buffer_nonces.cmd().read(&mut nonces).enq()?;
        self.q.finish()?;
        for i in 0..names.len() {
            print_event(names[i], &event_list[i]);
        }
        nonces.sort();
        let valid = nonces.windows(2).all(|entry| match entry {
            [p, n] => p < n,
            _ => true,
        });
        Ok((nonces, valid))
    }

    pub unsafe fn run(&self, k: &[u64; 4]) -> ocl::Result<Vec<u32>> {
        let mut kernel_seed_a = kernel_builder!(self, "FluffySeed2A", 2048 * 128)
            .arg(k[0])
            .arg(k[1])
            .arg(k[2])
            .arg(k[3])
            .arg(None::<&Buffer<Ulong4>>)
            .arg(None::<&Buffer<Ulong4>>)
            .arg(None::<&Buffer<u32>>)
            .build()?;
        if self.is_nvidia {
            kernel_seed_a.set_default_local_work_size(SpatialDims::One(128));
        }
        kernel_seed_a.set_arg_unchecked(4, ArgVal::mem(&self.buffer_b))?;
        kernel_seed_a.set_arg_unchecked(5, ArgVal::mem(&self.buffer_a1))?;
        kernel_seed_a.set_arg_unchecked(6, ArgVal::mem(&self.buffer_i1))?;

        let mut kernel_seed_b1 = kernel_builder!(self, "FluffySeed2B", 1024 * 128)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<Ulong4>>)
            .arg(None::<&Buffer<Ulong4>>)
            .arg(None::<&Buffer<i32>>)
            .arg(None::<&Buffer<i32>>)
            .arg(32)
            .build()?;
        if self.is_nvidia {
            kernel_seed_b1.set_default_local_work_size(SpatialDims::One(128));
        }
        kernel_seed_b1.set_arg_unchecked(0, ArgVal::mem(&self.buffer_a1))?;
        kernel_seed_b1.set_arg_unchecked(1, ArgVal::mem(&self.buffer_a1))?;
        kernel_seed_b1.set_arg_unchecked(2, ArgVal::mem(&self.buffer_a2))?;
        kernel_seed_b1.set_arg_unchecked(3, ArgVal::mem(&self.buffer_i1))?;
        kernel_seed_b1.set_arg_unchecked(4, ArgVal::mem(&self.buffer_i2))?;

        let mut kernel_seed_b2 = kernel_builder!(self, "FluffySeed2B", 1024 * 128)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<Ulong4>>)
            .arg(None::<&Buffer<Ulong4>>)
            .arg(None::<&Buffer<i32>>)
            .arg(None::<&Buffer<i32>>)
            .arg(0)
            .build()?;
        if self.is_nvidia {
            kernel_seed_b2.set_default_local_work_size(SpatialDims::One(128));
        }

        kernel_seed_b2.set_arg_unchecked(0, ArgVal::mem(&self.buffer_b))?;
        kernel_seed_b2.set_arg_unchecked(1, ArgVal::mem(&self.buffer_a1))?;
        kernel_seed_b2.set_arg_unchecked(2, ArgVal::mem(&self.buffer_a2))?;
        kernel_seed_b2.set_arg_unchecked(3, ArgVal::mem(&self.buffer_i1))?;
        kernel_seed_b2.set_arg_unchecked(4, ArgVal::mem(&self.buffer_i2))?;

        let mut kernel_round1 = kernel_builder!(self, "FluffyRound1", 4096 * 1024)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<i32>>)
            .arg(None::<&Buffer<i32>>)
            .arg((DUCK_SIZE_A * 1024) as i32)
            .arg((DUCK_SIZE_B * 1024) as i32)
            .build()?;
        if self.is_nvidia {
            kernel_round1.set_default_local_work_size(SpatialDims::One(1024));
        }

        kernel_round1.set_arg_unchecked(0, ArgVal::mem(&self.buffer_a1))?;
        kernel_round1.set_arg_unchecked(1, ArgVal::mem(&self.buffer_a2))?;
        kernel_round1.set_arg_unchecked(2, ArgVal::mem(&self.buffer_b))?;
        kernel_round1.set_arg_unchecked(3, ArgVal::mem(&self.buffer_i2))?;
        kernel_round1.set_arg_unchecked(4, ArgVal::mem(&self.buffer_i1))?;

        let mut kernel_round0 = kernel_builder!(self, "FluffyRoundNO1", 4096 * 1024)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<i32>>)
            .arg(None::<&Buffer<i32>>)
            .build()?;
        if self.is_nvidia {
            kernel_round0.set_default_local_work_size(SpatialDims::One(1024));
        }
        kernel_round0.set_arg_unchecked(0, ArgVal::mem(&self.buffer_b))?;
        kernel_round0.set_arg_unchecked(1, ArgVal::mem(&self.buffer_a1))?;
        kernel_round0.set_arg_unchecked(2, ArgVal::mem(&self.buffer_i1))?;
        kernel_round0.set_arg_unchecked(3, ArgVal::mem(&self.buffer_i2))?;

        let mut kernel_round_na = kernel_builder!(self, "FluffyRoundNON", 4096 * 1024)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<i32>>)
            .arg(None::<&Buffer<i32>>)
            .build()?;
        if self.is_nvidia {
            kernel_round_na.set_default_local_work_size(SpatialDims::One(1024));
        }
        kernel_round_na.set_arg_unchecked(0, ArgVal::mem(&self.buffer_b))?;
        kernel_round_na.set_arg_unchecked(1, ArgVal::mem(&self.buffer_a1))?;
        kernel_round_na.set_arg_unchecked(2, ArgVal::mem(&self.buffer_i1))?;
        kernel_round_na.set_arg_unchecked(3, ArgVal::mem(&self.buffer_i2))?;

        let mut kernel_round_nb = kernel_builder!(self, "FluffyRoundNON", 4096 * 1024)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<i32>>)
            .arg(None::<&Buffer<i32>>)
            .build()?;
        if self.is_nvidia {
            kernel_round_nb.set_default_local_work_size(SpatialDims::One(1024));
        }
        kernel_round_nb.set_arg_unchecked(0, ArgVal::mem(&self.buffer_a1))?;
        kernel_round_nb.set_arg_unchecked(1, ArgVal::mem(&self.buffer_b))?;
        kernel_round_nb.set_arg_unchecked(2, ArgVal::mem(&self.buffer_i2))?;
        kernel_round_nb.set_arg_unchecked(3, ArgVal::mem(&self.buffer_i1))?;

        let mut kernel_tail = kernel_builder!(self, "FluffyTailO", 4096 * 1024)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<Uint2>>)
            .arg(None::<&Buffer<i32>>)
            .arg(None::<&Buffer<i32>>)
            .build()?;
        if self.is_nvidia {
            kernel_tail.set_default_local_work_size(SpatialDims::One(1024));
        }
        kernel_tail.set_arg_unchecked(0, ArgVal::mem(&self.buffer_b))?;
        kernel_tail.set_arg_unchecked(1, ArgVal::mem(&self.buffer_a1))?;
        kernel_tail.set_arg_unchecked(2, ArgVal::mem(&self.buffer_i1))?;
        kernel_tail.set_arg_unchecked(3, ArgVal::mem(&self.buffer_i2))?;

        let event_list = EventList::new();
        let names = vec![];

        let mut edges_count: Vec<u32> = vec![0; 1];
        clear_buffer!(self.buffer_i1);
        clear_buffer!(self.buffer_i2);
        kernel_enq!(kernel_seed_a, event_list, names, "seedA");
        kernel_enq!(kernel_seed_b1, event_list, names, "seedB1");
        kernel_enq!(kernel_seed_b2, event_list, names, "seedB2");
        clear_buffer!(self.buffer_i1);
        kernel_enq!(kernel_round1, event_list, names, "round1");
        clear_buffer!(self.buffer_i2);
        kernel_enq!(kernel_round0, event_list, names, "roundN0");
        clear_buffer!(self.buffer_i1);
        kernel_enq!(kernel_round_nb, event_list, names, "roundNB");
        for _ in 0..120 {
            clear_buffer!(self.buffer_i2);
            kernel_enq!(kernel_round_na, event_list, names, "roundNA");
            clear_buffer!(self.buffer_i1);
            kernel_enq!(kernel_round_nb, event_list, names, "roundNB");
        }
        clear_buffer!(self.buffer_i2);
        kernel_enq!(kernel_tail, event_list, names, "tail");

        self.buffer_i2.cmd().read(&mut edges_count).enq()?;

        let mut edges_left: Vec<u32> = vec![0; (edges_count[0] * 2) as usize];

        self.buffer_a1.cmd().read(&mut edges_left).enq()?;
        self.q.finish()?;
        for i in 0..names.len() {
            print_event(names[i], &event_list[i]);
        }
        clear_buffer!(self.buffer_i1);
        clear_buffer!(self.buffer_i2);
        self.q.finish()?;
        Ok(edges_left)
    }
}

#[cfg(feature = "profile")]
fn print_event(name: &str, ev: &Event) {
    let submit = ev
        .profiling_info(ProfilingInfo::Submit)
        .unwrap()
        .time()
        .unwrap();
    let queued = ev
        .profiling_info(ProfilingInfo::Queued)
        .unwrap()
        .time()
        .unwrap();
    let start = ev
        .profiling_info(ProfilingInfo::Start)
        .unwrap()
        .time()
        .unwrap();
    let end = ev
        .profiling_info(ProfilingInfo::End)
        .unwrap()
        .time()
        .unwrap();
    println!(
        "{}\t total {}ms \t queued->submit {}mc \t submit->start {}ms \t start->end {}ms",
        name,
        (end - queued) / 1_000_000,
        (submit - queued) / 1_000,
        (start - submit) / 1_000_000,
        (end - start) / 1_000_000
    );
}

#[cfg(not(feature = "profile"))]
fn print_event(_name: &str, _ev: &Event) {}

fn find_platform(selector: Option<&str>) -> Option<Platform> {
    match selector {
        None => Some(Platform::default()),
        Some(sel) => Platform::list().into_iter().find(|p| {
            if let Ok(vendor) = p.name() {
                vendor.contains(sel)
            } else {
                false
            }
        }),
    }
}

fn find_device(platform: &Platform, selector: Option<usize>) -> ocl::Result<Device> {
    match selector {
        None => Device::first(platform),
        Some(index) => Device::by_idx_wrap(platform, index),
    }
}

fn check_device_compatibility(
    device: &Device,
    buffers: &HashMap<String, ClBufferParams>,
) -> ocl::Result<()> {
    let max_alloc_size: u64 = get_device_info!(device, MaxMemAllocSize);
    let global_memory_size: u64 = get_device_info!(device, GlobalMemSize);
    let mut total_alloc: u64 = 0;

    // Check that no buffer is bigger than the max memory allocation size
    for (k, v) in buffers {
        total_alloc += v.size as u64;
        if v.size as u64 > max_alloc_size {
            return Err(ocl::Error::from(format!(
                "Buffer {} is bigger than maximum alloc size ({})",
                k, max_alloc_size
            )));
        }
    }

    // Check that total buffer allocation does not exceed global memory size
    if total_alloc > global_memory_size {
        return Err(ocl::Error::from(format!(
            "Total needed memory is bigger than device's capacity ({})",
            global_memory_size
        )));
    }

    Ok(())
}

fn build_buffer(params: Option<&ClBufferParams>, q: &Queue) -> ocl::Result<Buffer<u32>> {
    match params {
        None => Err(ocl::Error::from("Invalid parameters")),
        Some(p) => Buffer::<u32>::builder()
            .queue(q.clone())
            .len(p.size)
            .flags(p.flags)
            .fill_val(0)
            .build(),
    }
}

const SRC: &str = r#"
// Cuckaroo Cycle, a memory-hard proof-of-work by John Tromp and team Grin
// Copyright (c) 2018 Jiri Photon Vadura and John Tromp
// This GGM miner file is covered by the FAIR MINING license

#pragma OPENCL EXTENSION cl_khr_int64_base_atomics : enable
#pragma OPENCL EXTENSION cl_khr_int64_extended_atomics : enable

typedef uint8 u8;
typedef uint16 u16;
typedef uint u32;
typedef ulong u64;

typedef u32 node_t;
typedef u64 nonce_t;


#define DUCK_SIZE_A 129L
#define DUCK_SIZE_B 83L

#define DUCK_A_EDGES (DUCK_SIZE_A * 1024L)
#define DUCK_A_EDGES_64 (DUCK_A_EDGES * 64L)

#define DUCK_B_EDGES (DUCK_SIZE_B * 1024L)
#define DUCK_B_EDGES_64 (DUCK_B_EDGES * 64L)

#define EDGE_BLOCK_SIZE (64)
#define EDGE_BLOCK_MASK (EDGE_BLOCK_SIZE - 1)

#define EDGEBITS 29
// number of edges
#define NEDGES ((node_t)1 << EDGEBITS)
// used to mask siphash output
#define EDGEMASK (NEDGES - 1)

#define CTHREADS 1024
#define BKTMASK4K (4096-1)
#define BKTGRAN 32

#define SIPROUND \
  do { \
    v0 += v1; v2 += v3; v1 = rotate(v1,(ulong)13); \
    v3 = rotate(v3,(ulong)16); v1 ^= v0; v3 ^= v2; \
    v0 = rotate(v0,(ulong)32); v2 += v1; v0 += v3; \
    v1 = rotate(v1,(ulong)17);   v3 = rotate(v3,(ulong)21); \
    v1 ^= v2; v3 ^= v0; v2 = rotate(v2,(ulong)32); \
  } while(0)


void Increase2bCounter(__local u32 * ecounters, const int bucket)
{
	int word = bucket >> 5;
	unsigned char bit = bucket & 0x1F;
	u32 mask = 1 << bit;

	u32 old = atomic_or(ecounters + word, mask) & mask;

	if (old > 0)
		atomic_or(ecounters + word + 4096, mask);
}

bool Read2bCounter(__local u32 * ecounters, const int bucket)
{
	int word = bucket >> 5;
	unsigned char bit = bucket & 0x1F;
	u32 mask = 1 << bit;

	return (ecounters[word + 4096] & mask) > 0;
}

__attribute__((reqd_work_group_size(128, 1, 1)))
__kernel  void FluffySeed2A(const u64 v0i, const u64 v1i, const u64 v2i, const u64 v3i, __global ulong4 * bufferA, __global ulong4 * bufferB, __global u32 * indexes)
{
	const int gid = get_global_id(0);
	const short lid = get_local_id(0);

	__global ulong4 * buffer;
	__local u64 tmp[64][16];
	__local u32 counters[64];
	u64 sipblock[64];

	u64 v0;
	u64 v1;
	u64 v2;
	u64 v3;

	if (lid < 64)
		counters[lid] = 0;

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < 1024 * 2; i += EDGE_BLOCK_SIZE)
	{
		u64 blockNonce = gid * (1024 * 2) + i;

		v0 = v0i;
		v1 = v1i;
		v2 = v2i;
		v3 = v3i;

		for (u32 b = 0; b < EDGE_BLOCK_SIZE; b++)
		{
			v3 ^= blockNonce + b;
			for (int r = 0; r < 2; r++)
				SIPROUND;
			v0 ^= blockNonce + b;
			v2 ^= 0xff;
			for (int r = 0; r < 4; r++)
				SIPROUND;

			sipblock[b] = (v0 ^ v1) ^ (v2  ^ v3);

		}
		u64 last = sipblock[EDGE_BLOCK_MASK];

		for (short s = 0; s < EDGE_BLOCK_SIZE; s++)
		{
			u64 lookup = s == EDGE_BLOCK_MASK ? last : sipblock[s] ^ last;
			uint2 hash = (uint2)(lookup & EDGEMASK, (lookup >> 32) & EDGEMASK);
			int bucket = hash.x & 63;

			barrier(CLK_LOCAL_MEM_FENCE);

			int counter = atomic_add(counters + bucket, (u32)1);
			int counterLocal = counter % 16;
			tmp[bucket][counterLocal] = hash.x | ((u64)hash.y << 32);

			barrier(CLK_LOCAL_MEM_FENCE);

			if ((counter > 0) && (counterLocal == 0 || counterLocal == 8))
			{
				int cnt = min((int)atomic_add(indexes + bucket, 8), (int)(DUCK_A_EDGES_64 - 8));
				int idx = ((bucket < 32 ? bucket : bucket - 32) * DUCK_A_EDGES_64 + cnt) / 4;
				buffer = bucket < 32 ? bufferA : bufferB;

				buffer[idx] = (ulong4)(
					atom_xchg(&tmp[bucket][8 - counterLocal], (u64)0),
					atom_xchg(&tmp[bucket][9 - counterLocal], (u64)0),
					atom_xchg(&tmp[bucket][10 - counterLocal], (u64)0),
					atom_xchg(&tmp[bucket][11 - counterLocal], (u64)0)
				);
				buffer[idx + 1] = (ulong4)(
					atom_xchg(&tmp[bucket][12 - counterLocal], (u64)0),
					atom_xchg(&tmp[bucket][13 - counterLocal], (u64)0),
					atom_xchg(&tmp[bucket][14 - counterLocal], (u64)0),
					atom_xchg(&tmp[bucket][15 - counterLocal], (u64)0)
				);
			}

		}
	}

	barrier(CLK_LOCAL_MEM_FENCE);

	if (lid < 64)
	{
		int counter = counters[lid];
		int counterBase = (counter % 16) >= 8 ? 8 : 0;
		int counterCount = (counter % 8);
		for (int i = 0; i < (8 - counterCount); i++)
			tmp[lid][counterBase + counterCount + i] = 0;
		int cnt = min((int)atomic_add(indexes + lid, 8), (int)(DUCK_A_EDGES_64 - 8));
		int idx = ( (lid < 32 ? lid : lid - 32) * DUCK_A_EDGES_64 + cnt) / 4;
		buffer = lid < 32 ? bufferA : bufferB;
		buffer[idx] = (ulong4)(tmp[lid][counterBase], tmp[lid][counterBase + 1], tmp[lid][counterBase + 2], tmp[lid][counterBase + 3]);
		buffer[idx + 1] = (ulong4)(tmp[lid][counterBase + 4], tmp[lid][counterBase + 5], tmp[lid][counterBase + 6], tmp[lid][counterBase + 7]);
	}

}

__attribute__((reqd_work_group_size(128, 1, 1)))
__kernel  void FluffySeed2B(const __global uint2 * source, __global ulong4 * destination1, __global ulong4 * destination2, const __global int * sourceIndexes, __global int * destinationIndexes, int startBlock)
{
	const int lid = get_local_id(0);
	const int group = get_group_id(0);

	__global ulong4 * destination = destination1;
	__local u64 tmp[64][16];
	__local int counters[64];

	if (lid < 64)
		counters[lid] = 0;

	barrier(CLK_LOCAL_MEM_FENCE);

	int offsetMem = startBlock * DUCK_A_EDGES_64;
	int offsetBucket = 0;
	const int myBucket = group / BKTGRAN;
	const int microBlockNo = group % BKTGRAN;
	const int bucketEdges = min(sourceIndexes[myBucket + startBlock], (int)(DUCK_A_EDGES_64));
	const int microBlockEdgesCount = (DUCK_A_EDGES_64 / BKTGRAN);
	const int loops = (microBlockEdgesCount / 128);

	if ((startBlock == 32) && (myBucket >= 30))
	{
		offsetMem = 0;
		destination = destination2;
		offsetBucket = 30;
	}

	for (int i = 0; i < loops; i++)
	{
		int edgeIndex = (microBlockNo * microBlockEdgesCount) + (128 * i) + lid;

		{
			uint2 edge = source[/*offsetMem + */(myBucket * DUCK_A_EDGES_64) + edgeIndex];
			bool skip = (edgeIndex >= bucketEdges) || (edge.x == 0 && edge.y == 0);

			int bucket = (edge.x >> 6) & (64 - 1);

			barrier(CLK_LOCAL_MEM_FENCE);

			int counter = 0;
			int counterLocal = 0;

			if (!skip)
			{
				counter = atomic_add(counters + bucket, (u32)1);
				counterLocal = counter % 16;
				tmp[bucket][counterLocal] = edge.x | ((u64)edge.y << 32);
			}

			barrier(CLK_LOCAL_MEM_FENCE);

			if ((counter > 0) && (counterLocal == 0 || counterLocal == 8))
			{
				int cnt = min((int)atomic_add(destinationIndexes + startBlock * 64 + myBucket * 64 + bucket, 8), (int)(DUCK_A_EDGES - 8));
				int idx = (offsetMem + (((myBucket - offsetBucket) * 64 + bucket) * DUCK_A_EDGES + cnt)) / 4;

				destination[idx] = (ulong4)(
					atom_xchg(&tmp[bucket][8 - counterLocal], 0),
					atom_xchg(&tmp[bucket][9 - counterLocal], 0),
					atom_xchg(&tmp[bucket][10 - counterLocal], 0),
					atom_xchg(&tmp[bucket][11 - counterLocal], 0)
				);
				destination[idx + 1] = (ulong4)(
					atom_xchg(&tmp[bucket][12 - counterLocal], 0),
					atom_xchg(&tmp[bucket][13 - counterLocal], 0),
					atom_xchg(&tmp[bucket][14 - counterLocal], 0),
					atom_xchg(&tmp[bucket][15 - counterLocal], 0)
				);
			}
		}
	}

	barrier(CLK_LOCAL_MEM_FENCE);

	if (lid < 64)
	{
		int counter = counters[lid];
		int counterBase = (counter % 16) >= 8 ? 8 : 0;
		int cnt = min((int)atomic_add(destinationIndexes + startBlock * 64 + myBucket * 64 + lid, 8), (int)(DUCK_A_EDGES - 8));
		int idx = (offsetMem + (((myBucket - offsetBucket) * 64 + lid) * DUCK_A_EDGES + cnt)) / 4;
		destination[idx] = (ulong4)(tmp[lid][counterBase], tmp[lid][counterBase + 1], tmp[lid][counterBase + 2], tmp[lid][counterBase + 3]);
		destination[idx + 1] = (ulong4)(tmp[lid][counterBase + 4], tmp[lid][counterBase + 5], tmp[lid][counterBase + 6], tmp[lid][counterBase + 7]);
	}
}

__attribute__((reqd_work_group_size(1024, 1, 1)))
__kernel   void FluffyRound1(const __global uint2 * source1, const __global uint2 * source2, __global uint2 * destination, const __global int * sourceIndexes, __global int * destinationIndexes, const int bktInSize, const int bktOutSize)
{
	const int lid = get_local_id(0);
	const int group = get_group_id(0);

	const __global uint2 * source = group < (62 * 64) ? source1 : source2;
	int groupRead                 = group < (62 * 64) ? group : group - (62 * 64);

	__local u32 ecounters[8192];

	const int edgesInBucket = min(sourceIndexes[group], bktInSize);
	const int loops = (edgesInBucket + CTHREADS) / CTHREADS;

	for (int i = 0; i < 8; i++)
		ecounters[lid + (1024 * i)] = 0;

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < loops; i++)
	{
		const int lindex = (i * CTHREADS) + lid;

		if (lindex < edgesInBucket)
		{

			const int index = (bktInSize * groupRead) + lindex;

			uint2 edge = source[index];

			if (edge.x == 0 && edge.y == 0) continue;

			Increase2bCounter(ecounters, (edge.x & EDGEMASK) >> 12);
		}
	}

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < loops; i++)
	{
		const int lindex = (i * CTHREADS) + lid;

		if (lindex < edgesInBucket)
		{
			const int index = (bktInSize * groupRead) + lindex;

			uint2 edge = source[index];

			if (edge.x == 0 && edge.y == 0) continue;

			if (Read2bCounter(ecounters, (edge.x & EDGEMASK) >> 12))
			{
				const int bucket = edge.y & BKTMASK4K;
				const int bktIdx = min(atomic_add(destinationIndexes + bucket, 1), bktOutSize - 1);
				destination[(bucket * bktOutSize) + bktIdx] = (uint2)(edge.y, edge.x);
			}
		}
	}

}

__attribute__((reqd_work_group_size(1024, 1, 1)))
__kernel   void FluffyRoundN(const __global uint2 * source, __global uint2 * destination, const __global int * sourceIndexes, __global int * destinationIndexes)
{
	const int lid = get_local_id(0);
	const int group = get_group_id(0);

	const int bktInSize = DUCK_B_EDGES;
	const int bktOutSize = DUCK_B_EDGES;

	__local u32 ecounters[8192];

	const int edgesInBucket = min(sourceIndexes[group], bktInSize);
	const int loops = (edgesInBucket + CTHREADS) / CTHREADS;

	for (int i = 0; i < 8; i++)
		ecounters[lid + (1024 * i)] = 0;

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < loops; i++)
	{
		const int lindex = (i * CTHREADS) + lid;

		if (lindex < edgesInBucket)
		{

			const int index = (bktInSize * group) + lindex;

			uint2 edge = source[index];

			if (edge.x == 0 && edge.y == 0) continue;

			Increase2bCounter(ecounters, (edge.x & EDGEMASK) >> 12);
		}
	}

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < loops; i++)
	{
		const int lindex = (i * CTHREADS) + lid;

		if (lindex < edgesInBucket)
		{
			const int index = (bktInSize * group) + lindex;

			uint2 edge = source[index];

			if (edge.x == 0 && edge.y == 0) continue;

			if (Read2bCounter(ecounters, (edge.x & EDGEMASK) >> 12))
			{
				const int bucket = edge.y & BKTMASK4K;
				const int bktIdx = min(atomic_add(destinationIndexes + bucket, 1), bktOutSize - 1);
				destination[(bucket * bktOutSize) + bktIdx] = (uint2)(edge.y, edge.x);
			}
		}
	}

}

__attribute__((reqd_work_group_size(64, 1, 1)))
__kernel   void FluffyRoundN_64(const __global uint2 * source, __global uint2 * destination, const __global int * sourceIndexes, __global int * destinationIndexes)
{
	const int lid = get_local_id(0);
	const int group = get_group_id(0);

	const int bktInSize = DUCK_B_EDGES;
	const int bktOutSize = DUCK_B_EDGES;

	__local u32 ecounters[8192];

	const int edgesInBucket = min(sourceIndexes[group], bktInSize);
	const int loops = (edgesInBucket + 64) / 64;

	for (int i = 0; i < 8*16; i++)
		ecounters[lid + (64 * i)] = 0;

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < loops; i++)
	{
		const int lindex = (i * 64) + lid;

		if (lindex < edgesInBucket)
		{

			const int index = (bktInSize * group) + lindex;

			uint2 edge = source[index];

			if (edge.x == 0 && edge.y == 0) continue;

			Increase2bCounter(ecounters, (edge.x & EDGEMASK) >> 12);
		}
	}

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < loops; i++)
	{
		const int lindex = (i * 64) + lid;

		if (lindex < edgesInBucket)
		{
			const int index = (bktInSize * group) + lindex;

			uint2 edge = source[index];

			if (edge.x == 0 && edge.y == 0) continue;

			if (Read2bCounter(ecounters, (edge.x & EDGEMASK) >> 12))
			{
				const int bucket = edge.y & BKTMASK4K;
				const int bktIdx = min(atomic_add(destinationIndexes + bucket, 1), bktOutSize - 1);
				destination[(bucket * bktOutSize) + bktIdx] = (uint2)(edge.y, edge.x);
			}
		}
	}

}

__attribute__((reqd_work_group_size(1024, 1, 1)))
__kernel void FluffyTail(const __global uint2 * source, __global uint2 * destination, const __global int * sourceIndexes, __global int * destinationIndexes)
{
	const int lid = get_local_id(0);
	const int group = get_group_id(0);

	int myEdges = sourceIndexes[group];
	__local int destIdx;

	if (lid == 0)
		destIdx = atomic_add(destinationIndexes, myEdges);

	barrier(CLK_LOCAL_MEM_FENCE);

	if (lid < myEdges)
	{
		destination[destIdx + lid] = source[group * DUCK_B_EDGES + lid];
	}
}

__attribute__((reqd_work_group_size(256, 1, 1)))
__kernel   void FluffyRecovery(const u64 v0i, const u64 v1i, const u64 v2i, const u64 v3i, const __constant u64 * recovery, __global int * indexes)
{
	const int gid = get_global_id(0);
	const short lid = get_local_id(0);

	__local u32 nonces[42];
	u64 sipblock[64];

	u64 v0;
	u64 v1;
	u64 v2;
	u64 v3;

	if (lid < 42) nonces[lid] = 0;

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < 1024; i += EDGE_BLOCK_SIZE)
	{
		u64 blockNonce = gid * 1024 + i;

		v0 = v0i;
		v1 = v1i;
		v2 = v2i;
		v3 = v3i;

		for (u32 b = 0; b < EDGE_BLOCK_SIZE; b++)
		{
			v3 ^= blockNonce + b;
			SIPROUND; SIPROUND;
			v0 ^= blockNonce + b;
			v2 ^= 0xff;
			SIPROUND; SIPROUND; SIPROUND; SIPROUND;

			sipblock[b] = (v0 ^ v1) ^ (v2  ^ v3);

		}
		const u64 last = sipblock[EDGE_BLOCK_MASK];

		for (short s = EDGE_BLOCK_MASK; s >= 0; s--)
		{
			u64 lookup = s == EDGE_BLOCK_MASK ? last : sipblock[s] ^ last;
			u64 u = lookup & EDGEMASK;
			u64 v = (lookup >> 32) & EDGEMASK;

			u64 a = u | (v << 32);
			u64 b = v | (u << 32);

			for (int i = 0; i < 42; i++)
			{
				if ((recovery[i] == a) || (recovery[i] == b))
					nonces[i] = blockNonce + s;
			}
		}
	}

	barrier(CLK_LOCAL_MEM_FENCE);

	if (lid < 42)
	{
		if (nonces[lid] > 0)
			indexes[lid] = nonces[lid];
	}
}



// ---------------
#define BKT_OFFSET 255
#define BKT_STEP 32
__attribute__((reqd_work_group_size(1024, 1, 1)))
__kernel   void FluffyRoundNO1(const __global uint2 * source, __global uint2 * destination, const __global int * sourceIndexes, __global int * destinationIndexes)
{
	const int lid = get_local_id(0);
	const int group = get_group_id(0);

	const int bktInSize = DUCK_B_EDGES;
	const int bktOutSize = DUCK_B_EDGES;

	__local u32 ecounters[8192];

	const int edgesInBucket = min(sourceIndexes[group], bktInSize);
	const int loops = (edgesInBucket + CTHREADS) / CTHREADS;

	for (int i = 0; i < 8; i++)
		ecounters[lid + (1024 * i)] = 0;

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < loops; i++)
	{
		const int lindex = (i * CTHREADS) + lid;

		if (lindex < edgesInBucket)
		{

			const int index =(bktInSize * group) + lindex;

			uint2 edge = source[index];

			if (edge.x == 0 && edge.y == 0) continue;

			Increase2bCounter(ecounters, (edge.x & EDGEMASK) >> 12);
		}
	}

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < loops; i++)
	{
		const int lindex = (i * CTHREADS) + lid;

		if (lindex < edgesInBucket)
		{
			const int index = (bktInSize * group) + lindex;

			uint2 edge = source[index];

			if (edge.x == 0 && edge.y == 0) continue;

			if (Read2bCounter(ecounters, (edge.x & EDGEMASK) >> 12))
			{
				const int bucket = edge.y & BKTMASK4K;
				const int bktIdx = min(atomic_add(destinationIndexes + bucket, 1), bktOutSize - 1 - ((bucket & BKT_OFFSET) * BKT_STEP));
				destination[((bucket & BKT_OFFSET) * BKT_STEP) + (bucket * bktOutSize) + bktIdx] = (uint2)(edge.y, edge.x);
			}
		}
	}

}

__attribute__((reqd_work_group_size(1024, 1, 1)))
__kernel   void FluffyRoundNON(const __global uint2 * source, __global uint2 * destination, const __global int * sourceIndexes, __global int * destinationIndexes)
{
	const int lid = get_local_id(0);
	const int group = get_group_id(0);

	const int bktInSize = DUCK_B_EDGES;
	const int bktOutSize = DUCK_B_EDGES;

	__local u32 ecounters[8192];

	const int edgesInBucket = min(sourceIndexes[group], bktInSize);
	const int loops = (edgesInBucket + CTHREADS) / CTHREADS;

	for (int i = 0; i < 8; i++)
		ecounters[lid + (1024 * i)] = 0;

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < loops; i++)
	{
		const int lindex = (i * CTHREADS) + lid;

		if (lindex < edgesInBucket)
		{

			const int index = ((group & BKT_OFFSET) * BKT_STEP) + (bktInSize * group) + lindex;

			uint2 edge = source[index];

			if (edge.x == 0 && edge.y == 0) continue;

			Increase2bCounter(ecounters, (edge.x & EDGEMASK) >> 12);
		}
	}

	barrier(CLK_LOCAL_MEM_FENCE);

	for (int i = 0; i < loops; i++)
	{
		const int lindex = (i * CTHREADS) + lid;

		if (lindex < edgesInBucket)
		{
			const int index = ((group & BKT_OFFSET) * BKT_STEP) + (bktInSize * group) + lindex;

			uint2 edge = source[index];

			if (edge.x == 0 && edge.y == 0) continue;

			if (Read2bCounter(ecounters, (edge.x & EDGEMASK) >> 12))
			{
				const int bucket = edge.y & BKTMASK4K;
				const int bktIdx = min(atomic_add(destinationIndexes + bucket, 1), bktOutSize - 1 - ((bucket & BKT_OFFSET) * BKT_STEP));
				destination[((bucket & BKT_OFFSET) * BKT_STEP) + (bucket * bktOutSize) + bktIdx] = (uint2)(edge.y, edge.x);
			}
		}
	}

}

__attribute__((reqd_work_group_size(1024, 1, 1)))
__kernel void FluffyTailO(const __global uint2 * source, __global uint2 * destination, const __global int * sourceIndexes, __global int * destinationIndexes)
{
	const int lid = get_local_id(0);
	const int group = get_group_id(0);

	int myEdges = sourceIndexes[group];
	__local int destIdx;

	if (lid == 0)
		destIdx = atomic_add(destinationIndexes, myEdges);

	barrier(CLK_LOCAL_MEM_FENCE);

	if (lid < myEdges)
	{
		destination[destIdx + lid] = source[((group & BKT_OFFSET) * BKT_STEP) + group * DUCK_B_EDGES + lid];
	}
}

"#;
