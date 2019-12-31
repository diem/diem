use ocl;
use ocl::{Buffer, Context, Device, Kernel, Platform, Program, Queue, SpatialDims};

const RES_BUFFER_SIZE: usize = 4_000_000;
const LOCAL_WORK_SIZE: usize = 256;
const GLOBAL_WORK_SIZE: usize = 1024 * LOCAL_WORK_SIZE;

enum Mode {
    SetCnt = 1,
    Trim = 2,
    Extract = 3,
}

pub struct Trimmer {
    edge_bits: u8,
    q: Queue,
    program: Program,
    edges: Buffer<u32>,
    counters: Buffer<u32>,
    result: Buffer<u32>,
    res_buf: Vec<u32>,
    pub device_name: String,
    pub device_id: usize,
}

impl Trimmer {
    pub fn build(
        platform_name: Option<&str>,
        device_id: Option<usize>,
        edge_bits: u8,
    ) -> ocl::Result<Trimmer> {
        let platform = find_platform(platform_name)
            .ok_or::<ocl::Error>("Can't find OpenCL platform".into())?;
        let device = find_device(&platform, device_id)?;

        let el_count = (1024 * 1024 * 16) << (edge_bits - 29);
        let res_buf: Vec<u32> = vec![0; RES_BUFFER_SIZE];

        let context = Context::builder()
            .platform(platform)
            .devices(device)
            .build()?;

        let q = Queue::new(&context, device, None)?;

        let program = Program::builder()
            .devices(device)
            .src(SRC)
            .cmplr_def("EDGEBITS", edge_bits as i32)
            .build(&context)?;

        let edges = Buffer::<u32>::builder()
            .queue(q.clone())
            .len(el_count)
            .fill_val(0xFFFFFFFF)
            .build()?;
        let counters = Buffer::<u32>::builder()
            .queue(q.clone())
            .len(el_count)
            .fill_val(0)
            .build()?;
        let result = unsafe {
            Buffer::<u32>::builder()
                .queue(q.clone())
                .len(RES_BUFFER_SIZE)
                .fill_val(0)
                .use_host_slice(&res_buf[..])
                .build()?
        };

        Ok(Trimmer {
            edge_bits,
            q,
            program,
            edges,
            counters,
            result,
            res_buf,
            device_name: device.name()?,
            device_id: device_id.unwrap_or(0),
        })
    }

    pub fn run(&self, k: &[u64; 4]) -> ocl::Result<Vec<u32>> {
        let mut current_mode = Mode::SetCnt;
        let mut current_uorv: u32 = 0;
        let trims = if self.edge_bits >= 29 { 128 } else { 256 };
        let enqs = 8 << (self.edge_bits - 29);

        let mut kernel = Kernel::builder()
            .name("LeanRound")
            .program(&self.program)
            .queue(self.q.clone())
            .global_work_size(GLOBAL_WORK_SIZE)
            .local_work_size(SpatialDims::One(LOCAL_WORK_SIZE))
            .arg(k[0])
            .arg(k[1])
            .arg(k[2])
            .arg(k[3])
            .arg(&self.edges)
            .arg(&self.counters)
            .arg(&self.result)
            .arg(current_mode as u32)
            .arg(current_uorv)
            .build()?;

        let mut offset;

        macro_rules! kernel_enq (
        ($num:expr) => (
        for i in 0..$num {
            offset = i * GLOBAL_WORK_SIZE;
            unsafe {
                kernel
                    .set_default_global_work_offset(SpatialDims::One(offset))
                    .enq()?;
            }
        }
        ));

        for l in 0..trims {
            current_uorv = l & 1 as u32;
            current_mode = Mode::SetCnt;
            kernel.set_arg(7, current_mode as u32)?;
            kernel.set_arg(8, current_uorv)?;
            kernel_enq!(enqs);

            current_mode = if l == (trims - 1) {
                Mode::Extract
            } else {
                Mode::Trim
            };
            kernel.set_arg(7, current_mode as u32)?;
            kernel_enq!(enqs);
            // prepare for the next round
            self.counters.cmd().fill(0, None).enq()?;
        }
        unsafe {
            self.result.map().enq()?;
        }
        self.q.finish()?;
        let ret = self.res_buf.clone();
        self.edges.cmd().fill(0xFFFFFFFF, None).enq()?;
        self.result.cmd().fill(0, None).enq()?;
        self.q.finish()?;
        Ok(ret)
    }
}

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

const SRC: &str = r#"
typedef uint8 u8;
typedef uint16 u16;
typedef uint u32;
typedef ulong u64;
typedef u32 node_t;
typedef u64 nonce_t;

#define DEBUG 0

// number of edges
#define NEDGES ((u64)1 << EDGEBITS)
// used to mask siphash output
#define EDGEMASK (NEDGES - 1)

#define SIPROUND \
  do { \
    v0 += v1; v2 += v3; v1 = rotate(v1,(ulong)13); \
    v3 = rotate(v3,(ulong)16); v1 ^= v0; v3 ^= v2; \
    v0 = rotate(v0,(ulong)32); v2 += v1; v0 += v3; \
    v1 = rotate(v1,(ulong)17);   v3 = rotate(v3,(ulong)21); \
    v1 ^= v2; v3 ^= v0; v2 = rotate(v2,(ulong)32); \
  } while(0)

