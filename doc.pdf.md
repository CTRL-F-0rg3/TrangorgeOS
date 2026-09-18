# TrangorgeOS Documentation

**Generated:** 2026-09-18 20:46:40

**Source:** `/home/ctrl/TrangorgeOS`

---

## 1. Public API Definitions

### /home/ctrl/TrangorgeOS/comgrub/src/boot.asm

**LABEL: _start** (line 30)

```asm
global _start
```

---

### /home/ctrl/TrangorgeOS/comlimine/src/main.c

**FUNCTION: _start** (line 28)

```c
void _start(void) {
```

---

### /home/ctrl/TrangorgeOS/drivers/audiodriver/src/jacklib.rs

**STRUCT: JackMgr** (line 4)

```rust
pub struct JackMgr {
```

---

**CONST: fn** (line 10)

```rust
pub const fn new() -> Self {
```

---

**FN: tick** (line 14)

```rust
pub fn tick(&mut self) {
```

---

**FN: set_amp** (line 30)

```rust
pub fn set_amp(&mut self, on: bool) {
```

---

**FN: present** (line 35)

```rust
pub fn present(&self) -> bool {
```

---

**FN: amp_enabled** (line 40)

```rust
pub fn amp_enabled(&self) -> bool {
```

Czy wzmacniacz (amp) jest aktualnie wlaczony.

---

### /home/ctrl/TrangorgeOS/drivers/audiodriver/src/lib.rs

**FN: init** (line 16)

```rust
pub fn init(nam_va: u64, bm_va: u64) -> bool {
```

---

**FN: play** (line 20)

```rust
pub fn play(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> bool {
```

---

**FN: capture** (line 24)

```rust
pub fn capture(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> bool {
```

---

**FN: stop** (line 28)

```rust
pub fn stop() {
```

---

**FN: jack_present** (line 32)

```rust
pub fn jack_present() -> bool {
```

---

**FN: set_amp** (line 36)

```rust
pub fn set_amp(on: bool) {
```

---

**FN: position** (line 40)

```rust
pub fn position() -> u32 {
```

---

### /home/ctrl/TrangorgeOS/drivers/audiodriver/src/odin/driver.odin

**STRUCT: Bdl_Entry** (line 8)

```odin
Bdl_Entry :: struct {
```

---

**PROC: nam16** (line 16)

```odin
nam16 :: proc(off: u32) -> ^u16 {
```

mixer (NAM) — 16-bit

---

**PROC: bm8** (line 21)

```odin
bm8 :: proc(off: u32) -> ^u8 {
```

bus master — 8/16/32

---

**PROC: bm16** (line 25)

```odin
bm16 :: proc(off: u32) -> ^u16 {
```

---

**PROC: bm32** (line 29)

```odin
bm32 :: proc(off: u32) -> ^u32 {
```

---

**PROC: ad_init** (line 50)

```odin
ad_init :: proc "C" (nam_va: u64, bm_va: u64) -> i32 {
```

---

**PROC: ad_stop** (line 73)

```odin
ad_stop :: proc "C" () -> i32 {
```

---

**PROC: ad_position** (line 80)

```odin
ad_position :: proc "C" () -> u32 {
```

---

### /home/ctrl/TrangorgeOS/drivers/audiodriver/src/odin/jack.odin

**PROC: ad_jack_present** (line 8)

```odin
ad_jack_present :: proc "C" () -> i32 {
```

---

**PROC: ad_set_amp** (line 21)

```odin
ad_set_amp :: proc "C" (on: i32) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/drivers/audiodriver/src/odin/microphone.odin

**PROC: ad_capture** (line 6)

```odin
ad_capture :: proc "C" (data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/drivers/audiodriver/src/odin/speaker.odin

**PROC: ad_play** (line 6)

```odin
ad_play :: proc "C" (data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/drivers/audiodriver/src/tone.rs

**FN: fill_square** (line 1)

```rust
pub fn fill_square(buf: &mut [u8], periods: u32) {
```

---

### /home/ctrl/TrangorgeOS/drivers/vgpu/vgpu.h

**FUNCTION: vgpu_clear** (line 25)

```c
void vgpu_clear(uint32_t color);
```

---

**FUNCTION: vgpu_pixel** (line 26)

```c
void vgpu_pixel(uint32_t x, uint32_t y, uint32_t color);
```

---

**FUNCTION: vgpu_process_ring** (line 31)

```c
void vgpu_process_ring(volatile void *ring);
```

---

### /home/ctrl/TrangorgeOS/drivers/vgpu/vgpu_driver.c

**FUNCTION: vgpu_pixel** (line 104)

```c
void vgpu_pixel(uint32_t x, uint32_t y, uint32_t color)
```

---

**FUNCTION: vgpu_clear** (line 113)

```c
void vgpu_clear(uint32_t color)
```

---

**FUNCTION: vgpu_process_ring** (line 190)

```c
void vgpu_process_ring(volatile vgpu_slot_t *ring)
```

---

**FUNCTION: ds_entry** (line 244)

```c
void ds_entry(uint64_t params_va)
```

---

### /home/ctrl/TrangorgeOS/driverspace/src/drivers/storage.rs

**STRUCT: StorageDrv** (line 9)

```rust
pub struct StorageDrv {
```

Storage driver stub. It only satisfies the driver registry so that
driver space can boot; a real disk driver can slot in later.

---

**CONST: fn** (line 14)

```rust
pub const fn new() -> Self {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/formal/ds-spark-core/src/ds_buddy_math.adb

**PACKAGE: body** (line 2)

```ada
package body DS_Buddy_Math with
```

formal/ds-spark-core/src/ds_buddy_math.adb

---

**FUNCTION: Power_Of_Two** (line 6)

```ada
function Power_Of_Two (Exponent : Block_Size) return Phys_Addr is
```

---

**FUNCTION: Is_Aligned** (line 11)

```ada
function Is_Aligned (Addr : Phys_Addr; Size_Exp : Block_Size) return Boolean is
```

---

**FUNCTION: Get_Buddy_Address** (line 18)

```ada
function Get_Buddy_Address (
```

---

**FUNCTION: Get_Left_Child_Address** (line 31)

```ada
function Get_Left_Child_Address (
```

---

**FUNCTION: Get_Right_Child_Address** (line 40)

```ada
function Get_Right_Child_Address (
```

---

**FUNCTION: Safe_Calculate_DMA_End** (line 50)

```ada
function Safe_Calculate_DMA_End (
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/formal/ds-spark-core/src/ds_buddy_math.ads

**PACKAGE: DS_Buddy_Math** (line 4)

```ada
package DS_Buddy_Math with
```

---

**TYPE: Phys_Addr** (line 9)

```ada
type Phys_Addr is new Unsigned_64;
```

Typy bazowe dla fizycznej pamięci (bare-metal)

---

**TYPE: Block_Size** (line 10)

```ada
type Block_Size is new Unsigned_64;
```

---

**FUNCTION: Power_Of_Two** (line 17)

```ada
function Power_Of_Two (Exponent : Block_Size) return Phys_Addr with
```

Funkcja pomocnicza: Obliczanie 2^N (Shift left)

---

**FUNCTION: Is_Aligned** (line 23)

```ada
function Is_Aligned (Addr : Phys_Addr; Size_Exp : Block_Size) return Boolean with
```

Sprawdzenie, czy adres jest poprawnie wyrównany do rozmiaru bloku

---

**FUNCTION: Get_Buddy_Address** (line 32)

```ada
function Get_Buddy_Address (
```

---

**FUNCTION: Get_Left_Child_Address** (line 47)

```ada
function Get_Left_Child_Address (
```

---

**FUNCTION: Get_Right_Child_Address** (line 55)

```ada
function Get_Right_Child_Address (
```

---

**FUNCTION: Safe_Calculate_DMA_End** (line 69)

```ada
function Safe_Calculate_DMA_End (
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-formal-ffi/src/lib.rs

**FN: ds_spark_safe_add_32** (line 7)

```rust
pub fn ds_spark_safe_add_32(a: u32, b: u32) -> u32;
```

---

**FN: ds_spark_safe_array_index** (line 10)

```rust
pub fn ds_spark_safe_array_index(index: u32, max_size: u32) -> u32;
```

---

**FN: safe_add** (line 21)

```rust
pub fn safe_add(a: u32, b: u32) -> u32 {
```

---

**FN: ds_spark_buddy_get_buddy** (line 28)

```rust
pub fn ds_spark_buddy_get_buddy(addr: u64, size_exp: u64) -> u64;
```

---

**FN: ds_spark_safe_dma_end** (line 29)

```rust
pub fn ds_spark_safe_dma_end(base: u64, len: u64) -> u64;
```

---

**FN: get_buddy** (line 35)

```rust
pub fn get_buddy(addr: u64, exp: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-audio/src/lib.rs

**FN: add** (line 1)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-block/src/request.rs

**ENUM: RequestState** (line 7)

```rust
pub enum RequestState {
```

---

**STRUCT: IoRequest** (line 15)

```rust
pub struct IoRequest {
```

High-level I/O request with metadata.

---

**FN: new** (line 28)

```rust
pub fn new(id: u64, lba: u64, block_count: u32, buffer: *mut u8, len: usize, is_write: bool) -> Self {
```

---

**FN: state** (line 42)

```rust
pub fn state(&self) -> RequestState {
```

---

**FN: mark_in_progress** (line 52)

```rust
pub fn mark_in_progress(&self) {
```

---

**FN: mark_completed** (line 56)

```rust
pub fn mark_completed(&self) {
```

---

**FN: mark_failed** (line 63)

```rust
pub fn mark_failed(&self) {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-block/src/traits.rs

**STRUCT: BlockRequest** (line 6)

```rust
pub struct BlockRequest {
```

Represents a single I/O request to a block device.

---

**TRAIT: BlockDevice** (line 16)

```rust
pub trait BlockDevice {
```

Core trait for all block devices.
Drivers must implement this to integrate with the block subsystem.

---

**STRUCT: BlockGeometry** (line 40)

```rust
pub struct BlockGeometry {
```

---

**FN: total_bytes** (line 47)

```rust
pub fn total_bytes(&self) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-gpu/src/lib.rs

**FN: add** (line 1)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-input/src/lib.rs

**FN: add** (line 1)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-ipc/src/client.rs

**STRUCT: ManagerClient** (line 7)

```rust
pub struct ManagerClient {
```

---

**CONST: fn** (line 12)

```rust
pub const fn new(ep: Handle) -> Self {
```

---

**FN: request_mmio** (line 16)

```rust
pub fn request_mmio(&self, phys_base: u64, size: u64) -> Result<u64, DsError> {
```

---

**FN: bind_irq** (line 41)

```rust
pub fn bind_irq(&self, irq_num: u32) -> Result<(), DsError> {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-ipc/src/dispatcher.rs

**TYPE: HandlerFn** (line 8)

```rust
pub type HandlerFn = fn(sender: Handle, payload_ptr: *const u8, payload_len: u32) -> Result<(), DsError>;
```

---

**STRUCT: MessageDispatcher** (line 10)

```rust
pub struct MessageDispatcher {
```

---

**CONST: fn** (line 15)

```rust
pub const fn new() -> Self {
```

---

**FN: register** (line 21)

```rust
pub fn register(&mut self, opcode: Opcode, handler: HandlerFn) -> Result<(), DsError> {
```

---

**FN: dispatch** (line 30)

```rust
pub fn dispatch(&self, sender: Handle, opcode_raw: u32, payload_ptr: *const u8, payload_len: u32) -> Result<(), DsError> {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-log/src/formatter.rs

**CONST: MAX_LOG_LINE_LEN** (line 5)

```rust
pub const MAX_LOG_LINE_LEN: usize = 256;
```

---

**STRUCT: LogBuffer** (line 7)

```rust
pub struct LogBuffer {
```

---

**CONST: fn** (line 13)

```rust
pub const fn new() -> Self {
```

---

**FN: as_bytes** (line 20)

```rust
pub fn as_bytes(&self) -> &[u8] {
```

---

**FN: reset** (line 24)

```rust
pub fn reset(&mut self) {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-log/src/levels.rs

**ENUM: LogLevel** (line 5)

```rust
pub enum LogLevel {
```

---

**CONST: fn** (line 14)

```rust
pub const fn as_str(&self) -> &'static str {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-log/src/lib.rs

**STRUCT: EndpointRef** (line 14)

```rust
pub struct EndpointRef;
```

---

**CONST: fn** (line 17)

```rust
pub const fn new() -> Self { Self }
```

---

**FN: is_valid** (line 19)

```rust
pub fn is_valid(&self) -> bool {
```

---

**FN: handle** (line 24)

```rust
pub fn handle(&self) -> u32 {
```

Handle endpointu Managera (0 = niezarejestrowany).

---

**FN: set** (line 28)

```rust
pub fn set(&self, handle: Handle) {
```

---

**STATIC: MANAGER_ENDPOINT** (line 33)

```rust
pub static MANAGER_ENDPOINT: EndpointRef = EndpointRef::new();
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-log/src/transport.rs

**FN: emit_log** (line 19)

```rust
pub fn emit_log(level: LogLevel, module: &str, buf: &LogBuffer) {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-mem/src/alloc/buddy.rs

**STRUCT: BuddyAllocator** (line 5)

```rust
pub struct BuddyAllocator {
```

---

**CONST: fn** (line 12)

```rust
pub const fn new(base_addr: usize, total_pages: usize) -> Self {
```

---

**FN: alloc_pages** (line 20)

```rust
pub fn alloc_pages(&mut self, order: usize) -> Option<usize> {
```

---

**FN: free_pages** (line 28)

```rust
pub fn free_pages(&mut self, addr: usize, order: usize) {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-mem/src/alloc/heap.rs

**STRUCT: DriverHeap** (line 6)

```rust
pub struct DriverHeap {
```

---

**CONST: fn** (line 12)

```rust
pub const fn new() -> Self {
```

---

**FN: init** (line 18)

```rust
pub fn init(&mut self, base_addr: usize, total_pages: usize) {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-mem/src/alloc/slab.rs

**STRUCT: SlabAllocator** (line 1)

```rust
pub struct SlabAllocator {
```

---

**CONST: fn** (line 7)

```rust
pub const fn new(object_size: usize) -> Self {
```

---

**FN: alloc** (line 14)

```rust
pub fn alloc(&mut self) -> Option<usize> {
```

---

**FN: free** (line 19)

```rust
pub fn free(&mut self, ptr: usize) {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-mem/src/dma/buffer.rs

**STRUCT: DmaFlags** (line 10)

```rust
pub struct DmaFlags: u32 {
```

---

**STRUCT: DmaBuffer** (line 17)

```rust
pub struct DmaBuffer {
```

---

**FN: allocate** (line 25)

```rust
pub fn allocate(manager_ep: Handle, size: u64, flags: DmaFlags) -> Result<Self, DsError> {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-mem/src/dma/mapping.rs

**STRUCT: MmioRegion** (line 8)

```rust
pub struct MmioRegion {
```

---

**FN: map** (line 16)

```rust
pub fn map(manager_ep: Handle, phys_base: u64, size: u64) -> Result<Self, DsError> {
```

---

**STRUCT: Volatile** (line 67)

```rust
pub struct Volatile<T>(pub T);
```

---

**FN: read** (line 71)

```rust
pub fn read(&self) -> T {
```

---

**FN: write** (line 76)

```rust
pub fn write(&mut self, value: T) {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-mem/src/vmm.rs

**FN: map_memory** (line 6)

```rust
pub fn map_memory(
```

---

**FN: unmap_memory** (line 33)

```rust
pub fn unmap_memory(manager_ep: Handle, virt_addr: u64, size: u64) -> Result<(), DsError> {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-abi/src/errors.rs

**ENUM: DsError** (line 7)

```rust
pub enum DsError {
```

---

**FN: from_u32** (line 44)

```rust
pub fn from_u32(val: u32) -> Self {
```

将 u32 转换为 DsError，如果值超出范围则返回 Unknown。

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-abi/src/opcodes.rs

**ENUM: Opcode** (line 6)

```rust
pub enum Opcode {
```

---

**FN: from_u32** (line 47)

```rust
pub fn from_u32(val: u32) -> Option<Self> {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-abi/src/payloads/dev.rs

**ENUM: BusType** (line 10)

```rust
pub enum BusType {
```

---

**STRUCT: DeviceFlags** (line 22)

```rust
pub struct DeviceFlags: u32 {
```

---

**STRUCT: DevAttachPayload** (line 38)

```rust
pub struct DevAttachPayload {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-abi/src/payloads/irq.rs

**STRUCT: IrqBindPayload** (line 6)

```rust
pub struct IrqBindPayload {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-abi/src/payloads/mem.rs

**STRUCT: MmioMapPayload** (line 8)

```rust
pub struct MmioMapPayload {
```

---

**STRUCT: DmaAllocPayload** (line 25)

```rust
pub struct DmaAllocPayload {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-abi/src/payloads/sys.rs

**STRUCT: LogPayload** (line 9)

```rust
pub struct LogPayload {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-abi/src/primitives.rs

**STRUCT: Handle** (line 8)

```rust
pub struct Handle(pub u32);
```

---

**CONST: NULL** (line 11)

```rust
pub const NULL: Self = Self(0);
```

---

**FN: is_valid** (line 14)

```rust
pub fn is_valid(&self) -> bool {
```

---

**STRUCT: CapId** (line 23)

```rust
pub struct CapId(pub u64);
```

---

**STRUCT: PhysAddr** (line 28)

```rust
pub struct PhysAddr(pub u64);
```

---

**STRUCT: VirtAddr** (line 33)

```rust
pub struct VirtAddr(pub u64);
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-syscall/src/lib.rs

**FN: sys_ipc_send** (line 40)

```rust
pub fn sys_ipc_send(
```

Wysyła wiadomość IPC (bez oczekiwania na odpowiedź).

---

**FN: sys_ipc_call** (line 50)

```rust
pub fn sys_ipc_call(
```

Wysyła żądanie IPC i odbiera odpowiedź.

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/abi.rs

**CONST: SVC_SYS** (line 1)

```rust
pub const SVC_SYS: u32 = 0;
```

---

**CONST: SVC_VIDEO** (line 2)

```rust
pub const SVC_VIDEO: u32 = 1;
```

---

**CONST: SVC_AUDIO** (line 3)

```rust
pub const SVC_AUDIO: u32 = 2;
```

---

**CONST: SVC_INPUT** (line 4)

```rust
pub const SVC_INPUT: u32 = 3;
```

---

**CONST: SVC_BLOCK** (line 5)

```rust
pub const SVC_BLOCK: u32 = 4;
```

---

**CONST: SVC_NET** (line 6)

```rust
pub const SVC_NET: u32 = 5;
```

---

**CONST: fn** (line 8)

```rust
pub const fn svc_cmd(class: u32, op: u32) -> u32 {
```

---

**CONST: fn** (line 12)

```rust
pub const fn svc_class(cmd: u32) -> u32 {
```

---

**CONST: fn** (line 16)

```rust
pub const fn svc_op(cmd: u32) -> u32 {
```

---

**CONST: VID_FB_INFO** (line 20)

```rust
pub const VID_FB_INFO: u32 = 1;
```

---

**CONST: VID_FB_TAKEOVER** (line 21)

```rust
pub const VID_FB_TAKEOVER: u32 = 2;
```

---

**CONST: VID_FB_RELEASE** (line 22)

```rust
pub const VID_FB_RELEASE: u32 = 3;
```

---

**CONST: IN_KEY_POLL** (line 23)

```rust
pub const IN_KEY_POLL: u32 = 1;
```

---

**CONST: AUD_PLAY** (line 24)

```rust
pub const AUD_PLAY: u32 = 1;
```

---

**CONST: AUD_STOP** (line 25)

```rust
pub const AUD_STOP: u32 = 2;
```

---

**CONST: AUD_JACK** (line 26)

```rust
pub const AUD_JACK: u32 = 3;
```

---

**CONST: AUD_AMP** (line 27)

```rust
pub const AUD_AMP: u32 = 4;
```

---

**CONST: BLK_COUNT** (line 28)

```rust
pub const BLK_COUNT: u32 = 1;
```

---

**CONST: BLK_READ** (line 29)

```rust
pub const BLK_READ: u32 = 2;
```

---

**CONST: BLK_WRITE** (line 30)

```rust
pub const BLK_WRITE: u32 = 3;
```

---

**CONST: DS_MAGIC** (line 37)

```rust
pub const DS_MAGIC: u64 = 0x4452_5653_5041_4345;
```

---

**CONST: DS_VERSION** (line 38)

```rust
pub const DS_VERSION: u32 = 1;
```

---

**CONST: DS_RING_CAP** (line 39)

```rust
pub const DS_RING_CAP: u64 = 16;
```

---

**CONST: DS_FLAG_RESPONSE** (line 41)

```rust
pub const DS_FLAG_RESPONSE: u32 = 1 << 0;
```

---

**CONST: DS_INIT_PARAMS_VA** (line 43)

```rust
pub const DS_INIT_PARAMS_VA: u64 = 0x4000_0000;
```

---

**CONST: DS_K2D_VA** (line 44)

```rust
pub const DS_K2D_VA: u64 = 0x4000_1000;
```

---

**CONST: DS_D2K_VA** (line 45)

```rust
pub const DS_D2K_VA: u64 = 0x4000_2000;
```

---

**CONST: DS_SCRATCH_VA** (line 46)

```rust
pub const DS_SCRATCH_VA: u64 = 0x4000_3000;
```

---

**CONST: DS_SCRATCH_SIZE** (line 47)

```rust
pub const DS_SCRATCH_SIZE: usize = 4096;
```

---

**CONST: DS_SWITCH_VA** (line 48)

```rust
pub const DS_SWITCH_VA: u64 = 0x4000_4000;
```

---

**ENUM: DsCmd** (line 53)

```rust
pub enum DsCmd {
```

---

**STRUCT: DsRing** (line 83)

```rust
pub struct DsRing {
```

---

**CONST: DS_RING_HDR_SIZE** (line 89)

```rust
pub const DS_RING_HDR_SIZE: usize = core::mem::size_of::<DsRing>();
```

---

**STRUCT: DsMsg** (line 94)

```rust
pub struct DsMsg {
```

---

**CONST: DS_MSG_SIZE** (line 105)

```rust
pub const DS_MSG_SIZE: usize = core::mem::size_of::<DsMsg>();
```

---

**STRUCT: DsInitParams** (line 110)

```rust
pub struct DsInitParams {
```

---

**ENUM: DsError** (line 123)

```rust
pub enum DsError {
```

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/audio.rs

**STRUCT: AudioBars** (line 4)

```rust
pub struct AudioBars {
```

---

**FN: info_req** (line 9)

```rust
pub fn info_req() -> u64 {
```

---

**FN: info_take** (line 13)

```rust
pub fn info_take(id: u64) -> Option<AudioBars> {
```

---

**FN: page_phys_req** (line 23)

```rust
pub fn page_phys_req(va: u64) -> u64 {
```

---

**FN: page_phys_take** (line 27)

```rust
pub fn page_phys_take(id: u64) -> Option<u64> {
```

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/driver.rs

**STRUCT: DeviceInfo** (line 6)

```rust
pub struct DeviceInfo {
```

---

**TRAIT: Driver** (line 14)

```rust
pub trait Driver {
```

Any driver that lives in driver space and is registered with the kernel
driver-space manager.

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/input.rs

**FN: key_req** (line 4)

```rust
pub fn key_req() -> u64 {
```

---

**FN: key_take** (line 8)

```rust
pub fn key_take(id: u64) -> Option<u8> {
```

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/jack.rs

**STRUCT: JackInfo** (line 4)

```rust
pub struct JackInfo {
```

---

**FN: query_req** (line 9)

```rust
pub fn query_req() -> u64 {
```

---

**FN: query_take** (line 13)

```rust
pub fn query_take(id: u64) -> Option<JackInfo> {
```

---

**FN: set_amp** (line 22)

```rust
pub fn set_amp(on: bool) {
```

---

**FN: play** (line 26)

```rust
pub fn play(va: u64, len: u32) -> u64 {
```

---

**FN: stop** (line 30)

```rust
pub fn stop() {
```

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/log.rs

**FN: ds_log** (line 10)

```rust
pub fn ds_log(msg: &str) {
```

Wysyła tekst do logu jądra (komenda `DsCmd::Log`).

---

**FN: ds_log_raw** (line 15)

```rust
pub fn ds_log_raw(ptr: *const u8, len: usize) {
```

To samo, ale dla surowego wskaźnika i długości (np. bufora bajtów).

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/runtime.rs

**FN: init_once** (line 38)

```rust
pub fn init_once(params_va: u64) {
```

Reads the boot parameters block and points our ring views at the
kernel-provided buffers.

---

**FN: register** (line 51)

```rust
pub fn register<D: Driver>(drv: &mut D) {
```

Registers a driver (by calling its `Driver::init` hook) and notifies
the kernel that this driver space is now managed.

---

**FN: request** (line 94)

```rust
pub fn request(cmd: DsCmd, a0: u64, a1: u64, a2: u64) -> u64 {
```

Sends a driver-space command to the kernel. Returns the id used later
with [`take_resp`].

---

**FN: request_raw** (line 99)

```rust
pub fn request_raw(cmd: u32, a0: u64, a1: u64, a2: u64) -> u64 {
```

Sends a raw (service-class packed) command to the kernel.

---

**FN: tick** (line 104)

```rust
pub fn tick() {
```

Drains pending kernel->driver responses into the local cache.

---

**FN: take_resp** (line 128)

```rust
pub fn take_resp(id: u64) -> Option<DsMsg> {
```

Returns (and consumes) the response with the given request id.

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/svc.rs

**FN: call** (line 4)

```rust
pub fn call(class: u32, op: u32, a0: u64, a1: u64, a2: u64) -> u64 {
```

---

**FN: take** (line 8)

```rust
pub fn take(id: u64) -> Option<DsMsg> {
```

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/video.rs

**STRUCT: FbInfo** (line 4)

```rust
pub struct FbInfo {
```

---

**FN: fb_info_req** (line 11)

```rust
pub fn fb_info_req() -> u64 {
```

---

**FN: fb_info_take** (line 15)

```rust
pub fn fb_info_take(id: u64) -> Option<FbInfo> {
```

---

**FN: takeover** (line 26)

```rust
pub fn takeover() -> u64 {
```

---

**FN: release** (line 30)

```rust
pub fn release() -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/C_base/kc-abi/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/Odin_base/odin-abi/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/Odin_base/odin-abi-bridge/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/base/kw-C-abi/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/base/kw-base/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/base/kw-libs/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/base/kw-odin-abi/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/kstd_alloc/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/kstd_base/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/kstd_core/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/kstd_data/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/kstd_io/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/linix_abi/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/linix_abi_com/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/linix_abi_driverspace/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/windows-com/src/lib.rs

**FN: add** (line 2)

```rust
pub fn add(left: u64, right: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/arch/mod.rs

**FN: init** (line 13)

```rust
pub fn init() {
```

---

**FN: hlt_loop** (line 18)

```rust
pub fn hlt_loop() -> ! {
```

---

**FN: now** (line 23)

```rust
pub fn now() -> u64 {
```

---

**FN: current_cpu** (line 28)

```rust
pub fn current_cpu() -> usize {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/arch/riscv64.rs

**FN: heap_init** (line 20)

```rust
pub fn heap_init() {
```

---

**STRUCT: KernelHeap** (line 27)

```rust
pub struct KernelHeap;
```

---

**FN: now** (line 51)

```rust
pub fn now() -> u64 {
```

---

**FN: init** (line 66)

```rust
pub fn init() {
```

---

**FN: current_cpu** (line 74)

```rust
pub fn current_cpu() -> usize {
```

---

**FN: hlt_loop** (line 78)

```rust
pub fn hlt_loop() -> ! {
```

---

**FN: early_uart** (line 86)

```rust
pub fn early_uart(s: &str) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/arch/x86_64.rs

**FN: init** (line 10)

```rust
pub fn init() {
```

---

**FN: hlt_loop** (line 18)

```rust
pub fn hlt_loop() -> ! {
```

---

**FN: current_cpu** (line 24)

```rust
pub fn current_cpu() -> usize {
```

---

**FN: now** (line 36)

```rust
pub fn now() -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/audio/jack.rs

**FN: init** (line 6)

```rust
pub fn init(base: u32) -> bool {
```

---

**FN: poll_jack** (line 10)

```rust
pub fn poll_jack() -> bool {
```

---

**FN: query** (line 23)

```rust
pub fn query() -> u32 {
```

---

**FN: set_amp** (line 29)

```rust
pub fn set_amp(on: bool) {
```

---

**FN: play_phys** (line 34)

```rust
pub fn play_phys(phys: u64, len: u32) -> bool {
```

---

**FN: stop** (line 38)

```rust
pub fn stop() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/battery/aut.rs

**CONST: BATT_STATUS** (line 1)

```rust
pub const BATT_STATUS: u32 = 1;
```

---

**CONST: BATT_THRESH** (line 2)

```rust
pub const BATT_THRESH: u32 = 2;
```

---

**FN: authorize** (line 4)

```rust
pub fn authorize(ring: u8, op: u8) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/battery/battery.h

**FUNCTION: battery_sim_tick** (line 28)

```c
void battery_sim_tick(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/battery/bridge.rs

**FN: batt_call** (line 9)

```rust
pub fn batt_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/battery/init.rs

**FN: init** (line 5)

```rust
pub fn init() -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/battery/operation.c

**FUNCTION: battery_register_backend** (line 38)

```c
void battery_register_backend(const battery_backend_t *b)
```

---

**FUNCTION: battery_sim_tick** (line 101)

```c
void battery_sim_tick(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/battery/operation.h

**FUNCTION: battery_register_backend** (line 11)

```c
void battery_register_backend(const battery_backend_t *b);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/bluetooth/aut.rs

**CONST: BT_INIT** (line 1)

```rust
pub const BT_INIT: u32 = 0;
```

---

**CONST: BT_INFO** (line 2)

```rust
pub const BT_INFO: u32 = 1;
```

---

**CONST: BT_CMD** (line 3)

```rust
pub const BT_CMD: u32 = 2;
```

---

**CONST: BT_EVT** (line 4)

```rust
pub const BT_EVT: u32 = 3;
```

---

**CONST: BT_ACL_OUT** (line 5)

```rust
pub const BT_ACL_OUT: u32 = 4;
```

---

**CONST: BT_ACL_IN** (line 6)

```rust
pub const BT_ACL_IN: u32 = 5;
```

---

**FN: authorize** (line 8)

```rust
pub fn authorize(ring: u8, op: u32) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/bluetooth/bridge.rs

**FN: bt_call** (line 20)

```rust
pub fn bt_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/bluetooth/bt.h

**FUNCTION: bt_info** (line 10)

```c
void bt_info(uint8_t *hci_ver, uint8_t *bdaddr);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/bluetooth/init.c

**FUNCTION: bt_info** (line 89)

```c
void bt_info(uint8_t *hci_ver, uint8_t *bdaddr)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/bluetooth/init.rs

**FN: init** (line 6)

```rust
pub fn init() -> bool {
```

---

**FN: ready** (line 10)

```rust
pub fn ready() -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/bluetooth/operation.c

**FUNCTION: bt_op_model_init** (line 36)

```c
void bt_op_model_init(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/bluetooth/operation.h

**FUNCTION: bt_op_model_init** (line 11)

```c
void bt_op_model_init(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/camera/aut.rs

**CONST: CAM_CAPS** (line 1)

```rust
pub const CAM_CAPS: u32 = 1;
```

---

**CONST: CAM_START** (line 2)

```rust
pub const CAM_START: u32 = 2;
```

---

**CONST: CAM_STOP** (line 3)

```rust
pub const CAM_STOP: u32 = 3;
```

---

**CONST: CAM_FRAME** (line 4)

```rust
pub const CAM_FRAME: u32 = 4;
```

---

**FN: authorize** (line 6)

```rust
pub fn authorize(ring: u8, op: u8) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/camera/bridge.rs

**FN: cam_call** (line 15)

```rust
pub fn cam_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/camera/init.rs

**FN: init** (line 5)

```rust
pub fn init() -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/camera/operation.c

**FUNCTION: camera_register_backend** (line 94)

```c
void camera_register_backend(const camera_backend_t *b)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/camera/operation.h

**FUNCTION: camera_register_backend** (line 13)

```c
void camera_register_backend(const camera_backend_t *b);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/audit.rs

**ENUM: EventKind** (line 6)

```rust
pub enum EventKind {
```

---

**STRUCT: AuditEvent** (line 16)

```rust
pub struct AuditEvent {
```

---

**FN: init_audit_log** (line 41)

```rust
pub fn init_audit_log() -> Result<(), &'static str> {
```

---

**FN: log_check** (line 73)

```rust
pub fn log_check(world: u32, cap: Capability, ok: bool) {
```

---

**FN: log_grant** (line 78)

```rust
pub fn log_grant(granter: u32, target: u32, cap: Capability, ok: bool) {
```

---

**FN: log_revoke** (line 84)

```rust
pub fn log_revoke(world: u32, cap: Capability, ok: bool) {
```

---

**FN: log_register** (line 90)

```rust
pub fn log_register(world: u32) {
```

---

**FN: log_unregister** (line 94)

```rust
pub fn log_unregister(world: u32) {
```

---

**FN: count** (line 98)

```rust
pub fn count() -> usize {
```

---

**FN: recent** (line 102)

```rust
pub fn recent(n: usize) -> Vec<AuditEvent> {
```

---

**FN: by_world** (line 119)

```rust
pub fn by_world(world: u32, limit: usize) -> Vec<AuditEvent> {
```

---

**FN: by_kind** (line 126)

```rust
pub fn by_kind(kind: EventKind, limit: usize) -> Vec<AuditEvent> {
```

---

**FN: deny_count** (line 133)

```rust
pub fn deny_count() -> usize {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/caps.h

**FUNCTION: caps_has** (line 14)

```c
int caps_has(unsigned int world_id, unsigned char cap_id);
```

---

**FUNCTION: caps_name** (line 16)

```c
int caps_name(unsigned char cap_id, unsigned char *buf, unsigned int len);
```

---

**FUNCTION: caps_request** (line 18)

```c
int caps_request(unsigned int target, unsigned char cap_id);
```

---

**FUNCTION: caps_release** (line 20)

```c
int caps_release(unsigned int world_id, unsigned char cap_id);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/check.rs

**FN: kernel_world_id** (line 10)

```rust
pub fn kernel_world_id() -> u32 {
```

---

**FN: set_kernel_world_id** (line 14)

```rust
pub fn set_kernel_world_id(id: u32) {
```

---

**FN: current_world_id_pub** (line 18)

```rust
pub fn current_world_id_pub() -> u32 {
```

---

**FN: set_current_world** (line 22)

```rust
pub fn set_current_world(world_id: u32) {
```

---

**FN: has_cap** (line 30)

```rust
pub fn has_cap(cap: Capability) -> bool {
```

---

**FN: world_has_cap** (line 35)

```rust
pub fn world_has_cap(world_id: u32, cap: Capability) -> bool {
```

---

**FN: require_cap** (line 39)

```rust
pub fn require_cap(cap: Capability) -> CapResult<()> {
```

---

**FN: require_world_cap** (line 54)

```rust
pub fn require_world_cap(world_id: u32, cap: Capability) -> CapResult<()> {
```

---

**FN: require_caps** (line 67)

```rust
pub fn require_caps(caps: &[Capability]) -> CapResult<()> {
```

---

**FN: with_cap** (line 74)

```rust
pub fn with_cap<T, F>(cap: Capability, f: F) -> CapResult<T>
```

---

**FN: with_caps** (line 82)

```rust
pub fn with_caps<T, F>(caps: &[Capability], f: F) -> CapResult<T>
```

---

**FN: assert_cap** (line 104)

```rust
pub fn assert_cap(cap: Capability) {
```

---

**FN: fast_check** (line 111)

```rust
pub fn fast_check(cap: Capability) -> bool {
```

---

**FN: if_cap** (line 115)

```rust
pub fn if_cap<T, F>(cap: Capability, f: F, default: T) -> T
```

---

**STRUCT: TemporaryCaps** (line 126)

```rust
pub struct TemporaryCaps {
```

---

**FN: enter** (line 132)

```rust
pub fn enter(extra: CapabilitySet) -> CapResult<Self> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/defaults.rs

**FN: default_for_ring** (line 6)

```rust
pub fn default_for_ring(ring: u8) -> CapabilitySet {
```

---

**FN: install_defaults** (line 15)

```rust
pub fn install_defaults() -> Result<(), &'static str> {
```

---

**FN: register_for_ring** (line 22)

```rust
pub fn register_for_ring(ring: u8, parent: Option<u32>) -> Result<u32, &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/grant.rs

**CONST: NON_INHERITABLE** (line 7)

```rust
pub const NON_INHERITABLE: &[Capability] = &[
```

---

**FN: grant_cap** (line 18)

```rust
pub fn grant_cap(granter: u32, target: u32, cap: Capability) -> CapResult<()> {
```

---

**FN: delegate_caps** (line 31)

```rust
pub fn delegate_caps(granter: u32, target: u32, caps: CapabilitySet) -> CapResult<()> {
```

---

**FN: inherit_set** (line 38)

```rust
pub fn inherit_set(parent_world: u32) -> CapabilitySet {
```

---

**FN: spawn_child** (line 51)

```rust
pub fn spawn_child(parent_world: u32) -> Result<u32, &'static str> {
```

---

**FN: grant_temporary** (line 78)

```rust
pub fn grant_temporary(granter: u32, target: u32, cap: Capability,
```

---

**FN: prune_expired** (line 97)

```rust
pub fn prune_expired() {
```

---

**FN: is_temporary** (line 110)

```rust
pub fn is_temporary(world_id: u32, cap: Capability) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/hierarchy.rs

**FN: parent** (line 5)

```rust
pub fn parent(cap: Capability) -> Option<Capability> {
```

---

**FN: implies** (line 47)

```rust
pub fn implies(held: Capability, required: Capability) -> bool {
```

---

**FN: set_implies** (line 62)

```rust
pub fn set_implies(held: CapabilitySet, required: Capability) -> bool {
```

---

**FN: expand_hierarchy** (line 71)

```rust
pub fn expand_hierarchy(set: CapabilitySet) -> CapabilitySet {
```

---

**FN: path_to_root** (line 85)

```rust
pub fn path_to_root(cap: Capability) -> Vec<Capability> {
```

---

**FN: depth** (line 96)

```rust
pub fn depth(cap: Capability) -> usize {
```

---

**FN: subtree** (line 106)

```rust
pub fn subtree(root: Capability) -> Vec<Capability> {
```

---

**FN: can_delegate** (line 123)

```rust
pub fn can_delegate(holder: CapabilitySet, candidate: Capability) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/mod.rs

**FN: init** (line 31)

```rust
pub fn init() -> Result<(), &'static str> {
```

---

**FN: snapshot** (line 38)

```rust
pub fn snapshot() -> SystemSnapshot {
```

---

**STRUCT: SystemSnapshot** (line 48)

```rust
pub struct SystemSnapshot {
```

---

**FN: self_test** (line 55)

```rust
pub fn self_test() -> crate::testing::TestResult {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/policy.rs

**TYPE: PolicyHook** (line 5)

```rust
pub type PolicyHook = fn(world: u32, cap: Capability) -> bool;
```

---

**FN: set_hook** (line 9)

```rust
pub fn set_hook(h: PolicyHook) {
```

---

**FN: enforce** (line 23)

```rust
pub fn enforce(world: u32, cap: Capability) -> CapResult<()> {
```

---

**FN: enforce_self** (line 35)

```rust
pub fn enforce_self(cap: Capability) -> CapResult<()> {
```

---

**FN: allowed** (line 40)

```rust
pub fn allowed(world: u32, cap: Capability) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/revoke.rs

**FN: revoke_from_world** (line 7)

```rust
pub fn revoke_from_world(world_id: u32, cap: Capability) -> CapResult<()> {
```

---

**FN: revoke_subtree** (line 15)

```rust
pub fn revoke_subtree(world_id: u32, cap: Capability) -> CapResult<()> {
```

---

**FN: revoke_global** (line 23)

```rust
pub fn revoke_global(cap: Capability) {
```

---

**FN: restore_global** (line 28)

```rust
pub fn restore_global(cap: Capability) {
```

---

**FN: revoked_list** (line 32)

```rust
pub fn revoked_list() -> Vec<Capability> {
```

---

**FN: is_globally_revoked** (line 43)

```rust
pub fn is_globally_revoked(cap: Capability) -> bool {
```

---

**FN: lockdown** (line 47)

```rust
pub fn lockdown(world_id: u32) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/sets.rs

**STRUCT: CapSetBuilder** (line 4)

```rust
pub struct CapSetBuilder {
```

---

**FN: new** (line 9)

```rust
pub fn new() -> Self {
```

---

**FN: empty** (line 13)

```rust
pub fn empty() -> Self {
```

---

**FN: add** (line 17)

```rust
pub fn add(mut self, cap: Capability) -> Self {
```

---

**FN: add_many** (line 22)

```rust
pub fn add_many(mut self, caps: &[Capability]) -> Self {
```

---

**FN: add_category** (line 29)

```rust
pub fn add_category(mut self, cat: super::types::CapCategory) -> Self {
```

---

**FN: remove** (line 38)

```rust
pub fn remove(mut self, cap: Capability) -> Self {
```

---

**FN: with_hierarchy** (line 43)

```rust
pub fn with_hierarchy(self) -> Self {
```

---

**FN: build** (line 47)

```rust
pub fn build(self) -> CapabilitySet {
```

---

**FN: minimal_user** (line 55)

```rust
pub fn minimal_user() -> CapabilitySet {
```

---

**FN: standard_user** (line 67)

```rust
pub fn standard_user() -> CapabilitySet {
```

---

**FN: privileged_user** (line 84)

```rust
pub fn privileged_user() -> CapabilitySet {
```

---

**FN: driver** (line 98)

```rust
pub fn driver() -> CapabilitySet {
```

---

**FN: kernel** (line 111)

```rust
pub fn kernel() -> CapabilitySet {
```

---

**FN: sandbox** (line 115)

```rust
pub fn sandbox() -> CapabilitySet {
```

---

**FN: validate_hierarchy** (line 125)

```rust
pub fn validate_hierarchy(set: CapabilitySet) -> Result<(), &'static str> {
```

---

**FN: effective** (line 141)

```rust
pub fn effective(set: CapabilitySet) -> CapabilitySet {
```

---

**FN: effective_diff** (line 145)

```rust
pub fn effective_diff(a: CapabilitySet, b: CapabilitySet) -> CapabilitySet {
```

---

**FN: is_subset_with_hierarchy** (line 151)

```rust
pub fn is_subset_with_hierarchy(subset: CapabilitySet, superset: CapabilitySet) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/store.rs

**STRUCT: WorldCaps** (line 9)

```rust
pub struct WorldCaps {
```

---

**FN: init_store** (line 28)

```rust
pub fn init_store() -> Result<(), &'static str> {
```

---

**FN: global_caps** (line 36)

```rust
pub fn global_caps() -> CapabilitySet {
```

---

**FN: register_world** (line 41)

```rust
pub fn register_world(parent_world: Option<u32>, initial: CapabilitySet) -> Result<u32, &'static str> {
```

---

**FN: get_world_caps** (line 82)

```rust
pub fn get_world_caps(world_id: u32) -> Result<CapabilitySet, &'static str> {
```

---

**FN: set_world_caps** (line 88)

```rust
pub fn set_world_caps(world_id: u32, new_caps: CapabilitySet) -> Result<(), &'static str> {
```

---

**FN: add_world_cap** (line 115)

```rust
pub fn add_world_cap(world_id: u32, cap: Capability) -> Result<(), &'static str> {
```

---

**FN: remove_world_cap** (line 120)

```rust
pub fn remove_world_cap(world_id: u32, cap: Capability) -> Result<(), &'static str> {
```

---

**FN: unregister_world** (line 138)

```rust
pub fn unregister_world(world_id: u32) -> Result<(), &'static str> {
```

---

**FN: world_has_cap** (line 153)

```rust
pub fn world_has_cap(world_id: u32, cap: Capability) -> bool {
```

---

**FN: world_count** (line 163)

```rust
pub fn world_count() -> usize {
```

---

**FN: iter_worlds** (line 168)

```rust
pub fn iter_worlds<F: FnMut(u32, CapabilitySet)>(mut f: F) {
```

---

**FN: add_global_revoked** (line 177)

```rust
pub fn add_global_revoked(cap: Capability) {
```

---

**FN: remove_global_revoked** (line 182)

```rust
pub fn remove_global_revoked(cap: Capability) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/syscalls.rs

**CONST: SYS_CAP_QUERY** (line 8)

```rust
pub const SYS_CAP_QUERY: u64 = 0x1070;
```

---

**CONST: SYS_CAP_REQUEST** (line 9)

```rust
pub const SYS_CAP_REQUEST: u64 = 0x1071;
```

---

**CONST: SYS_CAP_RELEASE** (line 10)

```rust
pub const SYS_CAP_RELEASE: u64 = 0x1072;
```

---

**CONST: SYS_CAP_AUDIT** (line 11)

```rust
pub const SYS_CAP_AUDIT: u64 = 0x1073;
```

---

**FN: cap_syscall** (line 17)

```rust
pub fn cap_syscall(num: u64, a0: u64, a1: u64, a2: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/types.rs

**TYPE: CapID** (line 3)

```rust
pub type CapID = u8;
```

---

**CONST: MAX_CAPS** (line 5)

```rust
pub const MAX_CAPS: usize = 32;
```

---

**ENUM: Capability** (line 10)

```rust
pub enum Capability {
```

---

**CONST: fn** (line 50)

```rust
pub const fn bit(self) -> u32 {
```

---

**CONST: fn** (line 54)

```rust
pub const fn id(self) -> CapID {
```

---

**CONST: fn** (line 58)

```rust
pub const fn name(self) -> &'static str {
```

---

**FN: iter_all** (line 99)

```rust
pub fn iter_all() -> impl Iterator<Item = Capability> {
```

---

**FN: category** (line 112)

```rust
pub fn category(self) -> CapCategory {
```

---

**ENUM: CapCategory** (line 127)

```rust
pub enum CapCategory {
```

---

**CONST: fn** (line 139)

```rust
pub const fn name(self) -> &'static str {
```

---

**STRUCT: CapabilitySet** (line 154)

```rust
pub struct CapabilitySet {
```

---

**CONST: fn** (line 159)

```rust
pub const fn empty() -> Self {
```

---

**CONST: fn** (line 163)

```rust
pub const fn all() -> Self {
```

---

**CONST: fn** (line 167)

```rust
pub const fn single(cap: Capability) -> Self {
```

---

**CONST: fn** (line 171)

```rust
pub const fn from_bits(bits: u32) -> Self {
```

---

**CONST: fn** (line 175)

```rust
pub const fn has(self, cap: Capability) -> bool {
```

---

**CONST: fn** (line 179)

```rust
pub const fn add(self, cap: Capability) -> Self {
```

---

**CONST: fn** (line 183)

```rust
pub const fn remove(self, cap: Capability) -> Self {
```

---

**CONST: fn** (line 187)

```rust
pub const fn intersect(self, other: Self) -> Self {
```

---

**CONST: fn** (line 191)

```rust
pub const fn union(self, other: Self) -> Self {
```

---

**CONST: fn** (line 195)

```rust
pub const fn diff(self, other: Self) -> Self {
```

---

**CONST: fn** (line 199)

```rust
pub const fn is_empty(self) -> bool {
```

---

**CONST: fn** (line 203)

```rust
pub const fn contains(self, other: Self) -> bool {
```

---

**CONST: fn** (line 207)

```rust
pub const fn count(self) -> usize {
```

---

**CONST: fn** (line 211)

```rust
pub const fn bits(self) -> u32 {
```

---

**FN: iter** (line 215)

```rust
pub fn iter(self) -> impl Iterator<Item = Capability> {
```

---

**STRUCT: CapabilityError** (line 234)

```rust
pub struct CapabilityError {
```

---

**TYPE: CapResult** (line 249)

```rust
pub type CapResult<T> = Result<T, CapabilityError>;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/ast.c

**FUNCTION: cl_arena_init** (line 3)

```c
void cl_arena_init(arena_t *a, uint8_t *buf, size_t cap)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/ast.h

**FUNCTION: cl_arena_init** (line 50)

```c
void cl_arena_init(arena_t *a, uint8_t *buf, size_t cap);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/bridge.c

**FUNCTION: cl_bridge_add** (line 179)

```c
int cl_bridge_add(cl_vm_t *vm, const char *name, cl_ext_fn fn)
```

---

**FUNCTION: cl_bridge_init** (line 184)

```c
int cl_bridge_init(cl_vm_t *vm, uint8_t ring)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/bridge.h

**FUNCTION: cl_bridge_init** (line 6)

```c
int cl_bridge_init(cl_vm_t *vm, uint8_t ring);
```

---

**FUNCTION: cl_bridge_add** (line 8)

```c
int cl_bridge_add(cl_vm_t *vm, const char *name, cl_ext_fn fn);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/lexer.c

**FUNCTION: cl_lexer_init** (line 360)

```c
void cl_lexer_init(lexer_t *l, const char *src, size_t len)
```

---

**FUNCTION: cl_lex_all** (line 369)

```c
int cl_lex_all(lexer_t *l, token_t *buf, size_t cap, size_t *count)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/lexer.h

**FUNCTION: cl_lexer_init** (line 16)

```c
void cl_lexer_init(lexer_t *l, const char *src, size_t len);
```

---

**FUNCTION: cl_lex_all** (line 20)

```c
int cl_lex_all(lexer_t *l, token_t *buf, size_t cap, size_t *count);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/native.h

**FUNCTION: uint64_t** (line 9)

```c
typedef uint64_t (*cl_native_fn)(uint64_t *, uint64_t *, cl_ext_fn *);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/run.c

**FUNCTION: cl_run_script** (line 7)

```c
int cl_run_script(const char *path, uint8_t ring, arena_t *ar)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/sema.c

**FUNCTION: cl_sema_run** (line 719)

```c
int cl_sema_run(sema_ctx_t *s, ast_node_t *prog)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/sema.h

**FUNCTION: cl_sema_run** (line 16)

```c
int cl_sema_run(sema_ctx_t *s, ast_node_t *prog);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/vm.c

**FUNCTION: cl_wbytes** (line 3)

```c
int cl_wbytes(int bits)
```

---

**FUNCTION: cl_mask** (line 14)

```c
void cl_mask(uint8_t *r, int w, int bits)
```

---

**FUNCTION: cl_add** (line 28)

```c
void cl_add(uint8_t *r, const uint8_t *a, const uint8_t *b, int w)
```

---

**FUNCTION: cl_sub** (line 40)

```c
void cl_sub(uint8_t *r, const uint8_t *a, const uint8_t *b, int w)
```

---

**FUNCTION: cl_mul** (line 52)

```c
void cl_mul(uint8_t *r, const uint8_t *a, const uint8_t *b, int w)
```

---

**FUNCTION: cl_cmp** (line 93)

```c
int cl_cmp(const uint8_t *a, const uint8_t *b, int w, bool sign)
```

---

**FUNCTION: cl_shl** (line 116)

```c
void cl_shl(uint8_t *r, const uint8_t *a, uint32_t n, int w)
```

---

**FUNCTION: cl_shr** (line 131)

```c
void cl_shr(uint8_t *r, const uint8_t *a, uint32_t n, int w)
```

---

**FUNCTION: cl_divmod** (line 146)

```c
void cl_divmod(const uint8_t *n, const uint8_t *d, int w,
```

---

**FUNCTION: cl_vm_init** (line 184)

```c
void cl_vm_init(cl_vm_t *vm, cl_prog_t *prog)
```

---

**FUNCTION: cl_vm_register_extern** (line 201)

```c
int cl_vm_register_extern(cl_vm_t *vm, const char *name, cl_ext_fn fn)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/core-lang/vm.h

**FUNCTION: uint64_t** (line 16)

```c
typedef uint64_t (*cl_ext_fn)(uint64_t, uint64_t, uint64_t,
```

---

**FUNCTION: cl_vm_init** (line 44)

```c
void cl_vm_init(cl_vm_t *vm, cl_prog_t *prog);
```

---

**FUNCTION: cl_vm_register_extern** (line 45)

```c
int cl_vm_register_extern(cl_vm_t *vm, const char *name, cl_ext_fn fn);
```

---

**FUNCTION: cl_wbytes** (line 48)

```c
int cl_wbytes(int bits);
```

---

**FUNCTION: cl_mask** (line 49)

```c
void cl_mask(uint8_t *r, int w, int bits);
```

---

**FUNCTION: cl_add** (line 50)

```c
void cl_add(uint8_t *r, const uint8_t *a, const uint8_t *b, int w);
```

---

**FUNCTION: cl_sub** (line 51)

```c
void cl_sub(uint8_t *r, const uint8_t *a, const uint8_t *b, int w);
```

---

**FUNCTION: cl_mul** (line 52)

```c
void cl_mul(uint8_t *r, const uint8_t *a, const uint8_t *b, int w);
```

---

**FUNCTION: cl_cmp** (line 53)

```c
int cl_cmp(const uint8_t *a, const uint8_t *b, int w, bool sign);
```

---

**FUNCTION: cl_shl** (line 54)

```c
void cl_shl(uint8_t *r, const uint8_t *a, uint32_t n, int w);
```

---

**FUNCTION: cl_shr** (line 55)

```c
void cl_shr(uint8_t *r, const uint8_t *a, uint32_t n, int w);
```

---

**FUNCTION: cl_divmod** (line 56)

```c
void cl_divmod(const uint8_t *n, const uint8_t *d, int w,
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/acpi.rs

**STRUCT: CpuEntry** (line 4)

```rust
pub struct CpuEntry {
```

---

**STRUCT: IoApic** (line 9)

```rust
pub struct IoApic {
```

---

**STRUCT: MadtInfo** (line 15)

```rust
pub struct MadtInfo {
```

---

**STRUCT: FadtInfo** (line 21)

```rust
pub struct FadtInfo {
```

---

**STRUCT: Rsdp** (line 25)

```rust
pub struct Rsdp {
```

---

**FN: find_rsdp** (line 57)

```rust
pub fn find_rsdp(phys_offset: u64) -> Option<u64> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/lapic.rs

**FN: init** (line 24)

```rust
pub fn init(base_phys: u64) -> bool {
```

---

**FN: is_x2apic** (line 46)

```rust
pub fn is_x2apic() -> bool {
```

---

**FN: read** (line 50)

```rust
pub fn read(reg: u32) -> u32 {
```

---

**FN: write** (line 60)

```rust
pub fn write(reg: u32, val: u32) {
```

---

**FN: id** (line 70)

```rust
pub fn id() -> u32 {
```

---

**FN: version** (line 78)

```rust
pub fn version() -> u32 {
```

---

**FN: enable_bsp** (line 82)

```rust
pub fn enable_bsp() {
```

---

**FN: enable_ap** (line 87)

```rust
pub fn enable_ap() {
```

---

**FN: eoi** (line 93)

```rust
pub fn eoi() {
```

---

**FN: send_ipi** (line 97)

```rust
pub fn send_ipi(icr: u32, dest_apic_id: u32) {
```

---

**FN: send_init_ipi** (line 109)

```rust
pub fn send_init_ipi() {
```

---

**FN: send_startup_ipi** (line 116)

```rust
pub fn send_startup_ipi(apic_id: u32, vector: u8) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/mod.rs

**FN: init_riscv** (line 18)

```rust
pub fn init_riscv() {
```

---

**FN: total_cpus** (line 24)

```rust
pub fn total_cpus() -> u32 {
```

---

**FN: total_cpus** (line 29)

```rust
pub fn total_cpus() -> u32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/riscv.rs

**FN: init** (line 7)

```rust
pub fn init() {
```

---

**FN: current_hart** (line 12)

```rust
pub fn current_hart() -> u64 {
```

---

**FN: poweroff** (line 17)

```rust
pub fn poweroff() -> ! {
```

---

**FN: reboot** (line 22)

```rust
pub fn reboot() -> ! {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/arch/risc_v/context_swich.rs

**FN: get_switch_count** (line 639)

```rust
pub fn get_switch_count() -> u64 {
```

---

**FN: get_fpu_switch_count** (line 643)

```rust
pub fn get_fpu_switch_count() -> u64 {
```

---

**FN: enable_lazy_fpu** (line 647)

```rust
pub fn enable_lazy_fpu() {
```

---

**FN: disable_lazy_fpu** (line 651)

```rust
pub fn disable_lazy_fpu() {
```

---

**FN: is_lazy_fpu_enabled** (line 655)

```rust
pub fn is_lazy_fpu_enabled() -> bool {
```

---

**FN: satp_mode_name** (line 692)

```rust
pub fn satp_mode_name(satp: u64) -> &'static str {
```

---

**FN: satp_asid** (line 703)

```rust
pub fn satp_asid(satp: u64) -> u64 {
```

---

**FN: satp_ppn** (line 707)

```rust
pub fn satp_ppn(satp: u64) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/arch/x86_64/context_swich.rs

**FN: get_switch_count** (line 731)

```rust
pub fn get_switch_count() -> u64 {
```

---

**FN: get_fpu_switch_count** (line 735)

```rust
pub fn get_fpu_switch_count() -> u64 {
```

---

**FN: enable_lazy_fpu** (line 739)

```rust
pub fn enable_lazy_fpu() {
```

---

**FN: disable_lazy_fpu** (line 743)

```rust
pub fn disable_lazy_fpu() {
```

---

**FN: is_lazy_fpu_enabled** (line 747)

```rust
pub fn is_lazy_fpu_enabled() -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/arch_hooks.rs

**CONST: LOCAL_APIC_BASE** (line 9)

```rust
pub const LOCAL_APIC_BASE: usize = 0xFEE0_0000;
```

---

**CONST: APIC_REG_ID** (line 10)

```rust
pub const APIC_REG_ID: usize = 0x020;
```

---

**CONST: APIC_REG_EOI** (line 11)

```rust
pub const APIC_REG_EOI: usize = 0x0B0;
```

---

**CONST: APIC_REG_SVR** (line 12)

```rust
pub const APIC_REG_SVR: usize = 0x0F0;
```

---

**CONST: APIC_REG_ICR_LOW** (line 13)

```rust
pub const APIC_REG_ICR_LOW: usize = 0x300;
```

---

**CONST: APIC_REG_ICR_HIGH** (line 14)

```rust
pub const APIC_REG_ICR_HIGH: usize = 0x310;
```

---

**CONST: APIC_REG_LVT_TIMER** (line 15)

```rust
pub const APIC_REG_LVT_TIMER: usize = 0x320;
```

---

**CONST: APIC_REG_TIMER_ICR** (line 16)

```rust
pub const APIC_REG_TIMER_ICR: usize = 0x380;
```

---

**CONST: APIC_REG_TIMER_CCR** (line 17)

```rust
pub const APIC_REG_TIMER_CCR: usize = 0x390;
```

---

**CONST: APIC_REG_TIMER_DIV** (line 18)

```rust
pub const APIC_REG_TIMER_DIV: usize = 0x3E0;
```

---

**CONST: RESCHED_VECTOR** (line 20)

```rust
pub const RESCHED_VECTOR: u8 = 0xFD;
```

---

**CONST: TLB_SHOOTDOWN_VECTOR** (line 21)

```rust
pub const TLB_SHOOTDOWN_VECTOR: u8 = 0xFC;
```

---

**CONST: TIMER_VECTOR** (line 22)

```rust
pub const TIMER_VECTOR: u8 = 0xEF;
```

---

**CONST: SPURIOUS_VECTOR** (line 23)

```rust
pub const SPURIOUS_VECTOR: u8 = 0xFF;
```

---

**CONST: NMI_WATCHDOG_VECTOR** (line 24)

```rust
pub const NMI_WATCHDOG_VECTOR: u8 = 0x02;
```

---

**CONST: TICK_HZ** (line 26)

```rust
pub const TICK_HZ: u64 = 1000;
```

---

**CONST: TICK_NS** (line 27)

```rust
pub const TICK_NS: u64 = 1_000_000_000 / TICK_HZ;
```

---

**CONST: RT_WATCHDOG_LIMIT_NS** (line 28)

```rust
pub const RT_WATCHDOG_LIMIT_NS: u64 = 4_000_000_000;
```

---

**CONST: AP_TRAMPOLINE_PAGE** (line 30)

```rust
pub const AP_TRAMPOLINE_PAGE: u32 = 0x8000;
```

---

**CONST: PIT_FREQUENCY_HZ** (line 31)

```rust
pub const PIT_FREQUENCY_HZ: u32 = 1_193_182;
```

---

**FN: register_runqueue** (line 49)

```rust
pub fn register_runqueue(cpu: u32, rq: *mut RunQueue) {
```

---

**FN: register_apic_id** (line 55)

```rust
pub fn register_apic_id(cpu: u32, apic_id: u32) {
```

---

**FN: snapshot_registry** (line 61)

```rust
pub fn snapshot_registry() -> [*mut RunQueue; MAX_CPUS] {
```

---

**FN: set_tsc_frequency** (line 81)

```rust
pub fn set_tsc_frequency(hz: u64) {
```

---

**FN: tsc_frequency** (line 85)

```rust
pub fn tsc_frequency() -> u64 {
```

---

**FN: booted_cpu_count** (line 89)

```rust
pub fn booted_cpu_count() -> u32 {
```

---

**FN: now_ns** (line 183)

```rust
pub fn now_ns() -> u64 {
```

---

**FN: current_cpu_id** (line 270)

```rust
pub fn current_cpu_id() -> u32 {
```

---

**CONST: IDT_SIZE** (line 332)

```rust
pub const IDT_SIZE: usize = 256;
```

---

**CONST: GATE_INTERRUPT** (line 333)

```rust
pub const GATE_INTERRUPT: u8 = 0x8E;
```

---

**CONST: GATE_TRAP** (line 334)

```rust
pub const GATE_TRAP: u8 = 0x8F;
```

---

**STRUCT: IdtEntry** (line 338)

```rust
pub struct IdtEntry {
```

---

**CONST: fn** (line 349)

```rust
pub const fn missing() -> Self {
```

---

**FN: set** (line 353)

```rust
pub fn set(&mut self, handler: usize, selector: u16, ist: u8, type_attr: u8) {
```

---

**FN: is_present** (line 363)

```rust
pub fn is_present(&self) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/class.rs

**STRUCT: SchedClassOps** (line 12)

```rust
pub struct SchedClassOps {
```

---

**STATIC: IDLE_SCHED_CLASS** (line 63)

```rust
pub static IDLE_SCHED_CLASS: SchedClassOps = SchedClassOps {
```

---

**STATIC: FAIR_SCHED_CLASS** (line 122)

```rust
pub static FAIR_SCHED_CLASS: SchedClassOps = SchedClassOps {
```

---

**STATIC: RT_SCHED_CLASS** (line 202)

```rust
pub static RT_SCHED_CLASS: SchedClassOps = SchedClassOps {
```

---

**STATIC: DL_SCHED_CLASS** (line 268)

```rust
pub static DL_SCHED_CLASS: SchedClassOps = SchedClassOps {
```

---

**STATIC: STOP_SCHED_CLASS** (line 317)

```rust
pub static STOP_SCHED_CLASS: SchedClassOps = SchedClassOps {
```

---

**FN: class_of** (line 336)

```rust
pub fn class_of(class: SchedClass) -> &'static SchedClassOps {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/bitmap.rs

**CONST: BITS_PER_WORD** (line 5)

```rust
pub const BITS_PER_WORD: usize = 64;
```

---

**CONST: fn** (line 7)

```rust
pub const fn words_for_bits(nbits: usize) -> usize {
```

---

**STRUCT: Bitmap** (line 27)

```rust
pub struct Bitmap<const WORDS: usize> {
```

---

**CONST: CAPACITY** (line 32)

```rust
pub const CAPACITY: usize = WORDS * BITS_PER_WORD;
```

---

**CONST: fn** (line 34)

```rust
pub const fn new() -> Self {
```

---

**CONST: fn** (line 38)

```rust
pub const fn filled() -> Self {
```

---

**FN: set** (line 43)

```rust
pub fn set(&mut self, bit: usize) {
```

---

**FN: clear** (line 50)

```rust
pub fn clear(&mut self, bit: usize) {
```

---

**FN: toggle** (line 57)

```rust
pub fn toggle(&mut self, bit: usize) {
```

---

**FN: test** (line 64)

```rust
pub fn test(&self, bit: usize) -> bool {
```

---

**FN: set_range** (line 68)

```rust
pub fn set_range(&mut self, start: usize, end: usize) {
```

---

**FN: is_empty** (line 77)

```rust
pub fn is_empty(&self) -> bool {
```

---

**FN: is_full** (line 81)

```rust
pub fn is_full(&self) -> bool {
```

---

**FN: weight** (line 85)

```rust
pub fn weight(&self) -> u32 {
```

---

**FN: find_first_set** (line 89)

```rust
pub fn find_first_set(&self) -> Option<usize> {
```

---

**FN: find_first_zero** (line 98)

```rust
pub fn find_first_zero(&self) -> Option<usize> {
```

---

**FN: find_next_set** (line 111)

```rust
pub fn find_next_set(&self, after: usize) -> Option<usize> {
```

---

**FN: and** (line 126)

```rust
pub fn and(&self, other: &Self) -> Self {
```

---

**FN: or** (line 134)

```rust
pub fn or(&self, other: &Self) -> Self {
```

---

**FN: xor** (line 142)

```rust
pub fn xor(&self, other: &Self) -> Self {
```

---

**FN: andnot** (line 150)

```rust
pub fn andnot(&self, other: &Self) -> Self {
```

---

**FN: intersects** (line 158)

```rust
pub fn intersects(&self, other: &Self) -> bool {
```

---

**FN: iter** (line 162)

```rust
pub fn iter(&self) -> BitmapIter<'_> {
```

---

**STRUCT: BitmapIter** (line 173)

```rust
pub struct BitmapIter<'a> {
```

---

**STRUCT: BitmapSlice** (line 200)

```rust
pub struct BitmapSlice<'a> {
```

---

**FN: new** (line 206)

```rust
pub fn new(words: &'a mut [u64], nbits: usize) -> Self {
```

---

**FN: capacity** (line 211)

```rust
pub fn capacity(&self) -> usize {
```

---

**FN: set** (line 215)

```rust
pub fn set(&mut self, bit: usize) {
```

---

**FN: clear** (line 221)

```rust
pub fn clear(&mut self, bit: usize) {
```

---

**FN: test** (line 227)

```rust
pub fn test(&self, bit: usize) -> bool {
```

---

**FN: weight** (line 231)

```rust
pub fn weight(&self) -> u32 {
```

---

**FN: find_first_zero** (line 235)

```rust
pub fn find_first_zero(&self) -> Option<usize> {
```

---

**FN: clear_all** (line 248)

```rust
pub fn clear_all(&mut self) {
```

---

**STRUCT: AtomicBitmap** (line 263)

```rust
pub struct AtomicBitmap<const WORDS: usize> {
```

---

**CONST: CAPACITY** (line 268)

```rust
pub const CAPACITY: usize = WORDS * BITS_PER_WORD;
```

---

**CONST: fn** (line 270)

```rust
pub const fn new() -> Self {
```

---

**FN: test** (line 275)

```rust
pub fn test(&self, bit: usize) -> bool {
```

---

**FN: test_and_set** (line 280)

```rust
pub fn test_and_set(&self, bit: usize) -> bool {
```

Ustawia bit atomowo, zwraca POPRZEDNI stan (true = już był ustawiony).

---

**FN: test_and_clear** (line 289)

```rust
pub fn test_and_clear(&self, bit: usize) -> bool {
```

Czyści bit atomowo, zwraca POPRZEDNI stan (true = był ustawiony).

---

**FN: weight** (line 297)

```rust
pub fn weight(&self) -> u32 {
```

---

**FN: is_full** (line 301)

```rust
pub fn is_full(&self) -> bool {
```

---

**FN: find_first_zero_and_set** (line 311)

```rust
pub fn find_first_zero_and_set(&self) -> Option<usize> {
```

Bez blokad: skanuje słowa w poszukiwaniu pierwszego wyzerowanego
bitu i próbuje go zająć przez `compare_exchange_weak`. Przegrany
wyścig oznacza, że ktoś inny właśnie zajął ten sam bit (albo inny

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/cpumask.rs

**STRUCT: AtomicCpuMask** (line 17)

```rust
pub struct AtomicCpuMask {
```

---

**CONST: fn** (line 23)

```rust
pub const fn new() -> Self {
```

---

**FN: set** (line 29)

```rust
pub fn set(&self, cpu: u32) {
```

---

**FN: clear** (line 37)

```rust
pub fn clear(&self, cpu: u32) {
```

---

**FN: test** (line 45)

```rust
pub fn test(&self, cpu: u32) -> bool {
```

---

**FN: snapshot** (line 51)

```rust
pub fn snapshot(&self) -> CpuMask {
```

---

**FN: mark_possible** (line 74)

```rust
pub fn mark_possible(cpu: u32) {
```

---

**FN: mark_present** (line 79)

```rust
pub fn mark_present(cpu: u32) {
```

---

**FN: mark_online** (line 84)

```rust
pub fn mark_online(cpu: u32) {
```

---

**FN: mark_offline** (line 90)

```rust
pub fn mark_offline(cpu: u32) {
```

---

**FN: is_online** (line 96)

```rust
pub fn is_online(cpu: u32) -> bool {
```

---

**FN: online_mask** (line 101)

```rust
pub fn online_mask() -> CpuMask {
```

---

**FN: active_mask** (line 106)

```rust
pub fn active_mask() -> CpuMask {
```

---

**FN: set_topology** (line 111)

```rust
pub fn set_topology(cpu: u32, pkg: u32, core: u32) {
```

---

**FN: sibling_mask** (line 119)

```rust
pub fn sibling_mask(cpu: u32) -> CpuMask {
```

---

**FN: package_mask** (line 137)

```rust
pub fn package_mask(cpu: u32) -> CpuMask {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/plist.rs

**STRUCT: PList** (line 44)

```rust
pub struct PList {
```

---

**CONST: fn** (line 49)

```rust
pub const fn new() -> Self {
```

---

**FN: is_empty** (line 53)

```rust
pub fn is_empty(&self) -> bool {
```

---

**FN: first** (line 152)

```rust
pub fn first(&self) -> *mut TaskStruct {
```

Zadanie o najwyższym priorytecie (head listy poziomów),
pierwsze w kolejności FIFO na tym poziomie. O(1).

---

**FN: last** (line 167)

```rust
pub fn last(&self) -> *mut TaskStruct {
```

Zadanie o najniższym priorytecie (ostatni poziom), ostatnie
w kolejności FIFO na tym poziomie.


---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/rbtree.rs

**STRUCT: RbTree** (line 62)

```rust
pub struct RbTree {
```

---

**CONST: fn** (line 68)

```rust
pub const fn new() -> Self {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/rt_array.rs

**STRUCT: RtArray** (line 4)

```rust
pub struct RtArray {
```

---

**CONST: fn** (line 11)

```rust
pub const fn new() -> Self {
```

---

**FN: set_bit** (line 20)

```rust
pub fn set_bit(&mut self, prio: usize) {
```

---

**FN: clear_bit** (line 27)

```rust
pub fn clear_bit(&mut self, prio: usize) {
```

---

**FN: highest_prio** (line 41)

```rust
pub fn highest_prio(&self) -> Option<usize> {
```

---

**FN: active_levels** (line 149)

```rust
pub fn active_levels(&self) -> u32 {
```

Liczba unikalnych aktywnych poziomów priorytetu (nie mylić
z `nr_running`, które liczy wszystkie zakolejkowane zadania).


---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/task.rs

**CONST: MAX_PID** (line 11)

```rust
pub const MAX_PID: u32 = 1_048_576;
```

---

**CONST: RESERVED_PIDS** (line 12)

```rust
pub const RESERVED_PIDS: u32 = 300;
```

---

**STRUCT: PidAllocator** (line 15)

```rust
pub struct PidAllocator {
```

---

**CONST: fn** (line 22)

```rust
pub const fn new() -> Self {
```

---

**FN: alloc** (line 31)

```rust
pub fn alloc(&self) -> Option<TaskId> {
```

---

**FN: free** (line 54)

```rust
pub fn free(&self, pid: TaskId) {
```

---

**FN: reserve** (line 64)

```rust
pub fn reserve(&self, pid: TaskId) -> bool {
```

---

**FN: is_used** (line 77)

```rust
pub fn is_used(&self, pid: TaskId) -> bool {
```

---

**FN: count_allocated** (line 85)

```rust
pub fn count_allocated(&self) -> u32 {
```

---

**CONST: MAX_TASKS** (line 91)

```rust
pub const MAX_TASKS: usize = 65536;
```

---

**STRUCT: TaskTable** (line 94)

```rust
pub struct TaskTable {
```

---

**STRUCT: RtFields** (line 100)

```rust
pub struct RtFields {
```

---

**CONST: fn** (line 107)

```rust
pub const fn new() -> Self {
```

---

**FN: insert** (line 119)

```rust
pub fn insert(&self, task: *mut TaskStruct) -> bool {
```

---

**FN: remove** (line 154)

```rust
pub fn remove(&self, pid: TaskId) -> *mut TaskStruct {
```

---

**FN: lookup_pid** (line 185)

```rust
pub fn lookup_pid(&self, pid: TaskId) -> *mut TaskStruct {
```

---

**FN: lookup_tgid** (line 196)

```rust
pub fn lookup_tgid(&self, tgid: TaskId) -> *mut TaskStruct {
```

---

**FN: count** (line 207)

```rust
pub fn count(&self) -> u32 {
```

---

**STRUCT: TaskIterator** (line 221)

```rust
pub struct TaskIterator<'a> {
```

---

**CONST: RB_LEFT_OFFSET** (line 226)

```rust
pub const RB_LEFT_OFFSET: usize = core::mem::offset_of!(TaskStruct, rb_left);
```

---

**CONST: RB_RIGHT_OFFSET** (line 227)

```rust
pub const RB_RIGHT_OFFSET: usize = core::mem::offset_of!(TaskStruct, rb_right);
```

---

**CONST: RB_PARENT_COLOR_OFFSET** (line 228)

```rust
pub const RB_PARENT_COLOR_OFFSET: usize = core::mem::offset_of!(TaskStruct, rb_parent_color);
```

---

**CONST: PLIST_PRIO_OFFSET** (line 230)

```rust
pub const PLIST_PRIO_OFFSET: usize = core::mem::offset_of!(TaskStruct, plist_prio);
```

---

**CONST: PLIST_SAME_PRIO_OFFSET** (line 231)

```rust
pub const PLIST_SAME_PRIO_OFFSET: usize = core::mem::offset_of!(TaskStruct, plist_same_prio);
```

---

**CONST: PLIST_NODE_OFFSET** (line 232)

```rust
pub const PLIST_NODE_OFFSET: usize = core::mem::offset_of!(TaskStruct, plist_node);
```

---

**CONST: RT_OFFSET** (line 234)

```rust
pub const RT_OFFSET: usize = core::mem::offset_of!(TaskStruct, rt);
```

---

**CONST: RT_RUN_LIST_OFFSET** (line 235)

```rust
pub const RT_RUN_LIST_OFFSET: usize = Self::RT_OFFSET + core::mem::offset_of!(RtFields, run_list);
```

---

**FN: new** (line 279)

```rust
pub fn new(table: &'a TaskTable) -> Self {
```

---

**STRUCT: ThreadGroupIterator** (line 299)

```rust
pub struct ThreadGroupIterator {
```

---

**STRUCT: ChildIterator** (line 334)

```rust
pub struct ChildIterator {
```

---

**STRUCT: DescendantIterator** (line 358)

```rust
pub struct DescendantIterator {
```

---

**STRUCT: ProcessGroup** (line 386)

```rust
pub struct ProcessGroup {
```

---

**FN: new** (line 397)

```rust
pub fn new(pgid: TaskId, leader: *mut TaskStruct) -> Self {
```

---

**FN: is_empty** (line 427)

```rust
pub fn is_empty(&self) -> bool {
```

---

**STRUCT: Session** (line 443)

```rust
pub struct Session {
```

---

**FN: new** (line 453)

```rust
pub fn new(sid: TaskId, leader: *mut TaskStruct) -> Self {
```

---

**FN: is_empty** (line 473)

```rust
pub fn is_empty(&self) -> bool {
```

---

**FN: set_foreground** (line 477)

```rust
pub fn set_foreground(&self, pgid: TaskId) {
```

---

**FN: get_foreground** (line 481)

```rust
pub fn get_foreground(&self) -> TaskId {
```

---

**CONST: MAX_UID** (line 486)

```rust
pub const MAX_UID: usize = 65536;
```

---

**STRUCT: UidTaskCount** (line 488)

```rust
pub struct UidTaskCount {
```

---

**CONST: fn** (line 493)

```rust
pub const fn new() -> Self {
```

---

**FN: inc** (line 498)

```rust
pub fn inc(&self, uid: u32) -> u32 {
```

---

**FN: dec** (line 503)

```rust
pub fn dec(&self, uid: u32) {
```

---

**FN: get** (line 510)

```rust
pub fn get(&self, uid: u32) -> u32 {
```

---

**ENUM: TaskRegistryError** (line 516)

```rust
pub enum TaskRegistryError {
```

---

**STRUCT: TaskRegistry** (line 524)

```rust
pub struct TaskRegistry {
```

---

**CONST: fn** (line 535)

```rust
pub const fn new() -> Self {
```

---

**FN: find_by_pid** (line 585)

```rust
pub fn find_by_pid(&self, pid: TaskId) -> *mut TaskStruct {
```

---

**FN: find_by_tgid** (line 589)

```rust
pub fn find_by_tgid(&self, tgid: TaskId) -> *mut TaskStruct {
```

---

**FN: iter** (line 685)

```rust
pub fn iter(&self) -> TaskIterator {
```

---

**FN: set_init_task** (line 700)

```rust
pub fn set_init_task(&self, task: *mut TaskStruct) {
```

---

**FN: stats** (line 704)

```rust
pub fn stats(&self) -> (u32, u64, u64) {
```

---

**STRUCT: PidNamespace** (line 713)

```rust
pub struct PidNamespace {
```

---

**CONST: fn** (line 722)

```rust
pub const fn new(id: u32, level: u32, parent: *mut PidNamespace) -> Self {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/core.rs

**CONST: PID_MAX** (line 18)

```rust
pub const PID_MAX: usize = 32_768;
```

---

**FN: alloc_pid** (line 22)

```rust
pub fn alloc_pid() -> Option<TaskId> {
```

---

**FN: free_pid** (line 26)

```rust
pub fn free_pid(pid: TaskId) {
```

---

**FN: reserve_pid** (line 34)

```rust
pub fn reserve_pid(pid: TaskId) -> bool {
```

Rezerwuje konkretny PID (np. 0 dla `idle`/`swapper`, albo odtworzenie
procesu z checkpointa). Zwraca `false`, jeśli PID był już zajęty.

---

**FN: pid_in_use** (line 41)

```rust
pub fn pid_in_use(pid: TaskId) -> bool {
```

---

**FN: sys_sched_get_priority_max** (line 219)

```rust
pub fn sys_sched_get_priority_max(policy: SchedPolicy) -> i32 {
```

---

**FN: sys_sched_get_priority_min** (line 227)

```rust
pub fn sys_sched_get_priority_min(policy: SchedPolicy) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/debug/tracepoints.rs

**STRUCT: Tracepoint** (line 1)

```rust
pub struct Tracepoint {}
```

---

**STRUCT: TracePointId** (line 3)

```rust
pub struct TracePointId {}
```

---

**STRUCT: Tracepointkind** (line 5)

```rust
pub struct Tracepointkind{}
```

---

**STRUCT: tracepoints** (line 7)

```rust
pub struct tracepoints {
```

---

**STRUCT: TraceEvent** (line 11)

```rust
pub struct TraceEvent{
```

---

**ENUM: TraceData** (line 15)

```rust
pub enum TraceData{
```

---

**STRUCT: ContextSwichTraceData** (line 19)

```rust
pub struct ContextSwichTraceData{
```

---

**STRUCT: TraceContext** (line 23)

```rust
pub struct TraceContext{
```

---

**STRUCT: TracepointRegistry** (line 27)

```rust
pub struct TracepointRegistry{
```

---

**STRUCT: RegisteredTracepoint** (line 31)

```rust
pub struct RegisteredTracepoint{}
```

---

**STRUCT: TraceBuffer** (line 33)

```rust
pub struct TraceBuffer{}
```

---

**STRUCT: PerCpuTracebuffer** (line 35)

```rust
pub struct PerCpuTracebuffer{}
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/entities/runqueue.rs

**STRUCT: EnqueueFlags** (line 19)

```rust
pub struct EnqueueFlags: u32 {
```

---

**STRUCT: DequeueFlags** (line 30)

```rust
pub struct DequeueFlags: u32 {
```

---

**CONST: MIN_GRANULARITY_NS** (line 37)

```rust
pub const MIN_GRANULARITY_NS: u64 = 750_000; // 0.75 ms
```

---

**CONST: TARGET_LATENCY_NS** (line 38)

```rust
pub const TARGET_LATENCY_NS: u64 = 6_000_000; // 6 ms
```

---

**CONST: BALANCE_INTERVAL_NS** (line 39)

```rust
pub const BALANCE_INTERVAL_NS: u64 = 4_000_000;
```

---

**CONST: IMBALANCE_THRESHOLD** (line 40)

```rust
pub const IMBALANCE_THRESHOLD: u32 = 2;
```

---

**TYPE: KeyOf** (line 45)

```rust
pub type KeyOf = fn(*const TaskStruct) -> u64;
```

---

**FN: insert** (line 240)

```rust
pub fn insert(root: &mut *mut TaskStruct, leftmost: &mut *mut TaskStruct, node: *mut TaskStruct, key_of: KeyOf) {
```

---

**FN: delete** (line 357)

```rust
pub fn delete(root: &mut *mut TaskStruct, leftmost: &mut *mut TaskStruct, z: *mut TaskStruct) {
```

---

**STRUCT: RqFair** (line 457)

```rust
pub struct RqFair {
```

---

**FN: is_empty** (line 478)

```rust
pub fn is_empty(&self) -> bool {
```

---

**FN: leftmost** (line 482)

```rust
pub fn leftmost(&self) -> *mut TaskStruct {
```

---

**FN: enqueue** (line 499)

```rust
pub fn enqueue(&mut self, task: *mut TaskStruct, flags: EnqueueFlags) {
```

Naprawa wady #2: prawdziwe wstawienie do drzewa RB (nie
pojedynczy wskaźnik udający drzewo).

---

**FN: dequeue** (line 510)

```rust
pub fn dequeue(&mut self, task: *mut TaskStruct) {
```

---

**FN: pick_first** (line 520)

```rust
pub fn pick_first(&self) -> *mut TaskStruct {
```

---

**FN: charge_exec** (line 524)

```rust
pub fn charge_exec(&mut self, task: *mut TaskStruct, delta_exec: u64, now: u64) -> u64 {
```

---

**FN: should_preempt** (line 540)

```rust
pub fn should_preempt(&self, current: *const TaskStruct, candidate: *const TaskStruct) -> bool {
```

---

**FN: rb_invariants_ok** (line 552)

```rust
pub fn rb_invariants_ok(&self) -> bool {
```

---

**FN: rb_count** (line 557)

```rust
pub fn rb_count(&self) -> usize {
```

---

**STRUCT: RqDl** (line 568)

```rust
pub struct RqDl {
```

---

**CONST: DL_BW_SCALE** (line 576)

```rust
pub const DL_BW_SCALE: u64 = 1 << 20;
```

---

**FN: is_empty** (line 591)

```rust
pub fn is_empty(&self) -> bool {
```

---

**FN: leftmost** (line 595)

```rust
pub fn leftmost(&self) -> *mut TaskStruct {
```

---

**FN: admission_control** (line 599)

```rust
pub fn admission_control(&self, dl_runtime: u64, dl_period: u64) -> bool {
```

---

**FN: enqueue** (line 618)

```rust
pub fn enqueue(&mut self, task: *mut TaskStruct, now: u64, flags: EnqueueFlags) {
```

---

**FN: dequeue** (line 639)

```rust
pub fn dequeue(&mut self, task: *mut TaskStruct, removing_permanently: bool) {
```

---

**FN: pick_first** (line 649)

```rust
pub fn pick_first(&self) -> *mut TaskStruct {
```

---

**FN: update_curr** (line 653)

```rust
pub fn update_curr(&mut self, task: *mut TaskStruct, delta_exec: u64, now: u64) {
```

---

**FN: should_preempt** (line 670)

```rust
pub fn should_preempt(&self, current: *const TaskStruct, candidate: *const TaskStruct) -> bool {
```

---

**FN: rb_invariants_ok** (line 675)

```rust
pub fn rb_invariants_ok(&self) -> bool {
```

---

**STRUCT: RqRt** (line 684)

```rust
pub struct RqRt {
```

---

**FN: new** (line 694)

```rust
pub fn new() -> Self {
```

---

**FN: is_empty** (line 710)

```rust
pub fn is_empty(&self) -> bool {
```

---

**FN: enqueue** (line 735)

```rust
pub fn enqueue(&mut self, task: *mut TaskStruct, flags: EnqueueFlags) {
```

---

**FN: dequeue** (line 769)

```rust
pub fn dequeue(&mut self, task: *mut TaskStruct) {
```

---

**FN: pick_first** (line 793)

```rust
pub fn pick_first(&self) -> *mut TaskStruct {
```

---

**FN: requeue** (line 809)

```rust
pub fn requeue(&mut self, task: *mut TaskStruct) {
```

---

**FN: highest_priority** (line 820)

```rust
pub fn highest_priority(&self) -> i32 {
```

---

**STRUCT: RqStop** (line 828)

```rust
pub struct RqStop {
```

---

**FN: is_empty** (line 833)

```rust
pub fn is_empty(&self) -> bool {
```

---

**FN: enqueue** (line 837)

```rust
pub fn enqueue(&mut self, task: *mut TaskStruct) {
```

---

**FN: dequeue** (line 843)

```rust
pub fn dequeue(&mut self, task: *mut TaskStruct) {
```

---

**FN: pick_first** (line 850)

```rust
pub fn pick_first(&self) -> *mut TaskStruct {
```

---

**STRUCT: RunQueue** (line 857)

```rust
pub struct RunQueue {
```

---

**FN: new** (line 882)

```rust
pub fn new(cpu: u32, idle_task: *mut TaskStruct) -> Self {
```

---

**FN: bind_idle_task** (line 914)

```rust
pub fn bind_idle_task(&self) {
```

---

**FN: current** (line 930)

```rust
pub fn current(&self) -> *mut TaskStruct {
```

---

**FN: nr_running** (line 938)

```rust
pub fn nr_running(&self) -> u32 {
```

---

**FN: is_idle** (line 942)

```rust
pub fn is_idle(&self) -> bool {
```

---

**FN: fair_root_for_test** (line 1214)

```rust
pub fn fair_root_for_test(&self) -> *mut TaskStruct {
```

---

**TYPE: RunQueueRegistry** (line 1267)

```rust
pub type RunQueueRegistry<'a> = &'a [*mut RunQueue];
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/entities/stats.rs

**FN: account_switch** (line 21)

```rust
pub fn account_switch() {
```

---

**FN: account_voluntary_switch** (line 25)

```rust
pub fn account_voluntary_switch(task: &mut TaskStruct) {
```

---

**FN: account_involuntary_switch** (line 30)

```rust
pub fn account_involuntary_switch(task: &mut TaskStruct) {
```

---

**FN: account_migration** (line 35)

```rust
pub fn account_migration(task: &mut TaskStruct) {
```

---

**FN: account_user_time** (line 41)

```rust
pub fn account_user_time(task: &mut TaskStruct, delta_ns: u64) {
```

---

**FN: account_system_time** (line 48)

```rust
pub fn account_system_time(task: &mut TaskStruct, delta_ns: u64) {
```

---

**FN: account_idle_time** (line 55)

```rust
pub fn account_idle_time(delta_ns: u64) {
```

---

**FN: account_iowait_time** (line 59)

```rust
pub fn account_iowait_time(delta_ns: u64) {
```

---

**FN: total_switches** (line 64)

```rust
pub fn total_switches() -> u64 { TOTAL_SWITCHES.load(Ordering::Relaxed) }
```

---

**FN: total_voluntary_switches** (line 65)

```rust
pub fn total_voluntary_switches() -> u64 { TOTAL_VOLUNTARY_SWITCHES.load(Ordering::Relaxed) }
```

---

**FN: total_involuntary_switches** (line 66)

```rust
pub fn total_involuntary_switches() -> u64 { TOTAL_INVOLUNTARY_SWITCHES.load(Ordering::Relaxed) }
```

---

**FN: total_migrations** (line 67)

```rust
pub fn total_migrations() -> u64 { TOTAL_MIGRATIONS.load(Ordering::Relaxed) }
```

---

**FN: total_user_time_ns** (line 68)

```rust
pub fn total_user_time_ns() -> u64 { TOTAL_USER_TIME_NS.load(Ordering::Relaxed) }
```

---

**FN: total_system_time_ns** (line 69)

```rust
pub fn total_system_time_ns() -> u64 { TOTAL_SYSTEM_TIME_NS.load(Ordering::Relaxed) }
```

---

**FN: total_idle_time_ns** (line 70)

```rust
pub fn total_idle_time_ns() -> u64 { TOTAL_IDLE_TIME_NS.load(Ordering::Relaxed) }
```

---

**FN: total_iowait_time_ns** (line 71)

```rust
pub fn total_iowait_time_ns() -> u64 { TOTAL_IOWAIT_TIME_NS.load(Ordering::Relaxed) }
```

---

**FN: total_running_time_ns** (line 72)

```rust
pub fn total_running_time_ns() -> u64 { TOTAL_RUNNING_TIME_NS.load(Ordering::Relaxed) }
```

---

**STRUCT: RqStats** (line 77)

```rust
pub struct RqStats {
```

---

**CONST: fn** (line 107)

```rust
pub const fn new() -> Self {
```

---

**FN: record_switch** (line 126)

```rust
pub fn record_switch(&mut self, voluntary: bool) {
```

---

**FN: record_migration** (line 143)

```rust
pub fn record_migration(&mut self) {
```

---

**FN: add_user_time** (line 149)

```rust
pub fn add_user_time(&mut self, delta_ns: u64) {
```

---

**FN: add_system_time** (line 157)

```rust
pub fn add_system_time(&mut self, delta_ns: u64) {
```

---

**FN: add_idle_time** (line 165)

```rust
pub fn add_idle_time(&mut self, delta_ns: u64) {
```

---

**FN: add_iowait_time** (line 171)

```rust
pub fn add_iowait_time(&mut self, delta_ns: u64) {
```

---

**FN: set_nr_running** (line 177)

```rust
pub fn set_nr_running(&mut self, nr: u32) {
```

---

**FN: set_nr_uninterruptible** (line 182)

```rust
pub fn set_nr_uninterruptible(&mut self, nr: u32) {
```

---

**FN: update_load_stats** (line 187)

```rust
pub fn update_load_stats(&mut self, sum_vruntime: u64, sum_weight: u64) {
```

---

**FN: load_percent** (line 193)

```rust
pub fn load_percent(&self) -> u32 {
```

Oblicza przybliżone obciążenie CPU (0-100).

---

**FN: load_avg_1min** (line 204)

```rust
pub fn load_avg_1min(&self) -> f32 {
```

---

**STRUCT: ExecTimeHistogram** (line 213)

```rust
pub struct ExecTimeHistogram {
```

---

**CONST: fn** (line 221)

```rust
pub const fn new() -> Self {
```

---

**FN: add_sample** (line 229)

```rust
pub fn add_sample(&mut self, exec_ns: u64) {
```

---

**FN: average_ns** (line 251)

```rust
pub fn average_ns(&self) -> f32 {
```

---

**FN: total_samples** (line 259)

```rust
pub fn total_samples(&self) -> u64 {
```

---

**STRUCT: SystemStats** (line 273)

```rust
pub struct SystemStats {
```

---

**STRUCT: CpuStatsSnapshot** (line 287)

```rust
pub struct CpuStatsSnapshot {
```

---

**FN: collect_system_stats** (line 298)

```rust
pub fn collect_system_stats() -> SystemStats {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/entities/task.rs

**TYPE: TaskId** (line 13)

```rust
pub type TaskId = u64;
```

---

**TYPE: Pid** (line 14)

```rust
pub type Pid = TaskId;
```

---

**TYPE: Tgid** (line 15)

```rust
pub type Tgid = TaskId;
```

---

**CONST: KERNEL_STACK_SIZE** (line 17)

```rust
pub const KERNEL_STACK_SIZE: usize = 16 * 1024;
```

---

**CONST: KERNEL_STACK_PAGES** (line 18)

```rust
pub const KERNEL_STACK_PAGES: usize = KERNEL_STACK_SIZE / 4096;
```

---

**CONST: TASK_COMM_LEN** (line 19)

```rust
pub const TASK_COMM_LEN: usize = 16;
```

---

**CONST: NICE_MIN** (line 21)

```rust
pub const NICE_MIN: i8 = -20;
```

---

**CONST: NICE_MAX** (line 22)

```rust
pub const NICE_MAX: i8 = 19;
```

---

**CONST: NICE_WIDTH** (line 23)

```rust
pub const NICE_WIDTH: usize = (NICE_MAX - NICE_MIN + 1) as usize;
```

---

**CONST: MAX_RT_PRIO** (line 25)

```rust
pub const MAX_RT_PRIO: i32 = 100;
```

---

**CONST: MAX_PRIO** (line 26)

```rust
pub const MAX_PRIO: i32 = 140;
```

---

**CONST: DEFAULT_PRIO** (line 27)

```rust
pub const DEFAULT_PRIO: i32 = MAX_RT_PRIO + 20;
```

---

**CONST: NICE_0_LOAD** (line 29)

```rust
pub const NICE_0_LOAD: u64 = 1024;
```

---

**CONST: MAX_CPUS** (line 31)

```rust
pub const MAX_CPUS: usize = 256;
```

---

**CONST: RLIM_NLIMITS** (line 34)

```rust
pub const RLIM_NLIMITS: usize = 10;
```

---

**CONST: RLIM_INFINITY** (line 35)

```rust
pub const RLIM_INFINITY: u64 = u64::MAX;
```

---

**CONST: CPU_NONE** (line 37)

```rust
pub const CPU_NONE: u32 = u32::MAX;
```

---

**STRUCT: TaskFlags** (line 44)

```rust
pub struct TaskFlags: u32 {
```

---

**STRUCT: AtomicTaskFlags** (line 73)

```rust
pub struct AtomicTaskFlags(AtomicU32);
```

---

**CONST: fn** (line 76)

```rust
pub const fn new(flags: TaskFlags) -> Self {
```

---

**CONST: fn** (line 80)

```rust
pub const fn empty() -> Self {
```

---

**FN: load** (line 85)

```rust
pub fn load(&self, order: Ordering) -> TaskFlags {
```

---

**FN: store** (line 90)

```rust
pub fn store(&self, flags: TaskFlags, order: Ordering) {
```

---

**FN: fetch_insert** (line 95)

```rust
pub fn fetch_insert(&self, flags: TaskFlags) -> TaskFlags {
```

---

**FN: fetch_remove** (line 102)

```rust
pub fn fetch_remove(&self, flags: TaskFlags) -> TaskFlags {
```

---

**FN: contains** (line 108)

```rust
pub fn contains(&self, flags: TaskFlags) -> bool {
```

---

**FN: test_and_set** (line 113)

```rust
pub fn test_and_set(&self, flags: TaskFlags) -> bool {
```

---

**ENUM: TaskState** (line 127)

```rust
pub enum TaskState {
```

---

**CONST: fn** (line 139)

```rust
pub const fn is_runnable(self) -> bool {
```

---

**CONST: fn** (line 143)

```rust
pub const fn is_terminal(self) -> bool {
```

---

**CONST: fn** (line 147)

```rust
pub const fn is_waiting(self) -> bool {
```

---

**CONST: fn** (line 157)

```rust
pub const fn can_transition_to(self, next: TaskState) -> bool {
```

---

**ENUM: SchedPolicy** (line 208)

```rust
pub enum SchedPolicy {
```

---

**CONST: fn** (line 219)

```rust
pub const fn is_realtime(self) -> bool {
```

---

**CONST: fn** (line 223)

```rust
pub const fn is_fair(self) -> bool {
```

---

**ENUM: SchedClass** (line 230)

```rust
pub enum SchedClass {
```

---

**STRUCT: CpuMask** (line 253)

```rust
pub struct CpuMask {
```

---

**CONST: fn** (line 258)

```rust
pub const fn empty() -> Self {
```

---

**CONST: fn** (line 262)

```rust
pub const fn all() -> Self {
```

---

**FN: single** (line 266)

```rust
pub fn single(cpu: u32) -> Self {
```

---

**FN: first_n** (line 272)

```rust
pub fn first_n(n: u32) -> Self {
```

---

**FN: set** (line 281)

```rust
pub fn set(&mut self, cpu: u32) {
```

---

**FN: clear** (line 288)

```rust
pub fn clear(&mut self, cpu: u32) {
```

---

**FN: is_set** (line 295)

```rust
pub fn is_set(&self, cpu: u32) -> bool {
```

---

**FN: is_empty** (line 300)

```rust
pub fn is_empty(&self) -> bool {
```

---

**FN: count** (line 304)

```rust
pub fn count(&self) -> u32 {
```

---

**FN: intersects** (line 308)

```rust
pub fn intersects(&self, other: &CpuMask) -> bool {
```

---

**FN: and** (line 312)

```rust
pub fn and(&self, other: &CpuMask) -> CpuMask {
```

---

**FN: or** (line 320)

```rust
pub fn or(&self, other: &CpuMask) -> CpuMask {
```

---

**FN: first** (line 328)

```rust
pub fn first(&self) -> Option<u32> {
```

---

**FN: next_after** (line 337)

```rust
pub fn next_after(&self, after: u32) -> Option<u32> {
```

---

**FN: iter** (line 352)

```rust
pub fn iter(&self) -> CpuMaskIter {
```

---

**STRUCT: CpuMaskIter** (line 363)

```rust
pub struct CpuMaskIter {
```

---

**FN: nice_to_weight** (line 406)

```rust
pub fn nice_to_weight(nice: i8) -> u64 {
```

---

**FN: nice_to_wmult** (line 411)

```rust
pub fn nice_to_wmult(nice: i8) -> u32 {
```

---

**FN: weight_to_nice** (line 416)

```rust
pub fn weight_to_nice(weight: u64) -> i8 {
```

---

**CONST: WMULT_SHIFT** (line 429)

```rust
pub const WMULT_SHIFT: u32 = 32;
```

---

**FN: calc_delta_fair** (line 431)

```rust
pub fn calc_delta_fair(delta_exec: u64, weight: u64, inv_weight: u32) -> u64 {
```

---

**STRUCT: InterruptFrame** (line 442)

```rust
pub struct InterruptFrame {
```

---

**STRUCT: FxSaveArea** (line 452)

```rust
pub struct FxSaveArea {
```

---

**CONST: fn** (line 457)

```rust
pub const fn zeroed() -> Self {
```

---

**STRUCT: CpuContext** (line 476)

```rust
pub struct CpuContext {
```

---

**FN: xsave_area_size** (line 548)

```rust
pub fn xsave_area_size() -> usize {
```

---

**CONST: RB_RED** (line 553)

```rust
pub const RB_RED: usize = 0;
```

---

**CONST: RB_BLACK** (line 554)

```rust
pub const RB_BLACK: usize = 1;
```

---

**FN: rb_parent** (line 557)

```rust
pub fn rb_parent(pc: usize) -> *mut TaskStruct {
```

---

**FN: rb_color** (line 562)

```rust
pub fn rb_color(pc: usize) -> usize {
```

---

**FN: rb_is_red** (line 567)

```rust
pub fn rb_is_red(pc: usize) -> bool {
```

---

**FN: rb_is_black** (line 572)

```rust
pub fn rb_is_black(pc: usize) -> bool {
```

---

**FN: rb_make_parent_color** (line 577)

```rust
pub fn rb_make_parent_color(parent: *mut TaskStruct, color: usize) -> usize {
```

---

**STRUCT: LoadAvg** (line 583)

```rust
pub struct LoadAvg {
```

---

**FN: accumulate** (line 595)

```rust
pub fn accumulate(&mut self, now: u64, delta_ns: u64, weight: u64, running: bool) {
```

---

**STRUCT: SchedEntity** (line 624)

```rust
pub struct SchedEntity {
```

---

**STRUCT: RtSchedEntity** (line 671)

```rust
pub struct RtSchedEntity {
```

---

**STRUCT: PlistNode** (line 703)

```rust
pub struct PlistNode {
```

---

**FN: init** (line 711)

```rust
pub fn init(&mut self, owner: *mut TaskStruct) {
```

---

**STRUCT: DlSchedEntity** (line 731)

```rust
pub struct DlSchedEntity {
```

---

**STRUCT: TaskStats** (line 762)

```rust
pub struct TaskStats {
```

---

**STRUCT: Credentials** (line 781)

```rust
pub struct Credentials {
```

---

**CONST: fn** (line 794)

```rust
pub const fn kernel() -> Self {
```

---

**CONST: fn** (line 808)

```rust
pub const fn user(uid: u32, gid: u32) -> Self {
```

---

**STRUCT: RLimit** (line 831)

```rust
pub struct RLimit {
```

---

**CONST: fn** (line 837)

```rust
pub const fn unlimited() -> Self {
```

---

**CONST: fn** (line 841)

```rust
pub const fn bounded(cur: u64, max: u64) -> Self {
```

---

**ENUM: RlimitResource** (line 848)

```rust
pub enum RlimitResource {
```

---

**CONST: fn** (line 861)

```rust
pub const fn default_rlimits() -> [RLimit; RLIM_NLIMITS] {
```

---

**STRUCT: SignalState** (line 872)

```rust
pub struct SignalState {
```

---

**FN: has_pending** (line 878)

```rust
pub fn has_pending(&self) -> bool {
```

---

**FN: raise** (line 882)

```rust
pub fn raise(&mut self, signum: u8) {
```

---

**FN: clear** (line 888)

```rust
pub fn clear(&mut self, signum: u8) {
```

---

**STRUCT: ListHead** (line 898)

```rust
pub struct ListHead {
```

---

**CONST: fn** (line 904)

```rust
pub const fn new() -> Self {
```

---

**FN: init** (line 908)

```rust
pub fn init(&mut self) {
```

---

**FN: is_empty** (line 914)

```rust
pub fn is_empty(&self) -> bool {
```

---

**FN: is_linked** (line 949)

```rust
pub fn is_linked(&self) -> bool {
```

---

**STRUCT: SpinLock** (line 997)

```rust
pub struct SpinLock {
```

---

**CONST: fn** (line 1004)

```rust
pub const fn new() -> Self {
```

---

**FN: lock** (line 1012)

```rust
pub fn lock(&self) {
```

---

**FN: unlock** (line 1024)

```rust
pub fn unlock(&self) {
```

---

**FN: try_lock** (line 1028)

```rust
pub fn try_lock(&self) -> bool {
```

---

**FN: is_locked** (line 1034)

```rust
pub fn is_locked(&self) -> bool {
```

---

**FN: lock_irqsave** (line 1038)

```rust
pub fn lock_irqsave(&self) -> usize {
```

---

**FN: unlock_irqrestore** (line 1043)

```rust
pub fn unlock_irqrestore(&self, flags: usize) {
```

---

**FN: lock_guard** (line 1048)

```rust
pub fn lock_guard(&self) -> SpinLockGuard<'_> {
```

---

**FN: lock_irqsave_guard** (line 1053)

```rust
pub fn lock_irqsave_guard(&self) -> SpinLockGuard<'_> {
```

---

**STRUCT: SpinLockGuard** (line 1060)

```rust
pub struct SpinLockGuard<'a> {
```

---

**ENUM: TaskError** (line 1076)

```rust
pub enum TaskError {
```

---

**STRUCT: TaskStruct** (line 1112)

```rust
pub struct TaskStruct {
```

---

**FN: state** (line 1403)

```rust
pub fn state(&self) -> TaskState {
```

---

**FN: set_state** (line 1410)

```rust
pub fn set_state(&self, new_state: TaskState) -> Result<(), TaskError> {
```

---

**FN: wake_up** (line 1433)

```rust
pub fn wake_up(&self) -> Result<(), TaskError> {
```

---

**FN: sleep** (line 1441)

```rust
pub fn sleep(&self, interruptible: bool) -> Result<(), TaskError> {
```

---

**FN: is_kernel_thread** (line 1446)

```rust
pub fn is_kernel_thread(&self) -> bool {
```

---

**FN: is_idle_task** (line 1450)

```rust
pub fn is_idle_task(&self) -> bool {
```

---

**FN: is_zombie** (line 1454)

```rust
pub fn is_zombie(&self) -> bool {
```

---

**FN: is_runnable** (line 1458)

```rust
pub fn is_runnable(&self) -> bool {
```

---

**FN: needs_resched** (line 1462)

```rust
pub fn needs_resched(&self) -> bool {
```

---

**FN: set_need_resched** (line 1466)

```rust
pub fn set_need_resched(&self) {
```

---

**FN: clear_need_resched** (line 1470)

```rust
pub fn clear_need_resched(&self) {
```

---

**FN: set_nice** (line 1474)

```rust
pub fn set_nice(&mut self, nice: i8) -> Result<(), TaskError> {
```

---

**FN: set_rt_priority** (line 1487)

```rust
pub fn set_rt_priority(&mut self, rt_priority: u8) -> Result<(), TaskError> {
```

---

**FN: effective_prio** (line 1501)

```rust
pub fn effective_prio(&self) -> i32 {
```

---

**FN: set_affinity** (line 1505)

```rust
pub fn set_affinity(&mut self, mask: CpuMask) -> Result<(), TaskError> {
```

---

**FN: can_run_on** (line 1517)

```rust
pub fn can_run_on(&self, cpu: u32) -> bool {
```

---

**FN: rq_ptr** (line 1522)

```rust
pub fn rq_ptr(&self) -> *mut core::ffi::c_void {
```

nigdzie.

---

**FN: set_rq_ptr** (line 1526)

```rust
pub fn set_rq_ptr(&self, rq: *mut core::ffi::c_void) {
```

---

**FN: charge_cputime** (line 1530)

```rust
pub fn charge_cputime(&mut self, delta_ns: u64) {
```

---

**FN: record_voluntary_switch** (line 1549)

```rust
pub fn record_voluntary_switch(&mut self) {
```

---

**FN: record_involuntary_switch** (line 1553)

```rust
pub fn record_involuntary_switch(&mut self) {
```

---

**FN: record_migration** (line 1557)

```rust
pub fn record_migration(&mut self, new_cpu: u32) {
```

---

**FN: set_comm** (line 1564)

```rust
pub fn set_comm(&mut self, name: &str) {
```

---

**FN: comm_str** (line 1571)

```rust
pub fn comm_str(&self) -> &str {
```

---

**STRUCT: ThreadInfoView** (line 1608)

```rust
pub struct ThreadInfoView<'a> {
```

---

**FN: thread_info** (line 1618)

```rust
pub fn thread_info(&mut self) -> ThreadInfoView<'_> {
```

---

**CONST: fn** (line 1630)

```rust
pub const fn default_time_slice(policy: SchedPolicy) -> u32 {
```

---

**FN: fair_has_priority** (line 1641)

```rust
pub fn fair_has_priority(a: &TaskStruct, b: &TaskStruct) -> bool {
```

---

**FN: deadline_has_priority** (line 1645)

```rust
pub fn deadline_has_priority(a: &TaskStruct, b: &TaskStruct) -> bool {
```

---

**FN: blank** (line 1652)

```rust
pub fn blank() -> TaskStruct {
```

---

**FN: init_test_stub** (line 1696)

```rust
pub fn init_test_stub(&mut self, pid: TaskId, policy: SchedPolicy, nice: i8) {
```

---

**CONST: PLIST_PRIO_OFFSET** (line 2188)

```rust
pub const PLIST_PRIO_OFFSET: usize = core::mem::offset_of!(TaskStruct, prio);
```

---

**CONST: PLIST_SAME_PRIO_OFFSET** (line 2189)

```rust
pub const PLIST_SAME_PRIO_OFFSET: usize = core::mem::offset_of!(TaskStruct, thread_group);
```

---

**CONST: PLIST_NODE_OFFSET** (line 2190)

```rust
pub const PLIST_NODE_OFFSET: usize = core::mem::offset_of!(TaskStruct, tasks);
```

---

**CONST: RB_LEFT_OFFSET** (line 2191)

```rust
pub const RB_LEFT_OFFSET: usize = core::mem::offset_of!(TaskStruct, se.rb_left);
```

---

**CONST: RB_RIGHT_OFFSET** (line 2192)

```rust
pub const RB_RIGHT_OFFSET: usize = core::mem::offset_of!(TaskStruct, se.rb_right);
```

---

**CONST: RB_PARENT_COLOR_OFFSET** (line 2193)

```rust
pub const RB_PARENT_COLOR_OFFSET: usize = core::mem::offset_of!(TaskStruct, se.rb_parent_color);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/mod.rs

**FN: self_test** (line 57)

```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

---

**FN: current_cpu_id** (line 68)

```rust
pub fn current_cpu_id() -> u32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/power/em.rs

**CONST: MAX_OPP_COUNT** (line 6)

```rust
pub const MAX_OPP_COUNT: usize = 16;
```

---

**CONST: MAX_PD_COUNT** (line 7)

```rust
pub const MAX_PD_COUNT: usize = 8;
```

---

**CONST: EM_MAX_POWER** (line 8)

```rust
pub const EM_MAX_POWER: u32 = 100_000_000;
```

---

**CONST: EM_MIN_POWER** (line 9)

```rust
pub const EM_MIN_POWER: u32 = 100;
```

---

**CONST: EM_FREQ_SCALE** (line 10)

```rust
pub const EM_FREQ_SCALE: u64 = 1_000_000;
```

---

**CONST: EM_POWER_SCALE** (line 11)

```rust
pub const EM_POWER_SCALE: u64 = 1_000;
```

---

**CONST: EM_CAPACITY_SCALE** (line 12)

```rust
pub const EM_CAPACITY_SCALE: u64 = 1024;
```

---

**STRUCT: CapacityState** (line 16)

```rust
pub struct CapacityState {
```

---

**CONST: fn** (line 28)

```rust
pub const fn empty() -> Self {
```

---

**CONST: fn** (line 41)

```rust
pub const fn new(freq: u32, power: u32, cap: u32, volt: u32, lat: u32) -> Self {
```

---

**FN: compute_cost** (line 54)

```rust
pub fn compute_cost(&mut self, max_cap: u32) {
```

---

**FN: is_valid** (line 64)

```rust
pub fn is_valid(&self) -> bool {
```

---

**STRUCT: PerformanceDomain** (line 70)

```rust
pub struct PerformanceDomain {
```

---

**CONST: fn** (line 94)

```rust
pub const fn empty() -> Self {
```

---

**FN: init** (line 120)

```rust
pub fn init(&mut self, id: u32, cpus: CpuMask) {
```

---

**FN: add_state** (line 132)

```rust
pub fn add_state(&mut self, state: CapacityState) -> bool {
```

---

**FN: finalize** (line 151)

```rust
pub fn finalize(&mut self) {
```

---

**FN: find_state_for_capacity** (line 173)

```rust
pub fn find_state_for_capacity(&self, target_cap: u32) -> Option<&CapacityState> {
```

---

**FN: find_state_for_frequency** (line 186)

```rust
pub fn find_state_for_frequency(&self, target_freq: u32) -> Option<&CapacityState> {
```

---

**FN: get_effective_capacity** (line 203)

```rust
pub fn get_effective_capacity(&self) -> u32 {
```

---

**FN: update_thermal_pressure** (line 207)

```rust
pub fn update_thermal_pressure(&self, pressure: u32) {
```

---

**FN: compute_power_at_state** (line 218)

```rust
pub fn compute_power_at_state(&self, state_idx: usize, temp_millideg: u32) -> u32 {
```

---

**FN: update_temperature** (line 237)

```rust
pub fn update_temperature(&self, power_mw: u32, delta_time_ms: u32) {
```

---

**FN: contains_cpu** (line 252)

```rust
pub fn contains_cpu(&self, cpu: u32) -> bool {
```

---

**FN: get_active_state_idx** (line 256)

```rust
pub fn get_active_state_idx(&self) -> u32 {
```

---

**FN: set_active_state_idx** (line 260)

```rust
pub fn set_active_state_idx(&self, idx: u32) {
```

---

**STRUCT: EnergyModel** (line 268)

```rust
pub struct EnergyModel {
```

---

**CONST: fn** (line 282)

```rust
pub const fn empty() -> Self {
```

---

**FN: init** (line 298)

```rust
pub fn init(&mut self) {
```

---

**FN: register_domain** (line 307)

```rust
pub fn register_domain(&mut self, pd: PerformanceDomain) -> bool {
```

---

**FN: get_pd_for_cpu** (line 329)

```rust
pub fn get_pd_for_cpu(&self, cpu: u32) -> Option<&PerformanceDomain> {
```

---

**FN: get_pd_for_cpu_mut** (line 340)

```rust
pub fn get_pd_for_cpu_mut(&mut self, cpu: u32) -> Option<&mut PerformanceDomain> {
```

---

**FN: compute_system_energy** (line 351)

```rust
pub fn compute_system_energy(&self, delta_time_ms: u32) -> u32 {
```

---

**FN: compute_task_energy_on_cpu** (line 365)

```rust
pub fn compute_task_energy_on_cpu(&self, task_util: u32, cpu: u32) -> u32 {
```

---

**FN: find_best_state_for_domain** (line 381)

```rust
pub fn find_best_state_for_domain(&self, pd_idx: u32, target_util: u32) -> Option<usize> {
```

---

**FN: invert_capacity_for_thermal** (line 404)

```rust
pub fn invert_capacity_for_thermal(&mut self, cpu: u32, thermal_pressure: u32) {
```

---

**FN: clear_thermal_pressure** (line 410)

```rust
pub fn clear_thermal_pressure(&mut self) {
```

---

**FN: get_max_capacity** (line 416)

```rust
pub fn get_max_capacity(&self) -> u32 {
```

---

**FN: get_min_capacity** (line 420)

```rust
pub fn get_min_capacity(&self) -> u32 {
```

---

**FN: is_cpu_big** (line 424)

```rust
pub fn is_cpu_big(&self, cpu: u32) -> bool {
```

---

**FN: is_cpu_little** (line 432)

```rust
pub fn is_cpu_little(&self, cpu: u32) -> bool {
```

---

**FN: dump_state** (line 440)

```rust
pub fn dump_state(&self) {
```

---

**STATIC: mut** (line 447)

```rust
pub static mut GLOBAL_ENERGY_MODEL: EnergyModel = EnergyModel::empty();
```

---

**FN: em_build_synthetic_little_pd** (line 484)

```rust
pub fn em_build_synthetic_little_pd(cpus: CpuMask) -> PerformanceDomain {
```

---

**FN: em_build_synthetic_big_pd** (line 502)

```rust
pub fn em_build_synthetic_big_pd(cpus: CpuMask) -> PerformanceDomain {
```

---

**FN: em_build_synthetic_huge_pd** (line 521)

```rust
pub fn em_build_synthetic_huge_pd(cpus: CpuMask) -> PerformanceDomain {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/power/mod.rs

**CONST: THERMAL_NORMAL** (line 8)

```rust
pub const THERMAL_NORMAL: u32 = 0;
```

---

**CONST: THERMAL_LIGHT** (line 9)

```rust
pub const THERMAL_LIGHT: u32 = 1;
```

---

**CONST: THERMAL_MEDIUM** (line 10)

```rust
pub const THERMAL_MEDIUM: u32 = 2;
```

---

**CONST: THERMAL_SEVERE** (line 11)

```rust
pub const THERMAL_SEVERE: u32 = 3;
```

---

**CONST: THERMAL_CRITICAL** (line 12)

```rust
pub const THERMAL_CRITICAL: u32 = 4;
```

---

**CONST: TEMP_NORMAL_MAX** (line 14)

```rust
pub const TEMP_NORMAL_MAX: u32 = 60000;
```

---

**CONST: TEMP_LIGHT_MAX** (line 15)

```rust
pub const TEMP_LIGHT_MAX: u32 = 75000;
```

---

**CONST: TEMP_MEDIUM_MAX** (line 16)

```rust
pub const TEMP_MEDIUM_MAX: u32 = 85000;
```

---

**CONST: TEMP_SEVERE_MAX** (line 17)

```rust
pub const TEMP_SEVERE_MAX: u32 = 95000;
```

---

**CONST: TEMP_CRITICAL_MAX** (line 18)

```rust
pub const TEMP_CRITICAL_MAX: u32 = 105000;
```

---

**CONST: PID_KP** (line 20)

```rust
pub const PID_KP: i64 = 50;
```

---

**CONST: PID_KI** (line 21)

```rust
pub const PID_KI: i64 = 10;
```

---

**CONST: PID_KD** (line 22)

```rust
pub const PID_KD: i64 = 20;
```

---

**CONST: PID_INTEGRAL_MAX** (line 23)

```rust
pub const PID_INTEGRAL_MAX: i64 = 10000;
```

---

**CONST: PID_INTEGRAL_MIN** (line 24)

```rust
pub const PID_INTEGRAL_MIN: i64 = -10000;
```

---

**STRUCT: PidState** (line 28)

```rust
pub struct PidState {
```

---

**CONST: fn** (line 36)

```rust
pub const fn new(setpoint: i64) -> Self {
```

---

**FN: update** (line 45)

```rust
pub fn update(&mut self, measurement: i64) -> i64 {
```

---

**FN: reset** (line 54)

```rust
pub fn reset(&mut self) {
```

---

**STRUCT: ThermalZone** (line 62)

```rust
pub struct ThermalZone {
```

---

**CONST: fn** (line 75)

```rust
pub const fn empty() -> Self {
```

---

**FN: init** (line 89)

```rust
pub fn init(&mut self, pd_idx: u32) {
```

---

**FN: update_temp** (line 99)

```rust
pub fn update_temp(&self, temp_millideg: u32) {
```

---

**FN: get_temp** (line 103)

```rust
pub fn get_temp(&self) -> u32 {
```

---

**FN: get_level** (line 107)

```rust
pub fn get_level(&self) -> u32 {
```

---

**FN: evaluate_level** (line 111)

```rust
pub fn evaluate_level(&self) -> u32 {
```

---

**FN: compute_pid_output** (line 148)

```rust
pub fn compute_pid_output(&mut self) -> i64 {
```

---

**STRUCT: PowerScheduler** (line 155)

```rust
pub struct PowerScheduler {
```

---

**CONST: fn** (line 169)

```rust
pub const fn empty() -> Self {
```

---

**FN: init** (line 186)

```rust
pub fn init(&mut self) {
```

---

**FN: register_zone** (line 199)

```rust
pub fn register_zone(&mut self, pd_idx: u32) -> bool {
```

---

**FN: get_zone_for_pd** (line 209)

```rust
pub fn get_zone_for_pd(&self, pd_idx: u32) -> Option<&ThermalZone> {
```

---

**FN: get_zone_for_pd_mut** (line 218)

```rust
pub fn get_zone_for_pd_mut(&mut self, pd_idx: u32) -> Option<&mut ThermalZone> {
```

---

**FN: update_tick** (line 227)

```rust
pub fn update_tick(&mut self, current_time_ns: u64) {
```

---

**FN: apply_mitigation** (line 258)

```rust
pub fn apply_mitigation(&mut self) {
```

---

**FN: get_max_allowed_capacity** (line 308)

```rust
pub fn get_max_allowed_capacity(&self, cpu: u32) -> u32 {
```

---

**FN: is_eas_enabled** (line 315)

```rust
pub fn is_eas_enabled(&self) -> bool {
```

---

**FN: enable_eas** (line 319)

```rust
pub fn enable_eas(&self) {
```

---

**FN: disable_eas** (line 323)

```rust
pub fn disable_eas(&self) {
```

---

**FN: is_emergency_shutdown** (line 327)

```rust
pub fn is_emergency_shutdown(&self) -> bool {
```

---

**FN: dump_thermal_state** (line 331)

```rust
pub fn dump_thermal_state(&self) {
```

---

**STATIC: mut** (line 337)

```rust
pub static mut GLOBAL_POWER_SCHEDULER: PowerScheduler = PowerScheduler::empty();
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/power/placement.rs

**CONST: EAS_MARGIN_PERCENT** (line 6)

```rust
pub const EAS_MARGIN_PERCENT: u32 = 20;
```

---

**CONST: EAS_MAX_IDLE_CPU_PERCENT** (line 7)

```rust
pub const EAS_MAX_IDLE_CPU_PERCENT: u32 = 15;
```

---

**CONST: EAS_PACKING_THRESHOLD** (line 8)

```rust
pub const EAS_PACKING_THRESHOLD: u32 = 80;
```

---

**CONST: EAS_MIGRATION_COST_NS** (line 9)

```rust
pub const EAS_MIGRATION_COST_NS: u64 = 500_000;
```

---

**CONST: EAS_CACHE_AFFINITY_BONUS** (line 10)

```rust
pub const EAS_CACHE_AFFINITY_BONUS: u32 = 50;
```

---

**STRUCT: CpuSnapshot** (line 14)

```rust
pub struct CpuSnapshot {
```

---

**CONST: fn** (line 28)

```rust
pub const fn empty() -> Self {
```

---

**STRUCT: DomainSnapshot** (line 35)

```rust
pub struct DomainSnapshot {
```

---

**CONST: fn** (line 47)

```rust
pub const fn empty() -> Self {
```

---

**STRUCT: EnergyCtx** (line 54)

```rust
pub struct EnergyCtx {
```

---

**CONST: fn** (line 69)

```rust
pub const fn empty() -> Self {
```

---

**STRUCT: PlacementResult** (line 77)

```rust
pub struct PlacementResult {
```

---

**CONST: fn** (line 86)

```rust
pub const fn empty() -> Self {
```

---

**CONST: REASON_ENERGY** (line 91)

```rust
pub const REASON_ENERGY: u32 = 1;
```

---

**CONST: REASON_CAPACITY** (line 92)

```rust
pub const REASON_CAPACITY: u32 = 2;
```

---

**CONST: REASON_AFFINITY** (line 93)

```rust
pub const REASON_AFFINITY: u32 = 3;
```

---

**CONST: REASON_THERMAL** (line 94)

```rust
pub const REASON_THERMAL: u32 = 4;
```

---

**CONST: REASON_FALLBACK** (line 95)

```rust
pub const REASON_FALLBACK: u32 = 5;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/balancing/calculate.rs

**CONST: MIGRATION_COST_NS** (line 9)

```rust
pub const MIGRATION_COST_NS: u64 = 500_000;
```

---

**CONST: MIN_BALANCE_INTERVAL_NS** (line 10)

```rust
pub const MIN_BALANCE_INTERVAL_NS: u64 = 1_000_000;
```

---

**CONST: IMBALANCE_PCT** (line 11)

```rust
pub const IMBALANCE_PCT: u32 = 125;
```

---

**CONST: GROUP_OVERLOAD_THRESHOLD** (line 12)

```rust
pub const GROUP_OVERLOAD_THRESHOLD: u32 = 90;
```

---

**CONST: MISFIT_TASK_THRESHOLD** (line 13)

```rust
pub const MISFIT_TASK_THRESHOLD: u32 = 80;
```

---

**ENUM: GroupType** (line 17)

```rust
pub enum GroupType {
```

---

**STRUCT: LoadCalculation** (line 28)

```rust
pub struct LoadCalculation {
```

---

**CONST: fn** (line 41)

```rust
pub const fn empty() -> Self {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/balancing/env.rs

**CONST: LBF_ALL_PINNED** (line 5)

```rust
pub const LBF_ALL_PINNED: u32 = 0x01;
```

---

**CONST: LBF_NEED_BREAK** (line 6)

```rust
pub const LBF_NEED_BREAK: u32 = 0x02;
```

---

**CONST: LBF_SOME_PINNED** (line 7)

```rust
pub const LBF_SOME_PINNED: u32 = 0x04;
```

---

**CONST: LBF_ACTIVE_LB** (line 8)

```rust
pub const LBF_ACTIVE_LB: u32 = 0x08;
```

---

**CONST: DEFAULT_LOOP_MAX** (line 10)

```rust
pub const DEFAULT_LOOP_MAX: u32 = 32;
```

---

**STRUCT: LoadBalanceEnv** (line 18)

```rust
pub struct LoadBalanceEnv {
```

---

**CONST: fn** (line 29)

```rust
pub const fn new(sd: *mut SchedDomain, src_cpu: u32, dst_cpu: u32) -> Self {
```

---

**FN: set_flag** (line 41)

```rust
pub fn set_flag(&mut self, flag: u32) {
```

---

**FN: clear_flag** (line 45)

```rust
pub fn clear_flag(&mut self, flag: u32) {
```

---

**FN: has_flag** (line 49)

```rust
pub fn has_flag(&self, flag: u32) -> bool {
```

---

**FN: should_stop** (line 55)

```rust
pub fn should_stop(&self) -> bool {
```

Czy przebieg powinien się zatrzymać (osiągnięto limit prób
przeniesienia zadań w tej iteracji `load_balance`).

---

**FN: record_attempt** (line 59)

```rust
pub fn record_attempt(&mut self) {
```

---

**FN: record_failure** (line 63)

```rust
pub fn record_failure(&mut self) {
```

---

**FN: reset_for_next_iteration** (line 68)

```rust
pub fn reset_for_next_iteration(&mut self) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/ipi.rs

**ENUM: IpiType** (line 8)

```rust
pub enum IpiType {
```

---

**CONST: IPI_COUNTER_INIT** (line 16)

```rust
pub const IPI_COUNTER_INIT: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: IPI_RESCHED_COUNT** (line 17)

```rust
pub static IPI_RESCHED_COUNT: [AtomicU64; MAX_CPUS] = [IPI_COUNTER_INIT; MAX_CPUS];
```

---

**STATIC: IPI_CALL_COUNT** (line 18)

```rust
pub static IPI_CALL_COUNT: [AtomicU64; MAX_CPUS] = [IPI_COUNTER_INIT; MAX_CPUS];
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/migration/stopper.rs

**ENUM: StopperState** (line 10)

```rust
pub enum StopperState {
```

---

**STRUCT: StopperWork** (line 19)

```rust
pub struct StopperWork {
```

---

**CONST: fn** (line 30)

```rust
pub const fn empty() -> Self {
```

---

**FN: get_state** (line 41)

```rust
pub fn get_state(&self) -> StopperState {
```

---

**FN: set_state** (line 52)

```rust
pub fn set_state(&self, new_state: StopperState) {
```

---

**TYPE: StopperFn** (line 57)

```rust
pub type StopperFn = unsafe extern "C" fn(*mut core::ffi::c_void) -> u32;
```

---

**STRUCT: CpuStopper** (line 60)

```rust
pub struct CpuStopper {
```

---

**CONST: fn** (line 71)

```rust
pub const fn empty() -> Self {
```

---

**FN: init** (line 83)

```rust
pub fn init(&mut self, cpu: u32) {
```

---

**FN: queue_work** (line 88)

```rust
pub fn queue_work(&self, func: StopperFn, arg: *mut core::ffi::c_void) -> bool {
```

---

**FN: execute_work** (line 120)

```rust
pub fn execute_work(&self) {
```

---

**FN: wait_for_completion** (line 146)

```rust
pub fn wait_for_completion(&self) -> u32 {
```

---

**STRUCT: StopperRegistry** (line 166)

```rust
pub struct StopperRegistry {
```

---

**CONST: fn** (line 171)

```rust
pub const fn empty() -> Self {
```

---

**FN: init_cpu** (line 178)

```rust
pub fn init_cpu(&mut self, cpu: u32) {
```

---

**FN: get** (line 184)

```rust
pub fn get(&self, cpu: u32) -> Option<&CpuStopper> {
```

---

**STATIC: mut** (line 193)

```rust
pub static mut GLOBAL_STOPPERS: StopperRegistry = StopperRegistry::empty();
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/topology.rs

**CONST: SD_LOAD_BALANCE** (line 7)

```rust
pub const SD_LOAD_BALANCE: u32      = 0x0001;
```

---

**CONST: SD_BALANCE_NEWIDLE** (line 8)

```rust
pub const SD_BALANCE_NEWIDLE: u32   = 0x0002;
```

---

**CONST: SD_BALANCE_EXEC** (line 9)

```rust
pub const SD_BALANCE_EXEC: u32      = 0x0004;
```

---

**CONST: SD_BALANCE_FORK** (line 10)

```rust
pub const SD_BALANCE_FORK: u32      = 0x0008;
```

---

**CONST: SD_BALANCE_WAKE** (line 11)

```rust
pub const SD_BALANCE_WAKE: u32      = 0x0010;
```

---

**CONST: SD_WAKE_AFFINE** (line 12)

```rust
pub const SD_WAKE_AFFINE: u32       = 0x0020;
```

---

**CONST: SD_SHARE_CPUCAPACITY** (line 13)

```rust
pub const SD_SHARE_CPUCAPACITY: u32 = 0x0040; // SMT (Hyper-threading)
```

---

**CONST: SD_SHARE_PKG_RESOURCES** (line 14)

```rust
pub const SD_SHARE_PKG_RESOURCES: u32= 0x0080; // Shared L2/L3 cache
```

---

**CONST: SD_SERIALIZE** (line 15)

```rust
pub const SD_SERIALIZE: u32         = 0x0100;
```

---

**CONST: SD_ASYM_PACKING** (line 16)

```rust
pub const SD_ASYM_PACKING: u32      = 0x0200;
```

---

**CONST: SD_NUMA** (line 17)

```rust
pub const SD_NUMA: u32              = 0x0400;
```

---

**CONST: MAX_SCHED_DOMAIN_LEVELS** (line 19)

```rust
pub const MAX_SCHED_DOMAIN_LEVELS: usize = 8;
```

---

**CONST: MAX_NUMA_NODES** (line 20)

```rust
pub const MAX_NUMA_NODES: usize = 16;
```

---

**ENUM: CacheLevel** (line 24)

```rust
pub enum CacheLevel {
```

---

**STRUCT: CacheTopology** (line 33)

```rust
pub struct CacheTopology {
```

---

**STRUCT: NumaNode** (line 41)

```rust
pub struct NumaNode {
```

---

**CONST: fn** (line 50)

```rust
pub const fn empty() -> Self {
```

---

**FN: distance_to** (line 60)

```rust
pub fn distance_to(&self, other_node: u32) -> u8 {
```

---

**STRUCT: SchedGroup** (line 67)

```rust
pub struct SchedGroup {
```

---

**CONST: fn** (line 80)

```rust
pub const fn empty() -> Self {
```

---

**STRUCT: SchedDomain** (line 96)

```rust
pub struct SchedDomain {
```

---

**CONST: fn** (line 116)

```rust
pub const fn empty() -> Self {
```

---

**FN: has_flag** (line 137)

```rust
pub fn has_flag(&self, flag: u32) -> bool {
```

---

**FN: is_numa** (line 141)

```rust
pub fn is_numa(&self) -> bool {
```

---

**FN: shares_cache** (line 145)

```rust
pub fn shares_cache(&self) -> bool {
```

---

**FN: is_smt** (line 149)

```rust
pub fn is_smt(&self) -> bool {
```

---

**STRUCT: CpuTopology** (line 155)

```rust
pub struct CpuTopology {
```

---

**CONST: fn** (line 171)

```rust
pub const fn empty() -> Self {
```

---

**STRUCT: TopologyState** (line 192)

```rust
pub struct TopologyState {
```

---

**CONST: fn** (line 206)

```rust
pub const fn empty() -> Self {
```

---

**FN: alloc_domain** (line 226)

```rust
pub fn alloc_domain(&mut self) -> Option<&mut SchedDomain> {
```

---

**FN: alloc_group** (line 234)

```rust
pub fn alloc_group(&mut self) -> Option<&mut SchedGroup> {
```

---

**STATIC: mut** (line 243)

```rust
pub static mut GLOBAL_TOPOLOGY: TopologyState = TopologyState::empty();
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/time/plist.rs

**STRUCT: PList** (line 42)

```rust
pub struct PList {
```

---

**CONST: fn** (line 47)

```rust
pub const fn new() -> Self {
```

---

**FN: is_empty** (line 51)

```rust
pub fn is_empty(&self) -> bool {
```

---

**FN: first** (line 150)

```rust
pub fn first(&self) -> *mut TaskStruct {
```

Zadanie o najwyższym priorytecie (head listy poziomów),
pierwsze w kolejności FIFO na tym poziomie. O(1).

---

**FN: last** (line 165)

```rust
pub fn last(&self) -> *mut TaskStruct {
```

Zadanie o najniższym priorytecie (ostatni poziom), ostatnie
w kolejności FIFO na tym poziomie.


---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/time/rbtree.rs

**STRUCT: RbTree** (line 62)

```rust
pub struct RbTree {
```

---

**CONST: fn** (line 68)

```rust
pub const fn new() -> Self {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/time/rt_array.rs

**STRUCT: RtArray** (line 4)

```rust
pub struct RtArray {
```

---

**CONST: fn** (line 11)

```rust
pub const fn new() -> Self {
```

---

**FN: set_bit** (line 20)

```rust
pub fn set_bit(&mut self, prio: usize) {
```

---

**FN: clear_bit** (line 27)

```rust
pub fn clear_bit(&mut self, prio: usize) {
```

---

**FN: highest_prio** (line 41)

```rust
pub fn highest_prio(&self) -> Option<usize> {
```

---

**FN: active_levels** (line 149)

```rust
pub fn active_levels(&self) -> u32 {
```

Liczba unikalnych aktywnych poziomów priorytetu (nie mylić
z `nr_running`, które liczy wszystkie zakolejkowane zadania).


---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/time/task.rs

**CONST: MAX_RT_PRIO** (line 3)

```rust
pub const MAX_RT_PRIO: u32 = 100;
```

---

**STRUCT: ListHead** (line 7)

```rust
pub struct ListHead {
```

---

**CONST: fn** (line 13)

```rust
pub const fn new() -> Self {
```

---

**FN: is_empty** (line 18)

```rust
pub fn is_empty(&self) -> bool {
```

---

**STRUCT: RtFields** (line 52)

```rust
pub struct RtFields {
```

---

**STRUCT: FairFields** (line 58)

```rust
pub struct FairFields {
```

---

**STRUCT: TaskStruct** (line 63)

```rust
pub struct TaskStruct {
```

---

**CONST: RB_LEFT_OFFSET** (line 79)

```rust
pub const RB_LEFT_OFFSET: usize = core::mem::offset_of!(TaskStruct, rb_left);
```

---

**CONST: RB_RIGHT_OFFSET** (line 80)

```rust
pub const RB_RIGHT_OFFSET: usize = core::mem::offset_of!(TaskStruct, rb_right);
```

---

**CONST: RB_PARENT_COLOR_OFFSET** (line 81)

```rust
pub const RB_PARENT_COLOR_OFFSET: usize = core::mem::offset_of!(TaskStruct, rb_parent_color);
```

---

**CONST: PLIST_PRIO_OFFSET** (line 83)

```rust
pub const PLIST_PRIO_OFFSET: usize = core::mem::offset_of!(TaskStruct, plist_prio);
```

---

**CONST: PLIST_SAME_PRIO_OFFSET** (line 84)

```rust
pub const PLIST_SAME_PRIO_OFFSET: usize = core::mem::offset_of!(TaskStruct, plist_same_prio);
```

---

**CONST: PLIST_NODE_OFFSET** (line 85)

```rust
pub const PLIST_NODE_OFFSET: usize = core::mem::offset_of!(TaskStruct, plist_node);
```

---

**CONST: RT_OFFSET** (line 87)

```rust
pub const RT_OFFSET: usize = core::mem::offset_of!(TaskStruct, rt);
```

---

**CONST: RT_RUN_LIST_OFFSET** (line 88)

```rust
pub const RT_RUN_LIST_OFFSET: usize = Self::RT_OFFSET + core::mem::offset_of!(RtFields, run_list);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/smp.rs

**CONST: MAX_CPUS** (line 13)

```rust
pub const MAX_CPUS: usize = 32;
```

---

**FN: flush_serial_no_panic** (line 91)

```rust
pub fn flush_serial_no_panic() {
```

Flush the serial port's transmit FIFO (COM1) so all pending bytes actually
hit the log file before QEMU is killed/timeout-terminated.

---

**FN: init** (line 141)

```rust
pub fn init(boot_info: &'static bootloader::BootInfo) {
```

---

**FN: total_cpus** (line 248)

```rust
pub fn total_cpus() -> u32 {
```

---

**FN: poweroff** (line 252)

```rust
pub fn poweroff() -> bool {
```

---

**FN: reboot** (line 276)

```rust
pub fn reboot() -> ! {
```

---

**FN: self_test** (line 293)

```rust
pub fn self_test() -> TestResult {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/trampoline.rs

**CONST: TRAMPOLINE_BASE** (line 4)

```rust
pub const TRAMPOLINE_BASE: u64 = 0x8000;
```

---

**FN: install** (line 23)

```rust
pub fn install(cr3: u64, entry: u64) {
```

---

**FN: set_stack_and_arg** (line 48)

```rust
pub fn set_stack_and_arg(stack_top: u64, arg: u64) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/common.rs

**STRUCT: PackageId** (line 2)

```rust
pub struct PackageId {
```

---

**STRUCT: Version** (line 8)

```rust
pub struct Version {
```

---

**ENUM: ComponentKind** (line 15)

```rust
pub enum ComponentKind {
```

---

**ENUM: PackageStatus** (line 23)

```rust
pub enum PackageStatus {
```

---

**ENUM: CtrlInstallError** (line 31)

```rust
pub enum CtrlInstallError {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/init/bootstrap.rs

**STRUCT: BootstrapResult** (line 1)

```rust
pub struct BootstrapResult {
```

---

**FN: bootstrap** (line 6)

```rust
pub fn bootstrap() -> Result<BootstrapResult, CtrlInstallError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/init/state.rs

**STRUCT: SystemState** (line 1)

```rust
pub struct SystemState {
```

---

**STRUCT: InstalledPackage** (line 6)

```rust
pub struct InstalledPackage {
```

---

**FN: is_installed** (line 16)

```rust
pub fn is_installed(&self, name: &str) -> bool { todo!() }
```

---

**FN: get_package** (line 17)

```rust
pub fn get_package(&self, name: &str) -> Option<&InstalledPackage> { todo!() }
```

---

**FN: register** (line 18)

```rust
pub fn register(&mut self, pkg: InstalledPackage) { todo!() }
```

---

**FN: unregister** (line 19)

```rust
pub fn unregister(&mut self, name: &str) -> Result<(), CtrlInstallError> { todo!() }
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/install/executor.rs

**STRUCT: InstallExecutor** (line 1)

```rust
pub struct InstallExecutor<'a> {
```

---

**FN: execute** (line 6)

```rust
pub fn execute(&mut self, tx: &Transaction) -> Result<(), CtrlInstallError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/install/resolver.rs

**STRUCT: ResolvedPlan** (line 1)

```rust
pub struct ResolvedPlan {
```

---

**FN: resolve** (line 5)

```rust
pub fn resolve(
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/install/transaction.rs

**ENUM: TransactionStep** (line 1)

```rust
pub enum TransactionStep {
```

---

**STRUCT: Transaction** (line 8)

```rust
pub struct Transaction {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/mod.rs

**STRUCT: CtrlInstall** (line 9)

```rust
pub struct CtrlInstall {
```

---

**FN: new** (line 15)

```rust
pub fn new() -> Result<Self, CtrlInstallError> {
```

---

**FN: install** (line 21)

```rust
pub fn install(&mut self, name: &str) -> Result<(), CtrlInstallError> {
```

---

**FN: update** (line 33)

```rust
pub fn update(&mut self) -> Result<(), CtrlInstallError> {
```

---

**FN: list_installed** (line 39)

```rust
pub fn list_installed(&self) -> &[init::state::InstalledPackage] {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/repo/fetch.rs

**TRAIT: PackageFetcher** (line 1)

```rust
pub trait PackageFetcher {
```

---

**STRUCT: LocalFetcher** (line 5)

```rust
pub struct LocalFetcher {
```

---

**STRUCT: RemoteFetcher** (line 9)

```rust
pub struct RemoteFetcher {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/repo/index.rs

**STRUCT: RepositoryIndex** (line 1)

```rust
pub struct RepositoryIndex {
```

---

**FN: find** (line 6)

```rust
pub fn find(&self, name: &str) -> Option<&PackageManifest> { todo!() }
```

---

**FN: find_by_kind** (line 7)

```rust
pub fn find_by_kind(&self, kind: ComponentKind) -> alloc::vec::Vec<&PackageManifest> { todo!() }
```

---

**FN: search** (line 8)

```rust
pub fn search(&self, query: &str) -> alloc::vec::Vec<&PackageManifest> { todo!() }
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/repo/manifest.rs

**STRUCT: PackageManifest** (line 1)

```rust
pub struct PackageManifest {
```

---

**STRUCT: Dependency** (line 10)

```rust
pub struct Dependency {
```

---

**STRUCT: FileEntry** (line 15)

```rust
pub struct FileEntry {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/update/diff.rs

**ENUM: UpdateAction** (line 1)

```rust
pub enum UpdateAction {
```

---

**FN: diff** (line 7)

```rust
pub fn diff(
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/update/upgrade.rs

**STRUCT: UpgradePlan** (line 1)

```rust
pub struct UpgradePlan {
```

---

**FN: plan_upgrade** (line 6)

```rust
pub fn plan_upgrade(
```

---

**FN: execute_upgrade** (line 13)

```rust
pub fn execute_upgrade(plan: &UpgradePlan, executor: &mut InstallExecutor) -> Result<(), CtrlInstallError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/aut.rs

**CONST: DP_STATUS** (line 1)

```rust
pub const DP_STATUS: u32 = 1;
```

---

**CONST: DP_LINK** (line 2)

```rust
pub const DP_LINK: u32 = 2;
```

---

**CONST: DP_MODES** (line 3)

```rust
pub const DP_MODES: u32 = 3;
```

---

**CONST: DP_MODE_SET** (line 4)

```rust
pub const DP_MODE_SET: u32 = 4;
```

---

**CONST: DP_FILL** (line 5)

```rust
pub const DP_FILL: u32 = 5;
```

---

**FN: authorize** (line 7)

```rust
pub fn authorize(ring: u8, op: u8) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/aux.c

**FUNCTION: dp_aux_sink_init** (line 35)

```c
void dp_aux_sink_init(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/aux.h

**FUNCTION: dp_aux_sink_init** (line 17)

```c
void dp_aux_sink_init(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/bridge.rs

**FN: dp_call** (line 13)

```rust
pub fn dp_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/dp.h

**FUNCTION: dp_link_info** (line 13)

```c
void dp_link_info(uint32_t *rate_mbps, uint32_t *lanes);
```

---

**FUNCTION: dp_caps** (line 20)

```c
void dp_caps(uint64_t *fb_phys, uint32_t *w, uint32_t *h, uint32_t *stride);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/init.c

**FUNCTION: dp_caps** (line 75)

```c
void dp_caps(uint64_t *fb_phys, uint32_t *w, uint32_t *h, uint32_t *stride)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/init.rs

**FN: init** (line 6)

```rust
pub fn init() -> bool {
```

---

**FN: ready** (line 10)

```rust
pub fn ready() -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/link.c

**FUNCTION: dp_link_info** (line 70)

```c
void dp_link_info(uint32_t *rate_mbps, uint32_t *lanes)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/link.h

**FUNCTION: dp_link_info** (line 8)

```c
void dp_link_info(uint32_t *rate_mbps, uint32_t *lanes);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/operation.c

**FUNCTION: dp_op_set_fb** (line 8)

```c
void dp_op_set_fb(uint64_t phys, uint32_t ww, uint32_t hh, uint32_t s)
```

---

**FUNCTION: dp_op_state** (line 16)

```c
void dp_op_state(uint64_t *phys, uint32_t *ww, uint32_t *hh, uint32_t *s)
```

---

**FUNCTION: dp_op_fill** (line 24)

```c
void dp_op_fill(uint32_t color, uint32_t x, uint32_t y,
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/operation.h

**FUNCTION: dp_op_set_fb** (line 6)

```c
void dp_op_set_fb(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride);
```

---

**FUNCTION: dp_op_state** (line 7)

```c
void dp_op_state(uint64_t *phys, uint32_t *w, uint32_t *h, uint32_t *stride);
```

---

**FUNCTION: dp_op_fill** (line 8)

```c
void dp_op_fill(uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/pci/mod.rs

**STRUCT: PciDev** (line 2)

```rust
pub struct PciDev {
```

---

**FN: read32** (line 22)

```rust
pub fn read32(d: PciDev, off: u32) -> u32 {
```

---

**FN: write32** (line 29)

```rust
pub fn write32(d: PciDev, off: u32, v: u32) {
```

---

**FN: port_out** (line 36)

```rust
pub fn port_out(port: u16, val: u32, size: u32) {
```

---

**FN: port_in** (line 46)

```rust
pub fn port_in(port: u16, size: u32) -> u32 {
```

---

**FN: read16** (line 68)

```rust
pub fn read16(d: PciDev, off: u32) -> u16 {
```

---

**FN: read8** (line 72)

```rust
pub fn read8(d: PciDev, off: u32) -> u8 {
```

---

**FN: vendor** (line 77)

```rust
pub fn vendor(self) -> u16 {
```

---

**FN: device_id** (line 81)

```rust
pub fn device_id(self) -> u16 {
```

---

**FN: class** (line 85)

```rust
pub fn class(self) -> u8 {
```

---

**FN: subclass** (line 89)

```rust
pub fn subclass(self) -> u8 {
```

---

**FN: prog_if** (line 93)

```rust
pub fn prog_if(self) -> u8 {
```

---

**FN: bar** (line 97)

```rust
pub fn bar(self, idx: u32) -> u64 {
```

---

**FN: enable_mmio** (line 108)

```rust
pub fn enable_mmio(self) {
```

---

**FN: find_class** (line 114)

```rust
pub fn find_class(class: u8, subclass: u8, prog_if: u8) -> Option<PciDev> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/hid/keyboard.rs

**FN: key_to_ascii** (line 18)

```rust
pub fn key_to_ascii(key: u8, shift: bool) -> Option<u8> {
```

---

**FN: push_char** (line 44)

```rust
pub fn push_char(c: u8) {
```

---

**FN: take_char** (line 55)

```rust
pub fn take_char() -> Option<u8> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/hid/mod.rs

**STRUCT: HidKeyboard** (line 16)

```rust
pub struct HidKeyboard {
```

---

**FN: attach** (line 31)

```rust
pub fn attach(x: &mut Xhci, dev: &mut UsbDevice) -> Result<bool, UsbError> {
```

---

**FN: poll** (line 105)

```rust
pub fn poll(x: &mut Xhci) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/hid/report.rs

**STRUCT: BootReport** (line 1)

```rust
pub struct BootReport {
```

---

**FN: parse_boot_keyboard** (line 6)

```rust
pub fn parse_boot_keyboard(report: &[u8]) -> Option<BootReport> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/mass/mod.rs

**CONST: CBW_SIG** (line 12)

```rust
pub const CBW_SIG: u32 = 0x4342_5355;
```

---

**CONST: CSW_SIG** (line 13)

```rust
pub const CSW_SIG: u32 = 0x5342_5355;
```

---

**STRUCT: UsbMass** (line 17)

```rust
pub struct UsbMass {
```

---

**FN: bulk_out** (line 31)

```rust
pub fn bulk_out(&mut self, x: &mut Xhci, phys: u64, len: u32) -> Result<(), UsbError> {
```

---

**FN: bulk_in** (line 44)

```rust
pub fn bulk_in(&mut self, x: &mut Xhci, phys: u64, len: u32) -> Result<(), UsbError> {
```

---

**FN: scsi_cmd** (line 57)

```rust
pub fn scsi_cmd(&mut self, x: &mut Xhci, cdb: &[u8],
```

---

**FN: attach** (line 101)

```rust
pub fn attach(x: &mut Xhci, dev: &mut UsbDevice) -> Result<bool, UsbError> {
```

---

**FN: with_controller** (line 186)

```rust
pub fn with_controller<F: FnOnce(&mut Xhci) -> R, R>(f: F) -> R {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/mass/scsi.rs

**FN: read_capacity** (line 5)

```rust
pub fn read_capacity(x: &mut Xhci, m: &mut UsbMass) -> Result<(u32, u32), UsbError> {
```

---

**FN: read10** (line 20)

```rust
pub fn read10(x: &mut Xhci, m: &mut UsbMass,
```

---

**FN: write10** (line 33)

```rust
pub fn write10(x: &mut Xhci, m: &mut UsbMass,
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/mod.rs

**TRAIT: ClassDriver** (line 5)

```rust
pub trait ClassDriver {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/core/descriptor.rs

**STRUCT: DeviceDesc** (line 2)

```rust
pub struct DeviceDesc {
```

---

**STRUCT: InterfaceDesc** (line 13)

```rust
pub struct InterfaceDesc {
```

---

**STRUCT: EndpointDesc** (line 22)

```rust
pub struct EndpointDesc {
```

---

**FN: parse_device** (line 29)

```rust
pub fn parse_device(buf: &[u8]) -> Option<DeviceDesc> {
```

---

**FN: parse_config** (line 45)

```rust
pub fn parse_config(buf: &[u8],
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/core/device.rs

**STRUCT: UsbDevice** (line 7)

```rust
pub struct UsbDevice {
```

---

**FN: new** (line 24)

```rust
pub fn new(slot: u8, speed: u32, ctx_size: usize) -> Result<Self, UsbError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/core/enumerate.rs

**FN: enumerate** (line 14)

```rust
pub fn enumerate(x: &mut Xhci, port: u32) -> Result<UsbDevice, UsbError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/core/request.rs

**CONST: GET_DESCRIPTOR** (line 1)

```rust
pub const GET_DESCRIPTOR: u8 = 6;
```

---

**CONST: SET_CONFIGURATION** (line 2)

```rust
pub const SET_CONFIGURATION: u8 = 9;
```

---

**CONST: DESC_DEVICE** (line 4)

```rust
pub const DESC_DEVICE: u8 = 1;
```

---

**CONST: DESC_CONFIG** (line 5)

```rust
pub const DESC_CONFIG: u8 = 2;
```

---

**CONST: DIR_IN** (line 7)

```rust
pub const DIR_IN: u8 = 0x80;
```

---

**CONST: DIR_OUT** (line 8)

```rust
pub const DIR_OUT: u8 = 0x00;
```

---

**CONST: TYPE_STANDARD** (line 9)

```rust
pub const TYPE_STANDARD: u8 = 0x00;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/core/speed.rs

**CONST: SPEED_FULL** (line 1)

```rust
pub const SPEED_FULL: u32 = 1;
```

---

**CONST: SPEED_LOW** (line 2)

```rust
pub const SPEED_LOW: u32 = 2;
```

---

**CONST: SPEED_HIGH** (line 3)

```rust
pub const SPEED_HIGH: u32 = 3;
```

---

**CONST: SPEED_SUPER** (line 4)

```rust
pub const SPEED_SUPER: u32 = 4;
```

---

**FN: default_ep0_mps** (line 6)

```rust
pub fn default_ep0_mps(speed: u32) -> u16 {
```

---

**CONST: EP_CONTROL** (line 14)

```rust
pub const EP_CONTROL: u8 = 0;
```

---

**CONST: EP_ISO** (line 15)

```rust
pub const EP_ISO: u8 = 1;
```

---

**CONST: EP_BULK** (line 16)

```rust
pub const EP_BULK: u8 = 2;
```

---

**CONST: EP_INTERRUPT** (line 17)

```rust
pub const EP_INTERRUPT: u8 = 3;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/dma.rs

**STRUCT: DmaBuf** (line 5)

```rust
pub struct DmaBuf {
```

---

**FN: new** (line 12)

```rust
pub fn new(len: usize) -> Result<Self, UsbError> {
```

---

**FN: zero** (line 31)

```rust
pub fn zero(&mut self) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/context.rs

**STRUCT: Contexts** (line 4)

```rust
pub struct Contexts {
```

---

**FN: setup_configure_ep** (line 11)

```rust
pub fn setup_configure_ep(&mut self, speed: u32, port: u32,
```

---

**FN: setup_configure_bulk_pair** (line 37)

```rust
pub fn setup_configure_bulk_pair(&mut self, speed: u32, port: u32,
```

---

**FN: new** (line 71)

```rust
pub fn new(ctx_size: usize) -> Result<Self, UsbError> {
```

---

**FN: setup_address_device** (line 97)

```rust
pub fn setup_address_device(&mut self, speed: u32, port: u32,
```

---

**FN: setup_evaluate_mps** (line 120)

```rust
pub fn setup_evaluate_mps(&mut self, mps: u16) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/control.rs

**FN: wait_transfer** (line 6)

```rust
pub fn wait_transfer(x: &mut Xhci, slot: u8) -> Result<u8, UsbError> {
```

---

**FN: wait_transfer_ep** (line 26)

```rust
pub fn wait_transfer_ep(x: &mut Xhci, slot: u8, ep: u8) -> Result<u8, UsbError> {
```

---

**FN: control** (line 46)

```rust
pub fn control(x: &mut Xhci,
```

---

**FN: control_in** (line 94)

```rust
pub fn control_in(x: &mut Xhci, dev: &mut UsbDevice,
```

---

**FN: control_out** (line 102)

```rust
pub fn control_out(x: &mut Xhci, dev: &mut UsbDevice,
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/event.rs

**FN: drain_events** (line 9)

```rust
pub fn drain_events(&mut self) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/init.rs

**CONST: RT_IMAN** (line 11)

```rust
pub const RT_IMAN: usize = 0x00;
```

---

**CONST: RT_IMOD** (line 12)

```rust
pub const RT_IMOD: usize = 0x04;
```

---

**CONST: RT_ERSTSZ** (line 13)

```rust
pub const RT_ERSTSZ: usize = 0x08;
```

---

**CONST: RT_ERSTBA** (line 14)

```rust
pub const RT_ERSTBA: usize = 0x10;
```

---

**CONST: RT_ERDP** (line 15)

```rust
pub const RT_ERDP: usize = 0x18;
```

---

**STRUCT: Xhci** (line 17)

```rust
pub struct Xhci {
```

---

**FN: init** (line 49)

```rust
pub fn init(regs: XhciRegs) -> Result<Xhci, UsbError> {
```

---

**FN: command** (line 103)

```rust
pub fn command(&mut self, trb: Trb) -> Result<Trb, UsbError> {
```

---

**FN: scan_ports** (line 130)

```rust
pub fn scan_ports(&mut self) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/regs.rs

**CONST: OP_USBCMD** (line 4)

```rust
pub const OP_USBCMD: usize = 0x00;
```

---

**CONST: OP_USBSTS** (line 5)

```rust
pub const OP_USBSTS: usize = 0x04;
```

---

**CONST: OP_PAGESIZE** (line 6)

```rust
pub const OP_PAGESIZE: usize = 0x08;
```

---

**CONST: OP_CRCR** (line 7)

```rust
pub const OP_CRCR: usize = 0x10;
```

---

**CONST: OP_DCBAAP** (line 8)

```rust
pub const OP_DCBAAP: usize = 0x30;
```

---

**CONST: OP_CONFIG** (line 9)

```rust
pub const OP_CONFIG: usize = 0x38;
```

---

**CONST: OP_PORTSC** (line 10)

```rust
pub const OP_PORTSC: usize = 0x400;
```

---

**CONST: CMD_RS** (line 12)

```rust
pub const CMD_RS: u32 = 1 << 0;
```

---

**CONST: CMD_HCRST** (line 13)

```rust
pub const CMD_HCRST: u32 = 1 << 1;
```

---

**CONST: CMD_INTE** (line 14)

```rust
pub const CMD_INTE: u32 = 1 << 2;
```

---

**CONST: STS_HCH** (line 16)

```rust
pub const STS_HCH: u32 = 1 << 0;
```

---

**CONST: STS_CNR** (line 17)

```rust
pub const STS_CNR: u32 = 1 << 11;
```

---

**CONST: PORTSC_CCS** (line 19)

```rust
pub const PORTSC_CCS: u32 = 1 << 0;
```

---

**CONST: PORTSC_PED** (line 20)

```rust
pub const PORTSC_PED: u32 = 1 << 1;
```

---

**CONST: PORTSC_PR** (line 21)

```rust
pub const PORTSC_PR: u32 = 1 << 4;
```

---

**CONST: PORTSC_PRC** (line 22)

```rust
pub const PORTSC_PRC: u32 = 1 << 21;
```

---

**CONST: PORTSC_CSC** (line 23)

```rust
pub const PORTSC_CSC: u32 = 1 << 17;
```

---

**CONST: PORTSC_SPEED** (line 24)

```rust
pub const PORTSC_SPEED: u32 = 0xF << 10;
```

---

**STRUCT: XhciRegs** (line 36)

```rust
pub struct XhciRegs {
```

---

**FN: new** (line 49)

```rust
pub fn new(phys: u64) -> Result<Self, UsbError> {
```

---

**FN: op_read** (line 97)

```rust
pub fn op_read(&self, off: usize) -> u32 {
```

---

**FN: op_write** (line 101)

```rust
pub fn op_write(&self, off: usize, v: u32) {
```

---

**FN: rt_read** (line 105)

```rust
pub fn rt_read(&self, off: usize) -> u32 {
```

---

**FN: rt_write** (line 109)

```rust
pub fn rt_write(&self, off: usize, v: u32) {
```

---

**FN: doorbell** (line 113)

```rust
pub fn doorbell(&self, slot: u32, target: u32, task: u32) {
```

---

**FN: port_sc** (line 120)

```rust
pub fn port_sc(&self, port: u32) -> u32 {
```

---

**FN: port_sc_write** (line 124)

```rust
pub fn port_sc_write(&self, port: u32, v: u32) {
```

---

**FN: port_speed** (line 128)

```rust
pub fn port_speed(&self, port: u32) -> u32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/ring.rs

**STRUCT: CmdRing** (line 5)

```rust
pub struct CmdRing {
```

---

**FN: new** (line 13)

```rust
pub fn new(count: usize) -> Result<Self, UsbError> {
```

---

**FN: phys** (line 24)

```rust
pub fn phys(&self) -> u64 {
```

---

**FN: enqueue** (line 28)

```rust
pub fn enqueue(&mut self, mut trb: Trb) {
```

---

**STRUCT: EventRing** (line 50)

```rust
pub struct EventRing {
```

---

**STRUCT: TransferRing** (line 58)

```rust
pub struct TransferRing {
```

---

**FN: new** (line 66)

```rust
pub fn new(count: usize) -> Result<Self, UsbError> {
```

---

**FN: phys** (line 77)

```rust
pub fn phys(&self) -> u64 {
```

---

**FN: enqueue** (line 81)

```rust
pub fn enqueue(&mut self, mut trb: Trb) {
```

---

**FN: new** (line 102)

```rust
pub fn new(count: usize) -> Result<Self, UsbError> {
```

---

**FN: erst_phys** (line 115)

```rust
pub fn erst_phys(&self) -> u64 {
```

---

**FN: pending** (line 119)

```rust
pub fn pending(&self) -> Option<Trb> {
```

---

**FN: pop** (line 131)

```rust
pub fn pop(&mut self) {
```

---

**FN: erdp** (line 140)

```rust
pub fn erdp(&self) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/trb.rs

**CONST: TRB_NORMAL** (line 1)

```rust
pub const TRB_NORMAL: u32 = 1;
```

---

**CONST: TRB_SETUP** (line 2)

```rust
pub const TRB_SETUP: u32 = 2;
```

---

**CONST: TRB_DATA** (line 3)

```rust
pub const TRB_DATA: u32 = 3;
```

---

**CONST: TRB_STATUS** (line 4)

```rust
pub const TRB_STATUS: u32 = 4;
```

---

**CONST: TRB_LINK** (line 5)

```rust
pub const TRB_LINK: u32 = 6;
```

---

**CONST: TRB_EVENT_DATA** (line 6)

```rust
pub const TRB_EVENT_DATA: u32 = 7;
```

---

**CONST: TRB_NOOP_TR** (line 7)

```rust
pub const TRB_NOOP_TR: u32 = 8;
```

---

**CONST: TRB_ENABLE_SLOT** (line 8)

```rust
pub const TRB_ENABLE_SLOT: u32 = 9;
```

---

**CONST: TRB_DISABLE_SLOT** (line 9)

```rust
pub const TRB_DISABLE_SLOT: u32 = 10;
```

---

**CONST: TRB_ADDRESS_DEVICE** (line 10)

```rust
pub const TRB_ADDRESS_DEVICE: u32 = 11;
```

---

**CONST: TRB_CONFIGURE_EP** (line 11)

```rust
pub const TRB_CONFIGURE_EP: u32 = 12;
```

---

**CONST: TRB_EVALUATE_CTX** (line 12)

```rust
pub const TRB_EVALUATE_CTX: u32 = 13;
```

---

**CONST: TRB_RESET_EP** (line 13)

```rust
pub const TRB_RESET_EP: u32 = 14;
```

---

**CONST: TRB_STOP_EP** (line 14)

```rust
pub const TRB_STOP_EP: u32 = 15;
```

---

**CONST: TRB_SET_DEQUEUE** (line 15)

```rust
pub const TRB_SET_DEQUEUE: u32 = 16;
```

---

**CONST: TRB_RESET_DEV** (line 16)

```rust
pub const TRB_RESET_DEV: u32 = 17;
```

---

**CONST: TRB_NOOP_CMD** (line 17)

```rust
pub const TRB_NOOP_CMD: u32 = 23;
```

---

**CONST: TRB_TRANSFER_EVENT** (line 18)

```rust
pub const TRB_TRANSFER_EVENT: u32 = 32;
```

---

**CONST: TRB_CMD_COMPLETION** (line 19)

```rust
pub const TRB_CMD_COMPLETION: u32 = 33;
```

---

**CONST: TRB_PORT_STATUS** (line 20)

```rust
pub const TRB_PORT_STATUS: u32 = 34;
```

---

**CONST: TRB_HC_EVENT** (line 21)

```rust
pub const TRB_HC_EVENT: u32 = 35;
```

---

**CONST: CC_SUCCESS** (line 23)

```rust
pub const CC_SUCCESS: u8 = 1;
```

---

**STRUCT: Trb** (line 29)

```rust
pub struct Trb {
```

---

**FN: typ** (line 36)

```rust
pub fn typ(&self) -> u32 {
```

---

**FN: cycle** (line 40)

```rust
pub fn cycle(&self) -> bool {
```

---

**FN: completion_code** (line 44)

```rust
pub fn completion_code(&self) -> u8 {
```

---

**FN: slot_id** (line 48)

```rust
pub fn slot_id(&self) -> u8 {
```

---

**FN: ep_id** (line 52)

```rust
pub fn ep_id(&self) -> u8 {
```

---

**FN: transfer_len** (line 56)

```rust
pub fn transfer_len(&self) -> u32 {
```

---

**FN: link** (line 60)

```rust
pub fn link(addr: u64) -> Self {
```

---

**FN: enable_slot** (line 68)

```rust
pub fn enable_slot() -> Self {
```

---

**FN: address_device** (line 72)

```rust
pub fn address_device(slot: u8, ctx_phys: u64) -> Self {
```

---

**FN: configure_ep** (line 80)

```rust
pub fn configure_ep(slot: u8, ctx_phys: u64) -> Self {
```

---

**FN: noop_cmd** (line 88)

```rust
pub fn noop_cmd() -> Self {
```

---

**FN: setup_stage** (line 92)

```rust
pub fn setup_stage(raw_setup: u64, trt: u32) -> Self {
```

---

**FN: data_stage** (line 100)

```rust
pub fn data_stage(addr: u64, len: u32, dir_in: bool) -> Self {
```

---

**FN: status_stage** (line 108)

```rust
pub fn status_stage(dir_in: bool) -> Self {
```

---

**FN: normal** (line 116)

```rust
pub fn normal(addr: u64, len: u32) -> Self {
```

---

**FN: evaluate_ctx** (line 124)

```rust
pub fn evaluate_ctx(slot: u8, ctx_phys: u64) -> Self {
```

---

**FN: pack_setup** (line 133)

```rust
pub fn pack_setup(bm_request: u8, b_request: u8, value: u16,
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/mod.rs

**ENUM: UsbError** (line 10)

```rust
pub enum UsbError {
```

---

**FN: init** (line 24)

```rust
pub fn init() -> Result<(), UsbError> {
```

---

**FN: poll** (line 61)

```rust
pub fn poll() {
```

---

**FN: self_test** (line 67)

```rust
pub fn self_test() -> TestResult {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/pci_glue.rs

**CONST: XHCI_CLASS** (line 4)

```rust
pub const XHCI_CLASS: u8 = 0x0C;
```

---

**CONST: XHCI_SUBCLASS** (line 5)

```rust
pub const XHCI_SUBCLASS: u8 = 0x03;
```

---

**CONST: XHCI_PROGIF** (line 6)

```rust
pub const XHCI_PROGIF: u8 = 0x30;
```

---

**STRUCT: XhciPci** (line 8)

```rust
pub struct XhciPci {
```

---

**FN: find_xhci** (line 13)

```rust
pub fn find_xhci() -> Result<XhciPci, UsbError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/driverspaceinit/abi/abi.rs

**CONST: DS_MAGIC** (line 1)

```rust
pub const DS_MAGIC: u64 = 0x4452_5653_5041_4345;
```

---

**CONST: DS_VERSION** (line 2)

```rust
pub const DS_VERSION: u32 = 1;
```

---

**CONST: DS_RING_CAP** (line 3)

```rust
pub const DS_RING_CAP: u64 = 16;
```

---

**CONST: DS_FLAG_RESPONSE** (line 5)

```rust
pub const DS_FLAG_RESPONSE: u32 = 1 << 0;
```

---

**ENUM: DsCmd** (line 9)

```rust
pub enum DsCmd {
```

---

**STRUCT: DsMsg** (line 38)

```rust
pub struct DsMsg {
```

---

**CONST: DS_MSG_SIZE** (line 49)

```rust
pub const DS_MSG_SIZE: usize = core::mem::size_of::<DsMsg>();
```

---

**STRUCT: DsRing** (line 53)

```rust
pub struct DsRing {
```

---

**CONST: DS_RING_HDR_SIZE** (line 59)

```rust
pub const DS_RING_HDR_SIZE: usize = core::mem::size_of::<DsRing>();
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/driverspaceinit/abi/src.rs

**FN: ring_bytes** (line 3)

```rust
pub fn ring_bytes(cap: u64) -> usize {
```

---

**STRUCT: RingView** (line 7)

```rust
pub struct RingView {
```

---

**FN: init** (line 20)

```rust
pub fn init(&self, cap: u64) {
```

---

**FN: push** (line 28)

```rust
pub fn push(&self, msg: &DsMsg) -> bool {
```

---

**FN: pop** (line 49)

```rust
pub fn pop(&self) -> Option<DsMsg> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/driverspaceinit/init/enter.rs

**FN: enter** (line 61)

```rust
pub fn enter() -> Result<(), DsError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/driverspaceinit/init/init.rs

**STRUCT: Driverspace** (line 9)

```rust
pub struct Driverspace {
```

---

**FN: prepare** (line 24)

```rust
pub fn prepare() -> Result<(), DsError> {
```

---

**FN: self_test** (line 91)

```rust
pub fn self_test() -> Result<(), DsError> {
```

---

**FN: ready** (line 134)

```rust
pub fn ready() -> bool {
```

---

**FN: k2d_view** (line 138)

```rust
pub fn k2d_view() -> Option<RingView> {
```

---

**FN: d2k_view** (line 142)

```rust
pub fn d2k_view() -> Option<RingView> {
```

---

**FN: scratch_view** (line 146)

```rust
pub fn scratch_view() -> Option<*mut u8> {
```

---

**FN: map_into_ds** (line 150)

```rust
pub fn map_into_ds(va: u64, phys: u64, len: usize, prot: space::ProtFlags) -> bool {
```

---

**FN: unmap_from_ds** (line 159)

```rust
pub fn unmap_from_ds(va: u64, len: usize) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/driverspaceinit/init/initabi.rs

**CONST: DS_SWITCH_VA** (line 1)

```rust
pub const DS_SWITCH_VA: u64 = 0x4000_4000;
```

---

**CONST: DS_STACK_VA** (line 2)

```rust
pub const DS_STACK_VA: u64 = 0x4FFF_0000;
```

---

**CONST: DS_STACK_SIZE** (line 3)

```rust
pub const DS_STACK_SIZE: u64 = 16 * 1024;
```

---

**STRUCT: DsSwitch** (line 7)

```rust
pub struct DsSwitch {
```

---

**CONST: SVC_SYS** (line 14)

```rust
pub const SVC_SYS: u32 = 0;
```

---

**CONST: SVC_VIDEO** (line 15)

```rust
pub const SVC_VIDEO: u32 = 1;
```

---

**CONST: SVC_AUDIO** (line 16)

```rust
pub const SVC_AUDIO: u32 = 2;
```

---

**CONST: SVC_INPUT** (line 17)

```rust
pub const SVC_INPUT: u32 = 3;
```

---

**CONST: SVC_BLOCK** (line 18)

```rust
pub const SVC_BLOCK: u32 = 4;
```

---

**CONST: SVC_NET** (line 19)

```rust
pub const SVC_NET: u32 = 5;
```

---

**CONST: SVC_BT** (line 20)

```rust
pub const SVC_BT: u32 = 6;
```

---

**CONST: VID_FB_INFO** (line 22)

```rust
pub const VID_FB_INFO: u32 = 1;
```

---

**CONST: VID_FB_TAKEOVER** (line 23)

```rust
pub const VID_FB_TAKEOVER: u32 = 2;
```

---

**CONST: VID_FB_RELEASE** (line 24)

```rust
pub const VID_FB_RELEASE: u32 = 3;
```

---

**CONST: VID_HDMI_FILL** (line 25)

```rust
pub const VID_HDMI_FILL: u32 = 4;
```

---

**CONST: VID_HDMI_CAPS** (line 26)

```rust
pub const VID_HDMI_CAPS: u32 = 6;
```

---

**CONST: IN_KEY_POLL** (line 28)

```rust
pub const IN_KEY_POLL: u32 = 1;
```

---

**CONST: AUD_PLAY** (line 30)

```rust
pub const AUD_PLAY: u32 = 1;
```

---

**CONST: AUD_STOP** (line 31)

```rust
pub const AUD_STOP: u32 = 2;
```

---

**CONST: AUD_JACK** (line 32)

```rust
pub const AUD_JACK: u32 = 3;
```

---

**CONST: AUD_AMP** (line 33)

```rust
pub const AUD_AMP: u32 = 4;
```

---

**CONST: BLK_COUNT** (line 35)

```rust
pub const BLK_COUNT: u32 = 1;
```

---

**CONST: BLK_READ** (line 36)

```rust
pub const BLK_READ: u32 = 2;
```

---

**CONST: BLK_WRITE** (line 37)

```rust
pub const BLK_WRITE: u32 = 3;
```

---

**CONST: DS_INIT_PARAMS_VA** (line 39)

```rust
pub const DS_INIT_PARAMS_VA: u64 = 0x4000_0000;
```

---

**CONST: DS_K2D_VA** (line 40)

```rust
pub const DS_K2D_VA: u64 = 0x4000_1000;
```

---

**CONST: DS_D2K_VA** (line 41)

```rust
pub const DS_D2K_VA: u64 = 0x4000_2000;
```

---

**CONST: DS_SCRATCH_VA** (line 42)

```rust
pub const DS_SCRATCH_VA: u64 = 0x4000_3000;
```

---

**CONST: DS_SCRATCH_SIZE** (line 43)

```rust
pub const DS_SCRATCH_SIZE: usize = 4096;
```

---

**STRUCT: DsInitParams** (line 47)

```rust
pub struct DsInitParams {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/driverspaceinit/init/initcommand.rs

**ENUM: DsError** (line 5)

```rust
pub enum DsError {
```

---

**STRUCT: InitHandshake** (line 14)

```rust
pub struct InitHandshake {
```

---

**FN: send** (line 21)

```rust
pub fn send(&mut self, cmd: DsCmd,
```

---

**FN: run** (line 60)

```rust
pub fn run(&mut self, params_va: u64) -> Result<(), DsError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/driverspaceinit/init/service.rs

**FN: vgpu_call** (line 86)

```rust
pub fn vgpu_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring_lvl: u8) -> i32 {
```

---

**FN: pci_call** (line 155)

```rust
pub fn pci_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
```

---

**FN: video_call** (line 247)

```rust
pub fn video_call(op: u32, m: &DsMsg, r: &mut DsMsg) -> i32 {
```

---

**FN: poll** (line 430)

```rust
pub fn poll() {
```

---

**FN: post_event** (line 459)

```rust
pub fn post_event(cmd: DsCmd, a0: u64, a1: u64) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ds_ipc/buffer.rs

**CONST: IPC_BUFFER_MAX_SIZE** (line 17)

```rust
pub const IPC_BUFFER_MAX_SIZE: usize = 0x100000;
```

---

**CONST: IPC_BUFFER_ALIGNMENT** (line 18)

```rust
pub const IPC_BUFFER_ALIGNMENT: usize = 0x1000;
```

---

**CONST: MAX_MR_BYTES** (line 19)

```rust
pub const MAX_MR_BYTES: usize = size_of::<MessageRegisters>();
```

---

**CONST: IPC_BUFFER_SLOT_INVALID** (line 20)

```rust
pub const IPC_BUFFER_SLOT_INVALID: u64 = 0;
```

---

**ENUM: BufferTransferMode** (line 24)

```rust
pub enum BufferTransferMode {
```

---

**ENUM: BufferError** (line 32)

```rust
pub enum BufferError {
```

---

**STRUCT: BufferFlags** (line 56)

```rust
pub struct BufferFlags: u32 {
```

---

**STRUCT: IpcBufferDescriptor** (line 66)

```rust
pub struct IpcBufferDescriptor {
```

---

**STRUCT: IpcBufferSlot** (line 74)

```rust
pub struct IpcBufferSlot {
```

---

**CONST: STATE_FREE** (line 84)

```rust
pub const STATE_FREE: u8 = 0;
```

---

**CONST: STATE_REGISTERED** (line 85)

```rust
pub const STATE_REGISTERED: u8 = 1;
```

---

**CONST: STATE_IN_TRANSFER** (line 86)

```rust
pub const STATE_IN_TRANSFER: u8 = 2;
```

---

**CONST: STATE_MAPPED_REMOTE** (line 87)

```rust
pub const STATE_MAPPED_REMOTE: u8 = 3;
```

---

**FN: new** (line 89)

```rust
pub fn new(id: u64, owner: u64) -> Self {
```

---

**FN: is_free** (line 106)

```rust
pub fn is_free(&self) -> bool {
```

---

**FN: mark_in_transfer** (line 110)

```rust
pub fn mark_in_transfer(&self) -> bool {
```

---

**FN: release** (line 119)

```rust
pub fn release(&self) {
```

---

**STRUCT: IpcBufferPool** (line 124)

```rust
pub struct IpcBufferPool {
```

---

**CONST: fn** (line 130)

```rust
pub const fn new() -> Self {
```

---

**FN: allocate_slot** (line 137)

```rust
pub fn allocate_slot(&self, owner_task_id: u64) -> Result<u64, BufferError> {
```

---

**FN: register_buffer** (line 150)

```rust
pub fn register_buffer(
```

---

**FN: get_slot** (line 192)

```rust
pub fn get_slot(&self, slot_id: u64) -> Option<core::sync::atomic::AtomicPtr<()>> {
```

---

**FN: with_slot** (line 196)

```rust
pub fn with_slot<F, R>(&self, slot_id: u64, f: F) -> Option<R>
```

---

**STRUCT: TransferContext** (line 206)

```rust
pub struct TransferContext<'a> {
```

---

**FN: determine_mode** (line 216)

```rust
pub fn determine_mode(&self) -> BufferTransferMode {
```

---

**FN: execute_transfer** (line 235)

```rust
pub fn execute_transfer(&mut self) -> Result<(), BufferError> {
```

---

**FN: revoke_ipc_buffer_capabilities** (line 498)

```rust
pub fn revoke_ipc_buffer_capabilities(cnode: &mut CNode, buffer_slot_id: u64) {
```

---

**FN: cleanup_task_ipc_buffers** (line 508)

```rust
pub fn cleanup_task_ipc_buffers(pool: &IpcBufferPool, task_id: u64) {
```

---

**FN: sys_ipc_buffer_register** (line 551)

```rust
pub fn sys_ipc_buffer_register(
```

---

**FN: sys_ipc_buffer_unregister** (line 582)

```rust
pub fn sys_ipc_buffer_unregister(
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ds_ipc/endpoint.rs

**STRUCT: Endpoint** (line 8)

```rust
pub struct Endpoint {
```

---

**FN: new** (line 23)

```rust
pub fn new(badge: u64) -> Self {
```

---

**FN: enqueue_sender** (line 34)

```rust
pub fn enqueue_sender(&self, task: *mut Task) {
```

---

**FN: enqueue_receiver** (line 39)

```rust
pub fn enqueue_receiver(&self, task: *mut Task) {
```

---

**FN: dequeue_sender** (line 44)

```rust
pub fn dequeue_sender(&self) -> Option<*mut Task> {
```

---

**FN: dequeue_receiver** (line 49)

```rust
pub fn dequeue_receiver(&self) -> Option<*mut Task> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ds_ipc/msg.rs

**STRUCT: MessageInfo** (line 5)

```rust
pub struct MessageInfo {
```

---

**CONST: fn** (line 14)

```rust
pub const fn new(label: u32, length: u8) -> Self {
```

---

**STRUCT: MessageRegisters** (line 27)

```rust
pub struct MessageRegisters {
```

---

**CONST: fn** (line 32)

```rust
pub const fn empty() -> Self {
```

---

**STRUCT: IpcMessage** (line 39)

```rust
pub struct IpcMessage {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ds_ipc/syscall.rs

**ENUM: IpcSyscall** (line 8)

```rust
pub enum IpcSyscall {
```

---

**FN: sys_ipc_call** (line 15)

```rust
pub fn sys_ipc_call(
```

---

**FN: sys_ipc_recv** (line 43)

```rust
pub fn sys_ipc_recv(ep_cap_idx: u64) -> Result<IpcMessage, DsError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/editor/editor.c

**FUNCTION: editor_run** (line 530)

```c
int editor_run(const char *path)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/editor/editor.h

**FUNCTION: editor_run** (line 21)

```c
int editor_run(const char *path);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/editor/mouse.c

**FUNCTION: mouse_init** (line 43)

```c
void mouse_init(void)
```

---

**FUNCTION: mouse_poll** (line 56)

```c
int mouse_poll(int *dx, int *dy, int *dz, int *buttons)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/editor/mouse.h

**FUNCTION: mouse_init** (line 6)

```c
void mouse_init(void);
```

---

**FUNCTION: mouse_poll** (line 8)

```c
int mouse_poll(int *dx, int *dy, int *dz, int *buttons);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/disc.rs

**ENUM: DiscError** (line 4)

```rust
pub enum DiscError {
```

---

**TYPE: Result** (line 13)

```rust
pub type Result<T> = core::result::Result<T, DiscError>;
```

---

**CONST: MIN_BLOCK_SIZE** (line 15)

```rust
pub const MIN_BLOCK_SIZE: usize = 512;
```

---

**TRAIT: BlockDevice** (line 17)

```rust
pub trait BlockDevice {
```

---

**ENUM: PartitionTableKind** (line 33)

```rust
pub enum PartitionTableKind {
```

---

**STRUCT: Partition** (line 40)

```rust
pub struct Partition {
```

---

**FN: end_lba** (line 49)

```rust
pub fn end_lba(&self) -> Option<u64> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/driver/ata_pio.rs

**STRUCT: AtaPio** (line 26)

```rust
pub struct AtaPio {
```

---

**STATIC: ATA0** (line 34)

```rust
pub static ATA0: AtaPio = AtaPio::master(0x1F0, 0x3F6);
```

---

**STATIC: ATA1** (line 35)

```rust
pub static ATA1: AtaPio = AtaPio::slave(0x1F0, 0x3F6);
```

---

**CONST: fn** (line 38)

```rust
pub const fn master(base: u16, ctrl: u16) -> Self {
```

---

**CONST: fn** (line 48)

```rust
pub const fn slave(base: u16, ctrl: u16) -> Self {
```

---

**FN: is_present** (line 66)

```rust
pub fn is_present(&self) -> bool {
```

---

**FN: identify** (line 129)

```rust
pub fn identify(&self) -> Result<[u16; 256], DriverError> {
```

---

**FN: probe** (line 253)

```rust
pub fn probe() -> usize {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/driver/block.rs

**ENUM: DriverError** (line 2)

```rust
pub enum DriverError {
```

---

**TRAIT: BlockDevice** (line 11)

```rust
pub trait BlockDevice {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/driver/lock.rs

**STRUCT: IrqGuard** (line 3)

```rust
pub struct IrqGuard {
```

---

**FN: lock** (line 8)

```rust
pub fn lock() -> Self {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/driver/mod.rs

**FN: init** (line 7)

```rust
pub fn init() {
```

---

**FN: self_test** (line 17)

```rust
pub fn self_test() -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/driver/registry.rs

**FN: register** (line 7)

```rust
pub fn register(dev: &'static dyn BlockDevice) -> bool {
```

---

**FN: get** (line 22)

```rust
pub fn get(index: usize) -> Option<&'static dyn BlockDevice> {
```

---

**FN: first** (line 28)

```rust
pub fn first() -> Option<&'static dyn BlockDevice> {
```

---

**FN: count** (line 32)

```rust
pub fn count() -> usize {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/ext4/mod.rs

**ENUM: ExtError** (line 6)

```rust
pub enum ExtError {
```

---

**STRUCT: Superblock** (line 15)

```rust
pub struct Superblock {
```

---

**STRUCT: RawInode** (line 27)

```rust
pub struct RawInode {
```

---

**STRUCT: DirEntry** (line 33)

```rust
pub struct DirEntry {
```

---

**STRUCT: Ext4** (line 39)

```rust
pub struct Ext4 {
```

---

**FN: mount** (line 65)

```rust
pub fn mount(disk: &'static dyn BlockDevice) -> Result<Self, ExtError> {
```

---

**FN: read_blk** (line 129)

```rust
pub fn read_blk(&self, blk: u64) -> Result<Vec<u8>, ExtError> {
```

---

**FN: read_inode** (line 156)

```rust
pub fn read_inode(&self, ino: u32) -> Result<RawInode, ExtError> {
```

---

**FN: read_file** (line 227)

```rust
pub fn read_file(&self, inode: &RawInode,
```

---

**FN: read_dir** (line 262)

```rust
pub fn read_dir(&self, inode: &RawInode) -> Result<Vec<DirEntry>, ExtError> {
```

---

**FN: resolve** (line 297)

```rust
pub fn resolve(&self, path: &str) -> Result<RawInode, ExtError> {
```

---

**FN: read_path** (line 332)

```rust
pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Result<usize, ExtError> {
```

---

**FN: list_path** (line 337)

```rust
pub fn list_path(&self, path: &str) -> Result<Vec<DirEntry>, ExtError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/fat32/mod.rs

**ENUM: FatError** (line 6)

```rust
pub enum FatError {
```

---

**STRUCT: Fat32** (line 14)

```rust
pub struct Fat32 {
```

---

**STRUCT: FatEntry** (line 23)

```rust
pub struct FatEntry {
```

---

**FN: mount** (line 39)

```rust
pub fn mount(disk: &'static dyn BlockDevice) -> Result<Self, FatError> {
```

---

**FN: read_dir_cluster** (line 142)

```rust
pub fn read_dir_cluster(&self, start: u32) -> Result<Vec<FatEntry>, FatError> {
```

---

**FN: resolve** (line 226)

```rust
pub fn resolve(&self, path: &str) -> Result<FatEntry, FatError> {
```

---

**FN: read_file** (line 264)

```rust
pub fn read_file(&self, entry: &FatEntry,
```

---

**FN: read_path** (line 290)

```rust
pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Result<usize, FatError> {
```

---

**FN: list_path** (line 295)

```rust
pub fn list_path(&self, path: &str) -> Result<Vec<FatEntry>, FatError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/mbr.rs

**STRUCT: MbrEntry** (line 9)

```rust
pub struct MbrEntry {
```

---

**STRUCT: Partition** (line 16)

```rust
pub struct Partition {
```

---

**FN: parse_mbr** (line 57)

```rust
pub fn parse_mbr(buf: &[u8]) -> Option<[MbrEntry; 4]> {
```

---

**FN: probe_disk** (line 80)

```rust
pub fn probe_disk(d: &'static dyn BlockDevice) -> usize {
```

---

**FN: init** (line 138)

```rust
pub fn init() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/mod.rs

**FN: init** (line 9)

```rust
pub fn init() {
```

---

**FN: self_test** (line 14)

```rust
pub fn self_test() -> TestResult {
```

---

**FN: ensure_formatted** (line 57)

```rust
pub fn ensure_formatted(dev: &dyn BlockDevice) -> Result<()> {
```

---

**FN: root_device** (line 113)

```rust
pub fn root_device() -> Option<&'static dyn driver::block::BlockDevice> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/btree.rs

**STRUCT: BtreeNode** (line 9)

```rust
pub struct BtreeNode {
```

---

**FN: lookup_dir_entry** (line 21)

```rust
pub fn lookup_dir_entry(fs: &TangFs, dir_ino: u64, name: &str) -> Result<u64> {
```

---

**FN: read_dir_entries** (line 91)

```rust
pub fn read_dir_entries(fs: &TangFs, dir_ino: u64) -> Result<Vec<DirEntry>> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/dir.rs

**STRUCT: DirEntry** (line 2)

```rust
pub struct DirEntry {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/extern.rs

**FN: allocate_block** (line 4)

```rust
pub fn allocate_block(fs: &TangFs) -> Result<u64> {
```

---

**FN: free_block** (line 35)

```rust
pub fn free_block(fs: &TangFs, block: u64) -> Result<()> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/file.rs

**FN: read_file** (line 4)

```rust
pub fn read_file(fs: &TangFs, inode: &Inode, mut offset: u64, buf: &mut [u8]) -> Result<usize> {
```

---

**FN: write_file** (line 50)

```rust
pub fn write_file(fs: &TangFs, inode: &mut Inode, mut offset: u64, data: &[u8]) -> Result<usize> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/inode.rs

**STRUCT: Inode** (line 7)

```rust
pub struct Inode {
```

---

**STRUCT: Extent** (line 28)

```rust
pub struct Extent {
```

---

**STRUCT: InodeHandle** (line 35)

```rust
pub struct InodeHandle {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/journal.rs

**STRUCT: Journal** (line 5)

```rust
pub struct Journal {
```

---

**FN: open** (line 13)

```rust
pub fn open(device: &'static dyn crate::fs::driver::BlockDevice, sb: &super::Superblock) -> Result<Self> {
```

---

**FN: write_block** (line 22)

```rust
pub fn write_block(&mut self, block: u64, data: &[u8]) -> Result<()> {
```

---

**FN: replay** (line 39)

```rust
pub fn replay(&self) -> Result<()> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/mod.rs

**STRUCT: TangFs** (line 18)

```rust
pub struct TangFs {
```

---

**FN: mount** (line 26)

```rust
pub fn mount(device: &'static dyn crate::fs::driver::BlockDevice) -> Result<Self> {
```

---

**FN: read_block** (line 49)

```rust
pub fn read_block(&self, block: u64) -> Result<Vec<u8>> {
```

---

**FN: write_block** (line 63)

```rust
pub fn write_block(&self, block: u64, data: &[u8]) -> Result<()> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/superblock.rs

**STRUCT: Superblock** (line 5)

```rust
pub struct Superblock {
```

---

**FN: read** (line 26)

```rust
pub fn read(device: &dyn BlockDevice) -> Result<Self> {
```

---

**FN: write** (line 45)

```rust
pub fn write(&self, device: &dyn BlockDevice) -> Result<()> {
```

---

**FN: generate_uuid** (line 77)

```rust
pub fn generate_uuid() -> [u8; 16] {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tfs.rs

**CONST: SUPER_MAGIC** (line 4)

```rust
pub const SUPER_MAGIC: [u8; 4] = *b"TFS1";
```

---

**CONST: ROOT_DIR** (line 12)

```rust
pub const ROOT_DIR: u32 = 2;
```

---

**ENUM: FsError** (line 19)

```rust
pub enum FsError {
```

---

**TYPE: Result** (line 30)

```rust
pub type Result<T> = core::result::Result<T, FsError>;
```

---

**STRUCT: Superblock** (line 33)

```rust
pub struct Superblock {
```

---

**FN: format** (line 48)

```rust
pub fn format(dev: &dyn BlockDevice) -> Result<()> {
```

---

**FN: read_superblock** (line 70)

```rust
pub fn read_superblock(dev: &dyn BlockDevice) -> Result<Superblock> {
```

---

**FN: write_superblock** (line 85)

```rust
pub fn write_superblock(dev: &dyn BlockDevice, sb: &Superblock) -> Result<()> {
```

---

**FN: list_dir** (line 156)

```rust
pub fn list_dir(dev: &dyn BlockDevice, dir: u32, out: &mut impl Write) -> Result<()> {
```

---

**FN: write_file** (line 186)

```rust
pub fn write_file(dev: &dyn BlockDevice, dir: u32, name: &str, data: &[u8]) -> Result<()> {
```

---

**FN: remove** (line 246)

```rust
pub fn remove(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<()> {
```

---

**FN: mkdir** (line 274)

```rust
pub fn mkdir(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<()> {
```

---

**FN: find_dir** (line 325)

```rust
pub fn find_dir(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<u32> {
```

---

**FN: entries** (line 333)

```rust
pub fn entries(dev: &dyn BlockDevice, dir: u32) -> Result<alloc::vec::Vec<(alloc::string::String, u32, u8)>> {
```

---

**FN: read_file** (line 352)

```rust
pub fn read_file(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<alloc::vec::Vec<u8>> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/vfs.rs

**STRUCT: FsEntry** (line 6)

```rust
pub struct FsEntry {
```

---

**ENUM: Mounted** (line 12)

```rust
pub enum Mounted {
```

---

**FN: read_path** (line 19)

```rust
pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Option<usize> {
```

---

**FN: list_path** (line 27)

```rust
pub fn list_path(&self, path: &str) -> Option<Vec<FsEntry>> {
```

---

**FN: mount_all** (line 56)

```rust
pub fn mount_all() {
```

---

**FN: root** (line 74)

```rust
pub fn root() -> Option<&'static Mounted> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gdt.rs

**CONST: DOUBLE_FAULT_IST_INDEX** (line 6)

```rust
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
```

---

**FN: init** (line 52)

```rust
pub fn init() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/console.rs

**CONST: MAX_COLS** (line 9)

```rust
pub const MAX_COLS: usize = 80;
```

---

**CONST: MAX_ROWS** (line 10)

```rust
pub const MAX_ROWS: usize = 25;
```

---

**FN: cols** (line 32)

```rust
pub fn cols() -> usize {
```

---

**FN: rows** (line 36)

```rust
pub fn rows() -> usize {
```

---

**FN: fb_info** (line 40)

```rust
pub fn fb_info() -> (u32, u32, u32, u64) {
```

---

**FN: set_enabled** (line 79)

```rust
pub fn set_enabled(enabled: bool) {
```

---

**FN: test_fill** (line 145)

```rust
pub fn test_fill(r: u32, g: u32, b: u32) -> bool {
```

---

**FN: resync_background** (line 164)

```rust
pub fn resync_background() -> bool {
```

---

**FN: init** (line 188)

```rust
pub fn init(fb_addr: u64, width: u32, height: u32, stride: u32, format: PixelFormat) -> bool {
```

---

**FN: refresh** (line 313)

```rust
pub fn refresh() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/font.rs

**CONST: FONT8X8** (line 1)

```rust
pub const FONT8X8: [[u8; 8]; 95] = [
```

---

**STATIC: font8x8** (line 112)

```rust
pub static font8x8: [[u8; 8]; 96] = make_font8x8();
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/framebuffer.rs

**STATIC: mut** (line 3)

```rust
pub static mut FLIP: bool = false;
```

---

**STATIC: mut** (line 5)

```rust
pub static mut FLIP_X: bool = false;
```

---

**ENUM: PixelFormat** (line 8)

```rust
pub enum PixelFormat {
```

---

**CONST: PALETTE16** (line 14)

```rust
pub const PALETTE16: [(u32, u32, u32); 16] = [
```

---

**STRUCT: Framebuffer** (line 21)

```rust
pub struct Framebuffer {
```

---

**FN: get** (line 69)

```rust
pub fn get(&self, x: usize, y: usize) -> u32 {
```

---

**FN: set** (line 85)

```rust
pub fn set(&mut self, x: usize, y: usize, c: u32) {
```

---

**FN: offset** (line 107)

```rust
pub fn offset(&self, x: usize, y: usize) -> usize {
```

---

**FN: add** (line 111)

```rust
pub fn add(&mut self, x: usize, y: usize, r: u32, g: u32, b: u32) {
```

---

**FN: rgb** (line 153)

```rust
pub fn rgb(r: u32, g: u32, b: u32) -> u32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/galaxy.rs

**FN: render** (line 96)

```rust
pub fn render(fb: &mut Framebuffer, t: u32) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/mod.rs

**CONST: FB_PHYS** (line 10)

```rust
pub const FB_PHYS: u64 = 0xA0000;
```

---

**FN: init** (line 16)

```rust
pub fn init() -> bool {
```

---

**FN: init_mode** (line 25)

```rust
pub fn init_mode(fb_phys: u64, width: u32, height: u32, stride: u32) -> bool {
```

---

**FN: set_resolution** (line 30)

```rust
pub fn set_resolution(mode: vga::VideoMode) -> bool {
```

---

**FN: set_resolution_w_h** (line 47)

```rust
pub fn set_resolution_w_h(width: u32, height: u32) -> bool {
```

---

**FN: current_resolution** (line 73)

```rust
pub fn current_resolution() -> (u32, u32) {
```

---

**FN: refresh** (line 77)

```rust
pub fn refresh() {
```

---

**FN: self_test** (line 81)

```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/panic_screen.rs

**FN: show** (line 11)

```rust
pub fn show(info: &PanicInfo) -> ! {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/vga.rs

**ENUM: VideoMode** (line 4)

```rust
pub enum VideoMode {
```

---

**FN: set_mode** (line 47)

```rust
pub fn set_mode(mode: VideoMode) {
```

---

**FN: bochs_version** (line 105)

```rust
pub fn bochs_version() -> Option<u16> {
```

---

**FN: bochs_disable** (line 114)

```rust
pub fn bochs_disable() {
```

---

**FN: bochs_set_mode** (line 120)

```rust
pub fn bochs_set_mode(width: u32, height: u32, bpp: u32) -> bool {
```

---

**FN: bochs_lfb_base** (line 139)

```rust
pub fn bochs_lfb_base() -> Option<u64> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/aut.rs

**CONST: VID_HDMI_INIT** (line 3)

```rust
pub const VID_HDMI_INIT: u32 = 3;
```

---

**CONST: VID_HDMI_FILL** (line 4)

```rust
pub const VID_HDMI_FILL: u32 = 4;
```

---

**CONST: VID_HDMI_POLL** (line 5)

```rust
pub const VID_HDMI_POLL: u32 = 5;
```

---

**CONST: VID_HDMI_CAPS** (line 6)

```rust
pub const VID_HDMI_CAPS: u32 = 6;
```

---

**CONST: VID_MODE_GET** (line 7)

```rust
pub const VID_MODE_GET: u32 = 7;
```

---

**CONST: VID_MODE_LIST** (line 8)

```rust
pub const VID_MODE_LIST: u32 = 8;
```

---

**CONST: VID_MODE_SET** (line 9)

```rust
pub const VID_MODE_SET: u32 = 9;
```

---

**CONST: VID_GRANT_FB** (line 10)

```rust
pub const VID_GRANT_FB: u32 = 10;
```

---

**CONST: VID_REVOKE_FB** (line 11)

```rust
pub const VID_REVOKE_FB: u32 = 11;
```

---

**CONST: VID_HDMI_ACQUIRE** (line 12)

```rust
pub const VID_HDMI_ACQUIRE: u32 = 12;
```

---

**CONST: VID_HDMI_RELEASE** (line 13)

```rust
pub const VID_HDMI_RELEASE: u32 = 13;
```

---

**FN: authorize** (line 18)

```rust
pub fn authorize(ring: u8, op: u32) -> bool {
```

---

**FN: tick_reset** (line 46)

```rust
pub fn tick_reset() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/bridge.rs

**FN: hdmi_call** (line 24)

```rust
pub fn hdmi_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8, owner: u32) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/hdmi.h

**FUNCTION: hdmi_caps** (line 43)

```c
void hdmi_caps(hdmi_caps_t *out);
```

---

**FUNCTION: hdmi_caps_raw** (line 51)

```c
void hdmi_caps_raw(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys);
```

---

**FUNCTION: hdmi_fb_revoke** (line 54)

```c
void hdmi_fb_revoke(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/init.c

**FUNCTION: hdmi_caps** (line 35)

```c
void hdmi_caps(hdmi_caps_t *out)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/init.rs

**FN: init** (line 6)

```rust
pub fn init() -> bool {
```

---

**FN: ready** (line 11)

```rust
pub fn ready() -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/interface.c

**FUNCTION: hdmi_iface_caps** (line 88)

```c
void hdmi_iface_caps(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/interface.h

**FUNCTION: hdmi_iface_caps** (line 24)

```c
void hdmi_iface_caps(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/operation.c

**FUNCTION: hdmi_op_state** (line 20)

```c
void hdmi_op_state(hdmi_caps_t *out)
```

---

**FUNCTION: hdmi_op_set_fb** (line 32)

```c
void hdmi_op_set_fb(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride)
```

---

**FUNCTION: hdmi_op_exec** (line 42)

```c
void hdmi_op_exec(hdmi_transfer_t *t)
```

---

**FUNCTION: hdmi_op_set_mode** (line 162)

```c
void hdmi_op_set_mode(const hdmi_mode_t *m)
```

---

**FUNCTION: hdmi_fb_revoke** (line 186)

```c
void hdmi_fb_revoke(void)
```

---

**FUNCTION: hdmi_caps_raw** (line 196)

```c
void hdmi_caps_raw(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/operation.h

**FUNCTION: hdmi_op_set_mode** (line 7)

```c
void hdmi_op_set_mode(const hdmi_mode_t *m);
```

---

**FUNCTION: hdmi_op_exec** (line 8)

```c
void hdmi_op_exec(hdmi_transfer_t *t);
```

---

**FUNCTION: hdmi_op_state** (line 9)

```c
void hdmi_op_state(hdmi_caps_t *out);
```

---

**FUNCTION: hdmi_op_set_fb** (line 10)

```c
void hdmi_op_set_fb(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/interrupts.rs

**STATIC: BREAKPOINT_HITS** (line 14)

```rust
pub static BREAKPOINT_HITS: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: TIMER_TICKS** (line 15)

```rust
pub static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: KEYBOARD_HITS** (line 16)

```rust
pub static KEYBOARD_HITS: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: IPI_HITS** (line 17)

```rust
pub static IPI_HITS: AtomicU64 = AtomicU64::new(0);
```

---

**CONST: IPI_VECTOR** (line 19)

```rust
pub const IPI_VECTOR: u8 = 0x30;
```

---

**CONST: PIC_1_OFFSET** (line 42)

```rust
pub const PIC_1_OFFSET: u8 = 32;
```

---

**CONST: PIC_2_OFFSET** (line 43)

```rust
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
```

---

**STATIC: PICS** (line 45)

```rust
pub static PICS: Mutex<ChainedPics> =
```

---

**ENUM: InterruptIndex** (line 50)

```rust
pub enum InterruptIndex {
```

---

**FN: init_idt** (line 83)

```rust
pub fn init_idt() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/main.rs

**FN: init** (line 141)

```rust
pub fn init() {
```

---

**FN: hlt_loop** (line 145)

```rust
pub fn hlt_loop() -> ! {
```

---

**FN: init_permissions** (line 149)

```rust
pub fn init_permissions() {
```

---

**FN: kernel_main** (line 162)

```rust
pub fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
```

---

**FN: kernel_main_riscv** (line 221)

```rust
pub fn kernel_main_riscv() -> ! {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/api/alloc.c

**FUNCTION: kfree** (line 44)

```c
void kfree(void *ptr) {
```

---

**FUNCTION: kfree_pages** (line 98)

```c
void kfree_pages(void *ptr, const size_t pages) {
```

---

**FUNCTION: kalloc_dump** (line 126)

```c
void kalloc_dump(void) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/api/alloc.h

**FUNCTION: kfree** (line 13)

```c
void kfree(void *ptr);
```

---

**FUNCTION: kfree_pages** (line 18)

```c
void kfree_pages(void *ptr, size_t pages);
```

---

**FUNCTION: kalloc_dump** (line 24)

```c
void kalloc_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/debug/alloc_debug.c

**FUNCTION: dbg_free** (line 56)

```c
void dbg_free(void *ptr) {
```

---

**FUNCTION: mm_debug_dump** (line 104)

```c
void mm_debug_dump(void) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/debug/alloc_debug.h

**FUNCTION: dbg_free** (line 9)

```c
void dbg_free(void *ptr);
```

---

**FUNCTION: mm_debug_dump** (line 13)

```c
void mm_debug_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/debug/leak.c

**FUNCTION: leak_dump** (line 74)

```c
void leak_dump(void) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/debug/leak.h

**FUNCTION: leak_dump** (line 16)

```c
void leak_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/debug/stats.c

**FUNCTION: alloc_stats_note_alloc** (line 15)

```c
void alloc_stats_note_alloc(size_t size) {
```

---

**FUNCTION: alloc_stats_note_free** (line 23)

```c
void alloc_stats_note_free(size_t size) {
```

---

**FUNCTION: alloc_stats_dump** (line 61)

```c
void alloc_stats_dump(void) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/debug/stats.h

**FUNCTION: alloc_stats_note_alloc** (line 8)

```c
void alloc_stats_note_alloc(size_t size);
```

---

**FUNCTION: alloc_stats_note_free** (line 9)

```c
void alloc_stats_note_free(size_t size);
```

---

**FUNCTION: alloc_stats_dump** (line 17)

```c
void alloc_stats_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/heap/buddy.c

**FUNCTION: buddy_free** (line 136)

```c
void buddy_free(void *ptr) {
```

---

**FUNCTION: buddy_dump** (line 173)

```c
void buddy_dump(void) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/heap/buddy.h

**FUNCTION: bool** (line 8)

```c
typedef bool (*buddy_map_cb)(uint64_t virt, size_t size);
```

---

**FUNCTION: void** (line 9)

```c
typedef void (*buddy_unmap_cb)(uint64_t virt, size_t size);
```

---

**FUNCTION: buddy_free** (line 20)

```c
void buddy_free(void *ptr);
```

---

**FUNCTION: buddy_dump** (line 27)

```c
void buddy_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/heap/heap.c

**FUNCTION: heap_free** (line 187)

```c
void heap_free(void *ptr)
```

---

**FUNCTION: heap_dump** (line 228)

```c
void heap_dump(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/heap/heap.h

**FUNCTION: heap_free** (line 15)

```c
void heap_free(void *ptr);
```

---

**FUNCTION: heap_dump** (line 19)

```c
void heap_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/heap/slab.c

**FUNCTION: slab_free** (line 286)

```c
void slab_free(void *ptr)
```

---

**FUNCTION: slab_dump** (line 436)

```c
void slab_dump(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/heap/slab.h

**FUNCTION: slab_free** (line 14)

```c
void slab_free(void *ptr);
```

---

**FUNCTION: slab_dump** (line 21)

```c
void slab_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/physical/bitmap.c

**FUNCTION: bitmap_init_virt** (line 84)

```c
void bitmap_init_virt(bitmap_t *bm, void *storage, size_t bit_count)
```

---

**FUNCTION: bitmap_fill** (line 119)

```c
void bitmap_fill(bitmap_t *bm, bool value)
```

---

**FUNCTION: bitmap_set** (line 137)

```c
void bitmap_set(bitmap_t *bm, size_t bit)
```

---

**FUNCTION: bitmap_clear** (line 149)

```c
void bitmap_clear(bitmap_t *bm, size_t bit)
```

---

**FUNCTION: bitmap_set_range** (line 173)

```c
void bitmap_set_range(bitmap_t *bm, size_t start, size_t count)
```

---

**FUNCTION: bitmap_clear_range** (line 216)

```c
void bitmap_clear_range(bitmap_t *bm, size_t start, size_t count)
```

---

**FUNCTION: bitmap_free** (line 519)

```c
void bitmap_free(bitmap_t *bm, size_t bit)
```

---

**FUNCTION: bitmap_free_range** (line 534)

```c
void bitmap_free_range(bitmap_t *bm, size_t start, size_t count)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/physical/bitmap.h

**FUNCTION: bitmap_init_virt** (line 33)

```c
void bitmap_init_virt(bitmap_t *bm, void *storage, size_t bit_count);
```

---

**FUNCTION: bitmap_fill** (line 37)

```c
void bitmap_fill(bitmap_t *bm, bool value);
```

---

**FUNCTION: bitmap_set** (line 39)

```c
void bitmap_set(bitmap_t *bm, size_t bit);
```

---

**FUNCTION: bitmap_clear** (line 40)

```c
void bitmap_clear(bitmap_t *bm, size_t bit);
```

---

**FUNCTION: bitmap_fill** (line 43)

```c
void bitmap_fill(bitmap_t *bm, bool value);
```

---

**FUNCTION: bitmap_set** (line 45)

```c
void bitmap_set(bitmap_t *bm, size_t bit);
```

---

**FUNCTION: bitmap_clear** (line 46)

```c
void bitmap_clear(bitmap_t *bm, size_t bit);
```

---

**FUNCTION: bitmap_set_range** (line 49)

```c
void bitmap_set_range(bitmap_t *bm, size_t start, size_t count);
```

---

**FUNCTION: bitmap_clear_range** (line 50)

```c
void bitmap_clear_range(bitmap_t *bm, size_t start, size_t count);
```

---

**FUNCTION: bitmap_free** (line 61)

```c
void bitmap_free(bitmap_t *bm, size_t bit);
```

---

**FUNCTION: bitmap_free_range** (line 62)

```c
void bitmap_free_range(bitmap_t *bm, size_t start, size_t count);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/physical/pmm.c

**FUNCTION: pmm_dump** (line 449)

```c
void pmm_dump(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/physical/pmm.h

**FUNCTION: pmm_dump** (line 48)

```c
void pmm_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/special/contiguous.c

**FUNCTION: contig_free** (line 56)

```c
void contig_free(uint64_t phys, size_t bytes)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/special/contiguous.h

**FUNCTION: contig_free** (line 13)

```c
void contig_free(uint64_t phys, size_t bytes);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/special/dma.c

**FUNCTION: dma_free_coherent** (line 57)

```c
void dma_free_coherent(uint64_t phys, void *virt, size_t bytes)
```

---

**FUNCTION: dma_sync_for_device** (line 94)

```c
void dma_sync_for_device(void *virt, size_t len)
```

---

**FUNCTION: dma_sync_for_cpu** (line 103)

```c
void dma_sync_for_cpu(void *virt, size_t len)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/special/dma.h

**FUNCTION: dma_free_coherent** (line 16)

```c
void dma_free_coherent(uint64_t phys, void *virt, size_t bytes);
```

---

**FUNCTION: dma_sync_for_device** (line 18)

```c
void dma_sync_for_device(void *virt, size_t len);
```

---

**FUNCTION: dma_sync_for_cpu** (line 19)

```c
void dma_sync_for_cpu(void *virt, size_t len);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/virtual/mapping.c

**FUNCTION: mapping_init** (line 18)

```c
void mapping_init(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/virtual/mapping.h

**FUNCTION: mapping_init** (line 12)

```c
void mapping_init(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/virtual/page.c

**FUNCTION: page_set_type** (line 352)

```c
void page_set_type(page_t *page, uint32_t type)
```

---

**FUNCTION: page_dump** (line 387)

```c
void page_dump(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/virtual/page.h

**FUNCTION: page_set_type** (line 43)

```c
void page_set_type(page_t *page, uint32_t type);
```

---

**FUNCTION: page_dump** (line 48)

```c
void page_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/virtual/vmm.c

**FUNCTION: vmm_dump** (line 463)

```c
void vmm_dump(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/alloc/virtual/vmm.h

**FUNCTION: vmm_dump** (line 36)

```c
void vmm_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/api.rs

**FN: kmalloc** (line 5)

```rust
pub fn kmalloc(size: usize) -> Option<*mut u8> {
```

---

**FN: kzalloc** (line 11)

```rust
pub fn kzalloc(size: usize) -> Option<*mut u8> {
```

---

**FN: krealloc** (line 17)

```rust
pub fn krealloc(ptr: *mut u8, size: usize) -> Option<*mut u8> {
```

---

**FN: kfree** (line 23)

```rust
pub fn kfree(ptr: *mut u8) {
```

---

**FN: kalloc_pages** (line 27)

```rust
pub fn kalloc_pages(pages: usize) -> Option<*mut u8> {
```

---

**FN: kfree_pages** (line 33)

```rust
pub fn kfree_pages(ptr: *mut u8, _pages: usize) {
```

---

**FN: virt_to_phys** (line 37)

```rust
pub fn virt_to_phys(ptr: *mut u8) -> u64 {
```

---

**STRUCT: KernelAlloc** (line 41)

```rust
pub struct KernelAlloc;
```

---

**FN: self_test** (line 72)

```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/aarch64/memory.h

**FUNCTION: arch_memory_init** (line 18)

```c
void arch_memory_init(const arch_raw_mem_entry_t *entries,
```

---

**FUNCTION: arch_memory_reserve_range** (line 28)

```c
void arch_memory_reserve_range(uint64_t base, uint64_t len);
```

---

**FUNCTION: arch_memory_dump** (line 38)

```c
void arch_memory_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/aarch64/paging.c

**FUNCTION: paging_init** (line 267)

```c
void paging_init(uint64_t boot_phys_offset)
```

---

**FUNCTION: paging_write_cr3** (line 345)

```c
void paging_write_cr3(uint64_t pml4_phys)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/aarch64/paging.h

**FUNCTION: paging_init** (line 17)

```c
void paging_init(uint64_t boot_phys_offset);
```

---

**FUNCTION: paging_write_cr3** (line 22)

```c
void paging_write_cr3(uint64_t pml4_phys);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/aarch64/tlb.c

**FUNCTION: tlb_flush_all** (line 16)

```c
void tlb_flush_all(void)
```

---

**FUNCTION: tlb_flush_all_including_global** (line 24)

```c
void tlb_flush_all_including_global(void)
```

---

**FUNCTION: tlb_flush_page_addr** (line 29)

```c
void tlb_flush_page_addr(uint64_t addr)
```

---

**FUNCTION: tlb_flush_page** (line 37)

```c
void tlb_flush_page(const void *addr)
```

---

**FUNCTION: tlb_flush_range_addr** (line 42)

```c
void tlb_flush_range_addr(uint64_t addr, size_t pages)
```

---

**FUNCTION: tlb_flush_range** (line 54)

```c
void tlb_flush_range(const void *addr, size_t pages)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/aarch64/tlb.h

**FUNCTION: tlb_flush_all** (line 11)

```c
void tlb_flush_all(void);
```

---

**FUNCTION: tlb_flush_all_including_global** (line 12)

```c
void tlb_flush_all_including_global(void);
```

---

**FUNCTION: tlb_flush_page** (line 14)

```c
void tlb_flush_page(const void *addr);
```

---

**FUNCTION: tlb_flush_page_addr** (line 15)

```c
void tlb_flush_page_addr(uint64_t addr);
```

---

**FUNCTION: tlb_flush_range** (line 17)

```c
void tlb_flush_range(const void *addr, size_t pages);
```

---

**FUNCTION: tlb_flush_range_addr** (line 18)

```c
void tlb_flush_range_addr(uint64_t addr, size_t pages);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/x86_64/memory.c

**FUNCTION: arch_memory_init_multiboot2** (line 29)

```c
void arch_memory_init_multiboot2(uint32_t magic,
```

---

**FUNCTION: arch_memory_init** (line 444)

```c
void arch_memory_init(const arch_raw_mem_entry_t *entries,
```

---

**FUNCTION: arch_memory_reserve_range** (line 585)

```c
void arch_memory_reserve_range(uint64_t base, uint64_t len)
```

---

**FUNCTION: arch_memory_dump** (line 659)

```c
void arch_memory_dump(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/x86_64/memory.h

**FUNCTION: arch_memory_init** (line 64)

```c
void arch_memory_init(const arch_raw_mem_entry_t *entries,
```

---

**FUNCTION: arch_memory_reserve_range** (line 79)

```c
void arch_memory_reserve_range(uint64_t base, uint64_t len);
```

---

**FUNCTION: arch_memory_dump** (line 83)

```c
void arch_memory_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/x86_64/paging.c

**FUNCTION: paging_flush_tlb_all** (line 71)

```c
void paging_flush_tlb_all(void)
```

---

**FUNCTION: paging_flush_page** (line 76)

```c
void paging_flush_page(uint64_t addr)
```

---

**FUNCTION: paging_set_boot_phys_offset** (line 299)

```c
void paging_set_boot_phys_offset(uint64_t phys_offset)
```

---

**FUNCTION: paging_init_direct_map** (line 314)

```c
void paging_init_direct_map(void)
```

---

**FUNCTION: paging_init** (line 354)

```c
void paging_init(uint64_t phys_offset)
```

---

**FUNCTION: paging_enable_nx** (line 531)

```c
void paging_enable_nx(void)
```

---

**FUNCTION: paging_write_cr3** (line 549)

```c
void paging_write_cr3(uint64_t pml4_phys)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/x86_64/paging.h

**FUNCTION: paging_set_boot_phys_offset** (line 35)

```c
void paging_set_boot_phys_offset(uint64_t phys_offset);
```

---

**FUNCTION: paging_init_direct_map** (line 39)

```c
void paging_init_direct_map(void);
```

---

**FUNCTION: paging_init** (line 41)

```c
void paging_init(uint64_t boot_phys_offset);
```

---

**FUNCTION: paging_flush_tlb_all** (line 44)

```c
void paging_flush_tlb_all(void);
```

---

**FUNCTION: paging_flush_page** (line 45)

```c
void paging_flush_page(uint64_t addr);
```

---

**FUNCTION: paging_enable_nx** (line 56)

```c
void paging_enable_nx(void);
```

---

**FUNCTION: paging_destroy_pml4** (line 70)

```c
void paging_destroy_pml4(uint64_t pml4_phys);
```

---

**FUNCTION: paging_switch_pml4** (line 71)

```c
void paging_switch_pml4(uint64_t pml4_phys);
```

---

**FUNCTION: paging_enable_write_protect** (line 75)

```c
void paging_enable_write_protect(void);
```

---

**FUNCTION: paging_disable_write_protect** (line 76)

```c
void paging_disable_write_protect(void);
```

---

**FUNCTION: paging_enable_smep** (line 79)

```c
void paging_enable_smep(void);
```

---

**FUNCTION: paging_enable_smap** (line 80)

```c
void paging_enable_smap(void);
```

---

**FUNCTION: paging_enable_pcid** (line 85)

```c
void paging_enable_pcid(void);
```

---

**FUNCTION: paging_invpcid** (line 87)

```c
void paging_invpcid(uint64_t type, uint64_t pcid, uint64_t addr);
```

---

**FUNCTION: paging_assert_4level_paging** (line 91)

```c
void paging_assert_4level_paging(void);
```

---

**FUNCTION: paging_write_cr3** (line 92)

```c
void paging_write_cr3(uint64_t pml4_phys);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/x86_64/tlb.c

**FUNCTION: tlb_flush_all** (line 73)

```c
void tlb_flush_all(void)
```

---

**FUNCTION: tlb_flush_all_including_global** (line 84)

```c
void tlb_flush_all_including_global(void)
```

---

**FUNCTION: tlb_flush_page_addr** (line 101)

```c
void tlb_flush_page_addr(uint64_t addr)
```

---

**FUNCTION: tlb_flush_page** (line 106)

```c
void tlb_flush_page(const void *addr)
```

---

**FUNCTION: tlb_flush_range_addr** (line 116)

```c
void tlb_flush_range_addr(uint64_t addr, size_t pages)
```

---

**FUNCTION: tlb_flush_range** (line 140)

```c
void tlb_flush_range(const void *addr, size_t pages)
```

---

**FUNCTION: tlb_flush_pcid** (line 150)

```c
void tlb_flush_pcid(uint16_t pcid)
```

---

**FUNCTION: tlb_flush_pcid_addr** (line 163)

```c
void tlb_flush_pcid_addr(uint16_t pcid, uint64_t addr)
```

---

**FUNCTION: tlb_wbinvd** (line 174)

```c
void tlb_wbinvd(void)
```

---

**FUNCTION: tlb_clflush** (line 179)

```c
void tlb_clflush(uint64_t addr)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/x86_64/tlb.h

**FUNCTION: tlb_flush_all** (line 19)

```c
void tlb_flush_all(void);
```

---

**FUNCTION: tlb_flush_all_including_global** (line 20)

```c
void tlb_flush_all_including_global(void);
```

---

**FUNCTION: tlb_flush_page** (line 22)

```c
void tlb_flush_page(const void *addr);
```

---

**FUNCTION: tlb_flush_page_addr** (line 23)

```c
void tlb_flush_page_addr(uint64_t addr);
```

---

**FUNCTION: tlb_flush_range** (line 25)

```c
void tlb_flush_range(const void *addr, size_t pages);
```

---

**FUNCTION: tlb_flush_range_addr** (line 26)

```c
void tlb_flush_range_addr(uint64_t addr, size_t pages);
```

---

**FUNCTION: tlb_flush_pcid** (line 28)

```c
void tlb_flush_pcid(uint16_t pcid);
```

---

**FUNCTION: tlb_flush_pcid_addr** (line 29)

```c
void tlb_flush_pcid_addr(uint16_t pcid, uint64_t addr);
```

---

**FUNCTION: tlb_wbinvd** (line 31)

```c
void tlb_wbinvd(void);
```

---

**FUNCTION: tlb_clflush** (line 32)

```c
void tlb_clflush(uint64_t addr);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/arch/x86_64/tlb_asm.h

**FUNCTION: tlb_asm_invlpg** (line 6)

```c
void tlb_asm_invlpg(uint64_t addr);
```

---

**FUNCTION: tlb_asm_write_cr3** (line 9)

```c
void tlb_asm_write_cr3(uint64_t value);
```

---

**FUNCTION: tlb_asm_write_cr4** (line 12)

```c
void tlb_asm_write_cr4(uint64_t value);
```

---

**FUNCTION: tlb_asm_invpcid** (line 14)

```c
void tlb_asm_invpcid(uint64_t type, const void *desc);
```

---

**FUNCTION: tlb_asm_wbinvd** (line 16)

```c
void tlb_asm_wbinvd(void);
```

---

**FUNCTION: tlb_asm_clflush** (line 17)

```c
void tlb_asm_clflush(uint64_t addr);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/cache/cache.c

**FUNCTION: cache_register** (line 27)

```c
void cache_register(kcache_t *c)
```

---

**FUNCTION: cache_dump** (line 60)

```c
void cache_dump(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/cache/cache.h

**FUNCTION: cache_register** (line 10)

```c
void cache_register(kcache_t *c);
```

---

**FUNCTION: cache_dump** (line 13)

```c
void cache_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/cache/object_cache.c

**FUNCTION: kcache_destroy** (line 167)

```c
void kcache_destroy(kcache_t *c)
```

---

**FUNCTION: kcache_free** (line 241)

```c
void kcache_free(kcache_t *c, void *ptr)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/cache/object_cashe.h

**FUNCTION: void** (line 12)

```c
typedef void (*kcache_ctor_t)(void *obj);
```

---

**FUNCTION: kcache_destroy** (line 47)

```c
void kcache_destroy(kcache_t *c);
```

---

**FUNCTION: kcache_free** (line 51)

```c
void kcache_free(kcache_t *c, void *ptr);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/core/mm.c

**FUNCTION: mm_dump** (line 112)

```c
void mm_dump(void)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/core/mm.h

**FUNCTION: mm_dump** (line 32)

```c
void mm_dump(void);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/core/smp_lock.c

**FUNCTION: smp_lock_init** (line 49)

```c
void smp_lock_init(smp_ticket_lock_t *lock)
```

---

**FUNCTION: smp_lock_acquire** (line 58)

```c
void smp_lock_acquire(smp_ticket_lock_t *lock)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/core/smp_lock.h

**FUNCTION: smp_lock_init** (line 21)

```c
void smp_lock_init(smp_ticket_lock_t *lock);
```

---

**FUNCTION: smp_lock_acquire** (line 24)

```c
void smp_lock_acquire(smp_ticket_lock_t *lock);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/ffi.rs

**STRUCT: RawMemEntry** (line 5)

```rust
pub struct RawMemEntry {
```

---

**STRUCT: MmBootParams** (line 14)

```rust
pub struct MmBootParams {
```

---

**FN: mm_init** (line 25)

```rust
pub fn mm_init(params: *const MmBootParams) -> bool;
```

---

**FN: mm_ready** (line 26)

```rust
pub fn mm_ready() -> bool;
```

---

**FN: mm_total_ram** (line 27)

```rust
pub fn mm_total_ram() -> u64;
```

---

**FN: mm_free_ram** (line 28)

```rust
pub fn mm_free_ram() -> u64;
```

---

**FN: mm_dump** (line 29)

```rust
pub fn mm_dump();
```

---

**FN: arch_memory_ready** (line 31)

```rust
pub fn arch_memory_ready() -> bool;
```

---

**FN: arch_memory_reserve_range** (line 32)

```rust
pub fn arch_memory_reserve_range(base: u64, len: u64);
```

---

**FN: arch_memory_boot_alloc** (line 33)

```rust
pub fn arch_memory_boot_alloc(len: u64, align: u64, out: *mut u64) -> bool;
```

---

**FN: paging_init** (line 35)

```rust
pub fn paging_init(boot_phys_offset: u64);
```

---

**FN: paging_enable_nx** (line 36)

```rust
pub fn paging_enable_nx();
```

---

**FN: paging_read_cr3** (line 37)

```rust
pub fn paging_read_cr3() -> u64;
```

---

**FN: paging_is_mapped** (line 38)

```rust
pub fn paging_is_mapped(virt: u64) -> bool;
```

---

**FN: paging_aspace_switch** (line 39)

```rust
pub fn paging_aspace_switch(aspace: *mut c_void);
```

---

**FN: paging_map_page** (line 40)

```rust
pub fn paging_map_page(virt: u64, phys: u64, flags: u64) -> bool;
```

---

**FN: pmm_init** (line 42)

```rust
pub fn pmm_init() -> bool;
```

---

**FN: pmm_alloc_frame** (line 43)

```rust
pub fn pmm_alloc_frame(out: *mut u64) -> bool;
```

---

**FN: pmm_alloc_zero_frame** (line 44)

```rust
pub fn pmm_alloc_zero_frame(out: *mut u64) -> bool;
```

---

**FN: pmm_alloc_frames** (line 45)

```rust
pub fn pmm_alloc_frames(count: usize, out: *mut u64) -> bool;
```

---

**FN: pmm_alloc_frames_aligned** (line 46)

```rust
pub fn pmm_alloc_frames_aligned(count: usize,
```

---

**FN: pmm_free_frame** (line 49)

```rust
pub fn pmm_free_frame(phys: u64) -> bool;
```

---

**FN: pmm_free_frames** (line 50)

```rust
pub fn pmm_free_frames(phys: u64, count: usize) -> bool;
```

---

**FN: vmm_init** (line 53)

```rust
pub fn vmm_init() -> bool;
```

---

**FN: vmm_alloc** (line 54)

```rust
pub fn vmm_alloc(bytes: usize, flags: u32, out: *mut u64) -> bool;
```

---

**FN: vmm_free** (line 55)

```rust
pub fn vmm_free(virt: u64, bytes: usize) -> bool;
```

---

**FN: vmm_map_device** (line 56)

```rust
pub fn vmm_map_device(phys: u64, len: usize, out: *mut u64) -> bool;
```

---

**FN: vmm_unmap_device** (line 57)

```rust
pub fn vmm_unmap_device(virt: u64, len: usize) -> bool;
```

---

**FN: kmalloc** (line 59)

```rust
pub fn kmalloc(size: usize) -> *mut c_void;
```

---

**FN: kzalloc** (line 60)

```rust
pub fn kzalloc(size: usize) -> *mut c_void;
```

---

**FN: kcalloc** (line 61)

```rust
pub fn kcalloc(count: usize, size: usize) -> *mut c_void;
```

---

**FN: krealloc** (line 62)

```rust
pub fn krealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
```

---

**FN: kmalloc_aligned** (line 63)

```rust
pub fn kmalloc_aligned(size: usize, align: usize) -> *mut c_void;
```

---

**FN: kfree** (line 64)

```rust
pub fn kfree(ptr: *mut c_void);
```

---

**FN: kalloc_pages** (line 65)

```rust
pub fn kalloc_pages(pages: usize) -> *mut c_void;
```

---

**FN: kfree_pages** (line 66)

```rust
pub fn kfree_pages(ptr: *mut c_void, pages: usize);
```

---

**FN: kvirt_to_phys** (line 67)

```rust
pub fn kvirt_to_phys(ptr: *mut c_void) -> u64;
```

---

**FN: contig_alloc** (line 69)

```rust
pub fn contig_alloc(bytes: usize,
```

---

**FN: contig_free** (line 73)

```rust
pub fn contig_free(phys: u64, bytes: usize);
```

---

**FN: dma_alloc_coherent** (line 74)

```rust
pub fn dma_alloc_coherent(bytes: usize,
```

---

**FN: dma_free_coherent** (line 78)

```rust
pub fn dma_free_coherent(phys: u64, virt: *mut c_void, bytes: usize);
```

---

**FN: aspace_subsystem_init** (line 81)

```rust
pub fn aspace_subsystem_init() -> bool;
```

---

**FN: aspace_create** (line 82)

```rust
pub fn aspace_create() -> *mut c_void;
```

---

**FN: aspace_destroy** (line 83)

```rust
pub fn aspace_destroy(pa: *mut c_void);
```

---

**FN: aspace_paging_handle** (line 84)

```rust
pub fn aspace_paging_handle(pa: *mut c_void) -> *mut c_void;
```

---

**FN: aspace_map_anon** (line 85)

```rust
pub fn aspace_map_anon(pa: *mut c_void,
```

---

**FN: aspace_unmap** (line 89)

```rust
pub fn aspace_unmap(pa: *mut c_void, addr: u64, len: usize) -> bool;
```

---

**FN: aspace_protect** (line 90)

```rust
pub fn aspace_protect(pa: *mut c_void,
```

---

**FN: aspace_brk** (line 94)

```rust
pub fn aspace_brk(pa: *mut c_void, new_brk: u64) -> u64;
```

---

**FN: mmap** (line 95)

```rust
pub fn mmap(pa: *mut c_void,
```

---

**FN: paging_aspace_map** (line 100)

```rust
pub fn paging_aspace_map(aspace: *mut c_void,
```

---

**FN: munmap** (line 105)

```rust
pub fn munmap(pa: *mut c_void, addr: u64, len: usize) -> bool;
```

---

**FN: paging_aspace_cr3** (line 106)

```rust
pub fn paging_aspace_cr3(aspace: *mut c_void) -> u64;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/init.rs

**STRUCT: Driverspace** (line 9)

```rust
pub struct Driverspace {
```

---

**FN: prepare** (line 24)

```rust
pub fn prepare() -> Result<(), DsError> {
```

---

**FN: self_test** (line 91)

```rust
pub fn self_test() -> Result<(), DsError> {
```

---

**FN: ready** (line 134)

```rust
pub fn ready() -> bool {
```

---

**FN: k2d_view** (line 138)

```rust
pub fn k2d_view() -> Option<RingView> {
```

---

**FN: d2k_view** (line 142)

```rust
pub fn d2k_view() -> Option<RingView> {
```

---

**FN: scratch_view** (line 146)

```rust
pub fn scratch_view() -> Option<*mut u8> {
```

---

**FN: map_into_ds** (line 150)

```rust
pub fn map_into_ds(va: u64, phys: u64, len: usize, prot: space::ProtFlags) -> bool {
```

---

**FN: unmap_from_ds** (line 159)

```rust
pub fn unmap_from_ds(va: u64, len: usize) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/kprintf.c

**FUNCTION: kprintf** (line 84)

```c
void kprintf(const char *fmt, ...)
```

---

**FUNCTION: serial_write_str** (line 184)

```c
void serial_write_str(const char *s)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/mm_bridge.rs

**CONST: ARCH_RAW_MEM_USABLE** (line 3)

```rust
pub const ARCH_RAW_MEM_USABLE: u32 = 1;
```

---

**CONST: ARCH_RAW_MEM_RESERVED** (line 4)

```rust
pub const ARCH_RAW_MEM_RESERVED: u32 = 2;
```

---

**CONST: ARCH_RAW_MEM_ACPI_RECLAIM** (line 5)

```rust
pub const ARCH_RAW_MEM_ACPI_RECLAIM: u32 = 3;
```

---

**CONST: ARCH_RAW_MEM_ACPI_NVS** (line 6)

```rust
pub const ARCH_RAW_MEM_ACPI_NVS: u32 = 4;
```

---

**CONST: ARCH_RAW_MEM_BAD** (line 7)

```rust
pub const ARCH_RAW_MEM_BAD: u32 = 5;
```

---

**CONST: ARCH_RAW_MEM_BOOTLOADER** (line 8)

```rust
pub const ARCH_RAW_MEM_BOOTLOADER: u32 = 0x100;
```

---

**STRUCT: RawMemEntry** (line 14)

```rust
pub struct RawMemEntry {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/mod.rs

**FN: init** (line 20)

```rust
pub fn init(params: &ffi::MmBootParams) -> bool {
```

---

**FN: ready** (line 25)

```rust
pub fn ready() -> bool {
```

---

**FN: dump** (line 30)

```rust
pub fn dump() {
```

---

**FN: init_riscv** (line 35)

```rust
pub fn init_riscv() -> bool {
```

---

**FN: init_from_boot_info** (line 57)

```rust
pub fn init_from_boot_info(boot_info: &'static bootloader::BootInfo) -> bool {
```

---

**FN: self_test** (line 110)

```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/paging/paging.c

**FUNCTION: paging_aspace_destroy** (line 109)

```c
void paging_aspace_destroy(address_space_t *as)
```

---

**FUNCTION: paging_aspace_switch** (line 130)

```c
void paging_aspace_switch(address_space_t *as)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/paging/paging.h

**FUNCTION: paging_aspace_destroy** (line 22)

```c
void paging_aspace_destroy(address_space_t *as);
```

---

**FUNCTION: paging_aspace_switch** (line 23)

```c
void paging_aspace_switch(address_space_t *as);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/paging/tlb.c

**FUNCTION: tlb_batch_begin** (line 4)

```c
void tlb_batch_begin(tlb_batch_t *batch)
```

---

**FUNCTION: tlb_batch_full** (line 10)

```c
void tlb_batch_full(tlb_batch_t *batch)
```

---

**FUNCTION: tlb_batch_add** (line 15)

```c
void tlb_batch_add(tlb_batch_t *batch, uint64_t virt)
```

---

**FUNCTION: tlb_batch_commit** (line 30)

```c
void tlb_batch_commit(tlb_batch_t *batch)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/paging/tlb.h

**FUNCTION: tlb_batch_begin** (line 16)

```c
void tlb_batch_begin(tlb_batch_t *batch);
```

---

**FUNCTION: tlb_batch_add** (line 17)

```c
void tlb_batch_add(tlb_batch_t *batch, uint64_t virt);
```

---

**FUNCTION: tlb_batch_full** (line 18)

```c
void tlb_batch_full(tlb_batch_t *batch);
```

---

**FUNCTION: tlb_batch_commit** (line 19)

```c
void tlb_batch_commit(tlb_batch_t *batch);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/phys.rs

**FN: alloc_frame** (line 3)

```rust
pub fn alloc_frame() -> Option<u64> {
```

---

**FN: alloc_zero_frame** (line 13)

```rust
pub fn alloc_zero_frame() -> Option<u64> {
```

---

**FN: alloc_frames** (line 23)

```rust
pub fn alloc_frames(count: usize) -> Option<u64> {
```

---

**FN: alloc_frames_aligned** (line 33)

```rust
pub fn alloc_frames_aligned(count: usize, align: usize) -> Option<u64> {
```

---

**FN: free_frame** (line 43)

```rust
pub fn free_frame(phys: u64) -> bool {
```

---

**FN: free_frames** (line 47)

```rust
pub fn free_frames(phys: u64, count: usize) -> bool {
```

---

**FN: reserve** (line 51)

```rust
pub fn reserve(base: u64, len: u64) {
```

---

**FN: total_bytes** (line 55)

```rust
pub fn total_bytes() -> u64 {
```

---

**FN: free_bytes** (line 59)

```rust
pub fn free_bytes() -> u64 {
```

---

**CONST: DIRECT_MAP_BASE** (line 63)

```rust
pub const DIRECT_MAP_BASE: u64 = 0xFFFF888000000000;
```

---

**FN: phys_to_virt** (line 65)

```rust
pub fn phys_to_virt(phys: u64) -> *mut u8 {
```

---

**FN: self_test** (line 69)

```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/process/address_space.c

**FUNCTION: aspace_destroy** (line 235)

```c
void aspace_destroy(proc_aspace_t *pa)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/process/address_space.h

**FUNCTION: aspace_destroy** (line 33)

```c
void aspace_destroy(proc_aspace_t *pa);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/protection/permissions.c

**FUNCTION: perm_set_strict_wx** (line 34)

```c
void perm_set_strict_wx(bool on)
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/protection/permissions.h

**FUNCTION: perm_set_strict_wx** (line 11)

```c
void perm_set_strict_wx(bool on);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/riscv.rs

**FN: init** (line 29)

```rust
pub fn init() {
```

---

**FN: reserve** (line 47)

```rust
pub fn reserve(base: u64, len: u64) {
```

---

**FN: alloc_frame** (line 91)

```rust
pub fn alloc_frame() -> Option<u64> {
```

---

**FN: alloc_zero_frame** (line 95)

```rust
pub fn alloc_zero_frame() -> Option<u64> {
```

---

**FN: alloc_frames** (line 103)

```rust
pub fn alloc_frames(count: usize) -> Option<u64> {
```

---

**FN: alloc_frames_aligned** (line 108)

```rust
pub fn alloc_frames_aligned(count: usize, align_frames: usize) -> Option<u64> {
```

---

**FN: free_frame** (line 121)

```rust
pub fn free_frame(pa: u64) -> bool {
```

---

**FN: free_frames** (line 125)

```rust
pub fn free_frames(pa: u64, count: usize) -> bool {
```

---

**FN: total_bytes** (line 152)

```rust
pub fn total_bytes() -> u64 {
```

---

**FN: free_bytes** (line 156)

```rust
pub fn free_bytes() -> u64 {
```

---

**CONST: DIRECT_MAP_BASE** (line 163)

```rust
pub const DIRECT_MAP_BASE: u64 = 0;
```

---

**FN: phys_to_virt** (line 165)

```rust
pub fn phys_to_virt(phys: u64) -> *mut u8 {
```

---

**FN: init** (line 195)

```rust
pub fn init() {
```

---

**FN: pool_bytes** (line 206)

```rust
pub fn pool_bytes() -> u64 {
```

---

**FN: heap_free_bytes** (line 210)

```rust
pub fn heap_free_bytes() -> u64 {
```

---

**FN: kmalloc** (line 334)

```rust
pub fn kmalloc(size: usize) -> Option<*mut u8> {
```

---

**FN: kmalloc_aligned** (line 342)

```rust
pub fn kmalloc_aligned(size: usize, align: usize) -> Option<*mut u8> {
```

---

**FN: kzalloc** (line 350)

```rust
pub fn kzalloc(size: usize) -> Option<*mut u8> {
```

---

**FN: kcalloc** (line 356)

```rust
pub fn kcalloc(count: usize, size: usize) -> Option<*mut u8> {
```

---

**FN: kfree** (line 360)

```rust
pub fn kfree(ptr: *mut u8) {
```

---

**FN: krealloc** (line 378)

```rust
pub fn krealloc(ptr: *mut u8, size: usize) -> Option<*mut u8> {
```

---

**FN: kalloc_pages** (line 395)

```rust
pub fn kalloc_pages(pages: usize) -> Option<*mut u8> {
```

---

**FN: kfree_pages** (line 399)

```rust
pub fn kfree_pages(ptr: *mut u8, _pages: usize) {
```

---

**FN: kvirt_to_phys** (line 403)

```rust
pub fn kvirt_to_phys(ptr: *mut u8) -> u64 {
```

---

**STRUCT: VmmFlags** (line 420)

```rust
pub struct VmmFlags(u32);
```

---

**CONST: NONE** (line 423)

```rust
pub const NONE: VmmFlags = VmmFlags(0);
```

---

**CONST: WRITE** (line 424)

```rust
pub const WRITE: VmmFlags = VmmFlags(1 << 0);
```

---

**CONST: USER** (line 425)

```rust
pub const USER: VmmFlags = VmmFlags(1 << 1);
```

---

**CONST: NX** (line 426)

```rust
pub const NX: VmmFlags = VmmFlags(1 << 2);
```

---

**CONST: DEVICE** (line 427)

```rust
pub const DEVICE: VmmFlags = VmmFlags(1 << 3);
```

---

**CONST: ZERO** (line 428)

```rust
pub const ZERO: VmmFlags = VmmFlags(1 << 4);
```

---

**CONST: fn** (line 430)

```rust
pub const fn bits(self) -> u32 {
```

---

**CONST: WRITE** (line 443)

```rust
pub const WRITE: VmmFlags = VmmFlags::WRITE;
```

---

**CONST: USER** (line 444)

```rust
pub const USER: VmmFlags = VmmFlags::USER;
```

---

**CONST: NX** (line 445)

```rust
pub const NX: VmmFlags = VmmFlags::NX;
```

---

**CONST: DEVICE** (line 446)

```rust
pub const DEVICE: VmmFlags = VmmFlags::DEVICE;
```

---

**CONST: ZERO** (line 447)

```rust
pub const ZERO: VmmFlags = VmmFlags::ZERO;
```

---

**FN: init** (line 470)

```rust
pub fn init() {
```

---

**FN: alloc** (line 504)

```rust
pub fn alloc(bytes: usize, _flags: VmmFlags) -> Option<u64> {
```

---

**FN: free** (line 513)

```rust
pub fn free(va: u64, bytes: usize) -> bool {
```

---

**FN: map_device** (line 525)

```rust
pub fn map_device(phys: u64, len: usize) -> Option<u64> {
```

---

**FN: unmap_device** (line 536)

```rust
pub fn unmap_device(va: u64, len: usize) -> bool {
```

---

**STRUCT: ProtFlags** (line 570)

```rust
pub struct ProtFlags(u32);
```

---

**CONST: NONE** (line 573)

```rust
pub const NONE: ProtFlags = ProtFlags(0);
```

---

**CONST: READ** (line 574)

```rust
pub const READ: ProtFlags = ProtFlags(1 << 0);
```

---

**CONST: WRITE** (line 575)

```rust
pub const WRITE: ProtFlags = ProtFlags(1 << 1);
```

---

**CONST: EXEC** (line 576)

```rust
pub const EXEC: ProtFlags = ProtFlags(1 << 2);
```

---

**CONST: USER** (line 577)

```rust
pub const USER: ProtFlags = ProtFlags(1 << 3);
```

---

**CONST: DEVICE** (line 578)

```rust
pub const DEVICE: ProtFlags = ProtFlags(1 << 4);
```

---

**CONST: fn** (line 580)

```rust
pub const fn bits(self) -> u32 {
```

---

**CONST: PROT_READ** (line 593)

```rust
pub const PROT_READ: ProtFlags = ProtFlags::READ;
```

---

**CONST: PROT_WRITE** (line 594)

```rust
pub const PROT_WRITE: ProtFlags = ProtFlags::WRITE;
```

---

**CONST: PROT_EXEC** (line 595)

```rust
pub const PROT_EXEC: ProtFlags = ProtFlags::EXEC;
```

---

**CONST: PROT_USER** (line 596)

```rust
pub const PROT_USER: ProtFlags = ProtFlags::USER;
```

---

**CONST: PROT_DEVICE** (line 597)

```rust
pub const PROT_DEVICE: ProtFlags = ProtFlags::DEVICE;
```

---

**STRUCT: MapFlags** (line 601)

```rust
pub struct MapFlags(u32);
```

---

**CONST: NONE** (line 604)

```rust
pub const NONE: MapFlags = MapFlags(0);
```

---

**CONST: ANONYMOUS** (line 605)

```rust
pub const ANONYMOUS: MapFlags = MapFlags(1 << 0);
```

---

**CONST: PRIVATE** (line 606)

```rust
pub const PRIVATE: MapFlags = MapFlags(1 << 1);
```

---

**CONST: FIXED** (line 607)

```rust
pub const FIXED: MapFlags = MapFlags(1 << 3);
```

---

**CONST: fn** (line 609)

```rust
pub const fn bits(self) -> u32 {
```

---

**CONST: MAP_ANONYMOUS** (line 622)

```rust
pub const MAP_ANONYMOUS: MapFlags = MapFlags::ANONYMOUS;
```

---

**CONST: MAP_PRIVATE** (line 623)

```rust
pub const MAP_PRIVATE: MapFlags = MapFlags::PRIVATE;
```

---

**CONST: MAP_FIXED** (line 624)

```rust
pub const MAP_FIXED: MapFlags = MapFlags::FIXED;
```

---

**STRUCT: AddressSpace** (line 663)

```rust
pub struct AddressSpace {
```

---

**FN: new** (line 674)

```rust
pub fn new() -> Option<Self> {
```

---

**FN: handle** (line 687)

```rust
pub fn handle(&self) -> *mut c_void {
```

---

**FN: cr3** (line 691)

```rust
pub fn cr3(&self) -> u64 {
```

---

**FN: translate** (line 732)

```rust
pub fn translate(&self, va: u64) -> Option<u64> {
```

---

**FN: map_phys** (line 758)

```rust
pub fn map_phys(&self, virt: u64, phys: u64, len: usize, prot: ProtFlags) -> bool {
```

---

**FN: map_anon** (line 773)

```rust
pub fn map_anon(&self, hint: u64, len: usize, prot: ProtFlags) -> Option<u64> {
```

---

**FN: mmap** (line 807)

```rust
pub fn mmap(
```

---

**FN: munmap** (line 821)

```rust
pub fn munmap(&self, addr: u64, len: usize) -> bool {
```

---

**FN: protect** (line 840)

```rust
pub fn protect(&self, addr: u64, len: usize, prot: ProtFlags) -> bool {
```

---

**FN: brk** (line 878)

```rust
pub fn brk(&self, new_brk: u64) -> u64 {
```

---

**FN: switch** (line 886)

```rust
pub fn switch(&self) {
```

---

**FN: init** (line 931)

```rust
pub fn init() -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/space.rs

**STRUCT: ProtFlags** (line 7)

```rust
pub struct ProtFlags(u32);
```

---

**CONST: NONE** (line 10)

```rust
pub const NONE: ProtFlags = ProtFlags(0);
```

---

**CONST: READ** (line 11)

```rust
pub const READ: ProtFlags = ProtFlags(1 << 0);
```

---

**CONST: WRITE** (line 12)

```rust
pub const WRITE: ProtFlags = ProtFlags(1 << 1);
```

---

**CONST: EXEC** (line 13)

```rust
pub const EXEC: ProtFlags = ProtFlags(1 << 2);
```

---

**CONST: USER** (line 14)

```rust
pub const USER: ProtFlags = ProtFlags(1 << 3);
```

---

**CONST: DEVICE** (line 16)

```rust
pub const DEVICE: ProtFlags = ProtFlags(1 << 4);
```

---

**CONST: fn** (line 18)

```rust
pub const fn bits(self) -> u32 {
```

---

**CONST: PROT_READ** (line 31)

```rust
pub const PROT_READ: ProtFlags = ProtFlags::READ;
```

---

**CONST: PROT_WRITE** (line 32)

```rust
pub const PROT_WRITE: ProtFlags = ProtFlags::WRITE;
```

---

**CONST: PROT_EXEC** (line 33)

```rust
pub const PROT_EXEC: ProtFlags = ProtFlags::EXEC;
```

---

**CONST: PROT_USER** (line 34)

```rust
pub const PROT_USER: ProtFlags = ProtFlags::USER;
```

---

**CONST: PROT_DEVICE** (line 35)

```rust
pub const PROT_DEVICE: ProtFlags = ProtFlags::DEVICE;
```

---

**STRUCT: MapFlags** (line 39)

```rust
pub struct MapFlags(u32);
```

---

**CONST: NONE** (line 42)

```rust
pub const NONE: MapFlags = MapFlags(0);
```

---

**CONST: ANONYMOUS** (line 43)

```rust
pub const ANONYMOUS: MapFlags = MapFlags(1 << 0);
```

---

**CONST: PRIVATE** (line 44)

```rust
pub const PRIVATE: MapFlags = MapFlags(1 << 1);
```

---

**CONST: FIXED** (line 45)

```rust
pub const FIXED: MapFlags = MapFlags(1 << 3);
```

---

**CONST: fn** (line 47)

```rust
pub const fn bits(self) -> u32 {
```

---

**CONST: MAP_ANONYMOUS** (line 60)

```rust
pub const MAP_ANONYMOUS: MapFlags = MapFlags::ANONYMOUS;
```

---

**CONST: MAP_PRIVATE** (line 61)

```rust
pub const MAP_PRIVATE: MapFlags = MapFlags::PRIVATE;
```

---

**CONST: MAP_FIXED** (line 62)

```rust
pub const MAP_FIXED: MapFlags = MapFlags::FIXED;
```

---

**STRUCT: AddressSpace** (line 64)

```rust
pub struct AddressSpace {
```

---

**FN: new** (line 69)

```rust
pub fn new() -> Option<Self> {
```

---

**FN: handle** (line 79)

```rust
pub fn handle(&self) -> *mut c_void {
```

---

**FN: cr3** (line 83)

```rust
pub fn cr3(&self) -> u64 {
```

---

**FN: map_phys** (line 86)

```rust
pub fn map_phys(&self, virt: u64, phys: u64, len: usize, prot: ProtFlags) -> bool {
```

---

**FN: map_anon** (line 90)

```rust
pub fn map_anon(&self, hint: u64, len: usize, prot: ProtFlags) -> Option<u64> {
```

---

**FN: mmap** (line 96)

```rust
pub fn mmap(&self, addr: u64, len: usize, prot: ProtFlags, flags: MapFlags) -> Option<u64> {
```

---

**FN: munmap** (line 102)

```rust
pub fn munmap(&self, addr: u64, len: usize) -> bool {
```

---

**FN: protect** (line 106)

```rust
pub fn protect(&self, addr: u64, len: usize, prot: ProtFlags) -> bool {
```

---

**FN: brk** (line 110)

```rust
pub fn brk(&self, new_brk: u64) -> u64 {
```

---

**FN: switch** (line 114)

```rust
pub fn switch(&self) {
```

---

**FN: self_test** (line 126)

```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/virt.rs

**STRUCT: VmmFlags** (line 6)

```rust
pub struct VmmFlags(u32);
```

---

**CONST: NONE** (line 9)

```rust
pub const NONE: VmmFlags = VmmFlags(0);
```

---

**CONST: WRITE** (line 10)

```rust
pub const WRITE: VmmFlags = VmmFlags(1 << 0);
```

---

**CONST: USER** (line 11)

```rust
pub const USER: VmmFlags = VmmFlags(1 << 1);
```

---

**CONST: NX** (line 12)

```rust
pub const NX: VmmFlags = VmmFlags(1 << 2);
```

---

**CONST: DEVICE** (line 13)

```rust
pub const DEVICE: VmmFlags = VmmFlags(1 << 3);
```

---

**CONST: ZERO** (line 14)

```rust
pub const ZERO: VmmFlags = VmmFlags(1 << 4);
```

---

**CONST: fn** (line 16)

```rust
pub const fn bits(self) -> u32 {
```

---

**CONST: WRITE** (line 29)

```rust
pub const WRITE: VmmFlags = VmmFlags::WRITE;
```

---

**CONST: USER** (line 30)

```rust
pub const USER: VmmFlags = VmmFlags::USER;
```

---

**CONST: NX** (line 31)

```rust
pub const NX: VmmFlags = VmmFlags::NX;
```

---

**CONST: DEVICE** (line 32)

```rust
pub const DEVICE: VmmFlags = VmmFlags::DEVICE;
```

---

**CONST: ZERO** (line 33)

```rust
pub const ZERO: VmmFlags = VmmFlags::ZERO;
```

---

**FN: alloc** (line 35)

```rust
pub fn alloc(bytes: usize, flags: VmmFlags) -> Option<u64> {
```

---

**FN: free** (line 45)

```rust
pub fn free(virt: u64, bytes: usize) -> bool {
```

---

**FN: map_device** (line 49)

```rust
pub fn map_device(phys: u64, len: usize) -> Option<u64> {
```

---

**FN: unmap_device** (line 59)

```rust
pub fn unmap_device(virt: u64, len: usize) -> bool {
```

---

**FN: self_test** (line 63)

```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/arp.rs

**CONST: PACKET_LEN** (line 6)

```rust
pub const PACKET_LEN: usize = 28;
```

---

**CONST: HARDWARE_ETHERNET** (line 7)

```rust
pub const HARDWARE_ETHERNET: u16 = 1;
```

---

**CONST: PROTOCOL_IPV4** (line 8)

```rust
pub const PROTOCOL_IPV4: u16 = 0x0800;
```

---

**CONST: OPERATION_REQUEST** (line 9)

```rust
pub const OPERATION_REQUEST: u16 = 1;
```

---

**CONST: OPERATION_REPLY** (line 10)

```rust
pub const OPERATION_REPLY: u16 = 2;
```

---

**STRUCT: ArpPacket** (line 13)

```rust
pub struct ArpPacket {
```

---

**FN: parse** (line 22)

```rust
pub fn parse(bytes: &[u8]) -> Result<Self, PacketError> {
```

---

**FN: write_to** (line 62)

```rust
pub fn write_to(&self, out: &mut [u8]) -> Result<usize, PacketError> {
```

---

**CONST: fn** (line 79)

```rust
pub const fn request(
```

---

**CONST: fn** (line 94)

```rust
pub const fn reply(local_mac: MacAddress, local_ip: Ipv4Address, request: Self) -> Self {
```

---

**STRUCT: ArpCache** (line 112)

```rust
pub struct ArpCache<const N: usize> {
```

---

**CONST: fn** (line 117)

```rust
pub const fn new() -> Self {
```

---

**FN: lookup** (line 121)

```rust
pub fn lookup(&mut self, ip: Ipv4Address, now_ms: u64) -> Option<MacAddress> {
```

---

**FN: insert** (line 134)

```rust
pub fn insert(&mut self, ip: Ipv4Address, mac: MacAddress, expires_at_ms: u64) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/checksum.rs

**FN: ones_complement_sum** (line 2)

```rust
pub fn ones_complement_sum(bytes: &[u8]) -> u32 {
```

---

**FN: fold** (line 18)

```rust
pub fn fold(sum: u32) -> u16 {
```

---

**FN: checksum** (line 27)

```rust
pub fn checksum(bytes: &[u8]) -> u16 {
```

---

**FN: is_valid** (line 32)

```rust
pub fn is_valid(bytes: &[u8]) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/command.rs

**STRUCT: NetworkCommandRunner** (line 7)

```rust
pub struct NetworkCommandRunner<const ARP_ENTRIES: usize> {
```

---

**CONST: fn** (line 15)

```rust
pub const fn new(config: NetworkConfig, local_mac: MacAddress, identifier: u16) -> Self {
```

---

**FN: start_ping** (line 24)

```rust
pub fn start_ping(
```

---

**FN: poll** (line 49)

```rust
pub fn poll(
```

---

**FN: parse_ipv4** (line 69)

```rust
pub fn parse_ipv4(input: &str) -> Option<Ipv4Address> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/device.rs

**STRUCT: PollResult** (line 4)

```rust
pub struct PollResult {
```

---

**STRUCT: TxFrame** (line 11)

```rust
pub struct TxFrame<'a> {
```

---

**CONST: fn** (line 17)

```rust
pub const fn new(bytes: &'a [u8]) -> Self {
```

---

**STRUCT: RxFrame** (line 23)

```rust
pub struct RxFrame<'a> {
```

---

**TRAIT: NetworkDevice** (line 28)

```rust
pub trait NetworkDevice {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/driver.rs

**STRUCT: DriverRegistry** (line 5)

```rust
pub struct DriverRegistry<'a> {
```

---

**FN: new** (line 11)

```rust
pub fn new() -> Self {
```

---

**FN: register** (line 18)

```rust
pub fn register(&mut self, device: &'a mut dyn NetworkDevice) -> bool {
```

---

**FN: device** (line 27)

```rust
pub fn device(&mut self, index: usize) -> Option<&mut (dyn NetworkDevice + 'a)> {
```

---

**FN: len** (line 31)

```rust
pub fn len(&self) -> usize {
```

---

**FN: is_empty** (line 35)

```rust
pub fn is_empty(&self) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/error.rs

**ENUM: NetworkError** (line 2)

```rust
pub enum NetworkError {
```

---

**ENUM: PacketError** (line 18)

```rust
pub enum PacketError {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/ethernet.rs

**CONST: HEADER_LEN** (line 3)

```rust
pub const HEADER_LEN: usize = 14;
```

---

**CONST: MIN_FRAME_NO_FCS** (line 4)

```rust
pub const MIN_FRAME_NO_FCS: usize = 60;
```

---

**CONST: DEFAULT_MTU** (line 5)

```rust
pub const DEFAULT_MTU: usize = 1500;
```

---

**CONST: ETHERTYPE_IPV4** (line 7)

```rust
pub const ETHERTYPE_IPV4: u16 = 0x0800;
```

---

**CONST: ETHERTYPE_ARP** (line 8)

```rust
pub const ETHERTYPE_ARP: u16 = 0x0806;
```

---

**STRUCT: EthernetHeader** (line 11)

```rust
pub struct EthernetHeader {
```

---

**STRUCT: EthernetFrame** (line 18)

```rust
pub struct EthernetFrame<'a> {
```

---

**FN: parse** (line 24)

```rust
pub fn parse(bytes: &'a [u8]) -> Result<Self, PacketError> {
```

---

**FN: is_for** (line 46)

```rust
pub fn is_for(&self, local: MacAddress) -> bool {
```

---

**FN: build** (line 51)

```rust
pub fn build(
```

---

**FN: pad_to_minimum** (line 66)

```rust
pub fn pad_to_minimum(frame: &mut [u8], logical_len: usize) -> Result<usize, PacketError> {
```

---

**FN: self_test** (line 78)

```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/icmp.rs

**CONST: TYPE_ECHO_REPLY** (line 3)

```rust
pub const TYPE_ECHO_REPLY: u8 = 0;
```

---

**CONST: TYPE_ECHO_REQUEST** (line 4)

```rust
pub const TYPE_ECHO_REQUEST: u8 = 8;
```

---

**CONST: ECHO_HEADER_LEN** (line 5)

```rust
pub const ECHO_HEADER_LEN: usize = 8;
```

---

**STRUCT: IcmpPacket** (line 8)

```rust
pub struct IcmpPacket<'a> {
```

---

**FN: parse** (line 17)

```rust
pub fn parse(bytes: &'a [u8]) -> Result<Self, PacketError> {
```

---

**FN: is_reply_for** (line 37)

```rust
pub fn is_reply_for(&self, identifier: u16, sequence: u16) -> bool {
```

---

**FN: write_echo_request** (line 44)

```rust
pub fn write_echo_request(
```

---

**FN: write_echo_reply** (line 66)

```rust
pub fn write_echo_reply(out: &mut [u8], request: IcmpPacket<'_>) -> Result<usize, PacketError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/ipv4.rs

**CONST: MIN_HEADER_LEN** (line 3)

```rust
pub const MIN_HEADER_LEN: usize = 20;
```

---

**CONST: PROTOCOL_ICMP** (line 4)

```rust
pub const PROTOCOL_ICMP: u8 = 1;
```

---

**CONST: PROTOCOL_TCP** (line 5)

```rust
pub const PROTOCOL_TCP: u8 = 6;
```

---

**CONST: PROTOCOL_UDP** (line 6)

```rust
pub const PROTOCOL_UDP: u8 = 17;
```

---

**STRUCT: Ipv4Packet** (line 9)

```rust
pub struct Ipv4Packet<'a> {
```

---

**FN: parse** (line 19)

```rust
pub fn parse(bytes: &'a [u8]) -> Result<Self, PacketError> {
```

---

**STRUCT: Ipv4Header** (line 64)

```rust
pub struct Ipv4Header {
```

---

**FN: write** (line 73)

```rust
pub fn write<'a>(
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/packet.rs

**STRUCT: IcmpEchoRequest** (line 8)

```rust
pub struct IcmpEchoRequest<'a> {
```

---

**FN: build_icmp_echo** (line 19)

```rust
pub fn build_icmp_echo(
```

---

**FN: self_test** (line 43)

```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/ping.rs

**ENUM: PingResult** (line 9)

```rust
pub enum PingResult {
```

---

**STRUCT: PingClient** (line 16)

```rust
pub struct PingClient<const ARP_ENTRIES: usize> {
```

---

**CONST: fn** (line 26)

```rust
pub const fn new(config: NetworkConfig, local_mac: MacAddress) -> Self {
```

---

**FN: start** (line 37)

```rust
pub fn start(
```

---

**FN: poll** (line 51)

```rust
pub fn poll(
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/protocols.rs

**STRUCT: MacAddress** (line 3)

```rust
pub struct MacAddress(pub [u8; 6]);
```

---

**CONST: BROADCAST** (line 6)

```rust
pub const BROADCAST: Self = MacAddress([0xFF; 6]);
```

---

**CONST: ZERO** (line 7)

```rust
pub const ZERO: Self = MacAddress([0; 6]);
```

---

**FN: is_broadcast** (line 9)

```rust
pub fn is_broadcast(&self) -> bool {
```

---

**ENUM: EtherType** (line 16)

```rust
pub enum EtherType {
```

---

**STRUCT: EthernetHeader** (line 24)

```rust
pub struct EthernetHeader {
```

---

**STRUCT: Ipv4Header** (line 32)

```rust
pub struct Ipv4Header {
```

---

**STRUCT: UdpHeader** (line 47)

```rust
pub struct UdpHeader {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/runtime.rs

**CONST: DEFAULT_CONFIG** (line 15)

```rust
pub const DEFAULT_CONFIG: NetworkConfig = NetworkConfig {
```

---

**FN: init** (line 30)

```rust
pub fn init() -> Result<(), NetworkError> {
```

---

**FN: start_ping** (line 46)

```rust
pub fn start_ping(destination: Ipv4Address, now_ms: u64) -> Result<PingResult, NetworkError> {
```

---

**FN: poll** (line 54)

```rust
pub fn poll(now_ms: u64) -> Result<Option<PingResult>, NetworkError> {
```

---

**FN: is_ready** (line 63)

```rust
pub fn is_ready() -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/stack.rs

**STRUCT: NetworkConfig** (line 11)

```rust
pub struct NetworkConfig {
```

---

**CONST: fn** (line 21)

```rust
pub const fn next_hop(&self, destination: Ipv4Address) -> Ipv4Address {
```

---

**ENUM: StackEvent** (line 31)

```rust
pub enum StackEvent {
```

---

**STRUCT: PingRequest** (line 54)

```rust
pub struct PingRequest<'a> {
```

---

**STRUCT: NetworkStack** (line 62)

```rust
pub struct NetworkStack<const ARP_ENTRIES: usize> {
```

---

**CONST: fn** (line 69)

```rust
pub const fn new(config: NetworkConfig) -> Self {
```

---

**CONST: fn** (line 78)

```rust
pub const fn config(&self) -> NetworkConfig {
```

---

**FN: next_hop_mac** (line 83)

```rust
pub fn next_hop_mac(&mut self, destination: Ipv4Address, now_ms: u64) -> Option<MacAddress> {
```

---

**FN: build_arp_request** (line 87)

```rust
pub fn build_arp_request(
```

---

**FN: build_ping** (line 103)

```rust
pub fn build_ping(
```

---

**FN: process_rx** (line 132)

```rust
pub fn process_rx(
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/types.rs

**STRUCT: MacAddress** (line 5)

```rust
pub struct MacAddress(pub [u8; 6]);
```

---

**CONST: ZERO** (line 8)

```rust
pub const ZERO: Self = Self([0; 6]);
```

---

**CONST: BROADCAST** (line 9)

```rust
pub const BROADCAST: Self = Self([0xff; 6]);
```

---

**CONST: fn** (line 12)

```rust
pub const fn is_broadcast(self) -> bool {
```

---

**CONST: fn** (line 22)

```rust
pub const fn is_zero(self) -> bool {
```

---

**CONST: fn** (line 32)

```rust
pub const fn is_multicast(self) -> bool {
```

---

**STRUCT: Ipv4Address** (line 49)

```rust
pub struct Ipv4Address(pub [u8; 4]);
```

---

**CONST: UNSPECIFIED** (line 52)

```rust
pub const UNSPECIFIED: Self = Self([0, 0, 0, 0]);
```

---

**CONST: LIMITED_BROADCAST** (line 53)

```rust
pub const LIMITED_BROADCAST: Self = Self([255, 255, 255, 255]);
```

---

**CONST: fn** (line 56)

```rust
pub const fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
```

---

**CONST: fn** (line 61)

```rust
pub const fn as_u32_be(self) -> u32 {
```

---

**CONST: fn** (line 66)

```rust
pub const fn is_unspecified(self) -> bool {
```

---

**CONST: fn** (line 71)

```rust
pub const fn is_limited_broadcast(self) -> bool {
```

---

**CONST: fn** (line 76)

```rust
pub const fn is_in_subnet(self, other: Self, netmask: Self) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/descriptor.rs

**CONST: VIRTQ_DESC_F_NEXT** (line 1)

```rust
pub const VIRTQ_DESC_F_NEXT: u16 = 1;
```

---

**CONST: VIRTQ_DESC_F_WRITE** (line 2)

```rust
pub const VIRTQ_DESC_F_WRITE: u16 = 2;
```

---

**CONST: VIRTQ_DESC_F_INDIRECT** (line 3)

```rust
pub const VIRTQ_DESC_F_INDIRECT: u16 = 4;
```

---

**STRUCT: VirtqDescriptor** (line 7)

```rust
pub struct VirtqDescriptor {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/device.rs

**STRUCT: Capabilities** (line 5)

```rust
pub struct Capabilities {
```

---

**TRAIT: NetworkDevice** (line 11)

```rust
pub trait NetworkDevice {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/net.rs

**CONST: VIRTIO_NET_F_MAC** (line 11)

```rust
pub const VIRTIO_NET_F_MAC: u64 = 1 << 5;
```

---

**CONST: VIRTIO_NET_F_STATUS** (line 12)

```rust
pub const VIRTIO_NET_F_STATUS: u64 = 1 << 16;
```

---

**CONST: VIRTIO_NET_F_MRG_RXBUF** (line 13)

```rust
pub const VIRTIO_NET_F_MRG_RXBUF: u64 = 1 << 15;
```

---

**CONST: VIRTIO_NET_HDR_F_NEEDS_CSUM** (line 14)

```rust
pub const VIRTIO_NET_HDR_F_NEEDS_CSUM: u8 = 1;
```

---

**CONST: VIRTIO_NET_HDR_GSO_NONE** (line 15)

```rust
pub const VIRTIO_NET_HDR_GSO_NONE: u8 = 0;
```

---

**STRUCT: VirtioNetHeader** (line 48)

```rust
pub struct VirtioNetHeader {
```

---

**STRUCT: VirtioNetDevice** (line 169)

```rust
pub struct VirtioNetDevice {
```

---

**CONST: fn** (line 182)

```rust
pub const fn new(mmio_base: usize) -> Self {
```

---

**FN: mmio_base** (line 196)

```rust
pub fn mmio_base(&self) -> usize {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/pci_legacy.rs

**STRUCT: VirtioPciLegacyNetDevice** (line 159)

```rust
pub struct VirtioPciLegacyNetDevice {
```

---

**CONST: fn** (line 174)

```rust
pub const fn new(io_base: u16) -> Self {
```

---

**CONST: fn** (line 188)

```rust
pub const fn io_base(&self) -> u16 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/queue.rs

**CONST: VIRTQ_DESC_F_NEXT** (line 3)

```rust
pub const VIRTQ_DESC_F_NEXT: u16 = 1;
```

---

**CONST: VIRTQ_DESC_F_WRITE** (line 4)

```rust
pub const VIRTQ_DESC_F_WRITE: u16 = 2;
```

---

**STRUCT: Descriptor** (line 8)

```rust
pub struct Descriptor {
```

---

**CONST: EMPTY** (line 16)

```rust
pub const EMPTY: Self = Self {
```

---

**STRUCT: DescriptorId** (line 26)

```rust
pub struct DescriptorId(pub u16);
```

---

**STRUCT: DescriptorPool** (line 28)

```rust
pub struct DescriptorPool<const N: usize> {
```

---

**CONST: fn** (line 35)

```rust
pub const fn new() -> Self {
```

---

**CONST: fn** (line 50)

```rust
pub const fn capacity(&self) -> usize {
```

---

**CONST: fn** (line 55)

```rust
pub const fn free_count(&self) -> u16 {
```

---

**FN: allocate** (line 59)

```rust
pub fn allocate(&mut self) -> Result<DescriptorId, NetworkError> {
```

---

**FN: configure** (line 75)

```rust
pub fn configure(
```

---

**FN: descriptor** (line 101)

```rust
pub fn descriptor(&self, id: DescriptorId) -> Result<&Descriptor, NetworkError> {
```

---

**FN: release_chain** (line 107)

```rust
pub fn release_chain(&mut self, head: DescriptorId) -> Result<(), NetworkError> {
```

---

**STRUCT: QueueMemory** (line 141)

```rust
pub struct QueueMemory {
```

---

**FN: self_test** (line 148)

```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/transport.rs

**STRUCT: QueueSetup** (line 4)

```rust
pub struct QueueSetup {
```

---

**TRAIT: VirtioTransport** (line 11)

```rust
pub trait VirtioTransport {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nimcore/banner.nim

**CONST: ART** (line 1)

```nim
const ART = """
```

---

**PROC: nim_banner** (line 13)

```nim
proc nim_banner(buf: ptr uint8, cap: uint32): uint32 {.exportc, cdecl.} =
```

---

**VAR: o** (line 14)

```nim
var o = 0u32
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nimcore/format.nim

**PROC: nim_u64_to_str** (line 1)

```nim
proc nim_u64_to_str(v: uint64, base: uint8, buf: ptr uint8, cap: uint32): uint32
```

---

**VAR: tmp** (line 3)

```nim
var tmp: array[64, uint8]
```

---

**VAR: n** (line 4)

```nim
var n = 0u32
```

---

**VAR: x** (line 5)

```nim
var x = v
```

---

**LET: b** (line 7)

```nim
let b = if base < 2: 10u64 else: uint64(base)
```

---

**LET: d** (line 14)

```nim
let d = x mod b
```

---

**VAR: i** (line 22)

```nim
var i = 0u32
```

---

**PROC: nim_hex_dump** (line 29)

```nim
proc nim_hex_dump(src: ptr uint8, len: uint32, buf: ptr uint8, cap: uint32): uint32
```

---

**CONST: hexd** (line 31)

```nim
const hexd = "0123456789abcdef"
```

---

**VAR: o** (line 32)

```nim
var o = 0u32
```

---

**VAR: i** (line 33)

```nim
var i = 0u32
```

---

**LET: b** (line 36)

```nim
let b = src[i]
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nimcore/parse.nim

**PROC: nim_parse_u64** (line 1)

```nim
proc nim_parse_u64(s: ptr uint8, len: uint32, base: uint8, out_v: ptr uint64): uint8
```

---

**VAR: v** (line 3)

```nim
var v = 0u64
```

---

**VAR: i** (line 4)

```nim
var i = 0u32
```

---

**LET: b** (line 5)

```nim
let b = uint64(base)
```

---

**LET: c** (line 15)

```nim
let c = s[i]
```

---

**LET: d** (line 16)

```nim
let d =
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nimcore/ringbuf.nim

**TYPE: Rb** (line 1)

```nim
type Rb = object
```

---

**VAR: key_rb** (line 6)

```nim
var key_rb: Rb
```

---

**PROC: nim_rb_push** (line 8)

```nim
proc nim_rb_push(b: uint8): uint8 {.exportc, cdecl.} =
```

---

**LET: next** (line 9)

```nim
let next = (key_rb.head + 1) mod 1024
```

---

**PROC: nim_rb_pop** (line 15)

```nim
proc nim_rb_pop(): int32 {.exportc, cdecl.} =
```

---

**LET: b** (line 17)

```nim
let b = key_rb.data[key_rb.tail]
```

---

**PROC: nim_rb_len** (line 21)

```nim
proc nim_rb_len(): uint32 {.exportc, cdecl.} =
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nimcore/shell.nim

**TYPE: Handler** (line 1)

```nim
type Handler = proc(arg: ptr uint8, len: uint32): int32 {.cdecl.}
```

---

**TYPE: Cmd** (line 3)

```nim
type Cmd = object
```

---

**VAR: cmds** (line 9)

```nim
var cmds: array[32, Cmd]
```

---

**PROC: nim_shell_register** (line 11)

```nim
proc nim_shell_register(name: ptr uint8, nlen: uint32, h: Handler): uint8
```

---

**VAR: k** (line 17)

```nim
var k = 0u32
```

---

**PROC: eq_name** (line 28)

```nim
proc eq_name(c: var Cmd, line: ptr uint8, nlen: uint32): bool =
```

---

**VAR: k** (line 30)

```nim
var k = 0u32
```

---

**PROC: nim_shell_run** (line 36)

```nim
proc nim_shell_run(line: ptr uint8, len: uint32): int32
```

---

**VAR: sp** (line 38)

```nim
var sp = 0u32
```

---

**VAR: arg** (line 44)

```nim
var arg: ptr uint8 = nil
```

---

**VAR: alen** (line 45)

```nim
var alen = 0u32
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nimcore.rs

**TYPE: ShellHandler** (line 12)

```rust
pub type ShellHandler = extern "C" fn(*mut u8, u32) -> i32;
```

---

**FN: banner** (line 14)

```rust
pub fn banner(buf: &mut [u8]) -> usize {
```

---

**FN: shell_register** (line 18)

```rust
pub fn shell_register(name: &str, h: ShellHandler) -> bool {
```

---

**FN: shell_run** (line 22)

```rust
pub fn shell_run(line: &str) -> i32 {
```

---

**FN: key_push** (line 26)

```rust
pub fn key_push(c: u8) {
```

---

**FN: key_pop** (line 30)

```rust
pub fn key_pop() -> Option<u8> {
```

---

**FN: parse_u64** (line 35)

```rust
pub fn parse_u64(s: &str, base: u8) -> Option<u64> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/pci.rs

**STRUCT: PciAddress** (line 11)

```rust
pub struct PciAddress {
```

---

**STRUCT: PciDevice** (line 18)

```rust
pub struct PciDevice {
```

---

**FN: config_read_u32** (line 36)

```rust
pub fn config_read_u32(addr: PciAddress, offset: u8) -> u32 {
```

---

**FN: config_write_u32** (line 45)

```rust
pub fn config_write_u32(addr: PciAddress, offset: u8, value: u32) {
```

---

**FN: config_read_u16** (line 54)

```rust
pub fn config_read_u16(addr: PciAddress, offset: u8) -> u16 {
```

---

**FN: config_read_u8** (line 60)

```rust
pub fn config_read_u8(addr: PciAddress, offset: u8) -> u8 {
```

---

**FN: enumerate** (line 88)

```rust
pub fn enumerate() -> Vec<PciDevice> {
```

---

**CONST: RTL8139_VENDOR** (line 114)

```rust
pub const RTL8139_VENDOR: u16 = 0x10EC;
```

---

**CONST: RTL8139_DEVICE** (line 115)

```rust
pub const RTL8139_DEVICE: u16 = 0x8139;
```

---

**CONST: VIRTIO_VENDOR** (line 116)

```rust
pub const VIRTIO_VENDOR: u16 = 0x1AF4;
```

---

**CONST: VIRTIO_NET_LEGACY_DEVICE** (line 117)

```rust
pub const VIRTIO_NET_LEGACY_DEVICE: u16 = 0x1000;
```

---

**CONST: VIRTIO_NET_MODERN_DEVICE** (line 118)

```rust
pub const VIRTIO_NET_MODERN_DEVICE: u16 = 0x1041;
```

---

**ENUM: NicKind** (line 121)

```rust
pub enum NicKind {
```

---

**FN: find_nic** (line 126)

```rust
pub fn find_nic(devices: &[PciDevice]) -> Option<(PciDevice, NicKind)> {
```

---

**FN: bar** (line 142)

```rust
pub fn bar(addr: PciAddress, bar_index: u8) -> u32 {
```

---

**FN: io_base_from_bar** (line 147)

```rust
pub fn io_base_from_bar(bar_value: u32) -> Option<u16> {
```

---

**FN: mem_base_from_bar** (line 155)

```rust
pub fn mem_base_from_bar(bar_value: u32) -> Option<u32> {
```

---

**FN: enable_bus_mastering** (line 163)

```rust
pub fn enable_bus_mastering(addr: PciAddress) {
```

---

**STATIC: PCI_DEVICES** (line 170)

```rust
pub static PCI_DEVICES: Mutex<Vec<PciDevice>> = Mutex::new(Vec::new());
```

---

**FN: init** (line 172)

```rust
pub fn init() {
```

---

**FN: self_test** (line 186)

```rust
pub fn self_test() -> TestResult {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/policy/ada/policy.adb

**PACKAGE: body** (line 1)

```ada
package body Policy
```

---

**FUNCTION: Evaluate** (line 5)

```ada
function Evaluate (Ring  : Ring_Id;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/policy/ada/policy.ads

**PACKAGE: Policy** (line 3)

```ada
package Policy
```

---

**TYPE: Ring_Id** (line 7)

```ada
type Ring_Id is (Ring_Kernel, Ring_Driver, Ring_User)
```

---

**TYPE: Call_Class** (line 10)

```ada
type Call_Class is (Cls_Sys, Cls_Video, Cls_Audio,
```

---

**TYPE: Decision** (line 14)

```ada
type Decision is (Allow, Deny)
```

---

**TYPE: U64** (line 17)

```ada
type U64 is mod 2**64;
```

---

**FUNCTION: Evaluate** (line 21)

```ada
function Evaluate (Ring  : Ring_Id;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/policy/ada/policy_c.adb

**PACKAGE: body** (line 3)

```ada
package body Policy_C is
```

---

**FUNCTION: To_Ring** (line 5)

```ada
function To_Ring (V : Unsigned_8) return Ring_Id is
```

---

**FUNCTION: To_Class** (line 11)

```ada
function To_Class (V : Unsigned_8) return Call_Class is
```

---

**FUNCTION: policy_evaluate** (line 20)

```ada
function policy_evaluate (ring : Unsigned_8;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/policy/ada/policy_c.ads

**PACKAGE: Policy_C** (line 4)

```ada
package Policy_C is
```

---

**FUNCTION: policy_evaluate** (line 6)

```ada
function policy_evaluate (ring : Unsigned_8;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/policy/bridge.rs

**CONST: ALLOW** (line 6)

```rust
pub const ALLOW: u8 = 0;
```

---

**CONST: DENY** (line 7)

```rust
pub const DENY: u8 = 1;
```

---

**FN: check** (line 9)

```rust
pub fn check(ring: u8, cmd: u32, arg: u64) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/policy/mod.rs

**CONST: ALLOW** (line 4)

```rust
pub const ALLOW: u8 = 0;
```

---

**CONST: DENY** (line 5)

```rust
pub const DENY: u8 = 1;
```

---

**CONST: RING_KERNEL** (line 7)

```rust
pub const RING_KERNEL: u8 = 0;
```

---

**CONST: RING_DRIVER** (line 8)

```rust
pub const RING_DRIVER: u8 = 1;
```

---

**CONST: RING_USER** (line 9)

```rust
pub const RING_USER: u8 = 2;
```

---

**CONST: CLS_SYS** (line 11)

```rust
pub const CLS_SYS: u8 = 0;
```

---

**CONST: CLS_VIDEO** (line 12)

```rust
pub const CLS_VIDEO: u8 = 1;
```

---

**CONST: CLS_AUDIO** (line 13)

```rust
pub const CLS_AUDIO: u8 = 2;
```

---

**CONST: CLS_INPUT** (line 14)

```rust
pub const CLS_INPUT: u8 = 3;
```

---

**CONST: CLS_BLOCK** (line 15)

```rust
pub const CLS_BLOCK: u8 = 4;
```

---

**CONST: CLS_NET** (line 16)

```rust
pub const CLS_NET: u8 = 5;
```

---

**CONST: CLS_BT** (line 18)

```rust
pub const CLS_BT: u8 = 6;
```

---

**CONST: BLK_WRITE** (line 20)

```rust
pub const BLK_WRITE: u8 = 3;
```

---

**CONST: fn** (line 22)

```rust
pub const fn cmd(class: u8, op: u8) -> u32 {
```

---

**CONST: fn** (line 26)

```rust
pub const fn class_of(cmd: u32) -> u8 {
```

---

**CONST: fn** (line 30)

```rust
pub const fn op_of(cmd: u32) -> u8 {
```

---

**FN: evaluate** (line 34)

```rust
pub fn evaluate(ring: u8, class: u8, op: u8, _arg: u64) -> u8 {
```

---

**STRUCT: PolicyEntry** (line 59)

```rust
pub struct PolicyEntry {
```

---

**FN: policy_log** (line 82)

```rust
pub fn policy_log(ring: u8, cls: u8, op: u8, dec: u8) {
```

---

**FN: denies** (line 94)

```rust
pub fn denies() -> u64 {
```

---

**FN: total** (line 98)

```rust
pub fn total() -> u64 {
```

---

**FN: entry** (line 103)

```rust
pub fn entry(idx: usize) -> Option<PolicyEntry> {
```

---

**FN: ring_of** (line 112)

```rust
pub fn ring_of(world: u32) -> u8 {
```

---

**FN: required_caps** (line 125)

```rust
pub fn required_caps(class: u8) -> &'static [Capability] {
```

---

**FN: hook** (line 136)

```rust
pub fn hook(world: u32, cap: Capability) -> bool {
```

---

**FN: install** (line 149)

```rust
pub fn install() {
```

---

**FN: decide** (line 153)

```rust
pub fn decide(world: u32, cmd: u32, arg: u64) -> Result<(), &'static str> {
```

---

**FN: decide_current** (line 185)

```rust
pub fn decide_current(cmd: u32, arg: u64) -> Result<(), &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/policy/nim/policynim.nim

**PROC: nim_policy_log** (line 10)

```nim
proc nim_policy_log(ring, cls, op, dec: uint8) {.exportc, cdecl.} =
```

---

**PROC: nim_policy_denies** (line 17)

```nim
proc nim_policy_denies(): uint64 {.exportc, cdecl.} =
```

---

**PROC: nim_policy_get** (line 20)

```nim
proc nim_policy_get(idx: uint32,
```

---

**LET: e** (line 26)

```nim
let e = logbuf[idx]
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/process/fd.rs

**CONST: FD_MAX** (line 3)

```rust
pub const FD_MAX: usize = 16;
```

---

**ENUM: FdKind** (line 9)

```rust
pub enum FdKind {
```

---

**STRUCT: Fd** (line 16)

```rust
pub struct Fd {
```

---

**CONST: NONE** (line 24)

```rust
pub const NONE: Fd = Fd { kind: FdKind::None, pool: -1, offset: 0, len: 0 };
```

---

**CONST: STDIO** (line 25)

```rust
pub const STDIO: Fd = Fd { kind: FdKind::Stdio, pool: -1, offset: 0, len: 0 };
```

---

**FN: open_for** (line 54)

```rust
pub fn open_for(pid: u32, path: &str) -> i32 {
```

---

**FN: read_for** (line 105)

```rust
pub fn read_for(pid: u32, fd: i32, out: &mut [u8]) -> i32 {
```

---

**FN: write_for** (line 157)

```rust
pub fn write_for(pid: u32, fd: i32, data: &[u8]) -> i32 {
```

---

**FN: close_for** (line 187)

```rust
pub fn close_for(pid: u32, fd: i32) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/process/proc.rs

**STRUCT: Process** (line 3)

```rust
pub struct Process {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/serial.rs

**FN: init** (line 9)

```rust
pub fn init() {
```

---

**FN: write_byte** (line 32)

```rust
pub fn write_byte(byte: u8) {
```

---

**FN: init** (line 47)

```rust
pub fn init() {
```

---

**FN: write_byte** (line 54)

```rust
pub fn write_byte(byte: u8) {
```

---

**FN: init** (line 60)

```rust
pub fn init() {
```

---

**FN: write_byte** (line 67)

```rust
pub fn write_byte(byte: u8) {
```

---

**FN: write_str** (line 74)

```rust
pub fn write_str(s: &str) {
```

---

**STRUCT: SerialWriter** (line 80)

```rust
pub struct SerialWriter;
```

---

**FN: print_args** (line 89)

```rust
pub fn print_args(args: fmt::Arguments) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/terminal/mod.rs

**FN: push_scancode** (line 116)

```rust
pub fn push_scancode(code: u8) {
```

---

**FN: pop_keycode** (line 164)

```rust
pub fn pop_keycode() -> Option<u32> {
```

---

**FN: set_keycode_capture** (line 177)

```rust
pub fn set_keycode_capture(on: bool) {
```

---

**FN: init** (line 564)

```rust
pub fn init() {
```

---

**FN: run** (line 569)

```rust
pub fn run() -> ! {
```

---

**FN: self_test** (line 583)

```rust
pub fn self_test() -> crate::testing::TestResult {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/testing.rs

**TYPE: TestResult** (line 4)

```rust
pub type TestResult = Result<&'static str, &'static str>;
```

---

**STRUCT: Test** (line 6)

```rust
pub struct Test {
```

---

**FN: self_test** (line 14)

```rust
pub fn self_test() -> $crate::testing::TestResult {
```

---

**FN: run_test** (line 20)

```rust
pub fn run_test(test: &Test) -> bool {
```

---

**FN: run_all** (line 41)

```rust
pub fn run_all(tests: &[Test]) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/trampoline_rings/arch/aarch64/trampoline_rings.rs

**CONST: HYPER_YIELD** (line 3)

```rust
pub const HYPER_YIELD: u64 = 1;
```

---

**CONST: HYPER_LOG** (line 4)

```rust
pub const HYPER_LOG: u64 = 2;
```

---

**CONST: HYPER_TICK** (line 5)

```rust
pub const HYPER_TICK: u64 = 3;
```

---

**CONST: RING_KERNEL** (line 7)

```rust
pub const RING_KERNEL: u8 = 0;
```

---

**CONST: RING_DRIVER** (line 8)

```rust
pub const RING_DRIVER: u8 = 1;
```

---

**CONST: RING_USER** (line 9)

```rust
pub const RING_USER: u8 = 3;
```

---

**CONST: SPSR_EL0T** (line 11)

```rust
pub const SPSR_EL0T: u64 = 0x0;
```

---

**STRUCT: CpuCtx** (line 15)

```rust
pub struct CpuCtx {
```

---

**FN: tr_restore_ctx** (line 24)

```rust
pub fn tr_restore_ctx(ctx: *mut CpuCtx) -> !;
```

---

**STRUCT: World** (line 27)

```rust
pub struct World {
```

---

**FN: init** (line 40)

```rust
pub fn init() {
```

---

**FN: add_world** (line 46)

```rust
pub fn add_world(ring: u8, ttbr0: u64, entry: u64,
```

---

**FN: start** (line 90)

```rust
pub fn start() -> ! {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/trampoline_rings/arch/risc-v/trampoline_rings.rs

**CONST: HYPER_YIELD** (line 3)

```rust
pub const HYPER_YIELD: u64 = 1;
```

---

**CONST: HYPER_LOG** (line 4)

```rust
pub const HYPER_LOG: u64 = 2;
```

---

**CONST: HYPER_TICK** (line 5)

```rust
pub const HYPER_TICK: u64 = 3;
```

---

**CONST: RING_KERNEL** (line 7)

```rust
pub const RING_KERNEL: u8 = 0;
```

---

**CONST: RING_DRIVER** (line 8)

```rust
pub const RING_DRIVER: u8 = 1;
```

---

**CONST: RING_USER** (line 9)

```rust
pub const RING_USER: u8 = 3;
```

---

**CONST: SSTATUS_WORLD** (line 12)

```rust
pub const SSTATUS_WORLD: u64 = 1 << 5;
```

---

**STRUCT: CpuCtx** (line 16)

```rust
pub struct CpuCtx {
```

---

**FN: tr_restore_ctx** (line 26)

```rust
pub fn tr_restore_ctx(ctx: *mut CpuCtx) -> !;
```

---

**STRUCT: World** (line 29)

```rust
pub struct World {
```

---

**FN: init** (line 44)

```rust
pub fn init() {
```

---

**FN: add_world** (line 51)

```rust
pub fn add_world(ring: u8, satp: u64, entry: u64,
```

---

**FN: start** (line 95)

```rust
pub fn start() -> ! {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/trampoline_rings/arch/x86_64/trampoline_rings.rs

**CONST: SEL_KCODE** (line 3)

```rust
pub const SEL_KCODE: u16 = 0x08;
```

---

**CONST: SEL_KDATA** (line 4)

```rust
pub const SEL_KDATA: u16 = 0x10;
```

---

**CONST: SEL_R1CODE** (line 5)

```rust
pub const SEL_R1CODE: u16 = 0x18 | 1;
```

---

**CONST: SEL_R1DATA** (line 6)

```rust
pub const SEL_R1DATA: u16 = 0x20 | 1;
```

---

**CONST: SEL_R3CODE** (line 7)

```rust
pub const SEL_R3CODE: u16 = 0x28 | 3;
```

---

**CONST: SEL_R3DATA** (line 8)

```rust
pub const SEL_R3DATA: u16 = 0x30 | 3;
```

---

**CONST: HYPER_YIELD** (line 10)

```rust
pub const HYPER_YIELD: u64 = 1;
```

---

**CONST: HYPER_LOG** (line 11)

```rust
pub const HYPER_LOG: u64 = 2;
```

---

**CONST: HYPER_TICK** (line 12)

```rust
pub const HYPER_TICK: u64 = 3;
```

---

**CONST: RING_KERNEL** (line 14)

```rust
pub const RING_KERNEL: u8 = 0;
```

---

**CONST: RING_DRIVER** (line 15)

```rust
pub const RING_DRIVER: u8 = 1;
```

---

**CONST: RING_USER** (line 16)

```rust
pub const RING_USER: u8 = 3;
```

---

**STRUCT: CpuCtx** (line 20)

```rust
pub struct CpuCtx {
```

---

**FN: tr_restore_ctx** (line 30)

```rust
pub fn tr_restore_ctx(ctx: *mut CpuCtx) -> !;
```

---

**STRUCT: World** (line 65)

```rust
pub struct World {
```

---

**FN: init** (line 87)

```rust
pub fn init() {
```

---

**FN: add_world** (line 129)

```rust
pub fn add_world(ring: u8, cr3: u64, entry: u64,
```

---

**FN: start** (line 174)

```rust
pub fn start() -> ! {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/userspace/process/proc.rs

**CONST: MAX_PROCS** (line 1)

```rust
pub const MAX_PROCS: usize = 8;
```

---

**CONST: MAILBOX_LEN** (line 2)

```rust
pub const MAILBOX_LEN: usize = 8;
```

---

**STRUCT: IpcMsg** (line 5)

```rust
pub struct IpcMsg {
```

---

**STRUCT: Process** (line 13)

```rust
pub struct Process {
```

---

**FN: register** (line 29)

```rust
pub fn register(world: usize, parent: u32) -> Option<u32> {
```

---

**FN: by_pid** (line 55)

```rust
pub fn by_pid(pid: u32) -> Option<&'static mut Process> {
```

---

**FN: by_world** (line 69)

```rust
pub fn by_world(world: usize) -> Option<&'static mut Process> {
```

---

**FN: send** (line 83)

```rust
pub fn send(dst: u32, msg: IpcMsg) -> bool {
```

---

**FN: recv** (line 101)

```rust
pub fn recv(pid: u32) -> Option<IpcMsg> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/userspace/process/runcl.rs

**FN: run** (line 27)

```rust
pub fn run(path: &str) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/userspace/process/spawn.rs

**FN: spawn_init** (line 6)

```rust
pub fn spawn_init() -> Result<(), &'static str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/userspace/process/syscall.rs

**CONST: SYS_EXIT** (line 4)

```rust
pub const SYS_EXIT: u64    = 0x1000;
```

---

**CONST: SYS_SPAWN** (line 5)

```rust
pub const SYS_SPAWN: u64   = 0x1001;
```

---

**CONST: SYS_GETPID** (line 6)

```rust
pub const SYS_GETPID: u64  = 0x1002;
```

---

**CONST: SYS_IPC_SEND** (line 7)

```rust
pub const SYS_IPC_SEND: u64 = 0x1010;
```

---

**CONST: SYS_IPC_RECV** (line 8)

```rust
pub const SYS_IPC_RECV: u64 = 0x1011;
```

---

**CONST: SYS_KEY** (line 9)

```rust
pub const SYS_KEY: u64     = 0x1040;
```

---

**CONST: SYS_READDIR** (line 10)

```rust
pub const SYS_READDIR: u64 = 0x1025;
```

---

**CONST: SYS_RUNCL** (line 11)

```rust
pub const SYS_RUNCL: u64   = 0x1050;
```

---

**CONST: SYS_UI_OPEN** (line 12)

```rust
pub const SYS_UI_OPEN: u64 = 0x1060;
```

---

**CONST: UI_FB_VA** (line 14)

```rust
pub const UI_FB_VA: u64   = 0x5000_0000;
```

---

**CONST: UI_FONT_VA** (line 15)

```rust
pub const UI_FONT_VA: u64 = 0x6000_0000;
```

---

**FN: handle** (line 121)

```rust
pub fn handle(world: usize, c: &mut tr::CpuCtx) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/vga_buffer.rs

**ENUM: Color** (line 6)

```rust
pub enum Color {
```

---

**STRUCT: Writer** (line 57)

```rust
pub struct Writer {
```

---

**FN: write_byte** (line 64)

```rust
pub fn write_byte(&mut self, byte: u8) {
```

---

**FN: write_string** (line 104)

```rust
pub fn write_string(&mut self, s: &str) {
```

---

**FN: set_color** (line 113)

```rust
pub fn set_color(&mut self, foreground: Color) {
```

---

**FN: set_column** (line 117)

```rust
pub fn set_column(&mut self, col: usize) {
```

---

**FN: clear_to_end** (line 121)

```rust
pub fn clear_to_end(&mut self) {
```

---

**FN: write_byte_colored** (line 133)

```rust
pub fn write_byte_colored(&mut self, byte: u8, fg: Color) {
```

---

**FN: column** (line 140)

```rust
pub fn column(&self) -> usize {
```

---

**FN: clear_screen** (line 144)

```rust
pub fn clear_screen(&mut self) {
```

---

**STATIC: ref** (line 169)

```rust
pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
```

---

**FN: text_cell** (line 176)

```rust
pub fn text_cell(row: usize, col: usize) -> (u8, u8) {
```

---

**FN: _print** (line 222)

```rust
pub fn _print(args: fmt::Arguments) {
```

---

**FN: _print_colored** (line 229)

```rust
pub fn _print_colored(_color: Color, args: fmt::Arguments) {
```

---

### /home/ctrl/TrangorgeOS/kstd/include/kstd.h

**FUNCTION: tr_log** (line 12)

```c
void tr_log(const char *s);
```

---

### /home/ctrl/TrangorgeOS/kstd/include/kstd_audio.h

**FUNCTION: tr_free** (line 7)

```c
void tr_free(void *ptr, uint32_t bytes);
```

---

### /home/ctrl/TrangorgeOS/kstd/include/kstd_mem.h

**FUNCTION: tr_free** (line 7)

```c
void tr_free(void *ptr, uint32_t bytes);
```

---

### /home/ctrl/TrangorgeOS/kstd/src/kernel/log.c

**FUNCTION: tr_log** (line 5)

```c
void tr_log(const char *s)
```

---

### /home/ctrl/TrangorgeOS/kstd/src/kernel/mem.c

**FUNCTION: tr_free** (line 13)

```c
void tr_free(void *ptr, uint32_t bytes)
```

---

### /home/ctrl/TrangorgeOS/kstd/src/user/log.c

**FUNCTION: tr_log** (line 6)

```c
void tr_log(const char *s)
```

---

### /home/ctrl/TrangorgeOS/kstd/src/user/mem.c

**FUNCTION: tr_free** (line 27)

```c
void tr_free(void *ptr, uint32_t bytes)
```

---

### /home/ctrl/TrangorgeOS/libs/dsabi.h

**FUNCTION: ds_init** (line 124)

```c
void ds_init(uint64_t params_va);
```

---

**FUNCTION: ds_take** (line 127)

```c
int ds_take(uint64_t id, ds_msg_t *out);
```

---

**FUNCTION: ds_poll** (line 128)

```c
void ds_poll(void);
```

---

### /home/ctrl/TrangorgeOS/libs/dsclient.c

**FUNCTION: ds_init** (line 12)

```c
void ds_init(uint64_t params_va)
```

---

**FUNCTION: ds_poll** (line 49)

```c
void ds_poll(void)
```

---

**FUNCTION: ds_take** (line 67)

```c
int ds_take(uint64_t id, ds_msg_t *out)
```

---

### /home/ctrl/TrangorgeOS/trangorgelibc/src/abi/errno.rs

**ENUM: Errno** (line 5)

```rust
pub enum Errno {
```

---

**TYPE: TResult** (line 19)

```rust
pub type TResult<T> = Result<T, Errno>;
```

---

### /home/ctrl/TrangorgeOS/trangorgelibc/src/abi/ktable.rs

**STRUCT: KernelTable** (line 6)

```rust
pub struct KernelTable {
```

---

**FN: validate** (line 25)

```rust
pub fn validate(&self) -> bool {
```

---

**FN: print** (line 30)

```rust
pub fn print(&self, s: &str) {
```

---

### /home/ctrl/TrangorgeOS/trangorgelibc/src/abi/mod.rs

**CONST: ABI_VERSION_MAJOR** (line 7)

```rust
pub const ABI_VERSION_MAJOR: u32 = 0;
```

---

**CONST: ABI_VERSION_MINOR** (line 9)

```rust
pub const ABI_VERSION_MINOR: u32 = 1;
```

---

**CONST: ABI_MAGIC** (line 11)

```rust
pub const ABI_MAGIC: u64 = 0x5452_474F_5247_4500;
```

---

### /home/ctrl/TrangorgeOS/trangorgelibc/src/abi/syscall.rs

**ENUM: Syscall** (line 4)

```rust
pub enum Syscall {
```

---

### /home/ctrl/TrangorgeOS/trangorgelibc/src/abi/types.rs

**STRUCT: SystemInfo** (line 4)

```rust
pub struct SystemInfo {
```

---

**STRUCT: Handle** (line 14)

```rust
pub struct Handle(pub u64);
```

---

**CONST: INVALID** (line 17)

```rust
pub const INVALID: Handle = Handle(u64::MAX);
```

---

**FN: is_valid** (line 19)

```rust
pub fn is_valid(self) -> bool {
```

---

**STRUCT: Stat** (line 26)

```rust
pub struct Stat {
```

---

### /home/ctrl/TrangorgeOS/trangorgelibc/src/lib.rs

**CONST: SYS_YIELD** (line 9)

```rust
pub const SYS_YIELD: u64 = 1;
```

---

**CONST: SYS_LOG** (line 10)

```rust
pub const SYS_LOG: u64 = 2;
```

---

**CONST: SYS_EXIT** (line 11)

```rust
pub const SYS_EXIT: u64 = 0x1000;
```

---

**CONST: SYS_SPAWN** (line 12)

```rust
pub const SYS_SPAWN: u64 = 0x1001;
```

---

**CONST: SYS_GETPID** (line 13)

```rust
pub const SYS_GETPID: u64 = 0x1002;
```

---

**CONST: SYS_IPC_SEND** (line 14)

```rust
pub const SYS_IPC_SEND: u64 = 0x1010;
```

---

**CONST: SYS_IPC_RECV** (line 15)

```rust
pub const SYS_IPC_RECV: u64 = 0x1011;
```

---

**CONST: SYS_OPEN** (line 16)

```rust
pub const SYS_OPEN: u64 = 0x1020;
```

---

**CONST: SYS_READ** (line 17)

```rust
pub const SYS_READ: u64 = 0x1021;
```

---

**CONST: SYS_WRITE** (line 18)

```rust
pub const SYS_WRITE: u64 = 0x1022;
```

---

**CONST: SYS_CLOSE** (line 19)

```rust
pub const SYS_CLOSE: u64 = 0x1023;
```

---

**CONST: SYS_WAIT** (line 20)

```rust
pub const SYS_WAIT: u64 = 0x1024;
```

---

**CONST: SYS_READDIR** (line 21)

```rust
pub const SYS_READDIR: u64 = 0x1025;
```

---

**CONST: SYS_KEY** (line 22)

```rust
pub const SYS_KEY: u64 = 0x1040;
```

---

**CONST: SYS_RUNCL** (line 23)

```rust
pub const SYS_RUNCL: u64 = 0x1050;
```

---

**CONST: SYS_UI_OPEN** (line 24)

```rust
pub const SYS_UI_OPEN: u64 = 0x1060;
```

---

**FN: log** (line 69)

```rust
pub fn log(s: &str) {
```

---

**FN: print** (line 75)

```rust
pub fn print(s: &str) {
```

Wypisuje tekst na konsole uzytkownika (na razie tym samym kanalem co `log`).

---

**FN: yield_cpu** (line 79)

```rust
pub fn yield_cpu() { sc0(SYS_YIELD); }
```

---

**FN: exit** (line 81)

```rust
pub fn exit(code: i32) -> ! {
```

---

**FN: getpid** (line 86)

```rust
pub fn getpid() -> u32 { sc0(SYS_GETPID) as u32 }
```

---

**FN: spawn** (line 88)

```rust
pub fn spawn(path: &str) -> i32 {
```

---

**FN: key** (line 92)

```rust
pub fn key() -> Option<u8> {
```

---

**FN: ipc_send** (line 96)

```rust
pub fn ipc_send(pid: u32, a0: u64, a1: u64) -> bool {
```

---

**STRUCT: Mail** (line 100)

```rust
pub struct Mail { pub from: u32, pub a0: u64, pub a1: u64 }
```

---

**FN: ipc_recv** (line 102)

```rust
pub fn ipc_recv() -> Option<Mail> {
```

---

**FN: open** (line 119)

```rust
pub fn open(path: &str) -> i32 {
```

---

**FN: read** (line 124)

```rust
pub fn read(fd: i32, buf: &mut [u8]) -> i32 {
```

---

**FN: write** (line 128)

```rust
pub fn write(fd: i32, buf: &[u8]) -> i32 {
```

---

**FN: close** (line 132)

```rust
pub fn close(fd: i32) -> i32 {
```

---

**FN: wait** (line 137)

```rust
pub fn wait() -> Option<(u32, i32)> {
```

Czeka na zakonczenie dziecka. Zwraca `(pid, kod_wyjscia)` albo `None`.

---

**FN: readdir** (line 148)

```rust
pub fn readdir(idx: u64, name: &mut [u8]) -> Option<u8> {
```

Wpis katalogu: `Some(typ)` (1 = plik, 2 = katalog), nazwa trafia do `name`.

---

**FN: runcl** (line 154)

```rust
pub fn runcl(path: &str) -> i32 {
```

Uruchamia program w jezyku Trangorge (core-lang) z podanej sciezki.

---

**CONST: UI_FB_VA** (line 164)

```rust
pub const UI_FB_VA: usize = 0x5000_0000;
```

Adres wirtualny framebuffera w przestrzeni uzytkownika (patrz SYS_UI_OPEN).

---

**CONST: UI_FONT_VA** (line 166)

```rust
pub const UI_FONT_VA: usize = 0x6000_0000;
```

Adres wirtualny tablicy czcionki 8x8 w przestrzeni uzytkownika.

---

**FN: ui_open** (line 170)

```rust
pub fn ui_open() -> Option<(u32, u32, u32)> {
```

Mapuje framebuffer i czcionke do przestrzeni uzytkownika.
Zwraca `(szerokosc, wysokosc, stride_w_bajtach)`.

---

**FN: ui_pixel** (line 187)

```rust
pub fn ui_pixel(stride: u32, x: i32, y: i32, color: u32, w: u32, h: u32) {
```

Ustawia jeden piksel (32-bit, format framebuffera).

---

**FN: ui_clear** (line 199)

```rust
pub fn ui_clear(stride: u32, w: u32, h: u32, color: u32) {
```

Wypelnia caly ekran jednym kolorem.

---

**FN: ui_text** (line 215)

```rust
pub fn ui_text(stride: u32, x: i32, y: i32, s: &str, color: u32, w: u32, h: u32) {
```

Rysuje tekst czcionka 8x8 (`font8x8` z jadra: glif `c` ma indeks `c - 32`).

---

**FN: malloc** (line 242)

```rust
pub fn malloc(n: usize) -> *mut u8 {
```

---

**FN: put_u32** (line 253)

```rust
pub fn put_u32(v: u32) {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/out/x86_64/prog.asm

**LABEL: main** (line 7)

```asm
global main
```

---

### /home/ctrl/TrangorgeOS/triang-lang/out/x86_64/prog.c

**FUNCTION: main** (line 6)

```c
int main(void)
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/ast.rs

**STRUCT: Program** (line 2)

```rust
pub struct Program {
```

---

**STRUCT: Function** (line 7)

```rust
pub struct Function {
```

---

**ENUM: Param** (line 15)

```rust
pub enum Param {
```

---

**ENUM: Type** (line 21)

```rust
pub enum Type {
```

---

**STRUCT: Layout** (line 27)

```rust
pub struct Layout {
```

---

**ENUM: Stmt** (line 34)

```rust
pub enum Stmt {
```

---

**STRUCT: OpCall** (line 55)

```rust
pub struct OpCall {
```

---

**ENUM: Target** (line 62)

```rust
pub enum Target {
```

---

**ENUM: Expr** (line 69)

```rust
pub enum Expr {
```

---

**STRUCT: Cond** (line 76)

```rust
pub struct Cond {
```

---

**ENUM: CmpOp** (line 83)

```rust
pub enum CmpOp {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/codegen/asm.rs

**FN: emit** (line 44)

```rust
pub fn emit(ir: &[Ir]) -> String {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/codegen/c.rs

**FN: emit** (line 67)

```rust
pub fn emit(ir: &[Ir]) -> String {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/codegen/mod.rs

**ENUM: Emit** (line 6)

```rust
pub enum Emit {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/codegen/target.rs

**ENUM: Target** (line 2)

```rust
pub enum Target {
```

---

**FN: reg** (line 9)

```rust
pub fn reg(&self, name: &str) -> &'static str {
```

---

**FN: ret_reg** (line 19)

```rust
pub fn ret_reg(&self) -> &'static str {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/ir.rs

**ENUM: Val** (line 5)

```rust
pub enum Val {
```

---

**ENUM: BinOp** (line 11)

```rust
pub enum BinOp {
```

---

**ENUM: Ir** (line 22)

```rust
pub enum Ir {
```

---

**STRUCT: Lower** (line 43)

```rust
pub struct Lower {
```

---

**FN: new** (line 50)

```rust
pub fn new() -> Self {
```

---

**FN: lower_program** (line 63)

```rust
pub fn lower_program(mut self, program: &Program) -> Vec<Ir> {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/iso.rs

**STRUCT: IsoFile** (line 1)

```rust
pub struct IsoFile {
```

---

**FN: build** (line 58)

```rust
pub fn build(volume: &str, files: &[IsoFile]) -> Vec<u8> {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/lexer.rs

**STRUCT: LexError** (line 5)

```rust
pub struct LexError {
```

---

**STRUCT: Lexer** (line 11)

```rust
pub struct Lexer {
```

---

**FN: new** (line 19)

```rust
pub fn new(src: &str) -> Self {
```

---

**FN: tokenize** (line 50)

```rust
pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/parser.rs

**STRUCT: ParseError** (line 5)

```rust
pub struct ParseError {
```

---

**STRUCT: Parser** (line 11)

```rust
pub struct Parser {
```

---

**FN: new** (line 17)

```rust
pub fn new(tokens: Vec<Token>) -> Self {
```

---

**FN: parse_program** (line 71)

```rust
pub fn parse_program(&mut self) -> Result<Program, ParseError> {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/project.rs

**STRUCT: Project** (line 4)

```rust
pub struct Project {
```

---

**FN: load** (line 12)

```rust
pub fn load(path: &str) -> Result<Project, String> {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/sema.rs

**ENUM: Symbol** (line 5)

```rust
pub enum Symbol {
```

---

**STRUCT: SemaError** (line 11)

```rust
pub struct SemaError {
```

---

**STRUCT: Sema** (line 15)

```rust
pub struct Sema {
```

---

**FN: new** (line 21)

```rust
pub fn new() -> Self {
```

---

**FN: check** (line 32)

```rust
pub fn check(&mut self, program: &Program) -> Result<(), SemaError> {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/token.rs

**ENUM: TokenKind** (line 4)

```rust
pub enum TokenKind {
```

---

**STRUCT: Token** (line 41)

```rust
pub struct Token {
```

---

## 2. Internal Functions

### /home/ctrl/TrangorgeOS/c.rs

**STRUCT: BlockHeader** (line 54)

```rust
typedef struct BlockHeader {
```

---

**STRUCT: BlockHeader** (line 57)

```rust
struct BlockHeader* next;           /* Wskaźnik do następnego bloku */
```

---

**STRUCT: BlockHeader** (line 58)

```rust
struct BlockHeader* prev;           /* Wskaźnik do poprzedniego bloku */
```

---

**STATIC: uint8_t** (line 62)

```rust
static uint8_t memory_pool[MEMORY_POOL_SIZE];
```

---

**STATIC: BlockHeader** (line 63)

```rust
static BlockHeader* free_list = NULL;
```

---

**STATIC: int** (line 64)

```rust
static int allocator_initialized = 0;
```

---

**STATIC: BlockHeader** (line 89)

```rust
static BlockHeader* find_free_block(size_t size) {
```

---

**STATIC: void** (line 103)

```rust
static void split_block(BlockHeader* block, size_t size) {
```

---

**STATIC: void** (line 125)

```rust
static void merge_blocks(BlockHeader* block) {
```

---

### /home/ctrl/TrangorgeOS/comgrub/build.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/comgrub/src/main.rs

**FN: kernel_main** (line 11)

```rust
fn kernel_main(magic: u32, info: *const u8) -> !;
```

---

**FN: panic** (line 15)

```rust
fn panic(_info: &PanicInfo) -> ! {
```

---

### /home/ctrl/TrangorgeOS/comlimine/build.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/comlimine/src/main.rs

**FN: main** (line 1)

```rust
fn main() {}
```

---

### /home/ctrl/TrangorgeOS/drivers/amdgpu_driver/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/drivers/audiodriver/build.rs

**FN: main** (line 12)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/drivers/audiodriver/src/lib.rs

**FN: ad_init** (line 7)

```rust
fn ad_init(nam_va: u64, bm_va: u64) -> i32;
```

---

**FN: ad_play** (line 8)

```rust
fn ad_play(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32;
```

---

**FN: ad_capture** (line 9)

```rust
fn ad_capture(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32;
```

---

**FN: ad_stop** (line 10)

```rust
fn ad_stop() -> i32;
```

---

**FN: ad_jack_present** (line 11)

```rust
fn ad_jack_present() -> i32;
```

---

**FN: ad_set_amp** (line 12)

```rust
fn ad_set_amp(on: i32) -> i32;
```

---

**FN: ad_position** (line 13)

```rust
fn ad_position() -> u32;
```

---

### /home/ctrl/TrangorgeOS/drivers/audiodriver/src/main.rs

**FN: main** (line 8)

```rust
fn main() {
```

Punkt wejścia binarki pomocniczej.

Właściwy sterownik działa w driver space przez `ds_entry` (patrz

---

### /home/ctrl/TrangorgeOS/drivers/intelgpu_driver/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/drivers/netcam_driver/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/drivers/wacomgraphic_driver/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/driverspace/src/drivers/storage.rs

**FN: init** (line 20)

```rust
fn init(&mut self, _info: &DeviceInfo) -> Result<(), DsError> {
```

---

### /home/ctrl/TrangorgeOS/driverspace/src/main.rs

**STATIC: mut** (line 8)

```rust
static mut STORAGE: drivers::storage::StorageDrv = drivers::storage::StorageDrv::new();
```

---

**STATIC: mut** (line 9)

```rust
static mut REGISTERED: bool = false;
```

---

**FN: panic** (line 32)

```rust
fn panic(_info: &core::panic::PanicInfo) -> ! {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/crates/ds-manager/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/drivers/amdgpu-driver/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/drivers/audiodriver/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/drivers/intelgpu-driver/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/drivers/netcam_driver/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/drivers/vgpu/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/drivers/wacomgraphic_driver/src/main.rs

**FN: main** (line 1)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-formal-ffi/build.rs

**FN: main** (line 10)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-audio/src/lib.rs

**FN: it_works** (line 10)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-block/src/traits.rs

**FN: read_blocks** (line 18)

```rust
fn read_blocks(&mut self, lba: u64, block_count: u32, buffer: &mut [u8]) -> Result<(), DsError>;
```

Read blocks from the device starting at LBA.

---

**FN: write_blocks** (line 21)

```rust
fn write_blocks(&mut self, lba: u64, block_count: u32, buffer: &[u8]) -> Result<(), DsError>;
```

Write blocks to the device starting at LBA.

---

**FN: flush** (line 24)

```rust
fn flush(&mut self) -> Result<(), DsError>;
```

Flush internal caches to physical media.

---

**FN: geometry** (line 27)

```rust
fn geometry(&self) -> BlockGeometry;
```

Get device geometry (block size, total blocks).

---

**FN: supports_trim** (line 30)

```rust
fn supports_trim(&self) -> bool { false }
```

Check if device supports TRIM/discard.

---

**FN: submit_request** (line 33)

```rust
fn submit_request(&mut self, _req: BlockRequest) -> Result<(), DsError> {
```

Optional: Submit async request (for advanced schedulers).

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-gpu/src/lib.rs

**FN: it_works** (line 10)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-fw-input/src/lib.rs

**FN: it_works** (line 10)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-log/src/formatter.rs

**FN: write_str** (line 30)

```rust
fn write_str(&mut self, s: &str) -> fmt::Result {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-log/src/lib.rs

**STATIC: MANAGER_ENDPOINT_RAW** (line 12)

```rust
static MANAGER_ENDPOINT_RAW: AtomicU32 = AtomicU32::new(0);
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-log/src/transport.rs

**CONST: UART_PORT** (line 9)

```rust
const UART_PORT: u16 = 0x3F8;
```

---

**FN: uart_putchar** (line 12)

```rust
fn uart_putchar(c: u8) {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-mem/src/alloc/buddy.rs

**CONST: MAX_ORDER** (line 3)

```rust
const MAX_ORDER: usize = 10; // Np. do 4MB przy stronie 4KB
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/ds-mem/src/dma/buffer.rs

**CONST: COHERENT** (line 11)

```rust
const COHERENT   = 1 << 0;
```

---

**CONST: HIGH_MEM** (line 12)

```rust
const HIGH_MEM   = 1 << 1;
```

---

**CONST: CONTIGUOUS** (line 13)

```rust
const CONTIGUOUS = 1 << 2;
```

---

**FN: drop** (line 61)

```rust
fn drop(&mut self) {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-abi/build.rs

**FN: main** (line 9)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-abi/src/payloads/dev.rs

**CONST: DMA_CAPABLE** (line 24)

```rust
const DMA_CAPABLE   = 1 << 0;
```

设备支持 DMA (Direct Memory Access)

---

**CONST: BUS_MASTER** (line 26)

```rust
const BUS_MASTER    = 1 << 1;
```

设备支持总线主控 (Bus Mastering)

---

**CONST: HOTPLUGGABLE** (line 28)

```rust
const HOTPLUGGABLE  = 1 << 2;
```

设备是可热插拔的

---

**CONST: LOW_POWER** (line 30)

```rust
const LOW_POWER     = 1 << 3;
```

设备当前处于低功耗状态

---

### /home/ctrl/TrangorgeOS/driverspace_workspace/lib/kapi-syscall/src/lib.rs

**FN: kapi_ipc_send** (line 16)

```rust
fn kapi_ipc_send(target: u32, opcode: u32, payload: *const u8, payload_len: u32) -> i32;
```

Wysyła jednokierunkową wiadomość IPC do `target`.
Zwraca 0 przy sukcesie albo kod `DsError`.

---

**FN: kapi_ipc_call** (line 20)

```rust
fn kapi_ipc_call(
```

Wysyła żądanie IPC i czeka na odpowiedź do bufora `reply`.
Zwraca 0 przy sukcesie albo kod `DsError`.

---

**FN: status** (line 31)

```rust
fn status(rc: i32) -> Result<(), DsError> {
```

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/abi.rs

**FN: from** (line 133)

```rust
fn from(v: i32) -> Self {
```

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/driver.rs

**FN: init** (line 15)

```rust
fn init(&mut self, info: &DeviceInfo) -> Result<(), DsError>;
```

---

### /home/ctrl/TrangorgeOS/driverspacelib/src/runtime.rs

**CONST: RESP_CACHE** (line 15)

```rust
const RESP_CACHE: usize = 16;
```

---

**CONST: EMPTY_MSG** (line 17)

```rust
const EMPTY_MSG: DsMsg = DsMsg {
```

---

**STATIC: mut** (line 28)

```rust
static mut K2D: *mut DsRing = ptr::null_mut();
```

---

**STATIC: mut** (line 29)

```rust
static mut D2K: *mut DsRing = ptr::null_mut();
```

---

**STATIC: mut** (line 30)

```rust
static mut K2D_MSGS: *mut DsMsg = ptr::null_mut();
```

---

**STATIC: mut** (line 31)

```rust
static mut D2K_MSGS: *mut DsMsg = ptr::null_mut();
```

---

**STATIC: mut** (line 32)

```rust
static mut NEXT_ID: u64 = 5000;
```

---

**STATIC: mut** (line 33)

```rust
static mut CACHE_MSG: [DsMsg; RESP_CACHE] = [EMPTY_MSG; RESP_CACHE];
```

---

**STATIC: mut** (line 34)

```rust
static mut CACHE_USED: [bool; RESP_CACHE] = [false; RESP_CACHE];
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/C_base/kc-abi/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/C_base/kc-abi-menager/src/main.rs

**FN: main** (line 2)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/Odin_base/odin-abi/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/Odin_base/odin-abi-bridge/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/Odin_base/odin-bin-loader/src/main.rs

**FN: main** (line 2)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/base/kw-C-abi/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/base/kw-base/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/base/kw-libs/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/base/kw-odin-abi/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/kstd_alloc/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/kstd_base/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/kstd_core/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/kstd_data/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/kstd_io/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/linix_abi/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/linix_abi_com/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/linix_abi_driverspace/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/windows-com/src/lib.rs

**FN: it_works** (line 11)

```rust
fn it_works() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/build.rs

**FN: collect_c_files** (line 6)

```rust
fn collect_c_files(dir: &Path, out: &mut Vec<PathBuf>) {
```

---

**FN: clang_target** (line 23)

```rust
fn clang_target(arch: &str) -> &'static str {
```

---

**FN: main** (line 33)

```rust
fn main() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/arch/riscv64.rs

**CONST: HEAP_SIZE** (line 6)

```rust
const HEAP_SIZE: usize = 2 * 1024 * 1024;
```

---

**STRUCT: HeapSpace** (line 9)

```rust
struct HeapSpace([u8; HEAP_SIZE]);
```

---

**STATIC: HEAP_MEM** (line 11)

```rust
static HEAP_MEM: HeapSpace = HeapSpace([0; HEAP_SIZE]);
```

---

**STRUCT: Bump** (line 13)

```rust
struct Bump {
```

---

**STATIC: BUMP** (line 18)

```rust
static BUMP: Mutex<Bump> = Mutex::new(Bump { next: 0, end: 0 });
```

---

**STATIC: KERNEL_ALLOC** (line 49)

```rust
static KERNEL_ALLOC: KernelHeap = KernelHeap;
```

---

**FN: riscv_panic** (line 60)

```rust
fn riscv_panic(info: &PanicInfo) -> ! {
```

---

**CONST: THR** (line 87)

```rust
const THR: *mut u8 = 0x1000_0000 as *mut u8;
```

---

**CONST: LSR** (line 88)

```rust
const LSR: *const u8 = 0x1000_0005 as *const u8;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/arch/x86_64.rs

**FN: x86_panic** (line 6)

```rust
fn x86_panic(info: &PanicInfo) -> ! {
```

---

**FN: x86_boot** (line 44)

```rust
fn x86_boot(boot_info: &'static BootInfo) -> ! {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/audio/jack.rs

**STATIC: mut** (line 3)

```rust
static mut AMP_ON: bool = false;
```

---

**STATIC: mut** (line 4)

```rust
static mut PRESENT: bool = false;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/battery/bridge.rs

**FN: battery_status_packed** (line 5)

```rust
fn battery_status_packed(a0: *mut u64, a1: *mut u64, a2: *mut u64) -> bool;
```

---

**FN: battery_set_threshold** (line 6)

```rust
fn battery_set_threshold(pct: u32) -> bool;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/battery/init.rs

**FN: battery_init** (line 2)

```rust
fn battery_init() -> bool;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/bluetooth/bridge.rs

**FN: bt_init** (line 7)

```rust
fn bt_init() -> bool;
```

---

**FN: bt_ready** (line 8)

```rust
fn bt_ready() -> bool;
```

---

**FN: bt_info** (line 9)

```rust
fn bt_info(ver: *mut u8, bdaddr: *mut u8);
```

---

**FN: bt_hci_cmd** (line 10)

```rust
fn bt_hci_cmd(opcode: u16, params: *const u8, len: u8) -> bool;
```

---

**FN: bt_event_poll** (line 11)

```rust
fn bt_event_poll(buf: *mut u8, cap: u8, len: *mut u8) -> bool;
```

---

**FN: bt_acl_send** (line 12)

```rust
fn bt_acl_send(data: *const u8, len: u16) -> bool;
```

---

**FN: bt_acl_recv** (line 13)

```rust
fn bt_acl_recv(data: *mut u8, cap: u16, len: *mut u16) -> bool;
```

---

**CONST: HCI_EVENT_MAX** (line 16)

```rust
const HCI_EVENT_MAX: usize = 64;
```

---

**CONST: ACL_MAX** (line 17)

```rust
const ACL_MAX: usize = 256;
```

---

**CONST: HCI_PARAM_MAX** (line 18)

```rust
const HCI_PARAM_MAX: usize = 64;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/bluetooth/init.rs

**FN: bt_init** (line 2)

```rust
fn bt_init() -> bool;
```

---

**FN: bt_ready** (line 3)

```rust
fn bt_ready() -> bool;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/camera/bridge.rs

**FN: camera_caps_get_w** (line 5)

```rust
fn camera_caps_get_w(w: *mut u32, h: *mut u32, fmt: *mut u32, fps: *mut u32) -> bool;
```

---

**FN: camera_start** (line 6)

```rust
fn camera_start() -> bool;
```

---

**FN: camera_stop** (line 7)

```rust
fn camera_stop() -> bool;
```

---

**FN: camera_frame_to_phys** (line 8)

```rust
fn camera_frame_to_phys(phys: u64, cap: u32, fid: *mut u64) -> bool;
```

---

**FN: grant_phys** (line 11)

```rust
fn grant_phys(va: u64) -> Option<u64> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/camera/init.rs

**FN: camera_init** (line 2)

```rust
fn camera_init() -> bool;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/audit.rs

**CONST: AUDIT_CAP** (line 25)

```rust
const AUDIT_CAP: usize = 2048;
```

---

**STRUCT: AuditInner** (line 27)

```rust
struct AuditInner {
```

---

**STATIC: AUDIT** (line 34)

```rust
static AUDIT: Mutex<AuditInner> = Mutex::new(AuditInner {
```

---

**FN: now_tick** (line 52)

```rust
fn now_tick() -> u64 {
```

---

**FN: push** (line 56)

```rust
fn push(kind: EventKind, world: u32, target: u32, cap: Capability) {
```

---

**FN: test_audit** (line 142)

```rust
fn test_audit() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/check.rs

**STATIC: KERNEL_WORLD** (line 6)

```rust
static KERNEL_WORLD: AtomicU32 = AtomicU32::new(1);
```

---

**STATIC: CURRENT_WORLD** (line 8)

```rust
static CURRENT_WORLD: AtomicU32 = AtomicU32::new(1);
```

---

**FN: current_world_id** (line 26)

```rust
fn current_world_id() -> u32 {
```

---

**FN: drop** (line 147)

```rust
fn drop(&mut self) {
```

---

**FN: test_require** (line 158)

```rust
fn test_require() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/export.rs

**FN: cap_from_id** (line 7)

```rust
fn cap_from_id(id: u8) -> Option<Capability> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/grant.rs

**STRUCT: TempGrant** (line 58)

```rust
struct TempGrant {
```

---

**CONST: MAX_TEMP** (line 65)

```rust
const MAX_TEMP: usize = 128;
```

---

**STATIC: TEMP** (line 67)

```rust
static TEMP: Mutex<[TempGrant; MAX_TEMP]> = Mutex::new([TempGrant {
```

---

**FN: now_tick** (line 74)

```rust
fn now_tick() -> u64 {
```

---

**FN: test_inherit** (line 121)

```rust
fn test_inherit() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/hierarchy.rs

**FN: test_hierarchy** (line 132)

```rust
fn test_hierarchy() {
```

---

**FN: test_depth** (line 149)

```rust
fn test_depth() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/policy.rs

**STATIC: HOOK** (line 7)

```rust
static HOOK: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());
```

---

**FN: policy_allows** (line 14)

```rust
fn policy_allows(world: u32, cap: Capability) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/revoke.rs

**FN: test_revoke** (line 68)

```rust
fn test_revoke() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/sets.rs

**FN: test_presets** (line 162)

```rust
fn test_presets() {
```

---

**FN: test_effective** (line 174)

```rust
fn test_effective() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/store.rs

**CONST: MAX_WORLDS** (line 6)

```rust
const MAX_WORLDS: usize = 256;
```

---

**STATIC: STORE** (line 16)

```rust
static STORE: Mutex<StoreInner> = Mutex::new(StoreInner {
```

---

**STRUCT: StoreInner** (line 22)

```rust
struct StoreInner {
```

---

**FN: find_world_inner** (line 71)

```rust
fn find_world_inner(s: &StoreInner, world_id: u32) -> Result<WorldCaps, &'static str> {
```

---

**FN: test_world_registration** (line 192)

```rust
fn test_world_registration() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/syscalls.rs

**FN: cap_from_id** (line 13)

```rust
fn cap_from_id(id: u8) -> Option<Capability> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/tests.rs

**FN: test_full_flow** (line 8)

```rust
fn test_full_flow() {
```

---

**FN: test_hierarchy_enforcement** (line 29)

```rust
fn test_hierarchy_enforcement() {
```

---

**FN: test_global_revocation** (line 40)

```rust
fn test_global_revocation() {
```

---

**FN: test_audit_trail** (line 54)

```rust
fn test_audit_trail() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/caps/types.rs

**FN: fmt** (line 221)

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

---

**FN: fmt** (line 240)

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

---

**FN: test_set_operations** (line 256)

```rust
fn test_set_operations() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/acpi.rs

**CONST: RSDP_SIGNATURE** (line 31)

```rust
const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";
```

---

**CONST: MADT_SIGNATURE** (line 32)

```rust
const MADT_SIGNATURE: &[u8; 4] = b"APIC";
```

---

**CONST: FADT_SIGNATURE** (line 33)

```rust
const FADT_SIGNATURE: &[u8; 4] = b"FACP";
```

---

**FN: checksum** (line 43)

```rust
fn checksum(bytes: &[u8]) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/lapic.rs

**CONST: APIC_BASE_MSR** (line 4)

```rust
const APIC_BASE_MSR: u32 = 0x1B;
```

---

**CONST: REG_ID** (line 6)

```rust
const REG_ID: u32 = 0x020;
```

---

**CONST: REG_VERSION** (line 7)

```rust
const REG_VERSION: u32 = 0x030;
```

---

**CONST: REG_SVR** (line 8)

```rust
const REG_SVR: u32 = 0x0F0;
```

---

**CONST: REG_EOI** (line 9)

```rust
const REG_EOI: u32 = 0x0B0;
```

---

**CONST: REG_ICR0** (line 10)

```rust
const REG_ICR0: u32 = 0x300;
```

---

**CONST: REG_ICR1** (line 11)

```rust
const REG_ICR1: u32 = 0x310;
```

---

**CONST: REG_LVT_LINT0** (line 12)

```rust
const REG_LVT_LINT0: u32 = 0x350;
```

---

**CONST: REG_LVT_LINT1** (line 13)

```rust
const REG_LVT_LINT1: u32 = 0x360;
```

---

**CONST: SPURIOUS_ENABLE** (line 15)

```rust
const SPURIOUS_ENABLE: u32 = 1 << 8;
```

---

**STATIC: mut** (line 17)

```rust
static mut LAPIC_BASE: usize = 0;
```

---

**STATIC: mut** (line 18)

```rust
static mut X2APIC: bool = false;
```

---

**FN: x2apic_msr** (line 20)

```rust
fn x2apic_msr(reg: u32) -> u32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/riscv.rs

**CONST: SBI_EID_SRST** (line 1)

```rust
const SBI_EID_SRST: usize = 0x5352_5354;
```

---

**CONST: SBI_FID_SYSTEM_RESET** (line 2)

```rust
const SBI_FID_SYSTEM_RESET: usize = 0;
```

---

**CONST: SRST_TYPE_SHUTDOWN** (line 4)

```rust
const SRST_TYPE_SHUTDOWN: u32 = 0;
```

---

**CONST: SRST_TYPE_COLD_REBOOT** (line 5)

```rust
const SRST_TYPE_COLD_REBOOT: u32 = 1;
```

---

**FN: system_reset** (line 26)

```rust
fn system_reset(reset_type: u32) -> ! {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/arch/risc_v/context_swich.rs

**CONST: FPU_REG_SIZE** (line 11)

```rust
const FPU_REG_SIZE: usize = 8;
```

---

**CONST: FPU_NUM_REGS** (line 12)

```rust
const FPU_NUM_REGS: usize = 32;
```

---

**CONST: FPU_SAVE_SIZE** (line 13)

```rust
const FPU_SAVE_SIZE: usize = FPU_REG_SIZE * FPU_NUM_REGS;
```

---

**CONST: SATP_MODE_BARE** (line 15)

```rust
const SATP_MODE_BARE: u64 = 0;
```

---

**CONST: SATP_MODE_SV39** (line 16)

```rust
const SATP_MODE_SV39: u64 = 8;
```

---

**CONST: SATP_MODE_SV48** (line 17)

```rust
const SATP_MODE_SV48: u64 = 9;
```

---

**CONST: SATP_MODE_SV57** (line 18)

```rust
const SATP_MODE_SV57: u64 = 10;
```

---

**CONST: SATP_MODE_SHIFT** (line 19)

```rust
const SATP_MODE_SHIFT: u64 = 60;
```

---

**CONST: SATP_ASID_BITS** (line 20)

```rust
const SATP_ASID_BITS: u64 = 16;
```

---

**CONST: SATP_PPN_MASK** (line 21)

```rust
const SATP_PPN_MASK: u64 = (1u64 << 44) - 1;
```

---

**CONST: SSTATUS_FS_OFF** (line 23)

```rust
const SSTATUS_FS_OFF: u64 = 0;
```

---

**CONST: SSTATUS_FS_INITIAL** (line 24)

```rust
const SSTATUS_FS_INITIAL: u64 = 1;
```

---

**CONST: SSTATUS_FS_CLEAN** (line 25)

```rust
const SSTATUS_FS_CLEAN: u64 = 2;
```

---

**CONST: SSTATUS_FS_DIRTY** (line 26)

```rust
const SSTATUS_FS_DIRTY: u64 = 3;
```

---

**CONST: SSTATUS_FS_SHIFT** (line 27)

```rust
const SSTATUS_FS_SHIFT: u64 = 13;
```

---

**CONST: SSTATUS_FS_MASK** (line 28)

```rust
const SSTATUS_FS_MASK: u64 = 0x3 << SSTATUS_FS_SHIFT;
```

---

**CONST: SSTATUS_SD** (line 30)

```rust
const SSTATUS_SD: u64 = 1u64 << 63;
```

---

**CONST: SSTATUS_SPP** (line 31)

```rust
const SSTATUS_SPP: u64 = 1u64 << 8;
```

---

**CONST: SSTATUS_SPIE** (line 32)

```rust
const SSTATUS_SPIE: u64 = 1u64 << 5;
```

---

**CONST: SSTATUS_SIE** (line 33)

```rust
const SSTATUS_SIE: u64 = 1u64 << 1;
```

---

**CONST: CSR_SSTATUS** (line 35)

```rust
const CSR_SSTATUS: u64 = 0x100;
```

---

**CONST: CSR_SIE** (line 36)

```rust
const CSR_SIE: u64 = 0x104;
```

---

**CONST: CSR_STVEC** (line 37)

```rust
const CSR_STVEC: u64 = 0x105;
```

---

**CONST: CSR_SSCRATCH** (line 38)

```rust
const CSR_SSCRATCH: u64 = 0x140;
```

---

**CONST: CSR_SEPC** (line 39)

```rust
const CSR_SEPC: u64 = 0x141;
```

---

**CONST: CSR_SCAUSE** (line 40)

```rust
const CSR_SCAUSE: u64 = 0x142;
```

---

**CONST: CSR_STVAL** (line 41)

```rust
const CSR_STVAL: u64 = 0x143;
```

---

**CONST: CSR_SIP** (line 42)

```rust
const CSR_SIP: u64 = 0x144;
```

---

**CONST: CSR_SATP** (line 43)

```rust
const CSR_SATP: u64 = 0x180;
```

---

**CONST: CSR_FFLAGS** (line 45)

```rust
const CSR_FFLAGS: u64 = 0x001;
```

---

**CONST: CSR_FRM** (line 46)

```rust
const CSR_FRM: u64 = 0x002;
```

---

**CONST: CSR_FCSR** (line 47)

```rust
const CSR_FCSR: u64 = 0x003;
```

---

**CONST: DEBUG_REGISTERS_COUNT** (line 49)

```rust
const DEBUG_REGISTERS_COUNT: usize = 4;
```

---

**CONST: TDATA1_DMODE** (line 50)

```rust
const TDATA1_DMODE: u64 = 1u64 << 27;
```

---

**CONST: TDATA1_TYPE_MCONTROL** (line 51)

```rust
const TDATA1_TYPE_MCONTROL: u64 = 2u64 << 28;
```

---

**CONST: TDATA1_ACTION_EXCEPTION** (line 52)

```rust
const TDATA1_ACTION_EXCEPTION: u64 = 0;
```

---

**CONST: TDATA1_ACTION_DEBUG_MODE** (line 53)

```rust
const TDATA1_ACTION_DEBUG_MODE: u64 = 1;
```

---

**STATIC: SWITCH_COUNT** (line 55)

```rust
static SWITCH_COUNT: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: FPU_CONTEXT_SWITCHES** (line 56)

```rust
static FPU_CONTEXT_SWITCHES: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: LAZY_FPU_ENABLED** (line 57)

```rust
static LAZY_FPU_ENABLED: AtomicBool = AtomicBool::new(true);
```

---

**STATIC: mut** (line 59)

```rust
static mut LAST_FPU_TASK: *mut TaskStruct = core::ptr::null_mut();
```

---

**STATIC: mut** (line 60)

```rust
static mut FPU_AVAILABLE: bool = false;
```

---

**STATIC: mut** (line 61)

```rust
static mut FPU_REGS_SIZE: usize = FPU_SAVE_SIZE;
```

---

**FN: test_fpu_save_area_size** (line 716)

```rust
fn test_fpu_save_area_size() {
```

---

**FN: test_satp_constants** (line 722)

```rust
fn test_satp_constants() {
```

---

**FN: test_sstatus_fs_mask** (line 730)

```rust
fn test_sstatus_fs_mask() {
```

---

**FN: test_satp_mode_name** (line 739)

```rust
fn test_satp_mode_name() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/arch/x86_64/context_swich.rs

**CONST: XSAVE_AREA_SIZE** (line 11)

```rust
const XSAVE_AREA_SIZE: usize = 832;
```

---

**CONST: FXSAVE_AREA_SIZE** (line 12)

```rust
const FXSAVE_AREA_SIZE: usize = 512;
```

---

**CONST: XCR0_SSE** (line 13)

```rust
const XCR0_SSE: u64 = 1 << 1;
```

---

**CONST: XCR0_AVX** (line 14)

```rust
const XCR0_AVX: u64 = 1 << 2;
```

---

**CONST: XCR0_MPX** (line 15)

```rust
const XCR0_MPX: u64 = (1 << 3) | (1 << 4);
```

---

**CONST: XCR0_AVX512** (line 16)

```rust
const XCR0_AVX512: u64 = (1 << 5) | (1 << 6) | (1 << 7);
```

---

**CONST: CR3_PCID_MASK** (line 18)

```rust
const CR3_PCID_MASK: u64 = 0xFFF;
```

---

**CONST: CR3_NO_FLUSH** (line 19)

```rust
const CR3_NO_FLUSH: u64 = 1 << 63;
```

---

**CONST: MSR_FS_BASE** (line 21)

```rust
const MSR_FS_BASE: u32 = 0xC0000100;
```

---

**CONST: MSR_GS_BASE** (line 22)

```rust
const MSR_GS_BASE: u32 = 0xC0000101;
```

---

**CONST: MSR_KERNEL_GS_BASE** (line 23)

```rust
const MSR_KERNEL_GS_BASE: u32 = 0xC0000102;
```

---

**CONST: MSR_STAR** (line 24)

```rust
const MSR_STAR: u32 = 0xC0000081;
```

---

**CONST: MSR_LSTAR** (line 25)

```rust
const MSR_LSTAR: u32 = 0xC0000082;
```

---

**CONST: MSR_CSTAR** (line 26)

```rust
const MSR_CSTAR: u32 = 0xC0000083;
```

---

**CONST: MSR_SFMASK** (line 27)

```rust
const MSR_SFMASK: u32 = 0xC0000084;
```

---

**CONST: MSR_EFER** (line 28)

```rust
const MSR_EFER: u32 = 0xC0000080;
```

---

**CONST: MSR_TSC_AUX** (line 29)

```rust
const MSR_TSC_AUX: u32 = 0xC0000103;
```

---

**CONST: DEBUG_REGISTERS_COUNT** (line 31)

```rust
const DEBUG_REGISTERS_COUNT: usize = 8;
```

---

**STATIC: SWITCH_COUNT** (line 33)

```rust
static SWITCH_COUNT: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: FPU_CONTEXT_SWITCHES** (line 34)

```rust
static FPU_CONTEXT_SWITCHES: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: LAZY_FPU_ENABLED** (line 35)

```rust
static LAZY_FPU_ENABLED: AtomicBool = AtomicBool::new(true);
```

---

**STATIC: mut** (line 37)

```rust
static mut LAST_FPU_TASK: *mut TaskStruct = core::ptr::null_mut();
```

---

**STATIC: mut** (line 38)

```rust
static mut XSAVE_ENABLED: bool = false;
```

---

**STATIC: mut** (line 39)

```rust
static mut AVX_ENABLED: bool = false;
```

---

**STATIC: mut** (line 40)

```rust
static mut AVX512_ENABLED: bool = false;
```

---

**STATIC: mut** (line 41)

```rust
static mut MPX_ENABLED: bool = false;
```

---

**STATIC: mut** (line 42)

```rust
static mut XSAVE_SIZE: usize = FXSAVE_AREA_SIZE;
```

---

**STRUCT: XSaveArea** (line 45)

```rust
struct XSaveArea {
```

---

**CONST: fn** (line 50)

```rust
const fn new() -> Self {
```

---

**STRUCT: ExtendedContext** (line 56)

```rust
struct ExtendedContext {
```

---

**STRUCT: TssStruct** (line 458)

```rust
struct TssStruct {
```

---

**CONST: 0** (line 705)

```rust
const 0,
```

---

**CONST: 0** (line 715)

```rust
const 0,
```

---

**FN: test_xsave_area_alignment** (line 769)

```rust
fn test_xsave_area_alignment() {
```

---

**FN: test_msr_constants** (line 774)

```rust
fn test_msr_constants() {
```

---

**FN: test_cr3_masks** (line 781)

```rust
fn test_cr3_masks() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/arch_hooks.rs

**CONST: NULL_RQ** (line 37)

```rust
const NULL_RQ: AtomicPtr<RunQueue> = AtomicPtr::new(ptr::null_mut());
```

---

**STATIC: RQ_REGISTRY** (line 38)

```rust
static RQ_REGISTRY: [AtomicPtr<RunQueue>; MAX_CPUS] = [NULL_RQ; MAX_CPUS];
```

---

**CONST: NO_APIC_ID** (line 40)

```rust
const NO_APIC_ID: AtomicU32 = AtomicU32::new(u32::MAX);
```

---

**STATIC: APIC_ID_TABLE** (line 41)

```rust
static APIC_ID_TABLE: [AtomicU32; MAX_CPUS] = [NO_APIC_ID; MAX_CPUS];
```

---

**CONST: NO_PENDING** (line 43)

```rust
const NO_PENDING: AtomicBool = AtomicBool::new(false);
```

---

**STATIC: TLB_SHOOTDOWN_PENDING** (line 44)

```rust
static TLB_SHOOTDOWN_PENDING: [AtomicBool; MAX_CPUS] = [NO_PENDING; MAX_CPUS];
```

---

**STATIC: TSC_HZ** (line 46)

```rust
static TSC_HZ: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: BOOTED_CPUS** (line 47)

```rust
static BOOTED_CPUS: AtomicU32 = AtomicU32::new(0);
```

---

**FN: cpu_to_apic_id** (line 69)

```rust
fn cpu_to_apic_id(cpu: u32) -> Option<u32> {
```

---

**FN: ns_from_tsc** (line 175)

```rust
fn ns_from_tsc(tsc_delta: u64, hz: u64) -> u64 {
```

---

**CONST: PIT_GATE_PORT** (line 190)

```rust
const PIT_GATE_PORT: u16 = 0x61;
```

---

**CONST: PIT_CMD_PORT** (line 191)

```rust
const PIT_CMD_PORT: u16 = 0x43;
```

---

**CONST: PIT_CH2_DATA_PORT** (line 192)

```rust
const PIT_CH2_DATA_PORT: u16 = 0x42;
```

---

**FN: timer_count_for_hz** (line 217)

```rust
fn timer_count_for_hz(hz: u64) -> u32 {
```

---

**FN: cmp_max_1** (line 225)

```rust
fn cmp_max_1(v: u32) -> u32 {
```

---

**FN: default_timer_count** (line 233)

```rust
fn default_timer_count() -> u32 {
```

---

**STRUCT: IdtDescriptor** (line 369)

```rust
struct IdtDescriptor {
```

---

**STATIC: mut** (line 374)

```rust
static mut IDT: [IdtEntry; IDT_SIZE] = [IdtEntry::missing(); IDT_SIZE];
```

---

**FN: arch_context_switch** (line 437)

```rust
fn arch_context_switch(prev_ctx: *mut CpuContext, next_ctx: *const CpuContext);
```

---

**FN: maybe_balance** (line 462)

```rust
fn maybe_balance(cpu: u32, registry: &[*mut RunQueue; MAX_CPUS]) {
```

---

**FN: registry_defaults_to_null_for_untouched_cpu** (line 587)

```rust
fn registry_defaults_to_null_for_untouched_cpu() {
```

---

**FN: apic_id_table_defaults_to_none_then_roundtrips** (line 593)

```rust
fn apic_id_table_defaults_to_none_then_roundtrips() {
```

---

**FN: register_runqueue_ignores_out_of_range_cpu** (line 600)

```rust
fn register_runqueue_ignores_out_of_range_cpu() {
```

---

**FN: ns_from_tsc_is_zero_without_calibration** (line 605)

```rust
fn ns_from_tsc_is_zero_without_calibration() {
```

---

**FN: ns_from_tsc_scales_correctly** (line 610)

```rust
fn ns_from_tsc_scales_correctly() {
```

---

**FN: timer_count_for_hz_has_sane_fallback** (line 618)

```rust
fn timer_count_for_hz_has_sane_fallback() {
```

---

**FN: timer_count_for_hz_never_returns_zero** (line 624)

```rust
fn timer_count_for_hz_never_returns_zero() {
```

---

**FN: tlb_pending_flag_sets_and_clears** (line 629)

```rust
fn tlb_pending_flag_sets_and_clears() {
```

---

**FN: idt_entry_encodes_and_reports_present** (line 637)

```rust
fn idt_entry_encodes_and_reports_present() {
```

---

**FN: idt_entry_missing_is_not_present** (line 645)

```rust
fn idt_entry_missing_is_not_present() {
```

---

**FN: cpuid_leaf_zero_reports_a_nonzero_max_leaf** (line 651)

```rust
fn cpuid_leaf_zero_reports_a_nonzero_max_leaf() {
```

---

**FN: rdtsc_does_not_go_backwards_across_two_reads** (line 660)

```rust
fn rdtsc_does_not_go_backwards_across_two_reads() {
```

---

**FN: cpuid_and_rdtsc_are_zero_off_x86_64** (line 670)

```rust
fn cpuid_and_rdtsc_are_zero_off_x86_64() {
```

---

**FN: set_and_read_tsc_frequency_is_stable_within_this_test** (line 678)

```rust
fn set_and_read_tsc_frequency_is_stable_within_this_test() {
```

---

**FN: booted_cpu_count_starts_at_or_above_zero** (line 684)

```rust
fn booted_cpu_count_starts_at_or_above_zero() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/class.rs

**FN: make_idle** (line 514)

```rust
fn make_idle(pid: TaskId) -> TaskStruct {
```

---

**FN: make_task** (line 520)

```rust
fn make_task(pid: TaskId, policy: SchedPolicy, nice: i8) -> TaskStruct {
```

---

**FN: ptr_of** (line 526)

```rust
fn ptr_of(t: &mut TaskStruct) -> *mut TaskStruct {
```

---

**FN: chain_is_ordered_stop_dl_rt_fair_idle** (line 531)

```rust
fn chain_is_ordered_stop_dl_rt_fair_idle() {
```

---

**FN: class_of_maps_every_variant_correctly** (line 545)

```rust
fn class_of_maps_every_variant_correctly() {
```

---

**FN: pick_next_walks_the_chain_in_priority_order** (line 554)

```rust
fn pick_next_walks_the_chain_in_priority_order() {
```

---

**FN: rt_get_rr_interval_is_nonzero_only_for_round_robin** (line 596)

```rust
fn rt_get_rr_interval_is_nonzero_only_for_round_robin() {
```

---

**FN: fair_and_deadline_and_idle_have_zero_rr_interval** (line 606)

```rust
fn fair_and_deadline_and_idle_have_zero_rr_interval() {
```

---

**FN: check_preempt_cross_class_ignores_intra_class_ops** (line 618)

```rust
fn check_preempt_cross_class_ignores_intra_class_ops() {
```

---

**FN: switched_to_reschedules_when_new_class_outranks_current** (line 639)

```rust
fn switched_to_reschedules_when_new_class_outranks_current() {
```

---

**FN: change_task_class_moves_task_between_underlying_queues** (line 664)

```rust
fn change_task_class_moves_task_between_underlying_queues() {
```

---

**FN: change_task_class_on_non_queued_task_does_not_touch_queues** (line 688)

```rust
fn change_task_class_on_non_queued_task_does_not_touch_queues() {
```

---

**FN: rt_task_tick_only_requeues_round_robin_on_slice_expiry** (line 706)

```rust
fn rt_task_tick_only_requeues_round_robin_on_slice_expiry() {
```

---

**FN: rt_task_tick_marks_resched_when_round_robin_slice_expires** (line 727)

```rust
fn rt_task_tick_marks_resched_when_round_robin_slice_expires() {
```

---

**FN: dl_task_tick_dequeues_when_throttled** (line 750)

```rust
fn dl_task_tick_dequeues_when_throttled() {
```

---

**FN: fair_charge_dispatches_to_underlying_rqfair** (line 776)

```rust
fn fair_charge_dispatches_to_underlying_rqfair() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/bitmap.rs

**CONST: fn** (line 12)

```rust
const fn word_of(bit: usize) -> usize {
```

---

**CONST: fn** (line 17)

```rust
const fn mask_of(bit: usize) -> u64 {
```

---

**FN: default** (line 168)

```rust
fn default() -> Self {
```

---

**TYPE: Item** (line 180)

```rust
type Item = usize;
```

---

**FN: next** (line 182)

```rust
fn next(&mut self) -> Option<usize> {
```

---

**CONST: ZERO_WORD** (line 261)

```rust
const ZERO_WORD: AtomicU64 = AtomicU64::new(0);
```

---

**FN: default** (line 340)

```rust
fn default() -> Self {
```

---

**FN: words_for_bits_rounds_up** (line 350)

```rust
fn words_for_bits_rounds_up() {
```

---

**FN: set_clear_test_roundtrip** (line 358)

```rust
fn set_clear_test_roundtrip() {
```

---

**FN: out_of_range_access_is_a_safe_noop** (line 369)

```rust
fn out_of_range_access_is_a_safe_noop() {
```

---

**FN: weight_counts_set_bits_across_words** (line 377)

```rust
fn weight_counts_set_bits_across_words() {
```

---

**FN: find_first_set_crosses_word_boundary** (line 387)

```rust
fn find_first_set_crosses_word_boundary() {
```

---

**FN: find_first_zero_skips_full_words** (line 394)

```rust
fn find_first_zero_skips_full_words() {
```

---

**FN: find_first_zero_none_when_full** (line 401)

```rust
fn find_first_zero_none_when_full() {
```

---

**FN: find_next_set_after_given_bit** (line 408)

```rust
fn find_next_set_after_given_bit() {
```

---

**FN: set_range_sets_contiguous_span** (line 417)

```rust
fn set_range_sets_contiguous_span() {
```

---

**FN: boolean_ops_behave_as_expected** (line 429)

```rust
fn boolean_ops_behave_as_expected() {
```

---

**FN: iter_yields_set_bits_in_order** (line 446)

```rust
fn iter_yields_set_bits_in_order() {
```

---

**FN: slice_bitmap_respects_logical_bit_count_not_word_count** (line 460)

```rust
fn slice_bitmap_respects_logical_bit_count_not_word_count() {
```

---

**FN: slice_bitmap_clear_all_resets_backing_storage** (line 472)

```rust
fn slice_bitmap_clear_all_resets_backing_storage() {
```

---

**FN: atomic_bitmap_test_and_set_reports_previous_state** (line 481)

```rust
fn atomic_bitmap_test_and_set_reports_previous_state() {
```

---

**FN: atomic_bitmap_test_and_clear_reports_previous_state** (line 489)

```rust
fn atomic_bitmap_test_and_clear_reports_previous_state() {
```

---

**FN: atomic_bitmap_find_first_zero_and_set_claims_sequentially** (line 498)

```rust
fn atomic_bitmap_find_first_zero_and_set_claims_sequentially() {
```

---

**FN: atomic_bitmap_find_first_zero_and_set_returns_none_when_full** (line 507)

```rust
fn atomic_bitmap_find_first_zero_and_set_returns_none_when_full() {
```

---

**FN: atomic_bitmap_weight_matches_manual_count** (line 517)

```rust
fn atomic_bitmap_weight_matches_manual_count() {
```

---

**FN: atomic_bitmap_out_of_range_set_is_harmless** (line 526)

```rust
fn atomic_bitmap_out_of_range_set_is_harmless() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/cpumask.rs

**CONST: WORDS** (line 14)

```rust
const WORDS: usize = MAX_CPUS / 64;
```

---

**CONST: INIT** (line 24)

```rust
const INIT: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: POSSIBLE** (line 62)

```rust
static POSSIBLE: AtomicCpuMask = AtomicCpuMask::new();
```

---

**STATIC: PRESENT** (line 63)

```rust
static PRESENT: AtomicCpuMask = AtomicCpuMask::new();
```

---

**STATIC: ONLINE** (line 64)

```rust
static ONLINE: AtomicCpuMask = AtomicCpuMask::new();
```

---

**STATIC: ACTIVE** (line 65)

```rust
static ACTIVE: AtomicCpuMask = AtomicCpuMask::new();
```

---

**CONST: NO_TOPO** (line 67)

```rust
const NO_TOPO: u32 = u32::MAX;
```

---

**STATIC: PACKAGE_ID** (line 68)

```rust
static PACKAGE_ID: [core::sync::atomic::AtomicU32; MAX_CPUS] =
```

---

**STATIC: CORE_ID** (line 70)

```rust
static CORE_ID: [core::sync::atomic::AtomicU32; MAX_CPUS] =
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/plist.rs

**FN: plist_prio** (line 20)

```rust
fn plist_prio(node: *mut TaskStruct) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/rbtree.rs

**CONST: RB_RED** (line 4)

```rust
const RB_RED: usize = 0;
```

---

**CONST: RB_BLACK** (line 5)

```rust
const RB_BLACK: usize = 1;
```

---

**FN: rb_parent** (line 23)

```rust
fn rb_parent(node: *mut TaskStruct) -> *mut TaskStruct {
```

---

**FN: rb_color** (line 29)

```rust
fn rb_color(node: *mut TaskStruct) -> usize {
```

---

**FN: rb_is_red** (line 35)

```rust
fn rb_is_red(node: *mut TaskStruct) -> bool {
```

---

**FN: rb_set_parent_color** (line 40)

```rust
fn rb_set_parent_color(node: *mut TaskStruct, parent: *mut TaskStruct, color: usize) {
```

---

**FN: rb_set_parent** (line 48)

```rust
fn rb_set_parent(node: *mut TaskStruct, parent: *mut TaskStruct) {
```

---

**FN: rb_set_color** (line 54)

```rust
fn rb_set_color(node: *mut TaskStruct, color: usize) {
```

---

**FN: rb_insert_fixup** (line 116)

```rust
fn rb_insert_fixup(&mut self, mut node: *mut TaskStruct) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/rt_array.rs

**FN: bit_is_set** (line 34)

```rust
fn bit_is_set(&self, prio: usize) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/collections/task.rs

**CONST: PID_BITMAP_WORDS** (line 13)

```rust
const PID_BITMAP_WORDS: usize = (MAX_PID as usize + 63) / 64;
```

---

**CONST: ZERO** (line 23)

```rust
const ZERO: AtomicU64 = AtomicU64::new(0);
```

---

**CONST: TGID_SLOTS** (line 92)

```rust
const TGID_SLOTS: usize = MAX_TASKS / 4;
```

---

**CONST: NULL_PTR** (line 108)

```rust
const NULL_PTR: AtomicPtr<TaskStruct> = AtomicPtr::new(ptr::null_mut());
```

---

**FN: hash_pid** (line 116)

```rust
fn hash_pid(pid: TaskId) -> usize { (pid as usize) % MAX_TASKS }
```

---

**FN: hash_tgid** (line 117)

```rust
fn hash_tgid(tgid: TaskId) -> usize { (tgid as usize) % TGID_SLOTS }
```

---

**TYPE: Item** (line 285)

```rust
type Item = *mut TaskStruct;
```

---

**FN: next** (line 287)

```rust
fn next(&mut self) -> Option<Self::Item> {
```

---

**TYPE: Item** (line 317)

```rust
type Item = *mut TaskStruct;
```

---

**FN: next** (line 319)

```rust
fn next(&mut self) -> Option<Self::Item> {
```

---

**TYPE: Item** (line 348)

```rust
type Item = *mut TaskStruct;
```

---

**FN: next** (line 350)

```rust
fn next(&mut self) -> Option<Self::Item> {
```

---

**TYPE: Item** (line 373)

```rust
type Item = *mut TaskStruct;
```

---

**FN: next** (line 375)

```rust
fn next(&mut self) -> Option<Self::Item> {
```

---

**CONST: ZERO** (line 494)

```rust
const ZERO: AtomicU32 = AtomicU32::new(0);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/core.rs

**CONST: PID_BITMAP_WORDS** (line 19)

```rust
const PID_BITMAP_WORDS: usize = words_for_bits(PID_MAX);
```

---

**STATIC: PID_BITMAP** (line 20)

```rust
static PID_BITMAP: AtomicBitmap<PID_BITMAP_WORDS> = AtomicBitmap::new();
```

---

**FN: make_idle** (line 240)

```rust
fn make_idle(pid: TaskId) -> TaskStructT {
```

---

**FN: make_task** (line 246)

```rust
fn make_task(pid: TaskId, policy: SchedPolicy, nice: i8) -> TaskStructT {
```

---

**FN: ptr_of** (line 252)

```rust
fn ptr_of(t: &mut TaskStructT) -> *mut TaskStructT {
```

---

**FN: two_consecutive_allocations_are_distinct** (line 257)

```rust
fn two_consecutive_allocations_are_distinct() {
```

---

**FN: reserve_and_free_pid_roundtrip_on_a_dedicated_high_pid** (line 266)

```rust
fn reserve_and_free_pid_roundtrip_on_a_dedicated_high_pid() {
```

---

**FN: reserve_pid_rejects_out_of_range** (line 278)

```rust
fn reserve_pid_rejects_out_of_range() {
```

---

**FN: boot_bsp_registers_apic_and_runqueue** (line 283)

```rust
fn boot_bsp_registers_apic_and_runqueue() {
```

---

**FN: wake_up_new_task_enqueues_and_marks_runnable** (line 294)

```rust
fn wake_up_new_task_enqueues_and_marks_runnable() {
```

---

**FN: exit_bookkeeping_dequeues_and_marks_zombie** (line 312)

```rust
fn exit_bookkeeping_dequeues_and_marks_zombie() {
```

---

**FN: exit_bookkeeping_on_unqueued_task_returns_null_rq** (line 332)

```rust
fn exit_bookkeeping_on_unqueued_task_returns_null_rq() {
```

---

**FN: reap_zombie_destroys_and_frees_pid** (line 342)

```rust
fn reap_zombie_destroys_and_frees_pid() {
```

---

**FN: maybe_reschedule_is_noop_when_not_needed** (line 358)

```rust
fn maybe_reschedule_is_noop_when_not_needed() {
```

---

**FN: sys_nice_clamps_to_valid_range** (line 369)

```rust
fn sys_nice_clamps_to_valid_range() {
```

---

**FN: sys_nice_rejects_non_fair_policy** (line 380)

```rust
fn sys_nice_rejects_non_fair_policy() {
```

---

**FN: sys_sched_setscheduler_moves_task_to_new_class** (line 388)

```rust
fn sys_sched_setscheduler_moves_task_to_new_class() {
```

---

**FN: sys_sched_rr_get_interval_matches_class_dispatch** (line 407)

```rust
fn sys_sched_rr_get_interval_matches_class_dispatch() {
```

---

**FN: sys_sched_affinity_roundtrips** (line 417)

```rust
fn sys_sched_affinity_roundtrips() {
```

---

**FN: priority_range_helpers_match_posix_expectations** (line 427)

```rust
fn priority_range_helpers_match_posix_expectations() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/entities/runqueue.rs

**CONST: ENQUEUE_RESTORE** (line 20)

```rust
const ENQUEUE_RESTORE   = 1 << 0;
```

---

**CONST: ENQUEUE_WAKEUP** (line 21)

```rust
const ENQUEUE_WAKEUP    = 1 << 1;
```

---

**CONST: ENQUEUE_NEW** (line 22)

```rust
const ENQUEUE_NEW       = 1 << 2;
```

---

**CONST: ENQUEUE_MIGRATED** (line 23)

```rust
const ENQUEUE_MIGRATED  = 1 << 3;
```

---

**CONST: ENQUEUE_HEAD** (line 24)

```rust
const ENQUEUE_HEAD      = 1 << 4;
```

---

**CONST: DEQUEUE_SLEEP** (line 31)

```rust
const DEQUEUE_SLEEP     = 1 << 0;
```

---

**CONST: DEQUEUE_SAVE** (line 32)

```rust
const DEQUEUE_SAVE      = 1 << 1;
```

---

**CONST: DEQUEUE_MIGRATING** (line 33)

```rust
const DEQUEUE_MIGRATING = 1 << 2;
```

---

**FN: insert_fixup** (line 189)

```rust
fn insert_fixup(root: &mut *mut TaskStruct, mut z: *mut TaskStruct) {
```

---

**FN: delete_fixup** (line 293)

```rust
fn delete_fixup(root: &mut *mut TaskStruct, mut x: *mut TaskStruct, mut x_parent: *mut TaskStruct) {
```

---

**FN: check** (line 423)

```rust
fn check(node: *mut TaskStruct, key_of: KeyOf) -> Result<usize, &'static str> {
```

---

**FN: fair_key_of** (line 452)

```rust
fn fair_key_of(node: *const TaskStruct) -> u64 {
```

---

**FN: default** (line 466)

```rust
fn default() -> Self {
```

---

**FN: place_entity** (line 486)

```rust
fn place_entity(&self, task: *mut TaskStruct, flags: EnqueueFlags) {
```

---

**FN: dl_key_of** (line 563)

```rust
fn dl_key_of(node: *const TaskStruct) -> u64 {
```

---

**FN: default** (line 579)

```rust
fn default() -> Self {
```

---

**FN: task_bw** (line 607)

```rust
fn task_bw(task: *const TaskStruct) -> u64 {
```

---

**CONST: RT_BITMAP_WORDS** (line 681)

```rust
const RT_BITMAP_WORDS: usize = (MAX_RT_PRIO as usize + 63) / 64;
```

---

**CONST: INIT** (line 695)

```rust
const INIT: ListHead = ListHead { prev: ptr::null_mut(), next: ptr::null_mut() };
```

---

**FN: set_bit** (line 714)

```rust
fn set_bit(&mut self, prio: usize) {
```

---

**FN: clear_bit** (line 718)

```rust
fn clear_bit(&mut self, prio: usize) {
```

---

**FN: sched_find_first_bit** (line 722)

```rust
fn sched_find_first_bit(&self) -> Option<usize> {
```

---

**FN: now** (line 920)

```rust
fn now(&self) -> u64 {
```

---

**FN: set_current** (line 934)

```rust
fn set_current(&self, task: *mut TaskStruct) {
```

---

**FN: nr_uninterruptible_dec_if** (line 1001)

```rust
fn nr_uninterruptible_dec_if(&self, task: *mut TaskStruct) {
```

---

**FN: fair_max_vruntime** (line 1202)

```rust
fn fair_max_vruntime(&self) -> Option<u64> {
```

---

**FN: fair_root_for_test** (line 1219)

```rust
fn fair_root_for_test(&self) -> *mut TaskStruct {
```

---

**FN: fair_root_internal** (line 1223)

```rust
fn fair_root_internal(&self) -> *mut TaskStruct {
```

---

**FN: fmt** (line 1246)

```rust
fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
```

---

**FN: can_migrate_task** (line 1294)

```rust
fn can_migrate_task(task: *const TaskStruct, dst_cpu: u32) -> bool {
```

---

**FN: make_idle** (line 1489)

```rust
fn make_idle(pid: TaskId, cpu: u32) -> TaskStruct {
```

---

**FN: make_task** (line 1496)

```rust
fn make_task(pid: TaskId, policy: SchedPolicy, nice: i8) -> TaskStruct {
```

---

**FN: ptr_of** (line 1502)

```rust
fn ptr_of(t: &mut TaskStruct) -> *mut TaskStruct {
```

---

**FN: fair_tree_maintains_rb_invariants_under_pseudorandom_inserts** (line 1507)

```rust
fn fair_tree_maintains_rb_invariants_under_pseudorandom_inserts() {
```

---

**CONST: N** (line 1508)

```rust
const N: usize = 40;
```

---

**FN: fair_tree_inorder_traversal_is_sorted_by_vruntime** (line 1537)

```rust
fn fair_tree_inorder_traversal_is_sorted_by_vruntime() {
```

---

**CONST: N** (line 1538)

```rust
const N: usize = 25;
```

---

**FN: fair_tree_survives_random_removals_keeping_invariants** (line 1566)

```rust
fn fair_tree_survives_random_removals_keeping_invariants() {
```

---

**CONST: N** (line 1567)

```rust
const N: usize = 30;
```

---

**FN: deadline_tree_orders_by_earliest_deadline** (line 1588)

```rust
fn deadline_tree_orders_by_earliest_deadline() {
```

---

**FN: rt_queue_fifo_order_at_same_priority_without_cycles** (line 1609)

```rust
fn rt_queue_fifo_order_at_same_priority_without_cycles() {
```

---

**FN: rt_queue_picks_highest_priority_across_levels** (line 1648)

```rust
fn rt_queue_picks_highest_priority_across_levels() {
```

---

**FN: rt_queue_requeue_moves_task_to_back_of_its_level** (line 1667)

```rust
fn rt_queue_requeue_moves_task_to_back_of_its_level() {
```

---

**FN: pick_next_task_respects_stop_over_deadline_over_rt_over_fair_over_idle** (line 1689)

```rust
fn pick_next_task_respects_stop_over_deadline_over_rt_over_fair_over_idle() {
```

---

**FN: pick_next_task_never_returns_null_even_when_fully_empty** (line 1739)

```rust
fn pick_next_task_never_returns_null_even_when_fully_empty() {
```

---

**FN: deadline_task_is_picked_even_before_its_deadline_has_passed** (line 1754)

```rust
fn deadline_task_is_picked_even_before_its_deadline_has_passed() {
```

---

**FN: min_vruntime_never_decreases_across_updates** (line 1783)

```rust
fn min_vruntime_never_decreases_across_updates() {
```

---

**FN: newly_woken_task_does_not_start_before_min_vruntime** (line 1797)

```rust
fn newly_woken_task_does_not_start_before_min_vruntime() {
```

---

**FN: higher_class_always_preempts_lower_class** (line 1817)

```rust
fn higher_class_always_preempts_lower_class() {
```

---

**FN: fair_preemption_requires_minimum_granularity** (line 1842)

```rust
fn fair_preemption_requires_minimum_granularity() {
```

---

**FN: deadline_preempts_via_earliest_deadline_first** (line 1860)

```rust
fn deadline_preempts_via_earliest_deadline_first() {
```

---

**FN: deadline_task_gets_throttled_when_budget_is_exhausted** (line 1874)

```rust
fn deadline_task_gets_throttled_when_budget_is_exhausted() {
```

---

**FN: deadline_task_is_replenished_after_period_rollover** (line 1892)

```rust
fn deadline_task_is_replenished_after_period_rollover() {
```

---

**FN: deadline_admission_control_rejects_overcommitted_bandwidth** (line 1909)

```rust
fn deadline_admission_control_rejects_overcommitted_bandwidth() {
```

---

**FN: activate_and_deactivate_round_trip_updates_bookkeeping** (line 1924)

```rust
fn activate_and_deactivate_round_trip_updates_bookkeeping() {
```

---

**FN: double_enqueue_is_rejected_in_debug_builds** (line 1951)

```rust
fn double_enqueue_is_rejected_in_debug_builds() {
```

---

**FN: select_task_rq_prefers_idle_cpu_with_cache_affinity** (line 1966)

```rust
fn select_task_rq_prefers_idle_cpu_with_cache_affinity() {
```

---

**FN: select_task_rq_picks_least_loaded_when_no_affinity_hit** (line 1985)

```rust
fn select_task_rq_picks_least_loaded_when_no_affinity_hit() {
```

---

**FN: wake_up_process_transitions_state_and_enqueues_remotely** (line 2011)

```rust
fn wake_up_process_transitions_state_and_enqueues_remotely() {
```

---

**FN: wake_up_process_is_idempotent_when_already_queued** (line 2030)

```rust
fn wake_up_process_is_idempotent_when_already_queued() {
```

---

**FN: idle_balance_pulls_one_task_from_busy_cpu** (line 2049)

```rust
fn idle_balance_pulls_one_task_from_busy_cpu() {
```

---

**FN: idle_balance_does_nothing_when_balanced** (line 2081)

```rust
fn idle_balance_does_nothing_when_balanced() {
```

---

**FN: migrated_task_gets_updated_cpu_and_rq_pointer** (line 2105)

```rust
fn migrated_task_gets_updated_cpu_and_rq_pointer() {
```

---

**FN: no_migration_respects_cpus_allowed_mask** (line 2140)

```rust
fn no_migration_respects_cpus_allowed_mask() {
```

---

**FN: yield_task_pushes_vruntime_to_back_of_fair_queue** (line 2169)

```rust
fn yield_task_pushes_vruntime_to_back_of_fair_queue() {
```

---

**FN: schedule_tail_records_voluntary_vs_involuntary_switch** (line 2192)

```rust
fn schedule_tail_records_voluntary_vs_involuntary_switch() {
```

---

**FN: clock_advances_and_update_curr_charges_real_delta** (line 2218)

```rust
fn clock_advances_and_update_curr_charges_real_delta() {
```

---

**FN: update_curr_is_a_noop_on_idle_task** (line 2243)

```rust
fn update_curr_is_a_noop_on_idle_task() {
```

---

**FN: lower_nice_task_accumulates_vruntime_slower** (line 2257)

```rust
fn lower_nice_task_accumulates_vruntime_slower() {
```

---

**FN: load_balance_moves_task_between_two_uneven_queues** (line 2281)

```rust
fn load_balance_moves_task_between_two_uneven_queues() {
```

---

**FN: load_balance_is_noop_below_imbalance_threshold** (line 2310)

```rust
fn load_balance_is_noop_below_imbalance_threshold() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/entities/stats.rs

**STATIC: TOTAL_SWITCHES** (line 9)

```rust
static TOTAL_SWITCHES: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: TOTAL_VOLUNTARY_SWITCHES** (line 10)

```rust
static TOTAL_VOLUNTARY_SWITCHES: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: TOTAL_INVOLUNTARY_SWITCHES** (line 11)

```rust
static TOTAL_INVOLUNTARY_SWITCHES: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: TOTAL_MIGRATIONS** (line 12)

```rust
static TOTAL_MIGRATIONS: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: TOTAL_USER_TIME_NS** (line 13)

```rust
static TOTAL_USER_TIME_NS: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: TOTAL_SYSTEM_TIME_NS** (line 14)

```rust
static TOTAL_SYSTEM_TIME_NS: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: TOTAL_IDLE_TIME_NS** (line 15)

```rust
static TOTAL_IDLE_TIME_NS: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: TOTAL_IOWAIT_TIME_NS** (line 16)

```rust
static TOTAL_IOWAIT_TIME_NS: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: TOTAL_RUNNING_TIME_NS** (line 17)

```rust
static TOTAL_RUNNING_TIME_NS: AtomicU64 = AtomicU64::new(0);
```

---

**FN: default** (line 265)

```rust
fn default() -> Self {
```

---

**FN: fmt** (line 315)

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

---

**FN: fmt** (line 332)

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/entities/task.rs

**CONST: CPUMASK_WORDS** (line 32)

```rust
const CPUMASK_WORDS: usize = MAX_CPUS / 64;
```

---

**CONST: PF_KTHREAD** (line 45)

```rust
const PF_KTHREAD        = 1 << 0;
```

---

**CONST: PF_EXITING** (line 46)

```rust
const PF_EXITING        = 1 << 1;
```

---

**CONST: PF_EXITPIDONE** (line 47)

```rust
const PF_EXITPIDONE     = 1 << 2;
```

---

**CONST: PF_FORKNOEXEC** (line 48)

```rust
const PF_FORKNOEXEC     = 1 << 3;
```

---

**CONST: PF_WQ_WORKER** (line 49)

```rust
const PF_WQ_WORKER      = 1 << 4;
```

---

**CONST: PF_NO_SETAFFINITY** (line 50)

```rust
const PF_NO_SETAFFINITY = 1 << 5;
```

---

**CONST: PF_IDLE** (line 51)

```rust
const PF_IDLE           = 1 << 6;
```

---

**CONST: PF_MEMALLOC** (line 52)

```rust
const PF_MEMALLOC       = 1 << 7;
```

---

**CONST: PF_FROZEN** (line 53)

```rust
const PF_FROZEN         = 1 << 8;
```

---

**CONST: PF_SUPERPRIV** (line 54)

```rust
const PF_SUPERPRIV      = 1 << 9;
```

---

**CONST: PF_DUMPCORE** (line 55)

```rust
const PF_DUMPCORE       = 1 << 10;
```

---

**CONST: PF_SIGNALED** (line 56)

```rust
const PF_SIGNALED       = 1 << 11;
```

---

**CONST: PF_MEMRECLAIM** (line 57)

```rust
const PF_MEMRECLAIM     = 1 << 12;
```

---

**CONST: PF_RANDOMIZE** (line 58)

```rust
const PF_RANDOMIZE      = 1 << 13;
```

---

**CONST: PF_CPU_BOUND** (line 59)

```rust
const PF_CPU_BOUND      = 1 << 14;
```

---

**CONST: PF_VCPU** (line 60)

```rust
const PF_VCPU           = 1 << 15;
```

---

**CONST: PF_IO_WORKER** (line 61)

```rust
const PF_IO_WORKER      = 1 << 16;
```

---

**CONST: PF_NEED_RESCHED** (line 62)

```rust
const PF_NEED_RESCHED   = 1 << 17;
```

---

**CONST: PF_MIGRATING** (line 63)

```rust
const PF_MIGRATING      = 1 << 18;
```

---

**CONST: PF_NO_SLEEP** (line 64)

```rust
const PF_NO_SLEEP       = 1 << 19;
```

---

**CONST: PF_NO_MIGRATE** (line 65)

```rust
const PF_NO_MIGRATE     = 1 << 20;
```

---

**CONST: PF_WAKING** (line 66)

```rust
const PF_WAKING         = 1 << 21;
```

---

**CONST: PF_DL_THROTTLED** (line 67)

```rust
const PF_DL_THROTTLED   = 1 << 22;
```

---

**FN: clone** (line 120)

```rust
fn clone(&self) -> Self {
```

---

**CONST: fn** (line 185)

```rust
const fn from_u8(v: u8) -> TaskState {
```

---

**FN: default** (line 200)

```rust
fn default() -> Self {
```

---

**FN: from** (line 239)

```rust
fn from(policy: SchedPolicy) -> Self {
```

---

**FN: default** (line 358)

```rust
fn default() -> Self {
```

---

**TYPE: Item** (line 369)

```rust
type Item = u32;
```

---

**FN: next** (line 371)

```rust
fn next(&mut self) -> Option<u32> {
```

---

**CONST: NICE_TO_WEIGHT_TABLE** (line 384)

```rust
const NICE_TO_WEIGHT_TABLE: [u64; NICE_WIDTH] = [
```

---

**CONST: NICE_TO_WMULT_TABLE** (line 395)

```rust
const NICE_TO_WMULT_TABLE: [u32; NICE_WIDTH] = [
```

---

**FN: default** (line 463)

```rust
fn default() -> Self {
```

---

**FN: fmt** (line 469)

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

---

**FN: default** (line 493)

```rust
fn default() -> Self {
```

---

**CONST: PELT_DECAY_SHIFT** (line 592)

```rust
const PELT_DECAY_SHIFT: u32 = 10;
```

---

**FN: default** (line 647)

```rust
fn default() -> Self {
```

---

**FN: default** (line 682)

```rust
fn default() -> Self {
```

---

**FN: default** (line 720)

```rust
fn default() -> Self {
```

---

**FN: default** (line 744)

```rust
fn default() -> Self {
```

---

**FN: default** (line 824)

```rust
fn default() -> Self {
```

---

**FN: default** (line 955)

```rust
fn default() -> Self {
```

---

**FN: drop** (line 1066)

```rust
fn drop(&mut self) {
```

---

**FN: fmt** (line 1089)

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

---

**CONST: STACK_TRAP_RETADDR** (line 1108)

```rust
const STACK_TRAP_RETADDR: u64 = 0xDEAD_0BAD_DEAD_0BAD;
```

---

**FN: set_state_unchecked** (line 1407)

```rust
fn set_state_unchecked(&self, new_state: TaskState) {
```

---

**FN: set_state_owned** (line 1429)

```rust
fn set_state_owned(&mut self, new_state: TaskState) -> Result<(), TaskError> {
```

---

**FN: fmt** (line 1578)

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

---

**FN: fmt** (line 1592)

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

---

**FN: nice_to_weight_is_monotonically_decreasing** (line 1760)

```rust
fn nice_to_weight_is_monotonically_decreasing() {
```

---

**FN: nice_zero_has_default_weight** (line 1770)

```rust
fn nice_zero_has_default_weight() {
```

---

**FN: nice_to_weight_clamps_out_of_range** (line 1775)

```rust
fn nice_to_weight_clamps_out_of_range() {
```

---

**FN: weight_to_nice_round_trip_is_close** (line 1781)

```rust
fn weight_to_nice_round_trip_is_close() {
```

---

**FN: calc_delta_fair_is_identity_at_nice_zero** (line 1789)

```rust
fn calc_delta_fair_is_identity_at_nice_zero() {
```

---

**FN: calc_delta_fair_grows_for_lower_weight** (line 1795)

```rust
fn calc_delta_fair_grows_for_lower_weight() {
```

---

**FN: cpumask_basic_set_clear** (line 1802)

```rust
fn cpumask_basic_set_clear() {
```

---

**FN: cpumask_all_contains_every_cpu** (line 1817)

```rust
fn cpumask_all_contains_every_cpu() {
```

---

**FN: cpumask_intersects** (line 1825)

```rust
fn cpumask_intersects() {
```

---

**FN: cpumask_and_or** (line 1836)

```rust
fn cpumask_and_or() {
```

---

**FN: cpumask_next_after_wraps** (line 1849)

```rust
fn cpumask_next_after_wraps() {
```

---

**FN: cpumask_first_n** (line 1858)

```rust
fn cpumask_first_n() {
```

---

**FN: task_state_valid_transitions** (line 1866)

```rust
fn task_state_valid_transitions() {
```

---

**FN: task_state_invalid_transitions_are_rejected** (line 1874)

```rust
fn task_state_invalid_transitions_are_rejected() {
```

---

**FN: default_rlimits_have_sane_stack_and_nofile** (line 1881)

```rust
fn default_rlimits_have_sane_stack_and_nofile() {
```

---

**FN: credentials_kernel_has_full_capabilities** (line 1889)

```rust
fn credentials_kernel_has_full_capabilities() {
```

---

**FN: credentials_user_has_no_extra_capabilities** (line 1896)

```rust
fn credentials_user_has_no_extra_capabilities() {
```

---

**FN: signal_state_pending_respects_blocked_mask** (line 1903)

```rust
fn signal_state_pending_respects_blocked_mask() {
```

---

**FN: list_head_insert_and_remove_roundtrip** (line 1914)

```rust
fn list_head_insert_and_remove_roundtrip() {
```

---

**FN: list_head_remove_never_creates_a_cycle_with_three_nodes** (line 1942)

```rust
fn list_head_remove_never_creates_a_cycle_with_three_nodes() {
```

---

**FN: spinlock_lock_unlock_cycle** (line 1966)

```rust
fn spinlock_lock_unlock_cycle() {
```

---

**FN: spinlock_try_lock_fails_when_held** (line 1976)

```rust
fn spinlock_try_lock_fails_when_held() {
```

---

**FN: spinlock_irqsave_guard_unlocks_on_drop** (line 1986)

```rust
fn spinlock_irqsave_guard_unlocks_on_drop() {
```

---

**FN: spinlock_plain_guard_unlocks_on_drop** (line 1996)

```rust
fn spinlock_plain_guard_unlocks_on_drop() {
```

---

**FN: atomic_task_flags_insert_remove_are_bitwise** (line 2006)

```rust
fn atomic_task_flags_insert_remove_are_bitwise() {
```

---

**FN: atomic_task_flags_test_and_set_is_edge_triggered** (line 2019)

```rust
fn atomic_task_flags_test_and_set_is_edge_triggered() {
```

---

**FN: rb_parent_color_encoding_roundtrips** (line 2026)

```rust
fn rb_parent_color_encoding_roundtrips() {
```

---

**FN: fx_save_area_default_is_zeroed** (line 2037)

```rust
fn fx_save_area_default_is_zeroed() {
```

---

**FN: cpu_context_default_has_no_valid_fpu_state** (line 2045)

```rust
fn cpu_context_default_has_no_valid_fpu_state() {
```

---

**FN: sched_policy_maps_to_expected_class** (line 2051)

```rust
fn sched_policy_maps_to_expected_class() {
```

---

**FN: sched_class_ordering_matches_pick_next_priority** (line 2062)

```rust
fn sched_class_ordering_matches_pick_next_priority() {
```

---

**FN: default_time_slice_matches_policy_expectations** (line 2070)

```rust
fn default_time_slice_matches_policy_expectations() {
```

---

**FN: load_avg_accumulate_grows_then_decays** (line 2077)

```rust
fn load_avg_accumulate_grows_then_decays() {
```

---

**FN: bare_task_for_state_tests** (line 2086)

```rust
fn bare_task_for_state_tests() -> TaskStruct {
```

---

**FN: set_state_via_shared_reference_does_not_need_mut** (line 2131)

```rust
fn set_state_via_shared_reference_does_not_need_mut() {
```

---

**FN: set_state_rejects_illegal_transition_via_shared_reference** (line 2141)

```rust
fn set_state_rejects_illegal_transition_via_shared_reference() {
```

---

**FN: wake_up_process_pattern_from_remote_cpu_context** (line 2149)

```rust
fn wake_up_process_pattern_from_remote_cpu_context() {
```

---

**FN: remote_wake** (line 2153)

```rust
fn remote_wake(t: &TaskStruct) -> Result<(), TaskError> {
```

---

**FN: rq_backpointer_roundtrips** (line 2162)

```rust
fn rq_backpointer_roundtrips() {
```

---

**FN: need_resched_flag_is_settable_from_shared_reference** (line 2171)

```rust
fn need_resched_flag_is_settable_from_shared_reference() {
```

---

**FN: rt_sched_entity_starts_with_initialized_list_and_no_queued_prio** (line 2181)

```rust
fn rt_sched_entity_starts_with_initialized_list_and_no_queued_prio() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/mod.rs

**STATIC: TOTAL_CPUS** (line 18)

```rust
static TOTAL_CPUS: AtomicU32 = AtomicU32::new(0);
```

---

**STATIC: mut** (line 19)

```rust
static mut RUN_QUEUES: [*mut RunQueue; MAX_CPUS] = [core::ptr::null_mut(); MAX_CPUS];
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/power/em.rs

**CONST: EMPTY_STATE** (line 95)

```rust
const EMPTY_STATE: CapacityState = CapacityState::empty();
```

---

**FN: sort_states** (line 160)

```rust
fn sort_states(&mut self) {
```

---

**CONST: EMPTY_PD** (line 283)

```rust
const EMPTY_PD: PerformanceDomain = PerformanceDomain::empty();
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/power/mod.rs

**CONST: EMPTY_ZONE** (line 170)

```rust
const EMPTY_ZONE: ThermalZone = ThermalZone::empty();
```

---

**CONST: ZERO_CAP** (line 171)

```rust
const ZERO_CAP: AtomicU32 = AtomicU32::new(0);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/power/placement.rs

**CONST: EMPTY_CPU** (line 48)

```rust
const EMPTY_CPU: CpuSnapshot = CpuSnapshot::empty();
```

---

**CONST: EMPTY_DOM** (line 70)

```rust
const EMPTY_DOM: DomainSnapshot = DomainSnapshot::empty();
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/balancing/active.rs

**STRUCT: ActiveBalanceArg** (line 13)

```rust
struct ActiveBalanceArg {
```

---

**FN: active_balance_is_noop_with_zero_imbalance** (line 84)

```rust
fn active_balance_is_noop_with_zero_imbalance() {
```

---

**FN: active_balance_is_noop_when_target_equals_busiest** (line 95)

```rust
fn active_balance_is_noop_when_target_equals_busiest() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/balancing/calculate.rs

**FN: load_calculation_defaults** (line 244)

```rust
fn load_calculation_defaults() {
```

---

**FN: group_classification_idle** (line 251)

```rust
fn group_classification_idle() {
```

---

**FN: group_classification_has_idle_beats_overloaded** (line 260)

```rust
fn group_classification_has_idle_beats_overloaded() {
```

---

**FN: group_classification_overloaded_when_busy_and_full** (line 271)

```rust
fn group_classification_overloaded_when_busy_and_full() {
```

---

**FN: calculate_imbalance_moves_load_toward_equilibrium** (line 281)

```rust
fn calculate_imbalance_moves_load_toward_equilibrium() {
```

---

**FN: calculate_imbalance_is_zero_for_balanced_groups** (line 304)

```rust
fn calculate_imbalance_is_zero_for_balanced_groups() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/balancing/env.rs

**FN: env** (line 79)

```rust
fn env() -> LoadBalanceEnv {
```

---

**FN: flags_set_clear_roundtrip** (line 84)

```rust
fn flags_set_clear_roundtrip() {
```

---

**FN: should_stop_once_loop_max_reached** (line 94)

```rust
fn should_stop_once_loop_max_reached() {
```

---

**FN: record_failure_sets_some_pinned** (line 106)

```rust
fn record_failure_sets_some_pinned() {
```

---

**FN: reset_clears_loop_counter_and_need_break** (line 115)

```rust
fn reset_clears_loop_counter_and_need_break() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/ipi.rs

**FN: migration_stop_ipi_increments_dedicated_counter** (line 75)

```rust
fn migration_stop_ipi_increments_dedicated_counter() {
```

---

**FN: out_of_range_cpu_is_a_safe_noop** (line 85)

```rust
fn out_of_range_cpu_is_a_safe_noop() {
```

---

**FN: handle_ipi_entry_ignores_unimplemented_types_without_panicking** (line 92)

```rust
fn handle_ipi_entry_ignores_unimplemented_types_without_panicking() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/migration/stopper.rs

**FN: wake_stopper_task** (line 108)

```rust
fn wake_stopper_task(&self) {
```

---

**CONST: EMPTY_STOPPER** (line 172)

```rust
const EMPTY_STOPPER: CpuStopper = CpuStopper::empty();
```

---

**FN: stopper_state_transitions** (line 280)

```rust
fn stopper_state_transitions() {
```

---

**FN: stopper_registry_bounds_check** (line 295)

```rust
fn stopper_registry_bounds_check() {
```

---

**FN: queue_execute_and_wait_roundtrip_returns_value_from_function** (line 302)

```rust
fn queue_execute_and_wait_roundtrip_returns_value_from_function() {
```

---

**FN: queue_work_rejects_a_second_job_while_busy** (line 317)

```rust
fn queue_work_rejects_a_second_job_while_busy() {
```

---

**FN: execute_work_without_queued_job_is_a_noop** (line 325)

```rust
fn execute_work_without_queued_job_is_a_noop() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/smp/topology.rs

**CONST: EMPTY_CACHE** (line 172)

```rust
const EMPTY_CACHE: CacheTopology = CacheTopology {
```

---

**CONST: EMPTY_CPU** (line 207)

```rust
const EMPTY_CPU: CpuTopology = CpuTopology::empty();
```

---

**CONST: EMPTY_NODE** (line 208)

```rust
const EMPTY_NODE: NumaNode = NumaNode::empty();
```

---

**CONST: EMPTY_SD** (line 209)

```rust
const EMPTY_SD: SchedDomain = SchedDomain::empty();
```

---

**CONST: EMPTY_SG** (line 210)

```rust
const EMPTY_SG: SchedGroup = SchedGroup::empty();
```

---

**FN: topology_alloc_domains_respects_limits** (line 393)

```rust
fn topology_alloc_domains_respects_limits() {
```

---

**FN: sched_domain_flags_logic** (line 406)

```rust
fn sched_domain_flags_logic() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/tests/test_balance_math.rs

**FN: make_idle** (line 5)

```rust
fn make_idle(pid: u64, cpu: u32) -> TaskStruct {
```

---

**FN: make_task** (line 12)

```rust
fn make_task(pid: u64, policy: SchedPolicy, nice: i8) -> TaskStruct {
```

---

**FN: ptr_of** (line 18)

```rust
fn ptr_of(t: &mut TaskStruct) -> *mut TaskStruct {
```

---

**FN: select_task_rq_returns_preferred_cpu_when_idle** (line 23)

```rust
fn select_task_rq_returns_preferred_cpu_when_idle() {
```

---

**FN: select_task_rq_falls_back_to_least_loaded** (line 42)

```rust
fn select_task_rq_falls_back_to_least_loaded() {
```

---

**FN: select_task_rq_respects_cpus_allowed_mask** (line 68)

```rust
fn select_task_rq_respects_cpus_allowed_mask() {
```

---

**FN: wake_up_process_enqueues_to_target_cpu** (line 88)

```rust
fn wake_up_process_enqueues_to_target_cpu() {
```

---

**FN: idle_balance_pulls_task_from_overloaded_cpu** (line 107)

```rust
fn idle_balance_pulls_task_from_overloaded_cpu() {
```

---

**FN: load_balance_moves_task_across_uneven_queues** (line 135)

```rust
fn load_balance_moves_task_across_uneven_queues() {
```

---

**FN: no_migration_when_cpus_allowed_forbids_it** (line 161)

```rust
fn no_migration_when_cpus_allowed_forbids_it() {
```

---

**FN: migrated_task_updates_cpu_and_rq_pointer** (line 189)

```rust
fn migrated_task_updates_cpu_and_rq_pointer() {
```

---

**FN: pf_no_migrate_blocks_migration** (line 224)

```rust
fn pf_no_migrate_blocks_migration() {
```

---

**FN: load_balance_is_noop_below_threshold** (line 250)

```rust
fn load_balance_is_noop_below_threshold() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/tests/test_pelt.rs

**FN: default_load_avg_is_zero** (line 4)

```rust
fn default_load_avg_is_zero() {
```

---

**FN: accumulate_zero_delta_does_not_change_state_except_time** (line 14)

```rust
fn accumulate_zero_delta_does_not_change_state_except_time() {
```

---

**FN: accumulate_running_task_increases_util_and_load** (line 23)

```rust
fn accumulate_running_task_increases_util_and_load() {
```

---

**FN: accumulate_idle_task_increases_load_but_not_util** (line 34)

```rust
fn accumulate_idle_task_increases_load_but_not_util() {
```

---

**FN: decay_reduces_load_avg_over_time** (line 44)

```rust
fn decay_reduces_load_avg_over_time() {
```

---

**FN: load_avg_never_exceeds_theoretical_maximum** (line 56)

```rust
fn load_avg_never_exceeds_theoretical_maximum() {
```

---

**FN: higher_weight_produces_higher_load_avg** (line 67)

```rust
fn higher_weight_produces_higher_load_avg() {
```

---

**FN: saturating_add_prevents_overflow_on_huge_delta** (line 78)

```rust
fn saturating_add_prevents_overflow_on_huge_delta() {
```

---

**FN: alternating_running_and_idle_smooths_util_avg** (line 86)

```rust
fn alternating_running_and_idle_smooths_util_avg() {
```

---

**FN: period_contrib_increments_on_every_accumulate** (line 99)

```rust
fn period_contrib_increments_on_every_accumulate() {
```

---

**FN: util_sum_accumulates_only_when_running** (line 109)

```rust
fn util_sum_accumulates_only_when_running() {
```

---

**FN: load_sum_accumulates_regardless_of_running_state** (line 118)

```rust
fn load_sum_accumulates_regardless_of_running_state() {
```

---

**FN: pelt_precision_under_one_millisecond** (line 127)

```rust
fn pelt_precision_under_one_millisecond() {
```

---

**FN: pelt_handles_max_weight_without_panic** (line 134)

```rust
fn pelt_handles_max_weight_without_panic() {
```

---

**FN: consecutive_accumulates_update_last_time_correctly** (line 141)

```rust
fn consecutive_accumulates_update_last_time_correctly() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/tests/test_rbtree.rs

**FN: fair_key** (line 5)

```rust
fn fair_key(node: *const TaskStruct) -> u64 {
```

---

**FN: dl_key** (line 9)

```rust
fn dl_key(node: *const TaskStruct) -> u64 {
```

---

**FN: make_task** (line 13)

```rust
fn make_task(pid: u64, vruntime: u64, deadline: u64) -> TaskStruct {
```

---

**FN: empty_tree_has_zero_count_and_ok_invariants** (line 22)

```rust
fn empty_tree_has_zero_count_and_ok_invariants() {
```

---

**FN: single_node_tree_is_black_and_valid** (line 33)

```rust
fn single_node_tree_is_black_and_valid() {
```

---

**FN: sequential_inserts_maintain_sorted_order_and_invariants** (line 46)

```rust
fn sequential_inserts_maintain_sorted_order_and_invariants() {
```

---

**CONST: N** (line 47)

```rust
const N: usize = 100;
```

---

**FN: reverse_inserts_maintain_invariants** (line 77)

```rust
fn reverse_inserts_maintain_invariants() {
```

---

**CONST: N** (line 78)

```rust
const N: usize = 50;
```

---

**FN: random_inserts_and_deletes_keep_tree_valid** (line 97)

```rust
fn random_inserts_and_deletes_keep_tree_valid() {
```

---

**CONST: N** (line 98)

```rust
const N: usize = 200;
```

---

**FN: delete_root_repeatedly_destroys_tree_cleanly** (line 136)

```rust
fn delete_root_repeatedly_destroys_tree_cleanly() {
```

---

**CONST: N** (line 137)

```rust
const N: usize = 30;
```

---

**FN: delete_successor_maintains_inorder_traversal** (line 160)

```rust
fn delete_successor_maintains_inorder_traversal() {
```

---

**CONST: N** (line 161)

```rust
const N: usize = 40;
```

---

**FN: duplicate_vruntime_inserts_are_placed_to_the_right** (line 206)

```rust
fn duplicate_vruntime_inserts_are_placed_to_the_right() {
```

---

**CONST: N** (line 207)

```rust
const N: usize = 10;
```

---

**FN: deadline_key_orders_by_earliest_deadline** (line 234)

```rust
fn deadline_key_orders_by_earliest_deadline() {
```

---

**FN: subtree_min_and_max_return_extremes** (line 254)

```rust
fn subtree_min_and_max_return_extremes() {
```

---

**FN: stress_test_insert_delete_cycles** (line 276)

```rust
fn stress_test_insert_delete_cycles() {
```

---

**CONST: N** (line 277)

```rust
const N: usize = 500;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/time/plist.rs

**FN: plist_prio** (line 20)

```rust
fn plist_prio(node: *mut TaskStruct) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/time/rbtree.rs

**CONST: RB_RED** (line 4)

```rust
const RB_RED: usize = 0;
```

---

**CONST: RB_BLACK** (line 5)

```rust
const RB_BLACK: usize = 1;
```

---

**FN: rb_parent** (line 23)

```rust
fn rb_parent(node: *mut TaskStruct) -> *mut TaskStruct {
```

---

**FN: rb_color** (line 29)

```rust
fn rb_color(node: *mut TaskStruct) -> usize {
```

---

**FN: rb_is_red** (line 35)

```rust
fn rb_is_red(node: *mut TaskStruct) -> bool {
```

---

**FN: rb_set_parent_color** (line 40)

```rust
fn rb_set_parent_color(node: *mut TaskStruct, parent: *mut TaskStruct, color: usize) {
```

---

**FN: rb_set_parent** (line 48)

```rust
fn rb_set_parent(node: *mut TaskStruct, parent: *mut TaskStruct) {
```

---

**FN: rb_set_color** (line 54)

```rust
fn rb_set_color(node: *mut TaskStruct, color: usize) {
```

---

**FN: rb_insert_fixup** (line 116)

```rust
fn rb_insert_fixup(&mut self, mut node: *mut TaskStruct) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/scheduler/time/rt_array.rs

**FN: bit_is_set** (line 34)

```rust
fn bit_is_set(&self, prio: usize) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/smp.rs

**CONST: AP_STACK_SIZE** (line 15)

```rust
const AP_STACK_SIZE: usize = 64 * 1024;
```

---

**STATIC: TOTAL_CPUS** (line 17)

```rust
static TOTAL_CPUS: AtomicU32 = AtomicU32::new(1);
```

---

**STATIC: PHYS_OFFSET** (line 19)

```rust
static PHYS_OFFSET: AtomicU64 = AtomicU64::new(0);
```

---

**STATIC: AP_STARTED** (line 21)

```rust
static AP_STARTED: [AtomicBool; MAX_CPUS] = [const { AtomicBool::new(false) }; MAX_CPUS];
```

---

**STATIC: AP_DONE** (line 22)

```rust
static AP_DONE: [AtomicBool; MAX_CPUS] = [const { AtomicBool::new(false) }; MAX_CPUS];
```

---

**STATIC: AP_APIC_ID_SEEN** (line 23)

```rust
static AP_APIC_ID_SEEN: [AtomicU32; MAX_CPUS] = [const { AtomicU32::new(0) }; MAX_CPUS];
```

---

**STATIC: EXPECTED_APIC_IDS** (line 24)

```rust
static EXPECTED_APIC_IDS: [AtomicU32; MAX_CPUS] = [const { AtomicU32::new(0) }; MAX_CPUS];
```

---

**STATIC: SHARED_COUNTER** (line 26)

```rust
static SHARED_COUNTER: AtomicU64 = AtomicU64::new(0);
```

---

**CONST: INCREMENTS_PER_AP** (line 28)

```rust
const INCREMENTS_PER_AP: u64 = 1000;
```

---

**STRUCT: ApStack** (line 32)

```rust
struct ApStack([u8; AP_STACK_SIZE]);
```

---

**STATIC: AP_STACKS** (line 34)

```rust
static AP_STACKS: [ApStack; MAX_CPUS] = [ApStack([0; AP_STACK_SIZE]); MAX_CPUS];
```

---

**FN: stack_top_of** (line 36)

```rust
fn stack_top_of(i: usize) -> u64 {
```

---

**FN: delay** (line 42)

```rust
fn delay(iterations: u64) {
```

---

**FN: delay_ms** (line 48)

```rust
fn delay_ms(ms: u64) {
```

---

**FN: load_cpu_gdt** (line 52)

```rust
fn load_cpu_gdt(ist_stack_top: VirtAddr) {
```

---

**FN: debug_halt** (line 76)

```rust
fn debug_halt(msg: &str) -> ! {
```

---

**FN: wait_for** (line 131)

```rust
fn wait_for(flag: &AtomicBool, max_ms: u64) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/cpu/trampoline.rs

**CONST: PTE_PRESENT** (line 6)

```rust
const PTE_PRESENT: u64 = 1;
```

---

**CONST: PTE_WRITABLE** (line 7)

```rust
const PTE_WRITABLE: u64 = 2;
```

---

**STATIC: mut** (line 10)

```rust
static mut trampoline_start: u8;
```

---

**STATIC: mut** (line 11)

```rust
static mut trampoline_end: u8;
```

---

**STATIC: mut** (line 12)

```rust
static mut trampoline_cr3: u64;
```

---

**STATIC: mut** (line 13)

```rust
static mut trampoline_entry: u64;
```

---

**STATIC: mut** (line 14)

```rust
static mut trampoline_stack: u64;
```

---

**STATIC: mut** (line 15)

```rust
static mut trampoline_arg: u64;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ctrlinstall/repo/fetch.rs

**FN: fetch** (line 2)

```rust
fn fetch(&mut self, manifest: &PackageManifest) -> Result<alloc::vec::Vec<u8>, CtrlInstallError>;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/bridge.rs

**FN: dp_ready** (line 5)

```rust
fn dp_ready() -> bool;
```

---

**FN: dp_link_info** (line 6)

```rust
fn dp_link_info(rate: *mut u32, lanes: *mut u32);
```

---

**FN: dp_mode_at** (line 7)

```rust
fn dp_mode_at(i: u32, id: *mut u32, w: *mut u32,
```

---

**FN: dp_mode_set_by_id** (line 9)

```rust
fn dp_mode_set_by_id(id: u32) -> bool;
```

---

**FN: dp_submit_fill** (line 10)

```rust
fn dp_submit_fill(color: u32, x: u32, y: u32, w: u32, h: u32) -> u64;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/displayport/init.rs

**FN: dp_init** (line 2)

```rust
fn dp_init() -> bool;
```

---

**FN: dp_ready** (line 3)

```rust
fn dp_ready() -> bool;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/pci/mod.rs

**FN: cfg_addr** (line 18)

```rust
fn cfg_addr(bus: u32, dev: u32, func: u32, off: u32) -> u32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/hid/keyboard.rs

**CONST: KEY_NORMAL** (line 1)

```rust
const KEY_NORMAL: [u8; 30] = [
```

---

**CONST: KEY_SHIFT** (line 8)

```rust
const KEY_SHIFT: [u8; 30] = [
```

---

**CONST: NUM_NORMAL** (line 15)

```rust
const NUM_NORMAL: [u8; 10] = *b"1234567890";
```

---

**CONST: NUM_SHIFT** (line 16)

```rust
const NUM_SHIFT: [u8; 10] = *b"!@#$%^&*()";
```

---

**STATIC: mut** (line 40)

```rust
static mut INBUF: [u8; 64] = [0; 64];
```

---

**STATIC: mut** (line 41)

```rust
static mut IN_HEAD: usize = 0;
```

---

**STATIC: mut** (line 42)

```rust
static mut IN_TAIL: usize = 0;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/hid/mod.rs

**CONST: SET_IDLE** (line 13)

```rust
const SET_IDLE: u8 = 0x0A;
```

---

**CONST: SET_PROTOCOL** (line 14)

```rust
const SET_PROTOCOL: u8 = 0x0B;
```

---

**STATIC: mut** (line 24)

```rust
static mut KEYS: [Option<HidKeyboard>; 2] = [None, None];
```

---

**FN: submit** (line 26)

```rust
fn submit(x: &mut Xhci, kb: &mut HidKeyboard) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/mass/blockdev.rs

**FN: name** (line 5)

```rust
fn name(&self) -> &'static str {
```

---

**FN: block_size** (line 9)

```rust
fn block_size(&self) -> usize {
```

---

**FN: block_count** (line 13)

```rust
fn block_count(&self) -> u64 {
```

---

**FN: read_block** (line 17)

```rust
fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError> {
```

---

**FN: write_block** (line 32)

```rust
fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/mass/mod.rs

**STATIC: mut** (line 15)

```rust
static mut XHCI_PTR: *mut Xhci = core::ptr::null_mut();
```

---

**STATIC: mut** (line 184)

```rust
static mut MASS0: Option<UsbMass> = None;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/class/mod.rs

**FN: probe** (line 6)

```rust
fn probe(&self, dev: &UsbDevice) -> bool;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/core/enumerate.rs

**FN: kprintf** (line 11)

```rust
fn kprintf(fmt: *const u8, ...);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/dma.rs

**FN: drop** (line 37)

```rust
fn drop(&mut self) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/context.rs

**FN: in_add_drop** (line 80)

```rust
fn in_add_drop(&mut self, add: u32, drop: u32) {
```

---

**FN: slot_ptr** (line 89)

```rust
fn slot_ptr(&mut self) -> *mut u32 {
```

---

**FN: ep0_ptr** (line 93)

```rust
fn ep0_ptr(&mut self) -> *mut u32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/event.rs

**FN: kprintf** (line 5)

```rust
fn kprintf(fmt: *const u8, ...);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/init.rs

**FN: kprintf** (line 8)

```rust
fn kprintf(fmt: *const u8, ...);
```

---

**FN: spin_wait** (line 27)

```rust
fn spin_wait<F: Fn() -> bool>(f: F) -> Result<(), UsbError> {
```

---

**FN: op_write64** (line 39)

```rust
fn op_write64(regs: &XhciRegs, off: usize, v: u64) {
```

---

**FN: attach_port** (line 173)

```rust
fn attach_port(&mut self, p: u32) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/regs.rs

**CONST: MAP_SIZE** (line 26)

```rust
const MAP_SIZE: usize = 0x10000;
```

---

**FN: op** (line 85)

```rust
fn op(&self) -> *mut u8 {
```

---

**FN: rt** (line 89)

```rust
fn rt(&self) -> *mut u8 {
```

---

**FN: db** (line 93)

```rust
fn db(&self) -> *mut u8 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/host/xhci/trb.rs

**CONST: ADDR_MASK** (line 25)

```rust
const ADDR_MASK: u64 = 0xFFFF_FFFF_FFFF_FF00;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/drivers/usb/mod.rs

**STATIC: mut** (line 22)

```rust
static mut CONTROLLER: Option<Xhci> = None;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/driverspaceinit/abi/src.rs

**FN: hdr** (line 16)

```rust
fn hdr(&self) -> *mut DsRing {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/driverspaceinit/init/init.rs

**CONST: DIRECT_BASE** (line 7)

```rust
const DIRECT_BASE: u64 = 0xFFFF888000000000;
```

---

**STATIC: mut** (line 18)

```rust
static mut DS: Option<Driverspace> = None;
```

---

**FN: kv** (line 20)

```rust
fn kv(phys: u64) -> *mut u8 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/driverspaceinit/init/service.rs

**FN: kprintf** (line 10)

```rust
fn kprintf(fmt: *const u8, ...);
```

---

**CONST: GRANT_RAM** (line 13)

```rust
const GRANT_RAM: u8 = 1;
```

---

**CONST: GRANT_MMIO** (line 14)

```rust
const GRANT_MMIO: u8 = 2;
```

---

**STRUCT: Grant** (line 17)

```rust
struct Grant {
```

---

**STATIC: mut** (line 25)

```rust
static mut GRANTS: [Grant; 32] = [Grant {
```

---

**STATIC: mut** (line 29)

```rust
static mut NEXT_VA: u64 = 0x4100_0000;
```

---

**STATIC: mut** (line 31)

```rust
static mut AC97_BARS: (u64, u64) = (0, 0);
```

---

**FN: ac97_bars** (line 33)

```rust
fn ac97_bars() -> (u64, u64) {
```

---

**FN: grant_add** (line 48)

```rust
fn grant_add(va: u64, phys: u64, pages: u64, kind: u8) -> bool {
```

---

**STATIC: mut** (line 61)

```rust
static mut VGPU_RING: u64 = 0;
```

---

**STATIC: mut** (line 62)

```rust
static mut VGPU_NEXT_ID: u64 = 1;
```

---

**STRUCT: Slot** (line 65)

```rust
struct Slot {
```

---

**FN: arch_phys_to_virt** (line 73)

```rust
fn arch_phys_to_virt(phys: u64) -> *mut u8;
```

---

**FN: ring** (line 76)

```rust
fn ring() -> Option<*mut Slot> {
```

---

**FN: grant_take** (line 222)

```rust
fn grant_take(va: u64) -> Option<Grant> {
```

---

**FN: grant_phys** (line 235)

```rust
fn grant_phys(va: u64) -> Option<(u64, u64)> {
```

---

**FN: handle** (line 271)

```rust
fn handle(m: &DsMsg, r: &mut DsMsg) -> i32 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ds_ipc/buffer.rs

**FN: from** (line 45)

```rust
fn from(e: BufferError) -> Self {
```

---

**CONST: READABLE** (line 57)

```rust
const READABLE      = 1 << 0;
```

---

**CONST: WRITABLE** (line 58)

```rust
const WRITABLE      = 1 << 1;
```

---

**CONST: ZERO_COPY_OK** (line 59)

```rust
const ZERO_COPY_OK  = 1 << 2;
```

---

**CONST: CACHE_WB** (line 60)

```rust
const CACHE_WB      = 1 << 3;
```

---

**CONST: PINNED** (line 61)

```rust
const PINNED        = 1 << 4;
```

---

**FN: can_do_zero_copy** (line 231)

```rust
fn can_do_zero_copy(&self) -> bool {
```

---

**FN: transfer_via_registers** (line 243)

```rust
fn transfer_via_registers(&mut self) -> Result<(), BufferError> {
```

---

**FN: transfer_zero_copy** (line 256)

```rust
fn transfer_zero_copy(&mut self) -> Result<(), BufferError> {
```

---

**FN: transfer_via_bounce** (line 344)

```rust
fn transfer_via_bounce(&mut self) -> Result<(), BufferError> {
```

---

**FN: arch_flush_tlb_single** (line 522)

```rust
fn arch_flush_tlb_single(task_id: u64, vaddr: VirtAddr) {
```

---

**FN: arch_clean_invalidate_dcache_region** (line 534)

```rust
fn arch_clean_invalidate_dcache_region(vaddr: VirtAddr, size: u64) {
```

---

**FN: test_buffer_alignment_check** (line 609)

```rust
fn test_buffer_alignment_check() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ds_ipc/endpoint.rs

**STATIC: NEXT_EP_ID** (line 6)

```rust
static NEXT_EP_ID: AtomicU64 = AtomicU64::new(1);
```

---

**STRUCT: EndpointState** (line 14)

```rust
struct EndpointState {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/ds_ipc/syscall.rs

**FN: transfer_message** (line 60)

```rust
fn transfer_message(_target: *mut Task, _info: MessageInfo, _regs: &MessageRegisters) {
```

---

**FN: extract_message** (line 64)

```rust
fn extract_message(_sender: *mut Task) -> IpcMessage {
```

---

**FN: block_current_task** (line 72)

```rust
fn block_current_task() {}
```

---

**FN: wake_task** (line 73)

```rust
fn wake_task(_task: *mut Task) {}
```

---

**FN: switch_context** (line 74)

```rust
fn switch_context() {}
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/disc.rs

**FN: block_size** (line 18)

```rust
fn block_size(&self) -> u64;
```

---

**FN: read_block** (line 20)

```rust
fn read_block(&mut self, lba: u64, buf: &[u8]) -> Result<()>;
```

---

**FN: write_block** (line 22)

```rust
fn write_block(&mut self, lba: u64, buf: &mut [ u8]) -> Result<()>{
```

---

**FN: flush** (line 27)

```rust
fn flush(&mut self) -> Result<()> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/driver/ata_pio.rs

**CONST: ATA_TIMEOUT** (line 6)

```rust
const ATA_TIMEOUT: u32 = 1_000_000;
```

---

**CONST: ATA_REG_DATA** (line 8)

```rust
const ATA_REG_DATA: u16 = 0;
```

---

**CONST: ATA_REG_ERROR** (line 9)

```rust
const ATA_REG_ERROR: u16 = 1;
```

---

**CONST: ATA_REG_SECT** (line 10)

```rust
const ATA_REG_SECT: u16 = 2;
```

---

**CONST: ATA_REG_LBA_LO** (line 11)

```rust
const ATA_REG_LBA_LO: u16 = 3;
```

---

**CONST: ATA_REG_LBA_MID** (line 12)

```rust
const ATA_REG_LBA_MID: u16 = 4;
```

---

**CONST: ATA_REG_LBA_HI** (line 13)

```rust
const ATA_REG_LBA_HI: u16 = 5;
```

---

**CONST: ATA_REG_DRIVE** (line 14)

```rust
const ATA_REG_DRIVE: u16 = 6;
```

---

**CONST: ATA_REG_STATUS** (line 15)

```rust
const ATA_REG_STATUS: u16 = 7;
```

---

**CONST: ATA_STATUS_ERR** (line 17)

```rust
const ATA_STATUS_ERR: u8 = 1 << 0;
```

---

**CONST: ATA_STATUS_DRQ** (line 18)

```rust
const ATA_STATUS_DRQ: u8 = 1 << 3;
```

---

**CONST: ATA_STATUS_BSY** (line 19)

```rust
const ATA_STATUS_BSY: u8 = 1 << 7;
```

---

**CONST: ATA_CMD_IDENTIFY** (line 21)

```rust
const ATA_CMD_IDENTIFY: u8 = 0xEC;
```

---

**CONST: ATA_CMD_READ28** (line 22)

```rust
const ATA_CMD_READ28: u8 = 0x20;
```

---

**CONST: ATA_CMD_WRITE28** (line 23)

```rust
const ATA_CMD_WRITE28: u8 = 0x30;
```

---

**CONST: ATA_CMD_FLUSH** (line 24)

```rust
const ATA_CMD_FLUSH: u8 = 0xE7;
```

---

**FN: drive_lba** (line 58)

```rust
fn drive_lba(&self) -> u8 {
```

---

**FN: drive_identify** (line 62)

```rust
fn drive_identify(&self) -> u8 {
```

---

**FN: disable_irq** (line 70)

```rust
fn disable_irq(&self) {
```

---

**FN: status** (line 74)

```rust
fn status(&self) -> u8 {
```

---

**FN: wait_not_bsy** (line 78)

```rust
fn wait_not_bsy(&self) -> Result<(), DriverError> {
```

---

**FN: wait_drq** (line 88)

```rust
fn wait_drq(&self) -> Result<(), DriverError> {
```

---

**FN: wait_ready** (line 104)

```rust
fn wait_ready(&self) -> Result<(), DriverError> {
```

---

**FN: setup_lba28** (line 116)

```rust
fn setup_lba28(&self, lba: u32, count: u8, cmd: u8) -> Result<(), DriverError> {
```

---

**FN: read_sector** (line 162)

```rust
fn read_sector(&self, lba: u32, buf: &mut [u8]) -> Result<(), DriverError> {
```

---

**FN: write_sector** (line 177)

```rust
fn write_sector(&self, lba: u32, buf: &[u8]) -> Result<(), DriverError> {
```

---

**FN: name** (line 196)

```rust
fn name(&self) -> &'static str {
```

---

**FN: block_size** (line 200)

```rust
fn block_size(&self) -> usize {
```

---

**FN: block_count** (line 204)

```rust
fn block_count(&self) -> u64 {
```

---

**FN: read_block** (line 208)

```rust
fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError> {
```

---

**FN: write_block** (line 224)

```rust
fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError> {
```

---

**FN: probe_dev** (line 241)

```rust
fn probe_dev(dev: &AtaPio) -> bool {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/driver/block.rs

**FN: name** (line 12)

```rust
fn name(&self) -> &'static str;
```

---

**FN: block_size** (line 13)

```rust
fn block_size(&self) -> usize;
```

---

**FN: block_count** (line 14)

```rust
fn block_count(&self) -> u64;
```

---

**FN: read_block** (line 16)

```rust
fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError>;
```

---

**FN: write_block** (line 17)

```rust
fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError>;
```

---

**FN: read_blocks** (line 19)

```rust
fn read_blocks(&self, start: u64, buf: &mut [u8]) -> Result<(), DriverError> {
```

---

**FN: write_blocks** (line 36)

```rust
fn write_blocks(&self, start: u64, buf: &[u8]) -> Result<(), DriverError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/driver/lock.rs

**FN: drop** (line 20)

```rust
fn drop(&mut self) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/driver/registry.rs

**STATIC: mut** (line 4)

```rust
static mut DEVICES: [Option<&'static dyn BlockDevice>; 4] =
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/ext4/mod.rs

**CONST: EXT4_MAGIC** (line 45)

```rust
const EXT4_MAGIC: u16 = 0xEF53;
```

---

**CONST: INCOMPAT_EXTENTS** (line 46)

```rust
const INCOMPAT_EXTENTS: u32 = 0x40;
```

---

**CONST: INCOMPAT_64BIT** (line 47)

```rust
const INCOMPAT_64BIT: u32 = 0x80;
```

---

**FN: u16le** (line 49)

```rust
fn u16le(b: &[u8], o: usize) -> u16 {
```

---

**FN: u32le** (line 53)

```rust
fn u32le(b: &[u8], o: usize) -> u32 {
```

---

**FN: u64le** (line 57)

```rust
fn u64le(b: &[u8], o: usize) -> u64 {
```

---

**FN: read_at** (line 107)

```rust
fn read_at(disk: &'static dyn BlockDevice,
```

---

**FN: inode_table** (line 138)

```rust
fn inode_table(&self, ino: u32) -> Result<u64, ExtError> {
```

---

**FN: block_map** (line 173)

```rust
fn block_map(&self, inode: &RawInode, file_blk: u32) -> Option<u64> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/fat32/mod.rs

**FN: u16le** (line 30)

```rust
fn u16le(b: &[u8], o: usize) -> u16 {
```

---

**FN: u32le** (line 34)

```rust
fn u32le(b: &[u8], o: usize) -> u32 {
```

---

**FN: read_at** (line 65)

```rust
fn read_at(&self, off: u64, buf: &mut [u8]) -> Result<(), FatError> {
```

---

**FN: cluster_off** (line 85)

```rust
fn cluster_off(&self, clus: u32) -> u64 {
```

---

**FN: read_cluster** (line 89)

```rust
fn read_cluster(&self, clus: u32) -> Result<Vec<u8>, FatError> {
```

---

**FN: next_cluster** (line 95)

```rust
fn next_cluster(&self, clus: u32) -> Option<u32> {
```

---

**FN: short_name** (line 113)

```rust
fn short_name(raw: &[u8]) -> Vec<u8> {
```

---

**FN: lfn_char** (line 138)

```rust
fn lfn_char(c: u16) -> u8 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/mbr.rs

**FN: kprintf** (line 5)

```rust
fn kprintf(fmt: *const u8, ...);
```

---

**STATIC: mut** (line 24)

```rust
static mut PARTS: [Option<Partition>; 8] =
```

---

**FN: name** (line 28)

```rust
fn name(&self) -> &'static str {
```

---

**FN: block_size** (line 32)

```rust
fn block_size(&self) -> usize {
```

---

**FN: block_count** (line 36)

```rust
fn block_count(&self) -> u64 {
```

---

**FN: read_block** (line 40)

```rust
fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError> {
```

---

**FN: write_block** (line 48)

```rust
fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/btree.rs

**CONST: BTREE_MAGIC** (line 6)

```rust
const BTREE_MAGIC: u32 = 0x42544E44;
```

---

**FN: search_leaf_node** (line 46)

```rust
fn search_leaf_node(node: &BtreeNode, name: &str) -> Result<u64> {
```

---

**FN: find_child_in_internal** (line 67)

```rust
fn find_child_in_internal(node: &BtreeNode, name: &str) -> Result<u64> {
```

---

**FN: collect_leaf_entries** (line 134)

```rust
fn collect_leaf_entries(node: &BtreeNode, entries: &mut Vec<DirEntry>) -> Result<()> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/file.rs

**FN: allocate_block_for_inode** (line 85)

```rust
fn allocate_block_for_inode(fs: &TangFs, inode: &mut Inode, logical_block: u64) -> Result<u64> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/inode.rs

**FN: load_inode** (line 41)

```rust
fn load_inode(&self) -> Result<Inode> {
```

---

**FN: save_inode** (line 55)

```rust
fn save_inode(&self, inode: &Inode) -> Result<()> {
```

---

**FN: lookup** (line 76)

```rust
fn lookup(&self, name: &str) -> Result<Box<dyn VfsInode>> {
```

---

**FN: readdir** (line 95)

```rust
fn readdir(&self) -> Result<Vec<DirEntry>> {
```

---

**FN: read** (line 105)

```rust
fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
```

---

**FN: write** (line 109)

```rust
fn write(&self, offset: u64, data: &[u8]) -> Result<usize> {
```

---

**FN: stat** (line 116)

```rust
fn stat(&self) -> Result<crate::fs::vfs::Stat> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/mod.rs

**FN: root_inode** (line 81)

```rust
fn root_inode(&self) -> Box<dyn VfsInode> {
```

---

**FN: statfs** (line 89)

```rust
fn statfs(&self) -> Result<crate::fs::vfs::StatFs> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tangfs/superblock.rs

**FN: calculate_checksum** (line 55)

```rust
fn calculate_checksum(sb: &Superblock) -> u32 {
```

---

**FN: clone** (line 89)

```rust
fn clone(&self) -> Self {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/tfs.rs

**CONST: DIR_BLOCKS** (line 6)

```rust
const DIR_BLOCKS: u32 = 16;
```

---

**CONST: DATA_START** (line 7)

```rust
const DATA_START: u32 = 1 + 1 + DIR_BLOCKS;
```

---

**CONST: ENTRY_SIZE** (line 8)

```rust
const ENTRY_SIZE: usize = 64;
```

---

**CONST: ENTRIES_PER_BLOCK** (line 9)

```rust
const ENTRIES_PER_BLOCK: usize = 512 / ENTRY_SIZE;
```

---

**CONST: MAX_NAME** (line 10)

```rust
const MAX_NAME: usize = 48;
```

---

**CONST: KIND_EMPTY** (line 14)

```rust
const KIND_EMPTY: u8 = 0;
```

---

**CONST: KIND_FILE** (line 15)

```rust
const KIND_FILE: u8 = 1;
```

---

**CONST: KIND_DIR** (line 16)

```rust
const KIND_DIR: u8 = 2;
```

---

**FN: rd32** (line 39)

```rust
fn rd32(b: &[u8], off: usize) -> u32 {
```

---

**FN: wr32** (line 43)

```rust
fn wr32(b: &mut [u8], off: usize, v: u32) {
```

---

**STRUCT: DirEntry** (line 98)

```rust
struct DirEntry {
```

---

**CONST: EMPTY_ENTRY** (line 105)

```rust
const EMPTY_ENTRY: DirEntry = DirEntry {
```

---

**FN: read_entry** (line 112)

```rust
fn read_entry(block: &[u8], idx: usize) -> DirEntry {
```

---

**FN: write_entry** (line 124)

```rust
fn write_entry(block: &mut [u8], idx: usize, e: &DirEntry) {
```

---

**FN: entry_name** (line 132)

```rust
fn entry_name(e: &DirEntry) -> &str {
```

---

**FN: find_entry** (line 137)

```rust
fn find_entry(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<(usize, DirEntry)> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/fs/vfs.rs

**STATIC: mut** (line 54)

```rust
static mut ROOT_FS: Option<Mounted> = None;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gdt.rs

**STRUCT: Selectors** (line 18)

```rust
struct Selectors {
```

---

**STATIC: ref** (line 24)

```rust
static ref TSS: TaskStateSegment = {
```

---

**CONST: STACK_SIZE** (line 27)

```rust
const STACK_SIZE: usize = 4096 * 5;
```

---

**STATIC: mut** (line 28)

```rust
static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
```

---

**STATIC: ref** (line 38)

```rust
static ref GDT: (GlobalDescriptorTable, Selectors) = {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/console.rs

**CONST: GLYPH_W** (line 6)

```rust
const GLYPH_W: usize = 8;
```

---

**CONST: GLYPH_H** (line 7)

```rust
const GLYPH_H: usize = 8;
```

---

**STATIC: mut** (line 12)

```rust
static mut FB: Option<Framebuffer> = None;
```

---

**STATIC: mut** (line 13)

```rust
static mut FB_PHYS: u64 = 0;
```

---

**STATIC: mut** (line 14)

```rust
static mut ENABLED: bool = true;
```

---

**STATIC: mut** (line 15)

```rust
static mut CLEAN: alloc::vec::Vec<u32> = alloc::vec::Vec::new();
```

---

**STATIC: mut** (line 17)

```rust
static mut COLS: usize = 0;
```

---

**STATIC: mut** (line 18)

```rust
static mut ROWS: usize = 0;
```

---

**STATIC: mut** (line 20)

```rust
static mut CELL_CACHE: [(u8, u8); MAX_COLS * MAX_ROWS] = [(0, 0); MAX_COLS * MAX_ROWS];
```

---

**STATIC: mut** (line 21)

```rust
static mut CACHE_VALID: bool = false;
```

---

**STATIC: mut** (line 23)

```rust
static mut FB_DEV_VIRT: u64 = 0;
```

---

**STATIC: mut** (line 24)

```rust
static mut FB_DEV_SIZE: usize = 0;
```

---

**STATIC: mut** (line 26)

```rust
static mut REVERSE_TEXT_COLS: bool = true;
```

---

**FN: fb** (line 28)

```rust
fn fb() -> &'static mut Framebuffer {
```

---

**FN: framebuffer_info** (line 54)

```rust
fn framebuffer_info() -> Option<(u32, u32, u32, u64, bool)> {
```

---

**FN: set_palette_rgb332** (line 90)

```rust
fn set_palette_rgb332() {
```

---

**FN: set_palette16** (line 111)

```rust
fn set_palette16() {
```

---

**FN: disable_text_cursor** (line 127)

```rust
fn disable_text_cursor() {
```

---

**FN: delay** (line 139)

```rust
fn delay() {
```

---

**FN: draw_cell** (line 363)

```rust
fn draw_cell(row: usize, col: usize, ch: u8, attr: u8) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/font.rs

**CONST: fn** (line 100)

```rust
const fn make_font8x8() -> [[u8; 8]; 96] {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/framebuffer.rs

**FN: rgb332_index** (line 29)

```rust
fn rgb332_index(r: u32, g: u32, b: u32) -> u8 {
```

---

**FN: rgb332_from_index** (line 33)

```rust
fn rgb332_from_index(idx: u8) -> (u32, u32, u32) {
```

---

**FN: rgb_to_index4** (line 40)

```rust
fn rgb_to_index4(r: u32, g: u32, b: u32) -> u8 {
```

---

**FN: index4_to_rgb** (line 56)

```rust
fn index4_to_rgb(idx: u8) -> (u32, u32, u32) {
```

---

**FN: ry** (line 61)

```rust
fn ry(&self, y: usize) -> usize {
```

---

**FN: rx** (line 65)

```rust
fn rx(&self, x: usize) -> usize {
```

---

**FN: planar_set** (line 119)

```rust
fn planar_set(&self, x: usize, y: usize, color: u8) {
```

---

**FN: planar_get** (line 134)

```rust
fn planar_get(&self, x: usize, y: usize) -> u8 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/galaxy.rs

**CONST: P_COS** (line 3)

```rust
const P_COS: i64 = 59636;
```

---

**CONST: P_SIN** (line 4)

```rust
const P_SIN: i64 = 27525;
```

---

**FN: hash2** (line 6)

```rust
fn hash2(x: i64, y: i64) -> u32 {
```

---

**FN: vnoise** (line 14)

```rust
fn vnoise(xq: i64, yq: i64) -> i64 {
```

---

**FN: fbm** (line 34)

```rust
fn fbm(xq: i64, yq: i64) -> i64 {
```

---

**FN: band_fall** (line 42)

```rust
fn band_fall(x: i64, y: i64, w: i64, h: i64) -> i64 {
```

---

**FN: green_fall** (line 60)

```rust
fn green_fall(x: i64, y: i64, w: i64, _h: i64) -> i64 {
```

---

**FN: px_add** (line 83)

```rust
fn px_add(fb: &mut Framebuffer, x: i64, y: i64,
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/mod.rs

**STATIC: mut** (line 12)

```rust
static mut CURRENT: vga::VideoMode = vga::VideoMode::Mode13h;
```

---

**STATIC: mut** (line 13)

```rust
static mut CURRENT_W: u32 = 320;
```

---

**STATIC: mut** (line 14)

```rust
static mut CURRENT_H: u32 = 200;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/panic_screen.rs

**CONST: BG** (line 5)

```rust
const BG: (u32, u32, u32) = (26, 0, 0);
```

---

**CONST: RULE** (line 7)

```rust
const RULE: &str =
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/gfx/vga.rs

**FN: write_regs** (line 9)

```rust
fn write_regs(misc: u8, seq: &[u8], crtc: &[u8], gfx: &[u8], attr: &[u8]) {
```

---

**CONST: DISPI_INDEX_PORT** (line 73)

```rust
const DISPI_INDEX_PORT: u16 = 0x01CE;
```

---

**CONST: DISPI_DATA_PORT** (line 74)

```rust
const DISPI_DATA_PORT: u16 = 0x01CF;
```

---

**CONST: DISPI_INDEX_ID** (line 76)

```rust
const DISPI_INDEX_ID: u16 = 0;
```

---

**CONST: DISPI_INDEX_XRES** (line 77)

```rust
const DISPI_INDEX_XRES: u16 = 1;
```

---

**CONST: DISPI_INDEX_YRES** (line 78)

```rust
const DISPI_INDEX_YRES: u16 = 2;
```

---

**CONST: DISPI_INDEX_BPP** (line 79)

```rust
const DISPI_INDEX_BPP: u16 = 3;
```

---

**CONST: DISPI_INDEX_ENABLE** (line 80)

```rust
const DISPI_INDEX_ENABLE: u16 = 4;
```

---

**CONST: DISPI_INDEX_VIRT_WIDTH** (line 81)

```rust
const DISPI_INDEX_VIRT_WIDTH: u16 = 6;
```

---

**CONST: DISPI_INDEX_VIRT_HEIGHT** (line 82)

```rust
const DISPI_INDEX_VIRT_HEIGHT: u16 = 7;
```

---

**CONST: DISPI_INDEX_X_OFFSET** (line 83)

```rust
const DISPI_INDEX_X_OFFSET: u16 = 8;
```

---

**CONST: DISPI_INDEX_Y_OFFSET** (line 84)

```rust
const DISPI_INDEX_Y_OFFSET: u16 = 9;
```

---

**CONST: DISPI_DISABLED** (line 86)

```rust
const DISPI_DISABLED: u16 = 0x00;
```

---

**CONST: DISPI_ENABLED** (line 87)

```rust
const DISPI_ENABLED: u16 = 0x01;
```

---

**CONST: DISPI_LFB_ENABLED** (line 88)

```rust
const DISPI_LFB_ENABLED: u16 = 0x40;
```

---

**CONST: DISPI_NOCLEARMEM** (line 89)

```rust
const DISPI_NOCLEARMEM: u16 = 0x80;
```

---

**FN: dispi_write** (line 91)

```rust
fn dispi_write(index: u16, value: u16) {
```

---

**FN: dispi_read** (line 98)

```rust
fn dispi_read(index: u16) -> u16 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/aut.rs

**CONST: BUDGET_MAX** (line 15)

```rust
const BUDGET_MAX: u32 = 8;
```

---

**STATIC: BUDGET** (line 16)

```rust
static BUDGET: AtomicU32 = AtomicU32::new(BUDGET_MAX);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/bridge.rs

**FN: hdmi_iface_acquire** (line 5)

```rust
fn hdmi_iface_acquire(owner: u32) -> bool;
```

---

**FN: hdmi_iface_release** (line 6)

```rust
fn hdmi_iface_release(owner: u32) -> bool;
```

---

**FN: hdmi_iface_init** (line 8)

```rust
fn hdmi_iface_init(owner: u32, fb_phys: u64, w: u32, h: u32, stride: u32) -> bool;
```

---

**FN: hdmi_iface_ready** (line 9)

```rust
fn hdmi_iface_ready() -> bool;
```

---

**FN: hdmi_iface_mode_set** (line 11)

```rust
fn hdmi_iface_mode_set(owner: u32, id: u32) -> bool;
```

---

**FN: hdmi_iface_mode_current** (line 12)

```rust
fn hdmi_iface_mode_current(id: *mut u32, w: *mut u32, h: *mut u32, r: *mut u32) -> bool;
```

---

**FN: hdmi_iface_mode_at** (line 13)

```rust
fn hdmi_iface_mode_at(i: u32, id: *mut u32, w: *mut u32, h: *mut u32, r: *mut u32) -> bool;
```

---

**FN: hdmi_iface_submit_fill** (line 15)

```rust
fn hdmi_iface_submit_fill(owner: u32, color: u32, x: u32, y: u32, w: u32, h: u32) -> u64;
```

---

**FN: hdmi_iface_poll** (line 16)

```rust
fn hdmi_iface_poll(owner: u32, out: *mut u64) -> bool;
```

---

**FN: hdmi_iface_caps** (line 18)

```rust
fn hdmi_iface_caps(w: *mut u32, h: *mut u32, s: *mut u32, phys: *mut u64);
```

---

**FN: hdmi_iface_fb_grant** (line 20)

```rust
fn hdmi_iface_fb_grant(owner: u32, phys: *mut u64, w: *mut u32, h: *mut u32, s: *mut u32) -> bool;
```

---

**FN: hdmi_iface_fb_revoke** (line 21)

```rust
fn hdmi_iface_fb_revoke(owner: u32) -> bool;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/hdmi/init.rs

**FN: hdmi_init_with** (line 2)

```rust
fn hdmi_init_with(fb_phys: u64, w: u32, h: u32, stride: u32) -> bool;
```

---

**FN: hdmi_ready** (line 3)

```rust
fn hdmi_ready() -> bool;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/interrupts.rs

**FN: as_u8** (line 56)

```rust
fn as_u8(self) -> u8 {
```

---

**FN: breakpoint_handler** (line 61)

```rust
fn breakpoint_handler(stack_frame: InterruptStackFrame) {
```

---

**STATIC: ref** (line 67)

```rust
static ref IDT: InterruptDescriptorTable = {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/kernel_glue.rs

**CONST: KSTD_PATH_MAX** (line 1)

```rust
const KSTD_PATH_MAX: usize = 256;
```

---

**FN: cstr_to_str** (line 3)

```rust
fn cstr_to_str<'a>(p: *const u8) -> Option<&'a str> {
```

---

**FN: tfs_read_path** (line 96)

```rust
fn tfs_read_path(dev: &dyn crate::fs::driver::block::BlockDevice, path: &str) -> Option<alloc::vec::Vec<u8>> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/kstd_glue.rs

**CONST: KSTD_PATH_MAX** (line 1)

```rust
const KSTD_PATH_MAX: usize = 256;
```

---

**FN: cstr_to_str** (line 3)

```rust
fn cstr_to_str<'a>(p: *const u8) -> Option<&'a str> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/main.rs

**STATIC: TESTS** (line 46)

```rust
static TESTS: &[Test] = &[
```

---

**STATIC: TESTS** (line 126)

```rust
static TESTS: &[Test] = &[
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/api.rs

**STATIC: ALLOCATOR** (line 44)

```rust
static ALLOCATOR: KernelAlloc = KernelAlloc;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/init.rs

**CONST: DIRECT_BASE** (line 7)

```rust
const DIRECT_BASE: u64 = 0xFFFF888000000000;
```

---

**STATIC: mut** (line 18)

```rust
static mut DS: Option<Driverspace> = None;
```

---

**FN: kv** (line 20)

```rust
fn kv(phys: u64) -> *mut u8 {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/mm_bridge.rs

**CONST: MAX_RAW_ENTRIES** (line 10)

```rust
const MAX_RAW_ENTRIES: usize = 256;
```

---

**CONST: fn** (line 22)

```rust
const fn zero() -> Self {
```

---

**FN: arch_memory_init** (line 33)

```rust
fn arch_memory_init(
```

---

**FN: arch_memory_dump** (line 42)

```rust
fn arch_memory_dump();
```

---

**STATIC: mut** (line 45)

```rust
static mut RAW_ENTRIES: [RawMemEntry; MAX_RAW_ENTRIES] =
```

---

**STATIC: mut** (line 48)

```rust
static mut RAW_COUNT: usize = 0;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/mod.rs

**CONST: MAX_RAW_ENTRIES** (line 40)

```rust
const MAX_RAW_ENTRIES: usize = 256;
```

---

**CONST: ZERO_ENTRY** (line 43)

```rust
const ZERO_ENTRY: ffi::RawMemEntry = ffi::RawMemEntry {
```

---

**STRUCT: EntryStorage** (line 51)

```rust
struct EntryStorage(UnsafeCell<[ffi::RawMemEntry; MAX_RAW_ENTRIES]>);
```

---

**STATIC: ENTRIES** (line 60)

```rust
static ENTRIES: EntryStorage =
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/riscv.rs

**CONST: FRAME_SIZE** (line 4)

```rust
const FRAME_SIZE: usize = 4096;
```

---

**CONST: FRAME_POOL_BYTES** (line 5)

```rust
const FRAME_POOL_BYTES: usize = 4 * 1024 * 1024;
```

---

**CONST: FRAME_COUNT** (line 6)

```rust
const FRAME_COUNT: usize = FRAME_POOL_BYTES / FRAME_SIZE;
```

---

**CONST: BITMAP_WORDS** (line 7)

```rust
const BITMAP_WORDS: usize = FRAME_COUNT / 64;
```

---

**STRUCT: FramePool** (line 10)

```rust
struct FramePool([u8; FRAME_POOL_BYTES]);
```

---

**STATIC: POOL** (line 12)

```rust
static POOL: FramePool = FramePool([0; FRAME_POOL_BYTES]);
```

---

**STRUCT: State** (line 14)

```rust
struct State {
```

---

**STATIC: STATE** (line 19)

```rust
static STATE: Mutex<State> = Mutex::new(State {
```

---

**FN: pool_base** (line 24)

```rust
fn pool_base() -> u64 {
```

---

**FN: bit_set** (line 35)

```rust
fn bit_set(b: &mut [u64; BITMAP_WORDS], idx: usize) {
```

---

**FN: bit_clear** (line 39)

```rust
fn bit_clear(b: &mut [u64; BITMAP_WORDS], idx: usize) {
```

---

**FN: bit_get** (line 43)

```rust
fn bit_get(b: &[u64; BITMAP_WORDS], idx: usize) -> bool {
```

---

**FN: scan_run** (line 66)

```rust
fn scan_run(s: &mut State, count: usize, align_frames: usize) -> Option<usize> {
```

---

**CONST: HEAP_BYTES** (line 175)

```rust
const HEAP_BYTES: usize = 2 * 1024 * 1024;
```

---

**CONST: HDR** (line 176)

```rust
const HDR: usize = 32;
```

---

**CONST: MIN_ALIGN** (line 177)

```rust
const MIN_ALIGN: usize = 16;
```

---

**STRUCT: HeapPool** (line 180)

```rust
struct HeapPool([u8; HEAP_BYTES]);
```

---

**STATIC: POOL** (line 182)

```rust
static POOL: HeapPool = HeapPool([0; HEAP_BYTES]);
```

---

**STRUCT: HeapState** (line 184)

```rust
struct HeapState {
```

---

**STATIC: STATE** (line 188)

```rust
static STATE: Mutex<HeapState> = Mutex::new(HeapState { head: 0 });
```

---

**FN: pool_base** (line 190)

```rust
fn pool_base() -> usize {
```

---

**FN: push_free** (line 224)

```rust
fn push_free(s: &mut HeapState, block: usize) {
```

---

**FN: raw_alloc** (line 231)

```rust
fn raw_alloc(size: usize, align: usize) -> *mut u8 {
```

---

**TYPE: Output** (line 436)

```rust
type Output = VmmFlags;
```

---

**FN: bitor** (line 438)

```rust
fn bitor(self, rhs: VmmFlags) -> VmmFlags {
```

---

**CONST: WINDOW_BASE** (line 449)

```rust
const WINDOW_BASE: u64 = 0x4000_0000;
```

---

**CONST: WINDOW_LEN** (line 450)

```rust
const WINDOW_LEN: u64 = 0x4000_0000;
```

---

**STRUCT: DeviceMap** (line 452)

```rust
struct DeviceMap {
```

---

**STRUCT: State** (line 458)

```rust
struct State {
```

---

**STATIC: STATE** (line 464)

```rust
static STATE: Mutex<State> = Mutex::new(State {
```

---

**FN: overlaps** (line 477)

```rust
fn overlaps(s: &State, va: u64, len: u64) -> bool {
```

---

**FN: take_range** (line 483)

```rust
fn take_range(s: &mut State, len: u64) -> Option<u64> {
```

---

**CONST: PTE_V** (line 558)

```rust
const PTE_V: u64 = 1 << 0;
```

---

**CONST: PTE_R** (line 559)

```rust
const PTE_R: u64 = 1 << 1;
```

---

**CONST: PTE_W** (line 560)

```rust
const PTE_W: u64 = 1 << 2;
```

---

**CONST: PTE_X** (line 561)

```rust
const PTE_X: u64 = 1 << 3;
```

---

**CONST: PTE_U** (line 562)

```rust
const PTE_U: u64 = 1 << 4;
```

---

**CONST: PTE_A** (line 563)

```rust
const PTE_A: u64 = 1 << 6;
```

---

**CONST: PTE_D** (line 564)

```rust
const PTE_D: u64 = 1 << 7;
```

---

**CONST: PAGE** (line 566)

```rust
const PAGE: usize = 4096;
```

---

**TYPE: Output** (line 586)

```rust
type Output = ProtFlags;
```

---

**FN: bitor** (line 588)

```rust
fn bitor(self, rhs: ProtFlags) -> ProtFlags {
```

---

**TYPE: Output** (line 615)

```rust
type Output = MapFlags;
```

---

**FN: bitor** (line 617)

```rust
fn bitor(self, rhs: MapFlags) -> MapFlags {
```

---

**FN: sfence_all** (line 626)

```rust
fn sfence_all() {
```

---

**FN: pte_for_prot** (line 632)

```rust
fn pte_for_prot(prot: ProtFlags) -> u64 {
```

---

**STRUCT: Range** (line 650)

```rust
struct Range {
```

---

**STRUCT: Inner** (line 656)

```rust
struct Inner {
```

---

**FN: alloc_table** (line 668)

```rust
fn alloc_table() -> Option<usize> {
```

---

**FN: ensure_table** (line 695)

```rust
fn ensure_table(entry: u64, tables: &mut Vec<u64>) -> Option<usize> {
```

---

**FN: map_page** (line 704)

```rust
fn map_page(
```

---

**FN: set_pte_flags** (line 852)

```rust
fn set_pte_flags(&self, va: u64, flags: u64) -> bool {
```

---

**FN: unmap_page_root** (line 891)

```rust
fn unmap_page_root(root: usize, va: u64) -> bool {
```

---

**FN: drop** (line 916)

```rust
fn drop(&mut self) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/space.rs

**TYPE: Output** (line 24)

```rust
type Output = ProtFlags;
```

---

**FN: bitor** (line 26)

```rust
fn bitor(self, rhs: ProtFlags) -> ProtFlags {
```

---

**TYPE: Output** (line 53)

```rust
type Output = MapFlags;
```

---

**FN: bitor** (line 55)

```rust
fn bitor(self, rhs: MapFlags) -> MapFlags {
```

---

**FN: drop** (line 121)

```rust
fn drop(&mut self) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/virt.rs

**TYPE: Output** (line 22)

```rust
type Output = VmmFlags;
```

---

**FN: bitor** (line 24)

```rust
fn bitor(self, rhs: VmmFlags) -> VmmFlags {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/arp.rs

**STRUCT: CacheEntry** (line 106)

```rust
struct CacheEntry {
```

---

**FN: default** (line 173)

```rust
fn default() -> Self {
```

---

**FN: arp_round_trip_and_cache_expiry** (line 183)

```rust
fn arp_round_trip_and_cache_expiry() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/checksum.rs

**FN: checksum_round_trip** (line 47)

```rust
fn checksum_round_trip() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/command.rs

**FN: parse_octet** (line 85)

```rust
fn parse_octet(input: &str) -> Option<u8> {
```

---

**FN: parses_ipv4** (line 107)

```rust
fn parses_ipv4() {
```

---

**FN: rejects_invalid_ipv4** (line 112)

```rust
fn rejects_invalid_ipv4() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/device.rs

**FN: init** (line 29)

```rust
fn init(&mut self) -> Result<(), NetworkError>;
```

---

**FN: mac_address** (line 30)

```rust
fn mac_address(&self) -> MacAddress;
```

---

**FN: mtu** (line 31)

```rust
fn mtu(&self) -> usize;
```

---

**FN: submit_tx** (line 32)

```rust
fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError>;
```

---

**FN: poll** (line 33)

```rust
fn poll(&mut self) -> Result<PollResult, NetworkError>;
```

---

**FN: take_rx** (line 34)

```rust
fn take_rx(&mut self) -> Option<RxFrame<'_>>;
```

---

**FN: recycle_rx** (line 35)

```rust
fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError>;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/driver.rs

**CONST: MAX_DEVICES** (line 3)

```rust
const MAX_DEVICES: usize = 4;
```

---

**FN: default** (line 41)

```rust
fn default() -> Self {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/ethernet.rs

**FN: ethernet_round_trip_and_padding** (line 97)

```rust
fn ethernet_round_trip_and_padding() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/icmp.rs

**FN: echo_request_and_reply_keep_identity_and_payload** (line 83)

```rust
fn echo_request_and_reply_keep_identity_and_payload() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/ipv4.rs

**FN: ipv4_build_then_parse** (line 105)

```rust
fn ipv4_build_then_parse() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/packet.rs

**FN: self_test_passes** (line 69)

```rust
fn self_test_passes() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/ping.rs

**CONST: FRAME_BYTES** (line 6)

```rust
const FRAME_BYTES: usize = 1536;
```

---

**FN: send_pending** (line 82)

```rust
fn send_pending(
```

---

**FN: map_packet_error** (line 115)

```rust
fn map_packet_error(error: PacketError) -> NetworkError {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/runtime.rs

**CONST: ARP_ENTRIES** (line 12)

```rust
const ARP_ENTRIES: usize = 4;
```

---

**CONST: IDENTIFIER** (line 13)

```rust
const IDENTIFIER: u16 = 0x5452;
```

---

**STRUCT: NetworkRuntime** (line 23)

```rust
struct NetworkRuntime {
```

---

**STATIC: RUNTIME** (line 28)

```rust
static RUNTIME: Mutex<Option<NetworkRuntime>> = Mutex::new(None);
```

---

**FN: legacy_virtio_device** (line 67)

```rust
fn legacy_virtio_device() -> Option<PciDevice> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/stack.rs

**FN: process_arp** (line 149)

```rust
fn process_arp(
```

---

**FN: process_ipv4** (line 182)

```rust
fn process_ipv4(&mut self, frame: EthernetFrame<'_>) -> Result<StackEvent, PacketError> {
```

---

**CONST: LOCAL_MAC** (line 211)

```rust
const LOCAL_MAC: MacAddress = MacAddress([2, 0, 0, 0, 0, 1]);
```

---

**CONST: LOCAL_IP** (line 212)

```rust
const LOCAL_IP: Ipv4Address = Ipv4Address::new(10, 0, 0, 2);
```

---

**CONST: GATEWAY** (line 213)

```rust
const GATEWAY: Ipv4Address = Ipv4Address::new(10, 0, 0, 1);
```

---

**FN: stack** (line 215)

```rust
fn stack() -> NetworkStack<4> {
```

---

**FN: outside_subnet_uses_gateway** (line 226)

```rust
fn outside_subnet_uses_gateway() {
```

---

**FN: arp_request_and_ping_are_ethernet_padded** (line 232)

```rust
fn arp_request_and_ping_are_ethernet_padded() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/types.rs

**FN: fmt** (line 38)

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

---

**FN: fmt** (line 82)

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

---

**FN: subnet_check_uses_mask** (line 92)

```rust
fn subnet_check_uses_mask() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/device.rs

**FN: mac_address** (line 12)

```rust
fn mac_address(&self) -> MacAddress;
```

---

**FN: mtu** (line 13)

```rust
fn mtu(&self) -> usize;
```

---

**FN: capabilities** (line 14)

```rust
fn capabilities(&self) -> Capabilities;
```

---

**FN: transmit** (line 15)

```rust
fn transmit(&mut self, packet: &[u8]) -> Result<(), NetworkError>;
```

---

**FN: receive** (line 16)

```rust
fn receive(&mut self) -> Option<&[u8]>;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/net.rs

**CONST: MMIO_MAGIC_VALUE** (line 17)

```rust
const MMIO_MAGIC_VALUE: usize = 0x000;
```

---

**CONST: MMIO_VERSION** (line 18)

```rust
const MMIO_VERSION: usize = 0x004;
```

---

**CONST: MMIO_DEVICE_ID** (line 19)

```rust
const MMIO_DEVICE_ID: usize = 0x008;
```

---

**CONST: MMIO_DEVICE_FEATURES** (line 20)

```rust
const MMIO_DEVICE_FEATURES: usize = 0x010;
```

---

**CONST: MMIO_DRIVER_FEATURES** (line 21)

```rust
const MMIO_DRIVER_FEATURES: usize = 0x020;
```

---

**CONST: MMIO_GUEST_PAGE_SIZE** (line 22)

```rust
const MMIO_GUEST_PAGE_SIZE: usize = 0x028;
```

---

**CONST: MMIO_QUEUE_SEL** (line 23)

```rust
const MMIO_QUEUE_SEL: usize = 0x030;
```

---

**CONST: MMIO_QUEUE_NUM_MAX** (line 24)

```rust
const MMIO_QUEUE_NUM_MAX: usize = 0x034;
```

---

**CONST: MMIO_QUEUE_NUM** (line 25)

```rust
const MMIO_QUEUE_NUM: usize = 0x038;
```

---

**CONST: MMIO_QUEUE_ALIGN** (line 26)

```rust
const MMIO_QUEUE_ALIGN: usize = 0x03c;
```

---

**CONST: MMIO_QUEUE_PFN** (line 27)

```rust
const MMIO_QUEUE_PFN: usize = 0x040;
```

---

**CONST: MMIO_QUEUE_NOTIFY** (line 28)

```rust
const MMIO_QUEUE_NOTIFY: usize = 0x050;
```

---

**CONST: MMIO_INTERRUPT_STATUS** (line 29)

```rust
const MMIO_INTERRUPT_STATUS: usize = 0x060;
```

---

**CONST: MMIO_INTERRUPT_ACK** (line 30)

```rust
const MMIO_INTERRUPT_ACK: usize = 0x064;
```

---

**CONST: MMIO_STATUS** (line 31)

```rust
const MMIO_STATUS: usize = 0x070;
```

---

**CONST: MMIO_CONFIG** (line 32)

```rust
const MMIO_CONFIG: usize = 0x100;
```

---

**CONST: STATUS_ACKNOWLEDGE** (line 34)

```rust
const STATUS_ACKNOWLEDGE: u32 = 1;
```

---

**CONST: STATUS_DRIVER** (line 35)

```rust
const STATUS_DRIVER: u32 = 2;
```

---

**CONST: STATUS_DRIVER_OK** (line 36)

```rust
const STATUS_DRIVER_OK: u32 = 4;
```

---

**CONST: STATUS_FAILED** (line 37)

```rust
const STATUS_FAILED: u32 = 128;
```

---

**CONST: RX_QUEUE_INDEX** (line 38)

```rust
const RX_QUEUE_INDEX: u16 = 0;
```

---

**CONST: TX_QUEUE_INDEX** (line 39)

```rust
const TX_QUEUE_INDEX: u16 = 1;
```

---

**CONST: QUEUE_SIZE** (line 40)

```rust
const QUEUE_SIZE: u16 = 4;
```

---

**CONST: FRAME_BYTES** (line 41)

```rust
const FRAME_BYTES: usize = 1536;
```

---

**CONST: HEADER_BYTES** (line 42)

```rust
const HEADER_BYTES: usize = core::mem::size_of::<VirtioNetHeader>();
```

---

**CONST: DMA_FRAME_BYTES** (line 43)

```rust
const DMA_FRAME_BYTES: usize = HEADER_BYTES + FRAME_BYTES;
```

---

**CONST: PAGE_SIZE** (line 44)

```rust
const PAGE_SIZE: usize = 4096;
```

---

**STRUCT: LegacyQueue** (line 58)

```rust
struct LegacyQueue {
```

---

**CONST: fn** (line 66)

```rust
const fn empty() -> Self {
```

---

**FN: allocate** (line 75)

```rust
fn allocate(size: u16) -> Result<Self, NetworkError> {
```

---

**FN: descriptor_offset** (line 101)

```rust
fn descriptor_offset(&self, index: u16) -> usize {
```

---

**FN: avail_offset** (line 105)

```rust
fn avail_offset(&self) -> usize {
```

---

**FN: used_offset** (line 109)

```rust
fn used_offset(&self) -> usize {
```

---

**STRUCT: UsedElement** (line 164)

```rust
struct UsedElement {
```

---

**FN: initialize** (line 200)

```rust
fn initialize(&mut self) -> Result<(), NetworkError> {
```

---

**FN: configure_queue** (line 246)

```rust
fn configure_queue(&mut self, index: u16, queue: LegacyQueue) -> Result<(), NetworkError> {
```

---

**FN: post_rx** (line 260)

```rust
fn post_rx(&mut self, index: u16) -> Result<(), NetworkError> {
```

---

**FN: reclaim_tx** (line 282)

```rust
fn reclaim_tx(&mut self) -> u16 {
```

---

**FN: notify** (line 291)

```rust
fn notify(&self, queue: u16) {
```

---

**FN: fail** (line 295)

```rust
fn fail(&mut self) {
```

---

**FN: mmio_read** (line 300)

```rust
fn mmio_read(&self, offset: usize) -> u32 {
```

---

**FN: mmio_write** (line 304)

```rust
fn mmio_write(&self, offset: usize, value: u32) {
```

---

**FN: init** (line 312)

```rust
fn init(&mut self) -> Result<(), NetworkError> {
```

---

**FN: mac_address** (line 316)

```rust
fn mac_address(&self) -> MacAddress {
```

---

**FN: mtu** (line 320)

```rust
fn mtu(&self) -> usize {
```

---

**FN: submit_tx** (line 324)

```rust
fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError> {
```

---

**FN: poll** (line 358)

```rust
fn poll(&mut self) -> Result<PollResult, NetworkError> {
```

---

**FN: take_rx** (line 375)

```rust
fn take_rx(&mut self) -> Option<RxFrame<'_>> {
```

---

**FN: recycle_rx** (line 393)

```rust
fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError> {
```

---

**CONST: fn** (line 404)

```rust
const fn queue_bytes(queue_size: u16) -> usize {
```

---

**CONST: fn** (line 412)

```rust
const fn align_up(value: usize, align: usize) -> usize {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/pci_legacy.rs

**CONST: VIRTIO_NET_F_MAC** (line 13)

```rust
const VIRTIO_NET_F_MAC: u32 = 1 << 5;
```

---

**CONST: STATUS_ACKNOWLEDGE** (line 14)

```rust
const STATUS_ACKNOWLEDGE: u8 = 1;
```

---

**CONST: STATUS_DRIVER** (line 15)

```rust
const STATUS_DRIVER: u8 = 2;
```

---

**CONST: STATUS_DRIVER_OK** (line 16)

```rust
const STATUS_DRIVER_OK: u8 = 4;
```

---

**CONST: STATUS_FAILED** (line 17)

```rust
const STATUS_FAILED: u8 = 128;
```

---

**CONST: RX_QUEUE_INDEX** (line 18)

```rust
const RX_QUEUE_INDEX: u16 = 0;
```

---

**CONST: TX_QUEUE_INDEX** (line 19)

```rust
const TX_QUEUE_INDEX: u16 = 1;
```

---

**CONST: QUEUE_SIZE** (line 20)

```rust
const QUEUE_SIZE: u16 = 4;
```

---

**CONST: FRAME_BYTES** (line 21)

```rust
const FRAME_BYTES: usize = 1536;
```

---

**CONST: PAGE_SIZE** (line 22)

```rust
const PAGE_SIZE: usize = 4096;
```

---

**CONST: HEADER_BYTES** (line 23)

```rust
const HEADER_BYTES: usize = core::mem::size_of::<VirtioNetHeader>();
```

---

**CONST: DMA_FRAME_BYTES** (line 24)

```rust
const DMA_FRAME_BYTES: usize = HEADER_BYTES + FRAME_BYTES;
```

---

**CONST: REG_DEVICE_FEATURES** (line 26)

```rust
const REG_DEVICE_FEATURES: u16 = 0;
```

---

**CONST: REG_GUEST_FEATURES** (line 27)

```rust
const REG_GUEST_FEATURES: u16 = 4;
```

---

**CONST: REG_QUEUE_ADDRESS** (line 28)

```rust
const REG_QUEUE_ADDRESS: u16 = 8;
```

---

**CONST: REG_QUEUE_SIZE** (line 29)

```rust
const REG_QUEUE_SIZE: u16 = 12;
```

---

**CONST: REG_QUEUE_SELECT** (line 30)

```rust
const REG_QUEUE_SELECT: u16 = 14;
```

---

**CONST: REG_QUEUE_NOTIFY** (line 31)

```rust
const REG_QUEUE_NOTIFY: u16 = 16;
```

---

**CONST: REG_DEVICE_STATUS** (line 32)

```rust
const REG_DEVICE_STATUS: u16 = 18;
```

---

**CONST: REG_ISR_STATUS** (line 33)

```rust
const REG_ISR_STATUS: u16 = 19;
```

---

**CONST: REG_DEVICE_CONFIG** (line 34)

```rust
const REG_DEVICE_CONFIG: u16 = 20;
```

---

**STRUCT: VirtioNetHeader** (line 38)

```rust
struct VirtioNetHeader {
```

---

**STRUCT: LegacyQueue** (line 48)

```rust
struct LegacyQueue {
```

---

**CONST: fn** (line 56)

```rust
const fn empty() -> Self {
```

---

**FN: allocate** (line 65)

```rust
fn allocate(size: u16) -> Result<Self, NetworkError> {
```

---

**FN: descriptor_offset** (line 91)

```rust
fn descriptor_offset(&self, index: u16) -> usize {
```

---

**FN: avail_offset** (line 95)

```rust
fn avail_offset(&self) -> usize {
```

---

**FN: used_offset** (line 99)

```rust
fn used_offset(&self) -> usize {
```

---

**STRUCT: UsedElement** (line 154)

```rust
struct UsedElement {
```

---

**FN: initialize** (line 192)

```rust
fn initialize(&mut self) -> Result<(), NetworkError> {
```

---

**FN: configure_queue** (line 233)

```rust
fn configure_queue(&mut self, index: u16, queue: LegacyQueue) -> Result<(), NetworkError> {
```

---

**FN: post_rx** (line 245)

```rust
fn post_rx(&mut self, index: u16) -> Result<(), NetworkError> {
```

---

**FN: reclaim_tx** (line 267)

```rust
fn reclaim_tx(&mut self) -> u16 {
```

---

**FN: notify** (line 276)

```rust
fn notify(&self, queue: u16) {
```

---

**FN: fail** (line 280)

```rust
fn fail(&mut self) {
```

---

**FN: read_u8** (line 285)

```rust
fn read_u8(&self, offset: u16) -> u8 {
```

---

**FN: read_u16** (line 290)

```rust
fn read_u16(&self, offset: u16) -> u16 {
```

---

**FN: read_u32** (line 295)

```rust
fn read_u32(&self, offset: u16) -> u32 {
```

---

**FN: write_u8** (line 300)

```rust
fn write_u8(&self, offset: u16, value: u8) {
```

---

**FN: write_u16** (line 307)

```rust
fn write_u16(&self, offset: u16, value: u16) {
```

---

**FN: write_u32** (line 314)

```rust
fn write_u32(&self, offset: u16, value: u32) {
```

---

**FN: init** (line 323)

```rust
fn init(&mut self) -> Result<(), NetworkError> {
```

---

**FN: mac_address** (line 327)

```rust
fn mac_address(&self) -> MacAddress {
```

---

**FN: mtu** (line 331)

```rust
fn mtu(&self) -> usize {
```

---

**FN: submit_tx** (line 335)

```rust
fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError> {
```

---

**FN: poll** (line 369)

```rust
fn poll(&mut self) -> Result<PollResult, NetworkError> {
```

---

**FN: take_rx** (line 383)

```rust
fn take_rx(&mut self) -> Option<RxFrame<'_>> {
```

---

**FN: recycle_rx** (line 401)

```rust
fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError> {
```

---

**CONST: fn** (line 412)

```rust
const fn queue_bytes(queue_size: u16) -> usize {
```

---

**CONST: fn** (line 420)

```rust
const fn align_up(value: usize, align: usize) -> usize {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/queue.rs

**FN: default** (line 135)

```rust
fn default() -> Self {
```

---

**FN: pool_allocates_and_releases_chain_once** (line 173)

```rust
fn pool_allocates_and_releases_chain_once() {
```

---

**FN: zero_sized_pool_fails_cleanly** (line 186)

```rust
fn zero_sized_pool_fails_cleanly() {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nic/virtio/transport.rs

**FN: reset** (line 12)

```rust
fn reset(&mut self) -> Result<(), NetworkError>;
```

---

**FN: status** (line 13)

```rust
fn status(&self) -> u8;
```

---

**FN: set_status** (line 14)

```rust
fn set_status(&mut self, status: u8);
```

---

**FN: device_features** (line 16)

```rust
fn device_features(&self) -> u64;
```

---

**FN: set_driver_features** (line 17)

```rust
fn set_driver_features(&mut self, features: u64);
```

---

**FN: queue_max_size** (line 19)

```rust
fn queue_max_size(&self, queue_index: u16) -> u16;
```

---

**FN: configure_queue** (line 20)

```rust
fn configure_queue(&mut self, queue_index: u16, setup: QueueSetup) -> Result<(), NetworkError>;
```

---

**FN: notify_queue** (line 22)

```rust
fn notify_queue(&mut self, queue_index: u16);
```

---

**FN: read_config** (line 24)

```rust
fn read_config(&self, offset: u16, out: &mut [u8]) -> Result<(), NetworkError>;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/nimcore.rs

**FN: nim_u64_to_str** (line 2)

```rust
fn nim_u64_to_str(v: u64, base: u8, buf: *mut u8, cap: u32) -> u32;
```

---

**FN: nim_parse_u64** (line 3)

```rust
fn nim_parse_u64(s: *const u8, len: u32, base: u8, out: *mut u64) -> u8;
```

---

**FN: nim_rb_push** (line 4)

```rust
fn nim_rb_push(b: u8) -> u8;
```

---

**FN: nim_rb_pop** (line 5)

```rust
fn nim_rb_pop() -> i32;
```

---

**FN: nim_shell_register** (line 6)

```rust
fn nim_shell_register(name: *const u8, nlen: u32,
```

---

**FN: nim_shell_run** (line 8)

```rust
fn nim_shell_run(line: *const u8, len: u32) -> i32;
```

---

**FN: nim_banner** (line 9)

```rust
fn nim_banner(buf: *mut u8, cap: u32) -> u32;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/pci.rs

**CONST: CONFIG_ADDRESS** (line 7)

```rust
const CONFIG_ADDRESS: u16 = 0xCF8;
```

---

**CONST: CONFIG_DATA** (line 8)

```rust
const CONFIG_DATA: u16 = 0xCFC;
```

---

**FN: config_address** (line 28)

```rust
fn config_address(addr: PciAddress, offset: u8) -> u32 {
```

---

**FN: probe_function** (line 66)

```rust
fn probe_function(bus: u8, device: u8, function: u8) -> Option<PciDevice> {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/policy/bridge.rs

**FN: policy_evaluate** (line 2)

```rust
fn policy_evaluate(ring: u8, cls: u8, op: u8, arg: u64) -> u8;
```

---

**FN: nim_policy_log** (line 3)

```rust
fn nim_policy_log(ring: u8, cls: u8, op: u8, dec: u8);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/policy/mod.rs

**CONST: LOG_CAP** (line 66)

```rust
const LOG_CAP: usize = 256;
```

---

**STRUCT: LogInner** (line 68)

```rust
struct LogInner {
```

---

**STATIC: LOG** (line 75)

```rust
static LOG: Mutex<LogInner> = Mutex::new(LogInner {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/process/fd.rs

**CONST: POOL_SLOTS** (line 5)

```rust
const POOL_SLOTS: usize = 4;
```

---

**CONST: POOL_SIZE** (line 6)

```rust
const POOL_SIZE: usize = 16 * 1024;
```

---

**STATIC: mut** (line 28)

```rust
static mut POOL: [[u8; POOL_SIZE]; POOL_SLOTS] = [[0; POOL_SIZE]; POOL_SLOTS];
```

---

**STATIC: mut** (line 29)

```rust
static mut POOL_USED: [bool; POOL_SLOTS] = [false; POOL_SLOTS];
```

---

**FN: kprintf** (line 32)

```rust
fn kprintf(fmt: *const u8, ...);
```

---

**FN: pool_take** (line 35)

```rust
fn pool_take() -> Option<i16> {
```

---

**FN: pool_drop** (line 48)

```rust
fn pool_drop(slot: i16) {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/serial.rs

**CONST: COM1** (line 7)

```rust
const COM1: u16 = 0x3F8;
```

---

**FN: tx_empty** (line 27)

```rust
fn tx_empty() -> bool {
```

---

**CONST: BASE** (line 43)

```rust
const BASE: usize = 0x1000_0000;
```

---

**CONST: THR** (line 44)

```rust
const THR: usize = 0;
```

---

**CONST: LSR** (line 45)

```rust
const LSR: usize = 5;
```

---

**FN: tx_empty** (line 50)

```rust
fn tx_empty() -> bool {
```

---

**FN: write_str** (line 83)

```rust
fn write_str(&mut self, s: &str) -> fmt::Result {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/terminal/mod.rs

**FN: editor_run** (line 7)

```rust
fn editor_run(path: *const u8) -> i32;
```

---

**CONST: KBUF_SIZE** (line 11)

```rust
const KBUF_SIZE: usize = 256;
```

---

**STATIC: KBUF** (line 12)

```rust
static KBUF: [AtomicU8; KBUF_SIZE] = [const { AtomicU8::new(0) }; KBUF_SIZE];
```

---

**STATIC: KHEAD** (line 13)

```rust
static KHEAD: AtomicUsize = AtomicUsize::new(0);
```

---

**STATIC: KTAIL** (line 14)

```rust
static KTAIL: AtomicUsize = AtomicUsize::new(0);
```

---

**STATIC: SHIFT** (line 15)

```rust
static SHIFT: AtomicBool = AtomicBool::new(false);
```

---

**CONST: KCODE_SIZE** (line 18)

```rust
const KCODE_SIZE: usize = 64;
```

---

**STATIC: KCODEBUF** (line 19)

```rust
static KCODEBUF: [AtomicU32; KCODE_SIZE] = [const { AtomicU32::new(0) }; KCODE_SIZE];
```

---

**STATIC: KCODE_HEAD** (line 20)

```rust
static KCODE_HEAD: AtomicUsize = AtomicUsize::new(0);
```

---

**STATIC: KCODE_TAIL** (line 21)

```rust
static KCODE_TAIL: AtomicUsize = AtomicUsize::new(0);
```

---

**STATIC: CAPTURE_KEYCODE** (line 22)

```rust
static CAPTURE_KEYCODE: AtomicBool = AtomicBool::new(false);
```

---

**FN: kbuf_push** (line 24)

```rust
fn kbuf_push(c: u8) {
```

---

**FN: kbuf_pop** (line 34)

```rust
fn kbuf_pop() -> Option<u8> {
```

---

**FN: scancode_to_char** (line 45)

```rust
fn scancode_to_char(code: u8) -> Option<char> {
```

---

**FN: scancode_to_keycode** (line 133)

```rust
fn scancode_to_keycode(code: u8) -> Option<u32> {
```

---

**FN: keycode_push** (line 152)

```rust
fn keycode_push(k: u32) {
```

---

**CONST: MAX_LINE** (line 181)

```rust
const MAX_LINE: usize = 120;
```

---

**CONST: PROMPT** (line 182)

```rust
const PROMPT: &str = "#$-=>";
```

---

**CONST: CURSOR** (line 183)

```rust
const CURSOR: &str = "_";
```

---

**CONST: CONSOLE_COLOR** (line 184)

```rust
const CONSOLE_COLOR: Color = Color::Green;
```

---

**CONST: INPUT_COLOR** (line 185)

```rust
const INPUT_COLOR: Color = Color::LightBlue;
```

---

**STATIC: LINE** (line 187)

```rust
static LINE: [AtomicU8; MAX_LINE] = [const { AtomicU8::new(0) }; MAX_LINE];
```

---

**STATIC: LINE_LEN** (line 188)

```rust
static LINE_LEN: AtomicUsize = AtomicUsize::new(0);
```

---

**STATIC: LINE_START_COL** (line 189)

```rust
static LINE_START_COL: AtomicUsize = AtomicUsize::new(0);
```

---

**STATIC: CURRENT_DIR** (line 192)

```rust
static CURRENT_DIR: AtomicU32 = AtomicU32::new(crate::fs::tfs::ROOT_DIR);
```

---

**STATIC: NET_TIME_MS** (line 193)

```rust
static NET_TIME_MS: AtomicU64 = AtomicU64::new(0);
```

---

**FN: line_as_str** (line 195)

```rust
fn line_as_str(buf: &mut [u8]) -> &str {
```

---

**FN: line_clear** (line 203)

```rust
fn line_clear() {
```

---

**FN: draw_prompt** (line 207)

```rust
fn draw_prompt() {
```

---

**FN: redraw_line** (line 226)

```rust
fn redraw_line() {
```

---

**FN: handle_char** (line 247)

```rust
fn handle_char(c: char) {
```

---

**FN: split_once_space** (line 296)

```rust
fn split_once_space(s: &str) -> (&str, &str) {
```

---

**FN: parse_resolution** (line 304)

```rust
fn parse_resolution(s: &str) -> Option<(u32, u32)> {
```

---

**FN: dev** (line 315)

```rust
fn dev() -> Option<&'static dyn BlockDevice> {
```

---

**FN: execute** (line 319)

```rust
fn execute(line: &str) {
```

---

**FN: print_ping_result** (line 528)

```rust
fn print_ping_result(result: crate::nic::PingResult) {
```

---

**FN: poll_network** (line 544)

```rust
fn poll_network() {
```

---

**STRUCT: Sink** (line 553)

```rust
struct Sink;
```

---

**FN: write_str** (line 556)

```rust
fn write_str(&mut self, s: &str) -> core::fmt::Result {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/trampoline_rings/arch/aarch64/trampoline_rings.rs

**FN: tr_init** (line 23)

```rust
fn tr_init();
```

---

**CONST: MAX_WORLDS** (line 34)

```rust
const MAX_WORLDS: usize = 4;
```

---

**STATIC: mut** (line 36)

```rust
static mut WORLDS: [Option<World>; MAX_WORLDS] = [None, None, None, None];
```

---

**STATIC: mut** (line 37)

```rust
static mut CURRENT: Option<usize> = None;
```

---

**STATIC: mut** (line 38)

```rust
static mut TICK: u64 = 0;
```

---

**FN: write_ttbr0** (line 68)

```rust
fn write_ttbr0(v: u64) {
```

---

**FN: pick_next** (line 74)

```rust
fn pick_next(from: Option<usize>) -> usize {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/trampoline_rings/arch/risc-v/trampoline_rings.rs

**FN: tr_init** (line 25)

```rust
fn tr_init(kernel_stack_top: u64);
```

---

**CONST: MAX_WORLDS** (line 36)

```rust
const MAX_WORLDS: usize = 4;
```

---

**STATIC: mut** (line 38)

```rust
static mut WORLDS: [Option<World>; MAX_WORLDS] = [None, None, None, None];
```

---

**STATIC: mut** (line 39)

```rust
static mut CURRENT: Option<usize> = None;
```

---

**STATIC: mut** (line 40)

```rust
static mut TICK: u64 = 0;
```

---

**STATIC: mut** (line 42)

```rust
static mut KSTACK: [u64; 4096] = [0; 4096];
```

---

**FN: write_satp** (line 73)

```rust
fn write_satp(v: u64) {
```

---

**FN: pick_next** (line 79)

```rust
fn pick_next(from: Option<usize>) -> usize {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/trampoline_rings/arch/x86_64/trampoline_rings.rs

**FN: tr_init** (line 29)

```rust
fn tr_init(rsp0: u64);
```

---

**STATIC: mut** (line 31)

```rust
static mut tr_tss_desc: [u64; 2];
```

---

**STATIC: mut** (line 32)

```rust
static mut isr_hypercall: u8;
```

---

**STATIC: mut** (line 33)

```rust
static mut isr_default: u8;
```

---

**STRUCT: IdtGate** (line 37)

```rust
struct IdtGate {
```

---

**STATIC: mut** (line 47)

```rust
static mut IDT: [IdtGate; 256] = [IdtGate {
```

---

**FN: set_gate** (line 51)

```rust
fn set_gate(i: usize, handler: u64, dpl: u8) {
```

---

**CONST: MAX_WORLDS** (line 72)

```rust
const MAX_WORLDS: usize = 4;
```

---

**STATIC: mut** (line 74)

```rust
static mut WORLDS: [Option<World>; MAX_WORLDS] = [None, None, None, None];
```

---

**STATIC: mut** (line 75)

```rust
static mut CURRENT: Option<usize> = None;
```

---

**STATIC: mut** (line 76)

```rust
static mut TICK: u64 = 0;
```

---

**STATIC: mut** (line 78)

```rust
static mut KSTACK: [u64; 4096] = [0; 4096];
```

---

**FN: sel_for** (line 80)

```rust
fn sel_for(ring: u8) -> (u64, u64) {
```

---

**STRUCT: IdtPtr** (line 122)

```rust
struct IdtPtr {
```

---

**STATIC: mut** (line 127)

```rust
static mut IDT_PTR: IdtPtr = IdtPtr { limit: 0, base: 0 };
```

---

**FN: write_cr3** (line 154)

```rust
fn write_cr3(v: u64) {
```

---

**FN: pick_next** (line 158)

```rust
fn pick_next(from: Option<usize>) -> usize {
```

---

**FN: ctx_mut** (line 227)

```rust
fn ctx_mut(&mut self) -> &mut CpuCtx {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/userspace/process/proc.rs

**STATIC: mut** (line 24)

```rust
static mut PROCS: [Option<Process>; MAX_PROCS] =
```

---

**STATIC: mut** (line 27)

```rust
static mut NEXT_PID: u32 = 1;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/userspace/process/runcl.rs

**FN: k_fs_read** (line 2)

```rust
fn k_fs_read(path: *const u8, buf: *mut u8, cap: u32) -> i32;
```

---

**FN: cl_compile_source** (line 4)

```rust
fn cl_compile_source(src: *const u8, len: usize, ar: *mut u8,
```

---

**FN: cl_vm_init** (line 8)

```rust
fn cl_vm_init(vm: *mut u8, prog: *mut u8);
```

---

**FN: cl_bridge_init** (line 9)

```rust
fn cl_bridge_init(vm: *mut u8, ring: u8) -> i32;
```

---

**FN: cl_vm_run** (line 10)

```rust
fn cl_vm_run(vm: *mut u8) -> i32;
```

---

**STATIC: mut** (line 13)

```rust
static mut SRC: [u8; 65536] = [0; 65536];
```

---

**STRUCT: ArenaBuf** (line 16)

```rust
struct ArenaBuf([u8; 196608]);
```

---

**STATIC: mut** (line 17)

```rust
static mut ARENA: ArenaBuf = ArenaBuf([0; 196608]);
```

---

**STRUCT: ProgBuf** (line 20)

```rust
struct ProgBuf([u8; 65536]);
```

---

**STATIC: mut** (line 21)

```rust
static mut PROG: ProgBuf = ProgBuf([0; 65536]);
```

---

**STRUCT: VmBuf** (line 24)

```rust
struct VmBuf([u8; 98304]);
```

---

**STATIC: mut** (line 25)

```rust
static mut VM: VmBuf = VmBuf([0; 98304]);
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/userspace/process/spawn.rs

**CONST: USER_STACK_TOP** (line 4)

```rust
const USER_STACK_TOP: u64 = 0x7FFF_0000_0000;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/userspace/process/syscall.rs

**FN: paging_translate_in** (line 17)

```rust
fn paging_translate_in(pml4: u64, virt: u64) -> u64;
```

---

**FN: kprintf** (line 18)

```rust
fn kprintf(fmt: *const u8, ...);
```

---

**CONST: DIRECT_BASE** (line 21)

```rust
const DIRECT_BASE: u64 = 0xFFFF888000000000;
```

---

**FN: user_copy_in** (line 23)

```rust
fn user_copy_in(cr3: u64, src: u64, dst: &mut [u8]) -> bool {
```

---

**FN: user_cstr** (line 50)

```rust
fn user_cstr(cr3: u64, ptr: u64, buf: &mut [u8]) -> Option<&str> {
```

---

**STATIC: mut** (line 68)

```rust
static mut ELF_BUF: [u8; 64 * 1024] = [0; 64 * 1024];
```

---

**FN: do_spawn** (line 70)

```rust
fn do_spawn(cr3: u64, path_ptr: u64, parent_pid: u32) -> i64 {
```

---

**FN: hdmi_caps_raw** (line 207)

```rust
fn hdmi_caps_raw(w: *mut u32, h: *mut u32,
```

---

**FN: kvirt_to_phys** (line 209)

```rust
fn kvirt_to_phys(p: *const u8) -> u64;
```

---

**FN: paging_map_page_in** (line 210)

```rust
fn paging_map_page_in(pml4: u64, virt: u64,
```

---

**STATIC: font8x8** (line 215)

```rust
static font8x8: u8;
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/vga_buffer.rs

**STRUCT: ColorCode** (line 34)

```rust
struct ColorCode(u8);
```

---

**FN: new** (line 37)

```rust
fn new(foreground: Color, background: Color) -> ColorCode {
```

---

**STRUCT: ScreenChar** (line 44)

```rust
struct ScreenChar {
```

---

**CONST: BUFFER_HEIGHT** (line 49)

```rust
const BUFFER_HEIGHT: usize = 25;
```

---

**CONST: BUFFER_WIDTH** (line 50)

```rust
const BUFFER_WIDTH: usize = 80;
```

---

**STRUCT: Buffer** (line 53)

```rust
struct Buffer {
```

---

**FN: new_line** (line 83)

```rust
fn new_line(&mut self) {
```

---

**FN: clear_row** (line 94)

```rust
fn clear_row(&mut self, row: usize) {
```

---

**FN: write_str** (line 153)

```rust
fn write_str(&mut self, s: &str) -> fmt::Result {
```

---

**CONST: BLANK** (line 159)

```rust
const BLANK: ScreenChar = ScreenChar {
```

---

**STATIC: mut** (line 164)

```rust
static mut TEXT_BUFFER: Buffer = Buffer {
```

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel-bin/src/main.rs

**FN: kernel_main** (line 8)

```rust
fn kernel_main(boot_info: &'static BootInfo) -> ! {
```

---

### /home/ctrl/TrangorgeOS/mp4_to_bmp/src/main.rs

**FN: main** (line 23)

```rust
fn main() {
```

---

**FN: run** (line 30)

```rust
fn run() -> Result<(), String> {
```

---

**FN: check_tool** (line 103)

```rust
fn check_tool(name: &str) -> Result<(), String> {
```

---

**FN: probe_resolution** (line 114)

```rust
fn probe_resolution(input: &str) -> Result<(usize, usize), String> {
```

Pobiera szerokosc i wysokosc pierwszego strumienia wideo przez ffprobe.

---

**FN: read_exact_or_eof** (line 152)

```rust
fn read_exact_or_eof<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<bool, String> {
```

Czyta dokladnie `buf.len()` bajtow. Zwraca Ok(true) jesli sie udalo,
Ok(false) jesli strumien skonczyl sie dokladnie na granicy klatki
(koniec pliku), Err jesli urwal sie w polowie klatki (uszkodzony strumien).

---

**FN: write_bmp** (line 173)

```rust
fn write_bmp(path: &str, pixels: &[u8], width: usize, height: usize) -> std::io::Result<()> {
```

Zapisuje surowa, nieskompresowana bitmape 24-bit BGR (BITMAPFILEHEADER +
BITMAPINFOHEADER + piksele). `pixels` to width*height*3 bajtow w kolejnosci
B,G,R, wiersz po wierszu, gora->dol (uzywamy ujemnej wysokosci w naglowku,

---

### /home/ctrl/TrangorgeOS/trangorgelibc/src/lib.rs

**FN: sc0** (line 31)

```rust
fn sc0(n: u64) -> u64 {
```

---

**FN: sc1** (line 38)

```rust
fn sc1(n: u64, a0: u64) -> u64 {
```

---

**FN: sc3** (line 45)

```rust
fn sc3(n: u64, a0: u64, a1: u64, a2: u64) -> u64 {
```

---

**FN: cstr** (line 58)

```rust
fn cstr(s: &str) -> ([u8; 256], usize) {
```

Kopiuje `s` do lokalnego bufora zakonczonego NUL.

Jadro czyta sciezki/teksty jako C-stringi, a `&str` z Rusta nie ma bajtu

---

**STATIC: mut** (line 239)

```rust
static mut HEAP: [u8; 256 * 1024] = [0; 256 * 1024];
```

---

**STATIC: mut** (line 240)

```rust
static mut HEAP_POS: usize = 0;
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/codegen/asm.rs

**CONST: SCRATCH_A** (line 4)

```rust
const SCRATCH_A: &str = "r14";
```

---

**CONST: SCRATCH_B** (line 5)

```rust
const SCRATCH_B: &str = "r15";
```

---

**CONST: SCRATCH_C** (line 6)

```rust
const SCRATCH_C: &str = "r12";
```

---

**STRUCT: Emitter** (line 8)

```rust
struct Emitter {
```

---

**FN: label** (line 14)

```rust
fn label(&mut self, tag: &str) -> String {
```

---

**FN: line** (line 19)

```rust
fn line(&mut self, s: &str) {
```

---

**FN: op** (line 26)

```rust
fn op(t: &Target, v: &Val) -> String {
```

---

**FN: mem_len** (line 33)

```rust
fn mem_len(ir: &[Ir], name: &str) -> u64 {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/codegen/c.rs

**STRUCT: FnChunk** (line 4)

```rust
struct FnChunk {
```

---

**FN: val** (line 11)

```rust
fn val(v: &Val) -> String {
```

---

**FN: binop** (line 18)

```rust
fn binop(op: BinOp) -> &'static str {
```

---

**FN: cmpop** (line 30)

```rust
fn cmpop(op: CmpOp) -> &'static str {
```

---

**FN: is_param** (line 37)

```rust
fn is_param(name: &str, args: usize) -> bool {
```

---

**FN: split** (line 47)

```rust
fn split(ir: &[Ir]) -> Vec<FnChunk> {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/ir.rs

**FN: label** (line 58)

```rust
fn label(&mut self, tag: &str) -> String {
```

---

**FN: lower_function** (line 70)

```rust
fn lower_function(&mut self, f: &Function) {
```

---

**FN: lower_stmt** (line 83)

```rust
fn lower_stmt(&mut self, s: &Stmt) {
```

---

**FN: lower_op** (line 143)

```rust
fn lower_op(&mut self, call: &OpCall) {
```

---

**FN: val** (line 263)

```rust
fn val(e: &Expr) -> Val {
```

---

**FN: invert** (line 271)

```rust
fn invert(op: CmpOp) -> CmpOp {
```

---

**FN: binop** (line 278)

```rust
fn binop(s: &str) -> BinOp {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/iso.rs

**CONST: SECTOR** (line 6)

```rust
const SECTOR: usize = 2048;
```

---

**FN: dstring** (line 8)

```rust
fn dstring(s: &str, len: usize) -> Vec<u8> {
```

---

**FN: push_both_u16** (line 18)

```rust
fn push_both_u16(out: &mut Vec<u8>, v: u16) {
```

---

**FN: push_both_u32** (line 23)

```rust
fn push_both_u32(out: &mut Vec<u8>, v: u32) {
```

---

**FN: put_both_u16** (line 28)

```rust
fn put_both_u16(buf: &mut [u8], off: usize, v: u16) {
```

---

**FN: put_both_u32** (line 33)

```rust
fn put_both_u32(buf: &mut [u8], off: usize, v: u32) {
```

---

**FN: dir_record** (line 38)

```rust
fn dir_record(name: &[u8], extent: u32, size: u32, flags: u8) -> Vec<u8> {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/lexer.rs

**FN: peek** (line 28)

```rust
fn peek(&self) -> Option<char> {
```

---

**FN: peek2** (line 32)

```rust
fn peek2(&self) -> Option<char> {
```

---

**FN: bump** (line 36)

```rust
fn bump(&mut self) -> Option<char> {
```

---

**FN: skip_ws_and_comments** (line 98)

```rust
fn skip_ws_and_comments(&mut self) {
```

---

**FN: read_number** (line 114)

```rust
fn read_number(&mut self) -> Result<TokenKind, LexError> {
```

---

**FN: read_ident** (line 161)

```rust
fn read_ident(&mut self) -> TokenKind {
```

---

**FN: is_ident_start** (line 192)

```rust
fn is_ident_start(c: char) -> bool {
```

---

**FN: is_ident_cont** (line 196)

```rust
fn is_ident_cont(c: char) -> bool {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/main.rs

**FN: main** (line 18)

```rust
fn main() {
```

---

**FN: read_libs** (line 29)

```rust
fn read_libs(dir: &str) -> Vec<String> {
```

---

**FN: pipeline** (line 50)

```rust
fn pipeline(srcs: &[String]) -> Vec<ir::Ir> {
```

---

**FN: cmd_new** (line 83)

```rust
fn cmd_new(args: &[String]) {
```

---

**FN: cmd_build** (line 114)

```rust
fn cmd_build(args: &[String]) {
```

---

**FN: cmd_file** (line 178)

```rust
fn cmd_file(args: &[String]) {
```

---

**FN: cmd_iso** (line 256)

```rust
fn cmd_iso(_args: &[String]) {
```

---

**FN: route_c** (line 288)

```rust
fn route_c(ir: &[ir::Ir], text_path: &str, elf_path: &str, bin_path: &str, want_elf: bool, want_bin: bool) {
```

---

**FN: route_asm** (line 309)

```rust
fn route_asm(
```

---

**FN: run_objcopy** (line 357)

```rust
fn run_objcopy(elf: &str, bin: &str) -> bool {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/parser.rs

**FN: cur** (line 21)

```rust
fn cur(&self) -> &Token {
```

---

**FN: kind** (line 25)

```rust
fn kind(&self) -> TokenKind {
```

---

**FN: at** (line 29)

```rust
fn at(&self, kind: &TokenKind) -> bool {
```

---

**FN: bump** (line 33)

```rust
fn bump(&mut self) -> TokenKind {
```

---

**FN: err** (line 41)

```rust
fn err(&self, msg: String) -> ParseError {
```

---

**FN: expect** (line 49)

```rust
fn expect(&mut self, want: TokenKind) -> Result<TokenKind, ParseError> {
```

---

**FN: expect_ident** (line 57)

```rust
fn expect_ident(&mut self) -> Result<String, ParseError> {
```

---

**FN: expect_int** (line 64)

```rust
fn expect_int(&mut self) -> Result<u64, ParseError> {
```

---

**FN: parse_function** (line 79)

```rust
fn parse_function(&mut self) -> Result<Function, ParseError> {
```

---

**FN: parse_block** (line 111)

```rust
fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
```

---

**FN: parse_stmt** (line 120)

```rust
fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
```

---

**FN: parse_type** (line 134)

```rust
fn parse_type(&mut self) -> Result<Type, ParseError> {
```

---

**FN: parse_layout** (line 148)

```rust
fn parse_layout(&mut self) -> Result<Layout, ParseError> {
```

---

**FN: parse_const** (line 160)

```rust
fn parse_const(&mut self) -> Result<Stmt, ParseError> {
```

---

**FN: parse_static** (line 169)

```rust
fn parse_static(&mut self) -> Result<Stmt, ParseError> {
```

---

**FN: parse_reg** (line 187)

```rust
fn parse_reg(&mut self) -> Result<Stmt, ParseError> {
```

---

**FN: parse_mem** (line 195)

```rust
fn parse_mem(&mut self) -> Result<Stmt, ParseError> {
```

---

**FN: parse_target** (line 207)

```rust
fn parse_target(&mut self) -> Result<Target, ParseError> {
```

---

**FN: parse_op_call** (line 219)

```rust
fn parse_op_call(&mut self) -> Result<OpCall, ParseError> {
```

---

**FN: parse_op_suffix** (line 224)

```rust
fn parse_op_suffix(&mut self, target: Target) -> Result<OpCall, ParseError> {
```

---

**FN: parse_op_stmt** (line 239)

```rust
fn parse_op_stmt(&mut self) -> Result<Stmt, ParseError> {
```

---

**FN: parse_expr** (line 245)

```rust
fn parse_expr(&mut self) -> Result<Expr, ParseError> {
```

---

**FN: parse_cond** (line 266)

```rust
fn parse_cond(&mut self) -> Result<Cond, ParseError> {
```

---

**FN: parse_if** (line 277)

```rust
fn parse_if(&mut self) -> Result<Stmt, ParseError> {
```

---

**FN: parse_while** (line 294)

```rust
fn parse_while(&mut self) -> Result<Stmt, ParseError> {
```

---

**FN: parse_return** (line 305)

```rust
fn parse_return(&mut self) -> Result<Stmt, ParseError> {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/project.rs

**FN: parse** (line 19)

```rust
fn parse(text: &str) -> Project {
```

---

### /home/ctrl/TrangorgeOS/triang-lang/src/sema.rs

**FN: err** (line 28)

```rust
fn err(&self, msg: String) -> SemaError {
```

---

**FN: check_function** (line 43)

```rust
fn check_function(&mut self, f: &Function) -> Result<(), SemaError> {
```

---

**FN: declare** (line 62)

```rust
fn declare(&mut self, name: &str, sym: Symbol) -> Result<(), SemaError> {
```

---

**FN: check_stmt** (line 70)

```rust
fn check_stmt(&mut self, s: &Stmt) -> Result<(), SemaError> {
```

---

**FN: check_cond** (line 104)

```rust
fn check_cond(&self, cond: &Cond) -> Result<(), SemaError> {
```

---

**FN: check_operand** (line 109)

```rust
fn check_operand(&self, e: &Expr) -> Result<(), SemaError> {
```

---

**FN: check_mem_access** (line 125)

```rust
fn check_mem_access(&self, name: &str, idx: u64) -> Result<(), SemaError> {
```

---

**FN: expect_arity** (line 141)

```rust
fn expect_arity(&self, call: &OpCall, n: usize) -> Result<(), SemaError> {
```

---

**FN: check_op** (line 149)

```rust
fn check_op(&self, call: &OpCall) -> Result<(), SemaError> {
```

---

### /home/ctrl/TrangorgeOS/userspace-legasy/demo/src/main.rs

**FN: panic** (line 24)

```rust
fn panic(_i: &core::panic::PanicInfo) -> ! { loop {} }
```

---

### /home/ctrl/TrangorgeOS/userspace-legasy/init/src/main.rs

**FN: load_autostart** (line 6)

```rust
fn load_autostart() -> bool {
```

---

**FN: panic** (line 73)

```rust
fn panic(_i: &core::panic::PanicInfo) -> ! {
```

---

### /home/ctrl/TrangorgeOS/userspace-legasy/shell/src/main.rs

**STATIC: mut** (line 9)

```rust
static mut LINE: [u8; 128] = [0; 128];
```

---

**STATIC: mut** (line 10)

```rust
static mut LEN: usize = 0;
```

---

**FN: cstr** (line 13)

```rust
fn cstr(buf: &[u8]) -> &str {
```

Zamienia bufor zakonczony NUL na `&str` (nazwy z `readdir`).

---

**FN: run** (line 48)

```rust
fn run(cmd: &str) {
```

---

**FN: panic** (line 141)

```rust
fn panic(_i: &core::panic::PanicInfo) -> ! { loop {} }
```

---

### /home/ctrl/TrangorgeOS/userspace-legasy/terminal/src/main.rs

**CONST: BG** (line 9)

```rust
const BG: u32 = 0xFF0A0A12;
```

---

**CONST: FG** (line 10)

```rust
const FG: u32 = 0xFFD0D0D0;
```

---

**CONST: ACC** (line 11)

```rust
const ACC: u32 = 0xFF4EC9B0;
```

---

**STATIC: mut** (line 13)

```rust
static mut HIST: [[u8; 64]; 24] = [[0; 64]; 24];
```

---

**STATIC: mut** (line 14)

```rust
static mut HN: usize = 0;
```

---

**FN: push_hist** (line 16)

```rust
fn push_hist(line: &[u8]) {
```

---

**FN: panic** (line 107)

```rust
fn panic(_i: &core::panic::PanicInfo) -> ! {
```

---

## 3. Markdown Documentation

### /home/ctrl/TrangorgeOS/README.md

# TrangorgeOS

# TrangorgeOS

> A modern, high-performance bare-metal operating system built ground-up on a custom **Separated Tri-Partition Architecture** (*Architektura Trójpodziału Rozdzielnego*).

---

## Overview & Philosophy

TrangorgeOS is built with a strict **from-scratch philosophy** (~35,000+ lines of code, actively expanding). It discards classic microkernel overhead and hybrid kernel bloat in favor of a strictly isolated, policy-enforced driver architecture. 

The project operates under an intensive development cycle aimed at producing a stable, bare-metal kernel fully functional on physical hardware.

---

## Visuals & Screenshots

| Kernel Loading & Memory Allocation Testing | Kernel Base Resolution | Kernel in 1080p |
|:---:|:---:|:---:|
| ![Kernel Loading](kernelloading.png) | ![Kernel Base Res](kernel_loader_base_res.png) | ![Kernel 1080p](kernelin1920x1080.png) |

---

## Architectural Model: Separated Tri-Partition Architecture

TrangorgeOS decouples driver functionality and system privilege into 4 distinct operational layers (3 main domains + userspace abstraction):


```

+-----------------------------------------------------------------+
|                         USERSPACE                               |
|   - Applications & High-Level Libraries                         |
+-----------------------------------------------------------------+
|                   USER DRIVER SPACE (UDS)                       |
|   - High-risk / Peripherals (Fault-isolated, strict boundary)   |
+-----------------------------------------------------------------+
|                     DRIVER SPACE (DS)                           |
|   * Dynamic Drivers   : GPU & complex hardware                  |
|   * Static Drivers    : Init & single-action hardware setup       |
|   * Library Drivers   : Inter-driver interface providers        |
+-----------------------------------------------------------------+
|                    KERNEL CORE / DRIVERS                        |
|   - Trusted Core Drivers (Network, USB stack base, Core MM)     |
|   - Fine-grained Kernel Policy Enforcement & Data Flow Control  |
+-----------------------------------------------------------------+

```

1. **Kernel Core & Trusted Drivers:** Holds only maximum-trust drivers (e.g., base network, USB stack core) to eliminate IPC latency for essential paths without compromising core stability. Exports explicit system interfaces.
2. **Driver Space (DS):** Modular driver execution environment with strict error margins:
   - **Dynamic Drivers:** Handle complex, stateful hardware (e.g., GPU control).
   - **Static Drivers:** Perform hardware initialization or non-exporting, single-purpose setup.
   - **Library Drivers:** Expose specialized interfaces (e.g., PCI, HDMI control) for other drivers to consume.
3. **User Driver Space (UDS):** High-level peripheral drivers isolated at a safe distance from the core to prevent system crashes on fault.
4. **Userspace & User Library Drivers:** Top-le

*[... content truncated ...]*


---

### /home/ctrl/TrangorgeOS/TrangorgeOS — TODO.md

# TrangorgeOS — TODO

> **Purpose:** This file tracks the remaining work required to move TrangorgeOS from the current `unstable` development state toward a reliable, testable, and eventually stable system.
>
> **Status convention:** `TODO` means not started or not verified; `WIP` means partially implemented; `BLOCKED` means dependent on another task; `DONE` should only be used after the stated acceptance criteria have been verified.
>
> **Priority convention:** `P0` blocks reliable boot or basic correctness; `P1` is required for the next serious development milestone; `P2` improves robustness and maintainability; `P3` is ecosystem or long-term work.

## 0. Current snapshot

| Area | Current state | Priority |
|---|---|---|
| Kernel boot and x86_64 baseline | Present, but requires repeatable build and boot verification | P0 |
| Memory subsystem | Layered implementation exists; initialization order is defined in `mm_init()` | P0 |
| Heap allocator | Buddy path is active; slab implementation exists but `HEAP_USE_SLAB` is `0` | P1 |
| Driver space | Separate address space, shared rings, initialization parameters, and services are present | P0 |
| Driver-space ABI | Implemented enough for experiments, not yet documented as stable | P0 |
| Filesystem | FAT32/EXT4 and additional filesystem code are present; correctness and integration tests are needed | P1 |
| USB/PCI | Significant infrastructure exists; device coverage and hardware validation are incomplete | P1 |
| Networking | NIC/VirtIO structures exist; network-driver work is explicitly a project priority | P1 |
| ARM64 and RISC-V | Public target goals; feature parity and build/boot status must be established | P2 |
| Package manager | Kernel-side `ctrlinstall` modules exist; end-to-end package workflow needs completion | P2 |
| Toolchain and ecosystem | `triang-lang`, libraries, ISO tooling, and auxiliary tools exist at different maturity levels | P2/P3 |
| Release process | No published release; stable branch criteria are not yet formalized | P1 |

## 1. P0 — immediate correctness blockers

### 1.1 Repair and compile-check driver-space initialization

**Status:** TODO  
**Area:** `kernel/src/driverspaceinit/init/init.rs`

The visible implementation creates a `Driverspace` structure with an address space, two ring physical addresses, an initialization-parameter page, and a preparation flag. Later helper functions reference scratch-page state that is not visibly represented in the structure. Resolve this inconsistency rather than masking it with an unrelated workaround.

**Tasks:**

- Add or remove scratch-page state consistently in the `Driverspace` structure and initialization path.
- Ensure the scratch page is allocated, zeroed, mapped, and released exactly once.
- Ensure `scratch_view()` cannot return a pointer after the driver-space instance has been destroyed.
- Ensure every failure after partial allocation releases previously allocated frames and address-space mappings.
- Compile the kernel

*[... content truncated ...]*


---

### /home/ctrl/TrangorgeOS/driverspace_workspace/Todo.md

# TODO.md: Driver Space Migration and Implementation Guide

## Overview
This document outlines the pending tasks, architectural constraints, and execution steps for migrating and implementing hardware drivers within the newly structured `driverspace_workspace`. 

The legacy monolithic implementations (`driverspacelib` and the old `driverspace` daemon) are deprecated. All driver development must now occur in the `drivers/` directory of the new workspace, strictly adhering to the modular architecture.

## 1. Prerequisites
Before beginning driver implementation, the foundational libraries located in `lib/` must be fully implemented and stabilized. Drivers cannot be compiled or tested without the following components:

- [ ] **`lib/kapi-abi`**: Finalize all IPC message structures, opcodes, and error codes. Ensure `cbindgen` is generating accurate C/Odin headers.
- [ ] **`lib/kapi-syscall`**: Implement architecture-specific SVC/syscall wrappers.
- [ ] **`lib/ds-ipc`**: Stabilize the channel, port, and shared memory abstractions.
- [ ] **`lib/ds-mem`**: Finalize the `no_std` allocators (Buddy/Slab) and DMA mapping utilities.
- [ ] **`lib/ds-fw-*`**: Define the core traits for device classes (`BlockDevice`, `AudioDevice`, `DisplayDevice`, `InputDevice`).

## 2. Driver Implementation Checklist
The following drivers require migration from the legacy codebase or complete reimplementation using the new framework traits.

### 2.1. Graphics and Display
- [ ] **`amdgpu_driver`**
  - [ ] Implement MMIO register read/write wrappers.
  - [ ] Implement ring buffer management for command submission.
  - [ ] Implement Display PHY initialization.
  - [ ] Implement the `DisplayDevice` and `GpuDevice` traits from `ds-fw-gpu`.
- [ ] **`intelgpu_driver`**
  - [ ] Implement Graphics Technology (GT) and Display Engine (DE) initialization.
  - [ ] Implement GuC (Graphics Microcontroller) firmware loading.
  - [ ] Implement the `DisplayDevice` trait from `ds-fw-gpu`.
- [ ] **`vgpu` (Virtual GPU)**
  - [ ] Set up the Rust-to-C FFI bridge (`build.rs` and `ffi.rs`).
  - [ ] Implement virtual framebuffer memory management.
  - [ ] Ensure the C implementation strictly uses the `kapi-abi` headers.

### 2.2. Audio
- [ ] **`audiodriver`**
  - [ ] Implement DMA stream management for audio buffers.
  - [ ] Implement codec communication protocols (I2C / Intel HDA).
  - [ ] Integrate the Odin/C DSP components via the defined FFI boundaries.
  - [ ] Implement the `AudioDevice` trait from `ds-fw-audio`.

### 2.3. Input and HID
- [ ] **`wacomgraphic_driver`**
  - [ ] Implement Wacom-specific HID report parsing.
  - [ ] Implement raw-to-logical data conversion for pen pressure and tilt (X/Y).
  - [ ] Implement the `InputDevice` trait from `ds-fw-input`.

### 2.4. Networking and Multimedia
- [ ] **`netcam_driver`**
  - [ ] Implement USB Video Class (UVC) protocol parsing.
  - [ ] Implement isochronous USB transfer handling.
  - [ ] Implement device control interfaces (brightness, contrast, z

*[... content truncated ...]*


---

### /home/ctrl/TrangorgeOS/driverspace_workspace/Tree_readme.md

driverspace_workspace/
├── Cargo.toml                     # Główny workspace manifest
├── rust-toolchain.toml            # Wymuszenie nightly i specyficznych targetów
│
├── lib/                           # 🧰 FUNDAMENT: API, INFRASTRUKTURA I FRAMEWORKI
│   │
│   ├── kapi-abi/                  # Nowe API Jądra: Definicje binarne (Single Source of Truth)
│   │   ├── Cargo.toml
│   │   ├── build.rs               # Generowanie nagłówków C/Odin przez cbindgen
│   │   └── src/
│   │       ├── lib.rs             # Tylko re-eksporty i flagi no_std
│   │       ├── primitives.rs      # Podstawowe typy (Handle, CapId, Status)
│   │       ├── opcodes.rs         # Enumy z numerami wywołań (Syscall/IPC opcodes)
│   │       ├── errors.rs          # Wspólne kody błędów (DsError)
│   │       ├── capabilities.rs    # Struktury uprawnień (Capability tokens)
│   │       ├── wire/              # Serializacja/Deserializacja wiadomości
│   │       │   ├── mod.rs
│   │       │   ├── encode.rs      # Zapis do bufora
│   │       │   ├── decode.rs      # Odczyt z bufora
│   │       │   └── endian.rs      # Konwersje kolejności bajtów
│   │       └── payloads/          # Konkretne struktury wiadomości (#[repr(C)])
│   │           ├── mod.rs
│   │           ├── mem.rs         # Żądania mapowania pamięci
│   │           ├── ipc.rs         # Żądania tworzenia kanałów
│   │           ├── dev.rs         # Żądania rejestracji urządzeń
│   │           └── irq.rs         # Żądania obsługi przerwań
│   │
│   ├── kapi-syscall/              # Surowe wywołania systemowe (SVC / INT)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── arch/              # Implementacje zależne od architektury
│   │       │   ├── mod.rs
│   │       │   ├── x86_64.rs      # Makra asm! dla x86_64 (syscall/sysenter)
│   │       │   └── riscv64.rs     # Makra asm! dla RISC-V (ecall)
│   │       └── wrappers.rs        # Bezpieczne (lub unsafe) wrappery nad surowym asm
│   │
│   ├── ds-ipc/                    # Infrastruktura IPC (Komunikacja między procesami)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── channel.rs         # Abstrakcja kanału (dwukierunkowego)
│   │       ├── port.rs            # Abstrakcja portu (nasłuchiwanie)
│   │       ├── router.rs          # Logika routowania wiadomości
│   │       ├── shared_mem.rs      # Zarządzanie buforami pamięci współdzielonej
│   │       └── sync/              # Primitivy synchronizacji dla IPC
│   │           ├── mod.rs
│   │           ├── spinlock.rs    # Spinlock (no_std)
│   │           └── semaphore.rs   # Semafory dla kolejek wiadomości
│   │
│   ├── ds-mem/                    # Zarządzanie pamięcią w przestrzeni sterownika
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── alloc/             # Własne alokatory (brak std::alloc)
│   │       │   ├── mod.rs
│   │       │   ├── buddy.rs       # Alokator Buddy (dla dużych bloków)
│   │       │   ├── slab.rs        # Alokator 

*[... content truncated ...]*


---

### /home/ctrl/TrangorgeOS/driverspace_workspace/Workspace.md

Oto profesjonalna dokumentacja architektury, przygotowana w języku angielskim, zgodnie z Twoimi wytycznymi.

***

# TrangorgeOS Driver Space Architecture and Workspace Documentation

## 1. Executive Summary
This document outlines the architectural redesign of the Driver Space subsystem within TrangorgeOS. The objective is to transition from a monolithic implementation (`driverspacelib` and `driverspace`) to a highly modular, strictly layered workspace. This restructuring enforces clear boundaries between the kernel interface, driver management daemons, and individual hardware drivers, ensuring maintainability, preventing code duplication, and establishing a robust foundation for the new Kernel API.

## 2. Workspace Topology
The `driverspace_workspace` is divided into three primary pillars. This strict separation of concerns dictates the dependency graph and ensures that hardware-specific logic remains isolated from system management and core abstractions.

```text
driverspace_workspace/
├── Cargo.toml                 # Workspace root configuration
├── lib/                       # Core libraries, frameworks, and the new Kernel API
├── crates/                    # Driver space management daemons and services
└── drivers/                   # Individual hardware driver implementations
```

## 3. Component Breakdown

### 3.1. `lib/` (Core Libraries and Kernel API)
The `lib/` directory serves as the foundational layer of the driver space. It contains the new Kernel API, infrastructure utilities, and device-class frameworks. **No hardware-specific logic resides here.**

*   **Kernel API (`kapi-*`)**
    *   `kapi-abi`: The single source of truth for all data structures, IPC message formats, opcodes, and error codes. All structures are strictly `#[repr(C)]` to ensure binary compatibility across different languages (Rust, C, Odin).
    *   `kapi-syscall`: Low-level, architecture-specific wrappers for kernel transitions (e.g., SVC calls, interrupt handling).
*   **Infrastructure (`ds-*`)**
    *   `ds-ipc`: Inter-process communication primitives, channel management, and message routing logic.
    *   `ds-mem`: Memory management utilities, including custom allocators for `no_std` environments, DMA mapping, and shared memory handling.
    *   `ds-log`: Standardized logging infrastructure that routes driver logs to the central manager or kernel console.
*   **Device Frameworks (`ds-fw-*`)**
    *   `ds-fw-block`, `ds-fw-audio`, `ds-fw-gpu`, `ds-fw-input`: High-level abstractions for specific device classes. These frameworks provide traits (e.g., `BlockDevice`) and handle protocol boilerplate, allowing driver authors to focus solely on hardware register manipulation and state machines.

### 3.2. `crates/` (Driver Space Management)
The `crates/` directory contains the privileged user-space (or kernel-space) daemons responsible for orchestrating the driver ecosystem. These components manage the lifecycle of drivers but do not interact directly with hardware.

* 

*[... content truncated ...]*


---

### /home/ctrl/TrangorgeOS/driverspace_workspace/formal/README.md



---

### /home/ctrl/TrangorgeOS/kernel_Workspace/README.md

       ┌──────────────┐
       │  kernel-bin  │  (wykonywalna binarka / punkt startowy)
       └──────┬───────┘
              │
      ┌───────┴──────────────┐
      ▼                      ▼
┌──────────┐          ┌──────────────┐
│  kernel  │ (lib)    │ core/* (lib) │ (kstd_core, gluecore, itp.)
└────┬─────┘          └──────┬───────┘
     │                       │
     └───────────┬───────────┘
                 ▼
       ┌──────────────────┐
       │ base/* / ABI libs│
       └──────────────────┘

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/core/core.md



---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/README.md

...

---

### /home/ctrl/TrangorgeOS/kernel_Workspace/kernel/src/mm/MM_PLAN_ULEPSZENIA.md

# Plan ulepszenia podsystemu MM w TrangorgeOS

**Repozytorium:** [CTRL-F-0rg3/TrangorgeOS](https://github.com/CTRL-F-0rg3/TrangorgeOS)  
**Analizowana gałąź:** `stabilizing`  
**Analizowany commit:** `7a0d836f2180874aac3e8c92493c791aec89a8b3`  
**Autor analizy:** ctrl 
**Data analizy:** 20 sierpnia 2026 r.
 
 
 masz kropke gdzie ostatnio dodałem


## 1. Zakres i podsumowanie

Przeanalizowany został cały katalog `kernel/src/mm`, w tym: inicjalizacja pamięci architektury, bitmapowy PMM, allocator ramek, mapowanie wirtualne, paging, TLB, sterta buddy/slab, API `kmalloc`, debug allocator, DMA, pamięć ciągła, cache, ochrona oraz przestrzenie adresowe procesów.

Architektura jest sensownie rozdzielona na warstwy. Obecny przepływ startowy (`arch_memory_init` → paging → PMM → VMM → heap → cache → paging subsystem → isolation → address spaces) daje dobrą bazę do dalszego rozwoju. Największym problemem nie jest brak komponentów, lecz to, że część API jest jeszcze prototypowa: ścieżka slab jest wyłączona, blokady nie zapewniają bezpieczeństwa SMP, operacje zakresowe nie wszędzie sprawdzają overflow, a obsługa VMA/mmap nie ma jeszcze pełnej semantyki systemowej.

> **Najważniejsza rekomendacja:** przed dodawaniem nowych funkcji należy ustabilizować kontrakty allocatorów, walidację zakresów i synchronizację. Bez tego rozwój procesów, sterowników i DMA będzie zwiększał ryzyko cichych uszkodzeń pamięci.

## 2. Ocena stanu obecnego

| Obszar | Stan | Ocena |
|---|---|---|
| Rozdzielenie PMM/VMM/heap/process | Istnieje i jest czytelne | Dobra baza architektoniczna |
| Bitmapa ramek | Obsługuje pojedyncze i ciągłe zakresy | Wymaga testów granicznych i lepszej wydajności |
| Buddy allocator | Działa jako główna ścieżka heap | Brak synchronizacji SMP i dokładnego rozmiaru żądania |
| Slab allocator | Zaimplementowany, ale `HEAP_USE_SLAB` wynosi `0` | Niewykorzystana optymalizacja; ma ryzyko double-free |
| API alokacji | `kmalloc`, `kzalloc`, `kcalloc`, `krealloc`, strony | Kontrakty `aligned/pages` są niepełne |
| Paging/VMM | Są mapowanie, translate, protect i address spaces | Brakuje pełnej transakcyjności i walidacji overflow |
| DMA/contiguous | Istnieją osobne moduły | Wymagają bezpiecznego liczenia rozmiarów i modelu cache |
| Izolacja | SMEP/SMAP/NX i audyt PML4 | Włączenie mechanizmów nie jest raportowane jako błąd krytyczny |
| Testy | Jest `kernel/src/testing.rs` i kilka self-testów | Brak automatycznej, szerokiej macierzy testów MM |
| Build/CI | W środowisku analizy `cargo` nie było dostępne | Nie udało się potwierdzić kompilacji; potrzebny CI z cross-toolchainem |

## 3. Problemy wymagające naprawy

### P0 — bezpieczeństwo i poprawność krytyczna

#### P0.1. Blokady wyłączające przerwania nie są blokadami SMP

PMM, slab i przestrzenie adresowe używają wzorca `pushfq; cli` oraz lokalnego licznika zagnieżdżenia. Wyłączenie przerwań chroni stan tylko przed przerwaniem na bieżącym CPU; nie chroni przed drugim rdzeniem. Przy konfiguracji wielordzeniowej (`-s

*[... content truncated ...]*


---

### /home/ctrl/TrangorgeOS/panic.md

# Kernel Hang / Panic Report — 2026-10-09
# Kernel Hang / Panic Report — LAPIC base sanity check

Key findings:
- `APIC_BASE_MSR` read reports x2APIC when bit 10 set; else LAPIC assumed at `base_phys` passed in.
- `lapic::init(base_phys)` uses `base_phys` straight as virtual if it's `>= 0xFFFF800000000000`.
- In SMP init, APs are booted with `send_startup_ipi` using `trampoline::TRAMPOLINE_BASE >> 12` as vector.

Open question:
- Where is actual LAPIC physical base loaded from? Current code does not show base-specific usage beyond `send_startup_ipi` vector.

Next active items:
1. Confirm TRAMPOLINE_BASE value and that identity mapping covers it.
2. Confirm `build.rs` linker path behavior for x86_64.
3. Add serial console capture in QEMU and reproduce hang location.


## Status
Kernel **builds and boots** in qemu (GTK display, 4 vCPUs) but **freezes during
runtime**. No console / serial output was captured from the GTK session — the VM
stays quiet and appears hung.

Attempted serial capture with qemu (`-serial file:/tmp/qemu_serial_out.txt`)
did not produce a readable guest log at the time of this report, so the exact
stop point is inferred from the boot path rather than from live kernel prints.

## Boot entry
`kernel_main(boot_info: &'static bootloader::BootInfo) -> !` (`src/main.rs`)

Sequence:
1. `init()`
2. `mm::init_from_boot_info(boot_info)`  → “[mm] allocator initialized OK” expected
3. `init_permissions()`
4. `gfx::init()`  → “[gfx] framebuffer initialized OK” expected
5. (optional) `hdmi::init::init()`
6. `pci::init()`
7. `nic::runtime::init()`
8. `bluetooth::init::init()`
9. `fs::init()`
10. **`cpu::init(boot_info)`**  ← SMP bring-up, tight waits per AP
11. `testing::run_all(TESTS)`
12. `println!("Welcome in my Galaxy!")`
13. `gfx::refresh()`, `terminal::init()`, `terminal::run()`

## Where the hang manifests
In the GTK qemu session the kernel reaches `cpu::init(boot_info)` (step 10) and
then stops producing visible progress. The SMP initialization polls for each AP
to reach its trampoline entry within a tight per-AP timeout (about 100 ms per
AP, 3 APs with 4 vCPUs). If an AP does not become ready in time, the boot path
blocks waiting for it without emitting a visible panic in the GTK display.

## Observations from the code
- SMP init lives in `src/cpu/mod.rs` / `src/cpu/smp.rs` and reaches into
  `src/cpu/scheduler/smp/mod.rs` plus the migration / stopper / balancing
  subtrees. AP boot uses a trampoline at `TRAMPOLINE_BASE`.
- The bootloader crate (`bootloader = 0.9` with `map_physical_memory` +
### Changed technical context

- Trampoline base is `TRAMPOLINE_BASE = 0x8000` in `kernel/src/cpu/trampoline.rs` and `.set TRAMPOLINE_BASE, 0x8000` in `kernel/src/cpu/trampoline.s`.
- SMP bringup uses `send_startup_ipi(apic_id, (trampoline::TRAMPOLINE_BASE >> 12) as u8)`.
- Paging init in `kernel/src/mm/arch/x86_64/paging.c` skips identity work when `boot_phys_offset == ARCH_DIRECT_MAP_BASE`, else it maps `boot_phys_offset` → `ARCH_DIRECT_MAP_B

*[... content truncated ...]*


---

