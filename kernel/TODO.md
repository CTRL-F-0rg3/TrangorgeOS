TrangorgeOS - memory management / allocator TODO

Current state: physical bitmap frame allocator, virtual address space bump allocator, buddy+slab kernel heap with a global allocator, all wired together and passing self-tests. This is a working baseline, not a finished subsystem. Below is what is missing or weak, roughly ordered by how blocking each item is.

Physical allocator (allocator/physical/)
- bitmap allocator has no support for allocating contiguous multi-frame runs, only single frames one at a time - anything needing physically contiguous memory (DMA buffers, some device drivers) cannot be served yet
- physical/buddy.rs is an empty placeholder - was meant as a second, contiguous-allocation-friendly physical allocator to sit next to the bitmap one, never implemented, needs a decision on when bitmap vs buddy should be used
- MAX_SUPPORTED_FRAMES is hardcoded to 2^20 frames (4GiB worth) in allocator/config.rs - machines with more RAM will silently lose everything above that, no warning is printed
- no reclamation of frames used for early boot structures (page tables set up by the bootloader, etc.) - they are permanently marked used even if never touched again
- BitmapFrameAllocator itself has no locking of its own - relies entirely on the outer Mutex in physical/mod.rs, fine for now but worth documenting/enforcing if the allocator is ever used outside that wrapper

Virtual address space (allocator/virt/)
- adress_space.rs only supports bump allocation, no way to release a reserved virtual range back - once reserved, a range is gone for the lifetime of the kernel
- only one static region exists (HEAP_REGION) - no general-purpose reservation API for other subsystems (MMIO mappings, future user-space regions, stacks for additional threads/cores)
- mapper.rs always maps PRESENT | WRITABLE with default page attributes - no support for read-only, no-execute, or uncacheable mappings, which will matter for MMIO and for W^X hardening later
- no unmap/cleanup path if map_range fails partway through - a partially mapped range is left mapped instead of being rolled back

Heap (allocator/heap/)
- SlabBucket has no way to return completely unused refill chunks back to the buddy allocator - once a chunk is carved into blocks, that memory can never go back to buddy even if every block in it is freed
- slab refill size (SLAB_REFILL_BLOCKS = 16) is a fixed constant, not tuned or configurable per bucket size
- buddy allocator's MAX_ORDER is a compile-time constant (22, 4MiB) - no way to allocate anything larger from the heap directly, large allocations would need a dedicated path
- no support for realloc - Rust's GlobalAlloc trait has an optional realloc method with a default implementation that just does alloc+copy+dealloc, which works but is wasteful for growing buffers (e.g. Vec growth) - a proper realloc that extends in place when possible would help performance
- no fragmentation metrics beyond raw allocated_bytes - would help to know free block count per order, largest free block available, etc.

Testing / correctness
- no stress test for the heap under many interleaved alloc/dealloc calls of varying sizes - current tests are correctness smoke tests, not fuzzing or stress testing
- no test exercising allocation failure paths (heap exhaustion, physical memory exhaustion) - only the happy path is currently verified
- traits.rs defines a SubAllocator trait that BuddyAllocator and SlabBucket do not implement yet - either wire it in for real polymorphism or remove it if it is not going to be used

Bigger, longer-term items
- no support for freeing/unmapping the heap region at all, it is set up once at boot and assumed to live forever
- no per-CPU heaps or any allocator-side preparation for SMP - once multiple cores exist, the current single global Mutex<KernelHeap> will be a contention point
- no huge page (2MiB/1GiB) support anywhere in the allocator stack, everything is 4KiB pages only