u64 dipnode(ulong v0i, ulong v1i, ulong v2i, ulong v3i, u64 nce, uint uorv) {
	ulong nonce = 2 * nce + uorv;
	ulong v0 = v0i, v1 = v1i, v2 = v2i, v3 = v3i ^ nonce;
	SIPROUND; SIPROUND;
	v0 ^= nonce;
	v2 ^= 0xff;
	SIPROUND; SIPROUND; SIPROUND; SIPROUND;
	return (v0 ^ v1 ^ v2  ^ v3) & EDGEMASK;
}

#define MODE_SETCNT 1
#define MODE_TRIM 2
#define MODE_EXTRACT 3

// Minimalistic cuckatoo lean trimmer
// This implementation is not optimal!
//
// 8 global kernel executions (hardcoded ATM)
// 1024 thread blocks, 256 threads each, 256 edges for each thread
// 8*1024*256*256 = 536 870 912 edges = cuckatoo29
__attribute__((reqd_work_group_size(256, 1, 1)))
__kernel  void LeanRound(const u64 v0i, const u64 v1i, const u64 v2i, const u64 v3i, __global uint8 * edges, __global uint * counters, __global u32 * aux, const u32 mode, const u32 uorv)
{
	const int blocks = NEDGES / 32;
	const int gid = get_global_id(0);
	const int lid = get_local_id(0);
	__local u32 el[256][8];

	{
		int lCount = 0;
		// what 256 nit block of edges are we processing
		u64 index = gid;
		u64 start = index * 256;
		// load all 256 bits (edges) to registers
		uint8 load = edges[index];
		// map to an array for easier indexing (depends on compiler/GPU, could be pushed out to cache)
		el[lid][0] = load.s0;
		el[lid][1] = load.s1;
		el[lid][2] = load.s2;
		el[lid][3] = load.s3;
		el[lid][4] = load.s4;
		el[lid][5] = load.s5;
		el[lid][6] = load.s6;
		el[lid][7] = load.s7;
		
		// process as 8 x 32bit segment, GPUs have 32bit ALUs 
		for (short i = 0; i < 8; i++)
		{
			// shortcut to current 32bit value
			uint ee = el[lid][i];
			// how many edges we process in the block
			short lEdges = popcount(ee);
			// whole warp will always execute worst case scenario, but it will help in the long run (not benched)
			
			// now a loop for every single living edge in current 32 edge block
			for (short e = 0; e < lEdges; e++)
			{
				// bit position of next living edge
				short pos = clz(ee);
				// position in the 256 edge block
				int subPos = (i * 32) + pos;
				// reconstruct value of noce for this edge
				int nonce = start + subPos;
				// calculate siphash24 for either U or V (host device control)
				u32 hash = dipnode(v0i, v1i, v2i, v3i, nonce, uorv);
				
				// this time we set edge bit counters - PASS 1
				if (mode == MODE_SETCNT)
				{
					// what global memory 32bit block we need to access
					int block = hash / 32;
					// what bit in the block we need to set
					u32 bit = hash % 32;
					// create a bitmask from that bit
					u32 mask = (u32)1 << bit;
					// global atomic or (set bit to 1 no matter what it was)
					atomic_or(&counters[block], mask);
				}
				// this time counters are already set so need to figure out if the edge lives - PASS 2
				else if ((mode == MODE_TRIM) || (mode == MODE_EXTRACT))
				{
					// cuckatoo XOR thing
					hash = hash ^ 1;
					// what global memory 32bit block we need to read
					int block = hash / 32;
					// what bit in the block we need to read
					u32 bit = hash % 32;
					// create a bitmask from that bit
					u32 mask = (u32)1 << bit;
					// does the edge live or not
					bool lives = ((counters[block]) & mask) > 0;
					// if edge is not alive, kill it (locally in registers)
					if (!lives)
					{
						el[lid][i] ^= ((u32)1<<31) >> pos; // 1 XOR 1 is 0
					}
					else
					{
						// debug counter of alive edges
						if (DEBUG)
							lCount++;

						// if this is last lean round we do, store all edges in one long list
						if (mode == MODE_EXTRACT) // PASS N_rounds
						{
							// obtain global pointer to final edge list
							int edgePos = atomic_inc(aux+1);
							// position in output array as multiple of 128bits (32bits will be empty)
							int auxIndex = 4 + (edgePos * 4);
							// debug failsafe
							//if (!(DEBUG && (auxIndex > (1024 * 1024))))
							{
								// store all information to global memory
								aux[auxIndex + 0] = dipnode(v0i, v1i, v2i, v3i, nonce, 0);
								aux[auxIndex + 1] = dipnode(v0i, v1i, v2i, v3i, nonce, 1);
								aux[auxIndex + 2] = nonce;
								aux[auxIndex + 3] = 0; // for clarity, competely useless operation
							}
						}
					}
				}
				// clear current edge position so that we can skip it in next run of ctz()
				ee ^= ((u32)1<<31) >> pos; // 1 XOR 1 is 0
			}
		}
		// return edge bits back to global memory if we are in second stage
		if (mode == MODE_TRIM)
			edges[index] = (uint8)(el[lid][0], el[lid][1], el[lid][2], el[lid][3], el[lid][4], el[lid][5], el[lid][6], el[lid][7]);
		// debug only, use aux buffer to count alive edges in this round
		if (DEBUG)
			atomic_add(aux, lCount);
	}
}
"#;
