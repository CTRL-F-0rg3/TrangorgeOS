# 📚 TrangorgeOS Architecture Documentation

**Generated:** 2026-09-18 20:54:30  
**Files Scanned:** `1710` | **API Items Extracted:** `4008`  
**Languages:** Rust, C/C++, Ada SPARK, Odin, Nim, Assembly

---

## 📑 Table of Contents

- [c.rs](#c.rs)
- [comgrub](#comgrub)
- [comgrub/src](#comgrub-src)
- [comlimine](#comlimine)
- [comlimine/src](#comlimine-src)
- [drivers/amdgpu_driver](#drivers-amdgpu-driver)
- [drivers/audiodriver](#drivers-audiodriver)
- [drivers/intelgpu_driver](#drivers-intelgpu-driver)
- [drivers/netcam_driver](#drivers-netcam-driver)
- [drivers/vgpu](#drivers-vgpu)
- [drivers/wacomgraphic_driver](#drivers-wacomgraphic-driver)
- [driverspace/src](#driverspace-src)
- [driverspace_workspace/crates](#driverspace-workspace-crates)
- [driverspace_workspace/drivers](#driverspace-workspace-drivers)
- [driverspace_workspace/formal](#driverspace-workspace-formal)
- [driverspace_workspace/lib](#driverspace-workspace-lib)
- [driverspacelib/src](#driverspacelib-src)
- [kernel_Workspace/C_base](#kernel-workspace-c-base)
- [kernel_Workspace/Odin_base](#kernel-workspace-odin-base)
- [kernel_Workspace/base](#kernel-workspace-base)
- [kernel_Workspace/core](#kernel-workspace-core)
- [kernel_Workspace/kernel](#kernel-workspace-kernel)
- [kernel_Workspace/kernel-bin](#kernel-workspace-kernel-bin)
- [kstd/include](#kstd-include)
- [kstd/src](#kstd-src)
- [libs](#libs)
- [mp4_to_bmp/src](#mp4-to-bmp-src)
- [trangorgelibc/src](#trangorgelibc-src)
- [triang-lang/src](#triang-lang-src)
- [userspace-legasy/demo](#userspace-legasy-demo)
- [userspace-legasy/init](#userspace-legasy-init)
- [userspace-legasy/shell](#userspace-legasy-shell)
- [userspace-legasy/terminal](#userspace-legasy-terminal)

---

## 📂 c.rs

<details>
<summary><b>📄 c.rs</b> (2 items)</summary>

#### `STRUCT`: **BlockHeader** <sub>line 57</sub>
```rust
struct BlockHeader* next;           /* Wskaźnik do następnego bloku */
```

#### `STRUCT`: **BlockHeader** <sub>line 58</sub>
```rust
struct BlockHeader* prev;           /* Wskaźnik do poprzedniego bloku */
```

</details>

## 📂 comgrub

<details>
<summary><b>📄 comgrub/build.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

## 📂 comgrub/src

<details>
<summary><b>📄 comgrub/src/main.rs</b> (2 items)</summary>

#### `FN`: **kernel_main** <sub>line 11</sub>
```rust
fn kernel_main(magic: u32, info: *const u8) -> !;
```

#### `FN`: **panic** <sub>line 15</sub>
```rust
fn panic(_info: &PanicInfo) -> ! {
```

</details>

## 📂 comlimine

<details>
<summary><b>📄 comlimine/build.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

## 📂 comlimine/src

<details>
<summary><b>📄 comlimine/src/main.c</b> (2 items)</summary>

#### `FUNCTION`: **kernel_main** <sub>line 6</sub>
```c
extern void kernel_main(uint64_t magic, void *info);
```

#### `FUNCTION`: **_start** <sub>line 28</sub>
```c
void _start(void) {
```

</details>

<details>
<summary><b>📄 comlimine/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {}
```

</details>

## 📂 drivers/amdgpu_driver

<details>
<summary><b>📄 drivers/amdgpu_driver/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

## 📂 drivers/audiodriver

<details>
<summary><b>📄 drivers/audiodriver/build.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 12</sub>
```rust
fn main() {
```

</details>

<details>
<summary><b>📄 drivers/audiodriver/src/jacklib.rs</b> (6 items)</summary>

#### `STRUCT`: **JackMgr** <sub>line 4</sub>
```rust
pub struct JackMgr {
```

#### `IMPL`: **JackMgr** <sub>line 9</sub>
```rust
impl JackMgr {
```

#### `FN`: **tick** <sub>line 14</sub>
```rust
pub fn tick(&mut self) {
```

#### `FN`: **set_amp** <sub>line 30</sub>
```rust
pub fn set_amp(&mut self, on: bool) {
```

#### `FN`: **present** <sub>line 35</sub>
```rust
pub fn present(&self) -> bool {
```

#### `FN`: **amp_enabled** <sub>line 40</sub>
```rust
pub fn amp_enabled(&self) -> bool {
```
> Czy wzmacniacz (amp) jest aktualnie wlaczony.

</details>

<details>
<summary><b>📄 drivers/audiodriver/src/lib.rs</b> (14 items)</summary>

#### `FN`: **ad_init** <sub>line 7</sub>
```rust
fn ad_init(nam_va: u64, bm_va: u64) -> i32;
```

#### `FN`: **ad_play** <sub>line 8</sub>
```rust
fn ad_play(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32;
```

#### `FN`: **ad_capture** <sub>line 9</sub>
```rust
fn ad_capture(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32;
```

#### `FN`: **ad_stop** <sub>line 10</sub>
```rust
fn ad_stop() -> i32;
```

#### `FN`: **ad_jack_present** <sub>line 11</sub>
```rust
fn ad_jack_present() -> i32;
```

#### `FN`: **ad_set_amp** <sub>line 12</sub>
```rust
fn ad_set_amp(on: i32) -> i32;
```

#### `FN`: **ad_position** <sub>line 13</sub>
```rust
fn ad_position() -> u32;
```

#### `FN`: **init** <sub>line 16</sub>
```rust
pub fn init(nam_va: u64, bm_va: u64) -> bool {
```

#### `FN`: **play** <sub>line 20</sub>
```rust
pub fn play(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> bool {
```

#### `FN`: **capture** <sub>line 24</sub>
```rust
pub fn capture(data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> bool {
```

#### `FN`: **stop** <sub>line 28</sub>
```rust
pub fn stop() {
```

#### `FN`: **jack_present** <sub>line 32</sub>
```rust
pub fn jack_present() -> bool {
```

#### `FN`: **set_amp** <sub>line 36</sub>
```rust
pub fn set_amp(on: bool) {
```

#### `FN`: **position** <sub>line 40</sub>
```rust
pub fn position() -> u32 {
```

</details>

<details>
<summary><b>📄 drivers/audiodriver/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 8</sub>
```rust
fn main() {
```
> Właściwy sterownik działa w driver space przez `ds_entry` (patrz 
> `driverspace/src/main.rs`); ta binarka jest cienkim wrapperem CLI, który 
> pozwala sprawdzić stan gniazda jack z poziomu hosta.

</details>

<details>
<summary><b>📄 drivers/audiodriver/src/odin/driver.odin</b> (8 items)</summary>

#### `STRUCT`: **Bdl_Entry** <sub>line 8</sub>
```odin
Bdl_Entry :: struct {
```

#### `PROC`: **nam16** <sub>line 16</sub>
```odin
nam16 :: proc(off: u32) -> ^u16 {
```
> mixer (NAM) — 16-bit

#### `PROC`: **bm8** <sub>line 21</sub>
```odin
bm8 :: proc(off: u32) -> ^u8 {
```
> bus master — 8/16/32

#### `PROC`: **bm16** <sub>line 25</sub>
```odin
bm16 :: proc(off: u32) -> ^u16 {
```

#### `PROC`: **bm32** <sub>line 29</sub>
```odin
bm32 :: proc(off: u32) -> ^u32 {
```

#### `PROC`: **ad_init** <sub>line 50</sub>
```odin
ad_init :: proc "C" (nam_va: u64, bm_va: u64) -> i32 {
```

#### `PROC`: **ad_stop** <sub>line 73</sub>
```odin
ad_stop :: proc "C" () -> i32 {
```

#### `PROC`: **ad_position** <sub>line 80</sub>
```odin
ad_position :: proc "C" () -> u32 {
```

</details>

<details>
<summary><b>📄 drivers/audiodriver/src/odin/jack.odin</b> (2 items)</summary>

#### `PROC`: **ad_jack_present** <sub>line 8</sub>
```odin
ad_jack_present :: proc "C" () -> i32 {
```

#### `PROC`: **ad_set_amp** <sub>line 21</sub>
```odin
ad_set_amp :: proc "C" (on: i32) -> i32 {
```

</details>

<details>
<summary><b>📄 drivers/audiodriver/src/odin/microphone.odin</b> (1 items)</summary>

#### `PROC`: **ad_capture** <sub>line 6</sub>
```odin
ad_capture :: proc "C" (data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32 {
```

</details>

<details>
<summary><b>📄 drivers/audiodriver/src/odin/speaker.odin</b> (1 items)</summary>

#### `PROC`: **ad_play** <sub>line 6</sub>
```odin
ad_play :: proc "C" (data_phys: u64, len: u32, bdl_va: u64, bdl_phys: u64) -> i32 {
```

</details>

<details>
<summary><b>📄 drivers/audiodriver/src/tone.rs</b> (1 items)</summary>

#### `FN`: **fill_square** <sub>line 1</sub>
```rust
pub fn fill_square(buf: &mut [u8], periods: u32) {
```

</details>

## 📂 drivers/intelgpu_driver

<details>
<summary><b>📄 drivers/intelgpu_driver/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

## 📂 drivers/netcam_driver

<details>
<summary><b>📄 drivers/netcam_driver/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

## 📂 drivers/vgpu

<details>
<summary><b>📄 drivers/vgpu/vgpu.h</b> (7 items)</summary>

#### `FUNCTION`: **vgpu_init** <sub>line 22</sub>
```c
bool vgpu_init(uint32_t w, uint32_t h);
```

#### `FUNCTION`: **vgpu_get_info** <sub>line 23</sub>
```c
vgpu_info_t vgpu_get_info(void);
```

#### `FUNCTION`: **vgpu_clear** <sub>line 25</sub>
```c
void vgpu_clear(uint32_t color);
```

#### `FUNCTION`: **vgpu_pixel** <sub>line 26</sub>
```c
void vgpu_pixel(uint32_t x, uint32_t y, uint32_t color);
```

#### `FUNCTION`: **vgpu_surface_create** <sub>line 28</sub>
```c
int32_t vgpu_surface_create(uint32_t w, uint32_t h, uint64_t *phys_out);
```

#### `FUNCTION`: **vgpu_surface_present** <sub>line 29</sub>
```c
bool vgpu_surface_present(int32_t id, uint32_t x, uint32_t y);
```

#### `FUNCTION`: **vgpu_process_ring** <sub>line 31</sub>
```c
void vgpu_process_ring(volatile void *ring);
```

</details>

<details>
<summary><b>📄 drivers/vgpu/vgpu_driver.c</b> (11 items)</summary>

#### `FUNCTION`: **pci_void** <sub>line 12</sub>
```c
static void pci_void(uint32_t op, uint64_t a0, uint64_t a1, uint64_t a2)
```

#### `FUNCTION`: **pci_val** <sub>line 20</sub>
```c
static uint64_t pci_val(uint32_t op, uint64_t a0, uint64_t a1, uint64_t a2)
```

#### `FUNCTION`: **vbe_write** <sub>line 33</sub>
```c
static void vbe_write(uint16_t idx, uint16_t val)
```

#### `FUNCTION`: **vgpu_init** <sub>line 40</sub>
```c
bool vgpu_init(uint32_t w, uint32_t h)
```

#### `FUNCTION`: **vgpu_get_info** <sub>line 98</sub>
```c
vgpu_info_t vgpu_get_info(void)
```

#### `FUNCTION`: **vgpu_pixel** <sub>line 104</sub>
```c
void vgpu_pixel(uint32_t x, uint32_t y, uint32_t color)
```

#### `FUNCTION`: **vgpu_clear** <sub>line 113</sub>
```c
void vgpu_clear(uint32_t color)
```

#### `FUNCTION`: **vgpu_surface_create** <sub>line 128</sub>
```c
int32_t vgpu_surface_create(uint32_t w, uint32_t h, uint64_t *phys_out)
```

#### `FUNCTION`: **vgpu_surface_present** <sub>line 168</sub>
```c
bool vgpu_surface_present(int32_t id, uint32_t x, uint32_t y)
```

#### `FUNCTION`: **vgpu_process_ring** <sub>line 190</sub>
```c
void vgpu_process_ring(volatile vgpu_slot_t *ring)
```

#### `FUNCTION`: **ds_entry** <sub>line 244</sub>
```c
void ds_entry(uint64_t params_va)
```

</details>

## 📂 drivers/wacomgraphic_driver

<details>
<summary><b>📄 drivers/wacomgraphic_driver/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

## 📂 driverspace/src

<details>
<summary><b>📄 driverspace/src/drivers/storage.rs</b> (4 items)</summary>

#### `STRUCT`: **StorageDrv** <sub>line 9</sub>
```rust
pub struct StorageDrv {
```
> Storage driver stub. It only satisfies the driver registry so that 
> driver space can boot; a real disk driver can slot in later.

#### `IMPL`: **StorageDrv** <sub>line 13</sub>
```rust
impl StorageDrv {
```

#### `IMPL`: **Driver** <sub>line 19</sub>
```rust
impl Driver for StorageDrv {
```

#### `FN`: **init** <sub>line 20</sub>
```rust
fn init(&mut self, _info: &DeviceInfo) -> Result<(), DsError> {
```

</details>

<details>
<summary><b>📄 driverspace/src/main.rs</b> (1 items)</summary>

#### `FN`: **panic** <sub>line 32</sub>
```rust
fn panic(_info: &core::panic::PanicInfo) -> ! {
```

</details>

## 📂 driverspace_workspace/crates

<details>
<summary><b>📄 driverspace_workspace/crates/ds-manager/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

## 📂 driverspace_workspace/drivers

<details>
<summary><b>📄 driverspace_workspace/drivers/amdgpu-driver/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/drivers/audiodriver/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/drivers/intelgpu-driver/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/drivers/netcam_driver/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/drivers/vgpu/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/drivers/wacomgraphic_driver/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 1</sub>
```rust
fn main() {
```

</details>

## 📂 driverspace_workspace/formal

<details>
<summary><b>📄 driverspace_workspace/formal/ds-spark-core/src/ds_buddy_math.adb</b> (7 items)</summary>

#### `PACKAGE`: **body** <sub>line 2</sub>
```ada
package body DS_Buddy_Math with
```
> formal/ds-spark-core/src/ds_buddy_math.adb

#### `FUNCTION`: **Power_Of_Two** <sub>line 6</sub>
```ada
function Power_Of_Two (Exponent : Block_Size) return Phys_Addr is
```

#### `FUNCTION`: **Is_Aligned** <sub>line 11</sub>
```ada
function Is_Aligned (Addr : Phys_Addr; Size_Exp : Block_Size) return Boolean is
```

#### `FUNCTION`: **Get_Buddy_Address** <sub>line 18</sub>
```ada
function Get_Buddy_Address (
```

#### `FUNCTION`: **Get_Left_Child_Address** <sub>line 31</sub>
```ada
function Get_Left_Child_Address (
```

#### `FUNCTION`: **Get_Right_Child_Address** <sub>line 40</sub>
```ada
function Get_Right_Child_Address (
```

#### `FUNCTION`: **Safe_Calculate_DMA_End** <sub>line 50</sub>
```ada
function Safe_Calculate_DMA_End (
```

</details>

<details>
<summary><b>📄 driverspace_workspace/formal/ds-spark-core/src/ds_buddy_math.ads</b> (9 items)</summary>

#### `PACKAGE`: **DS_Buddy_Math** <sub>line 4</sub>
```ada
package DS_Buddy_Math with
```

#### `TYPE`: **Phys_Addr** <sub>line 9</sub>
```ada
type Phys_Addr is new Unsigned_64;
```
> Typy bazowe dla fizycznej pamięci (bare-metal)

#### `TYPE`: **Block_Size** <sub>line 10</sub>
```ada
type Block_Size is new Unsigned_64;
```

#### `FUNCTION`: **Power_Of_Two** <sub>line 17</sub>
```ada
function Power_Of_Two (Exponent : Block_Size) return Phys_Addr with
```
> Funkcja pomocnicza: Obliczanie 2^N (Shift left)

#### `FUNCTION`: **Is_Aligned** <sub>line 23</sub>
```ada
function Is_Aligned (Addr : Phys_Addr; Size_Exp : Block_Size) return Boolean with
```
> Sprawdzenie, czy adres jest poprawnie wyrównany do rozmiaru bloku

#### `FUNCTION`: **Get_Buddy_Address** <sub>line 32</sub>
```ada
function Get_Buddy_Address (
```

#### `FUNCTION`: **Get_Left_Child_Address** <sub>line 47</sub>
```ada
function Get_Left_Child_Address (
```

#### `FUNCTION`: **Get_Right_Child_Address** <sub>line 55</sub>
```ada
function Get_Right_Child_Address (
```

#### `FUNCTION`: **Safe_Calculate_DMA_End** <sub>line 69</sub>
```ada
function Safe_Calculate_DMA_End (
```

</details>

## 📂 driverspace_workspace/lib

<details>
<summary><b>📄 driverspace_workspace/lib/ds-formal-ffi/build.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 10</sub>
```rust
fn main() {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-formal-ffi/src/lib.rs</b> (6 items)</summary>

#### `FN`: **ds_spark_safe_add_32** <sub>line 7</sub>
```rust
pub fn ds_spark_safe_add_32(a: u32, b: u32) -> u32;
```

#### `FN`: **ds_spark_safe_array_index** <sub>line 10</sub>
```rust
pub fn ds_spark_safe_array_index(index: u32, max_size: u32) -> u32;
```

#### `FN`: **safe_add** <sub>line 21</sub>
```rust
pub fn safe_add(a: u32, b: u32) -> u32 {
```

#### `FN`: **ds_spark_buddy_get_buddy** <sub>line 28</sub>
```rust
pub fn ds_spark_buddy_get_buddy(addr: u64, size_exp: u64) -> u64;
```

#### `FN`: **ds_spark_safe_dma_end** <sub>line 29</sub>
```rust
pub fn ds_spark_safe_dma_end(base: u64, len: u64) -> u64;
```

#### `FN`: **get_buddy** <sub>line 35</sub>
```rust
pub fn get_buddy(addr: u64, exp: u64) -> u64 {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-fw-audio/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 1</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 10</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-fw-block/src/request.rs</b> (8 items)</summary>

#### `ENUM`: **RequestState** <sub>line 7</sub>
```rust
pub enum RequestState {
```

#### `STRUCT`: **IoRequest** <sub>line 15</sub>
```rust
pub struct IoRequest {
```
> High-level I/O request with metadata.

#### `IMPL`: **IoRequest** <sub>line 27</sub>
```rust
impl IoRequest {
```

#### `FN`: **new** <sub>line 28</sub>
```rust
pub fn new(id: u64, lba: u64, block_count: u32, buffer: *mut u8, len: usize, is_write: bool) -> Self {
```

#### `FN`: **state** <sub>line 42</sub>
```rust
pub fn state(&self) -> RequestState {
```

#### `FN`: **mark_in_progress** <sub>line 52</sub>
```rust
pub fn mark_in_progress(&self) {
```

#### `FN`: **mark_completed** <sub>line 56</sub>
```rust
pub fn mark_completed(&self) {
```

#### `FN`: **mark_failed** <sub>line 63</sub>
```rust
pub fn mark_failed(&self) {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-fw-block/src/traits.rs</b> (11 items)</summary>

#### `STRUCT`: **BlockRequest** <sub>line 6</sub>
```rust
pub struct BlockRequest {
```
> Represents a single I/O request to a block device.

#### `TRAIT`: **BlockDevice** <sub>line 16</sub>
```rust
pub trait BlockDevice {
```
> Core trait for all block devices. 
> Drivers must implement this to integrate with the block subsystem.

#### `FN`: **read_blocks** <sub>line 18</sub>
```rust
fn read_blocks(&mut self, lba: u64, block_count: u32, buffer: &mut [u8]) -> Result<(), DsError>;
```
> Read blocks from the device starting at LBA.

#### `FN`: **write_blocks** <sub>line 21</sub>
```rust
fn write_blocks(&mut self, lba: u64, block_count: u32, buffer: &[u8]) -> Result<(), DsError>;
```
> Write blocks to the device starting at LBA.

#### `FN`: **flush** <sub>line 24</sub>
```rust
fn flush(&mut self) -> Result<(), DsError>;
```
> Flush internal caches to physical media.

#### `FN`: **geometry** <sub>line 27</sub>
```rust
fn geometry(&self) -> BlockGeometry;
```
> Get device geometry (block size, total blocks).

#### `FN`: **supports_trim** <sub>line 30</sub>
```rust
fn supports_trim(&self) -> bool { false }
```
> Check if device supports TRIM/discard.

#### `FN`: **submit_request** <sub>line 33</sub>
```rust
fn submit_request(&mut self, _req: BlockRequest) -> Result<(), DsError> {
```
> Optional: Submit async request (for advanced schedulers).

#### `STRUCT`: **BlockGeometry** <sub>line 40</sub>
```rust
pub struct BlockGeometry {
```

#### `IMPL`: **BlockGeometry** <sub>line 46</sub>
```rust
impl BlockGeometry {
```

#### `FN`: **total_bytes** <sub>line 47</sub>
```rust
pub fn total_bytes(&self) -> u64 {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-fw-gpu/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 1</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 10</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-fw-input/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 1</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 10</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-ipc/src/client.rs</b> (4 items)</summary>

#### `STRUCT`: **ManagerClient** <sub>line 7</sub>
```rust
pub struct ManagerClient {
```

#### `IMPL`: **ManagerClient** <sub>line 11</sub>
```rust
impl ManagerClient {
```

#### `FN`: **request_mmio** <sub>line 16</sub>
```rust
pub fn request_mmio(&self, phys_base: u64, size: u64) -> Result<u64, DsError> {
```

#### `FN`: **bind_irq** <sub>line 41</sub>
```rust
pub fn bind_irq(&self, irq_num: u32) -> Result<(), DsError> {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-ipc/src/dispatcher.rs</b> (5 items)</summary>

#### `TYPE`: **HandlerFn** <sub>line 8</sub>
```rust
pub type HandlerFn = fn(sender: Handle, payload_ptr: *const u8, payload_len: u32) -> Result<(), DsError>;
```

#### `STRUCT`: **MessageDispatcher** <sub>line 10</sub>
```rust
pub struct MessageDispatcher {
```

#### `IMPL`: **MessageDispatcher** <sub>line 14</sub>
```rust
impl MessageDispatcher {
```

#### `FN`: **register** <sub>line 21</sub>
```rust
pub fn register(&mut self, opcode: Opcode, handler: HandlerFn) -> Result<(), DsError> {
```

#### `FN`: **dispatch** <sub>line 30</sub>
```rust
pub fn dispatch(&self, sender: Handle, opcode_raw: u32, payload_ptr: *const u8, payload_len: u32) -> Result<(), DsError> {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-log/src/formatter.rs</b> (6 items)</summary>

#### `STRUCT`: **LogBuffer** <sub>line 7</sub>
```rust
pub struct LogBuffer {
```

#### `IMPL`: **LogBuffer** <sub>line 12</sub>
```rust
impl LogBuffer {
```

#### `FN`: **as_bytes** <sub>line 20</sub>
```rust
pub fn as_bytes(&self) -> &[u8] {
```

#### `FN`: **reset** <sub>line 24</sub>
```rust
pub fn reset(&mut self) {
```

#### `IMPL`: **Write** <sub>line 29</sub>
```rust
impl Write for LogBuffer {
```

#### `FN`: **write_str** <sub>line 30</sub>
```rust
fn write_str(&mut self, s: &str) -> fmt::Result {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-log/src/levels.rs</b> (2 items)</summary>

#### `ENUM`: **LogLevel** <sub>line 5</sub>
```rust
pub enum LogLevel {
```

#### `IMPL`: **LogLevel** <sub>line 13</sub>
```rust
impl LogLevel {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-log/src/lib.rs</b> (5 items)</summary>

#### `STRUCT`: **EndpointRef** <sub>line 14</sub>
```rust
pub struct EndpointRef;
```

#### `IMPL`: **EndpointRef** <sub>line 16</sub>
```rust
impl EndpointRef {
```

#### `FN`: **is_valid** <sub>line 19</sub>
```rust
pub fn is_valid(&self) -> bool {
```

#### `FN`: **handle** <sub>line 24</sub>
```rust
pub fn handle(&self) -> u32 {
```
> Handle endpointu Managera (0 = niezarejestrowany).

#### `FN`: **set** <sub>line 28</sub>
```rust
pub fn set(&self, handle: Handle) {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-log/src/transport.rs</b> (2 items)</summary>

#### `FN`: **uart_putchar** <sub>line 12</sub>
```rust
fn uart_putchar(c: u8) {
```

#### `FN`: **emit_log** <sub>line 19</sub>
```rust
pub fn emit_log(level: LogLevel, module: &str, buf: &LogBuffer) {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-mem/src/alloc/buddy.rs</b> (4 items)</summary>

#### `STRUCT`: **BuddyAllocator** <sub>line 5</sub>
```rust
pub struct BuddyAllocator {
```

#### `IMPL`: **BuddyAllocator** <sub>line 11</sub>
```rust
impl BuddyAllocator {
```

#### `FN`: **alloc_pages** <sub>line 20</sub>
```rust
pub fn alloc_pages(&mut self, order: usize) -> Option<usize> {
```

#### `FN`: **free_pages** <sub>line 28</sub>
```rust
pub fn free_pages(&mut self, addr: usize, order: usize) {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-mem/src/alloc/heap.rs</b> (3 items)</summary>

#### `STRUCT`: **DriverHeap** <sub>line 6</sub>
```rust
pub struct DriverHeap {
```

#### `IMPL`: **DriverHeap** <sub>line 11</sub>
```rust
impl DriverHeap {
```

#### `FN`: **init** <sub>line 18</sub>
```rust
pub fn init(&mut self, base_addr: usize, total_pages: usize) {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-mem/src/alloc/slab.rs</b> (4 items)</summary>

#### `STRUCT`: **SlabAllocator** <sub>line 1</sub>
```rust
pub struct SlabAllocator {
```

#### `IMPL`: **SlabAllocator** <sub>line 6</sub>
```rust
impl SlabAllocator {
```

#### `FN`: **alloc** <sub>line 14</sub>
```rust
pub fn alloc(&mut self) -> Option<usize> {
```

#### `FN`: **free** <sub>line 19</sub>
```rust
pub fn free(&mut self, ptr: usize) {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-mem/src/dma/buffer.rs</b> (6 items)</summary>

#### `STRUCT`: **DmaFlags** <sub>line 10</sub>
```rust
pub struct DmaFlags: u32 {
```

#### `STRUCT`: **DmaBuffer** <sub>line 17</sub>
```rust
pub struct DmaBuffer {
```

#### `IMPL`: **DmaBuffer** <sub>line 24</sub>
```rust
impl DmaBuffer {
```

#### `FN`: **allocate** <sub>line 25</sub>
```rust
pub fn allocate(manager_ep: Handle, size: u64, flags: DmaFlags) -> Result<Self, DsError> {
```

#### `IMPL`: **Drop** <sub>line 60</sub>
```rust
impl Drop for DmaBuffer {
```

#### `FN`: **drop** <sub>line 61</sub>
```rust
fn drop(&mut self) {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-mem/src/dma/mapping.rs</b> (6 items)</summary>

#### `STRUCT`: **MmioRegion** <sub>line 8</sub>
```rust
pub struct MmioRegion {
```

#### `IMPL`: **MmioRegion** <sub>line 15</sub>
```rust
impl MmioRegion {
```

#### `FN`: **map** <sub>line 16</sub>
```rust
pub fn map(manager_ep: Handle, phys_base: u64, size: u64) -> Result<Self, DsError> {
```

#### `STRUCT`: **Volatile** <sub>line 67</sub>
```rust
pub struct Volatile<T>(pub T);
```

#### `FN`: **read** <sub>line 71</sub>
```rust
pub fn read(&self) -> T {
```

#### `FN`: **write** <sub>line 76</sub>
```rust
pub fn write(&mut self, value: T) {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/ds-mem/src/vmm.rs</b> (2 items)</summary>

#### `FN`: **map_memory** <sub>line 6</sub>
```rust
pub fn map_memory(
```

#### `FN`: **unmap_memory** <sub>line 33</sub>
```rust
pub fn unmap_memory(manager_ep: Handle, virt_addr: u64, size: u64) -> Result<(), DsError> {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/kapi-abi/build.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 9</sub>
```rust
fn main() {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/kapi-abi/src/errors.rs</b> (3 items)</summary>

#### `ENUM`: **DsError** <sub>line 7</sub>
```rust
pub enum DsError {
```

#### `IMPL`: **DsError** <sub>line 42</sub>
```rust
impl DsError {
```

#### `FN`: **from_u32** <sub>line 44</sub>
```rust
pub fn from_u32(val: u32) -> Self {
```
> 将 u32 转换为 DsError，如果值超出范围则返回 Unknown。

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/kapi-abi/src/opcodes.rs</b> (3 items)</summary>

#### `ENUM`: **Opcode** <sub>line 6</sub>
```rust
pub enum Opcode {
```

#### `IMPL`: **Opcode** <sub>line 46</sub>
```rust
impl Opcode {
```

#### `FN`: **from_u32** <sub>line 47</sub>
```rust
pub fn from_u32(val: u32) -> Option<Self> {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/kapi-abi/src/payloads/dev.rs</b> (3 items)</summary>

#### `ENUM`: **BusType** <sub>line 10</sub>
```rust
pub enum BusType {
```

#### `STRUCT`: **DeviceFlags** <sub>line 22</sub>
```rust
pub struct DeviceFlags: u32 {
```

#### `STRUCT`: **DevAttachPayload** <sub>line 38</sub>
```rust
pub struct DevAttachPayload {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/kapi-abi/src/payloads/irq.rs</b> (1 items)</summary>

#### `STRUCT`: **IrqBindPayload** <sub>line 6</sub>
```rust
pub struct IrqBindPayload {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/kapi-abi/src/payloads/mem.rs</b> (2 items)</summary>

#### `STRUCT`: **MmioMapPayload** <sub>line 8</sub>
```rust
pub struct MmioMapPayload {
```

#### `STRUCT`: **DmaAllocPayload** <sub>line 25</sub>
```rust
pub struct DmaAllocPayload {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/kapi-abi/src/payloads/sys.rs</b> (1 items)</summary>

#### `STRUCT`: **LogPayload** <sub>line 9</sub>
```rust
pub struct LogPayload {
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/kapi-abi/src/primitives.rs</b> (6 items)</summary>

#### `STRUCT`: **Handle** <sub>line 8</sub>
```rust
pub struct Handle(pub u32);
```

#### `IMPL`: **Handle** <sub>line 10</sub>
```rust
impl Handle {
```

#### `FN`: **is_valid** <sub>line 14</sub>
```rust
pub fn is_valid(&self) -> bool {
```

#### `STRUCT`: **CapId** <sub>line 23</sub>
```rust
pub struct CapId(pub u64);
```

#### `STRUCT`: **PhysAddr** <sub>line 28</sub>
```rust
pub struct PhysAddr(pub u64);
```

#### `STRUCT`: **VirtAddr** <sub>line 33</sub>
```rust
pub struct VirtAddr(pub u64);
```

</details>

<details>
<summary><b>📄 driverspace_workspace/lib/kapi-syscall/src/lib.rs</b> (5 items)</summary>

#### `FN`: **kapi_ipc_send** <sub>line 16</sub>
```rust
fn kapi_ipc_send(target: u32, opcode: u32, payload: *const u8, payload_len: u32) -> i32;
```
> Wysyła jednokierunkową wiadomość IPC do `target`. 
> Zwraca 0 przy sukcesie albo kod `DsError`.

#### `FN`: **kapi_ipc_call** <sub>line 20</sub>
```rust
fn kapi_ipc_call(
```
> Wysyła żądanie IPC i czeka na odpowiedź do bufora `reply`. 
> Zwraca 0 przy sukcesie albo kod `DsError`.

#### `FN`: **status** <sub>line 31</sub>
```rust
fn status(rc: i32) -> Result<(), DsError> {
```

#### `FN`: **sys_ipc_send** <sub>line 40</sub>
```rust
pub fn sys_ipc_send(
```
> Wysyła wiadomość IPC (bez oczekiwania na odpowiedź).

#### `FN`: **sys_ipc_call** <sub>line 50</sub>
```rust
pub fn sys_ipc_call(
```
> Wysyła żądanie IPC i odbiera odpowiedź.

</details>

## 📂 driverspacelib/src

<details>
<summary><b>📄 driverspacelib/src/abi.rs</b> (7 items)</summary>

#### `ENUM`: **DsCmd** <sub>line 53</sub>
```rust
pub enum DsCmd {
```

#### `STRUCT`: **DsRing** <sub>line 83</sub>
```rust
pub struct DsRing {
```

#### `STRUCT`: **DsMsg** <sub>line 94</sub>
```rust
pub struct DsMsg {
```

#### `STRUCT`: **DsInitParams** <sub>line 110</sub>
```rust
pub struct DsInitParams {
```

#### `ENUM`: **DsError** <sub>line 123</sub>
```rust
pub enum DsError {
```

#### `IMPL`: **From** <sub>line 132</sub>
```rust
impl From<i32> for DsError {
```

#### `FN`: **from** <sub>line 133</sub>
```rust
fn from(v: i32) -> Self {
```

</details>

<details>
<summary><b>📄 driverspacelib/src/audio.rs</b> (5 items)</summary>

#### `STRUCT`: **AudioBars** <sub>line 4</sub>
```rust
pub struct AudioBars {
```

#### `FN`: **info_req** <sub>line 9</sub>
```rust
pub fn info_req() -> u64 {
```

#### `FN`: **info_take** <sub>line 13</sub>
```rust
pub fn info_take(id: u64) -> Option<AudioBars> {
```

#### `FN`: **page_phys_req** <sub>line 23</sub>
```rust
pub fn page_phys_req(va: u64) -> u64 {
```

#### `FN`: **page_phys_take** <sub>line 27</sub>
```rust
pub fn page_phys_take(id: u64) -> Option<u64> {
```

</details>

<details>
<summary><b>📄 driverspacelib/src/driver.rs</b> (3 items)</summary>

#### `STRUCT`: **DeviceInfo** <sub>line 6</sub>
```rust
pub struct DeviceInfo {
```

#### `TRAIT`: **Driver** <sub>line 14</sub>
```rust
pub trait Driver {
```
> Any driver that lives in driver space and is registered with the kernel 
> driver-space manager.

#### `FN`: **init** <sub>line 15</sub>
```rust
fn init(&mut self, info: &DeviceInfo) -> Result<(), DsError>;
```

</details>

<details>
<summary><b>📄 driverspacelib/src/input.rs</b> (2 items)</summary>

#### `FN`: **key_req** <sub>line 4</sub>
```rust
pub fn key_req() -> u64 {
```

#### `FN`: **key_take** <sub>line 8</sub>
```rust
pub fn key_take(id: u64) -> Option<u8> {
```

</details>

<details>
<summary><b>📄 driverspacelib/src/jack.rs</b> (6 items)</summary>

#### `STRUCT`: **JackInfo** <sub>line 4</sub>
```rust
pub struct JackInfo {
```

#### `FN`: **query_req** <sub>line 9</sub>
```rust
pub fn query_req() -> u64 {
```

#### `FN`: **query_take** <sub>line 13</sub>
```rust
pub fn query_take(id: u64) -> Option<JackInfo> {
```

#### `FN`: **set_amp** <sub>line 22</sub>
```rust
pub fn set_amp(on: bool) {
```

#### `FN`: **play** <sub>line 26</sub>
```rust
pub fn play(va: u64, len: u32) -> u64 {
```

#### `FN`: **stop** <sub>line 30</sub>
```rust
pub fn stop() {
```

</details>

<details>
<summary><b>📄 driverspacelib/src/log.rs</b> (2 items)</summary>

#### `FN`: **ds_log** <sub>line 10</sub>
```rust
pub fn ds_log(msg: &str) {
```
> Wysyła tekst do logu jądra (komenda `DsCmd::Log`).

#### `FN`: **ds_log_raw** <sub>line 15</sub>
```rust
pub fn ds_log_raw(ptr: *const u8, len: usize) {
```
> To samo, ale dla surowego wskaźnika i długości (np. bufora bajtów).

</details>

<details>
<summary><b>📄 driverspacelib/src/runtime.rs</b> (6 items)</summary>

#### `FN`: **init_once** <sub>line 38</sub>
```rust
pub fn init_once(params_va: u64) {
```
> Reads the boot parameters block and points our ring views at the 
> kernel-provided buffers.

#### `FN`: **register** <sub>line 51</sub>
```rust
pub fn register<D: Driver>(drv: &mut D) {
```
> Registers a driver (by calling its `Driver::init` hook) and notifies 
> the kernel that this driver space is now managed.

#### `FN`: **request** <sub>line 94</sub>
```rust
pub fn request(cmd: DsCmd, a0: u64, a1: u64, a2: u64) -> u64 {
```
> Sends a driver-space command to the kernel. Returns the id used later 
> with [`take_resp`].

#### `FN`: **request_raw** <sub>line 99</sub>
```rust
pub fn request_raw(cmd: u32, a0: u64, a1: u64, a2: u64) -> u64 {
```
> Sends a raw (service-class packed) command to the kernel.

#### `FN`: **tick** <sub>line 104</sub>
```rust
pub fn tick() {
```
> Drains pending kernel->driver responses into the local cache.

#### `FN`: **take_resp** <sub>line 128</sub>
```rust
pub fn take_resp(id: u64) -> Option<DsMsg> {
```
> Returns (and consumes) the response with the given request id.

</details>

<details>
<summary><b>📄 driverspacelib/src/svc.rs</b> (2 items)</summary>

#### `FN`: **call** <sub>line 4</sub>
```rust
pub fn call(class: u32, op: u32, a0: u64, a1: u64, a2: u64) -> u64 {
```

#### `FN`: **take** <sub>line 8</sub>
```rust
pub fn take(id: u64) -> Option<DsMsg> {
```

</details>

<details>
<summary><b>📄 driverspacelib/src/video.rs</b> (5 items)</summary>

#### `STRUCT`: **FbInfo** <sub>line 4</sub>
```rust
pub struct FbInfo {
```

#### `FN`: **fb_info_req** <sub>line 11</sub>
```rust
pub fn fb_info_req() -> u64 {
```

#### `FN`: **fb_info_take** <sub>line 15</sub>
```rust
pub fn fb_info_take(id: u64) -> Option<FbInfo> {
```

#### `FN`: **takeover** <sub>line 26</sub>
```rust
pub fn takeover() -> u64 {
```

#### `FN`: **release** <sub>line 30</sub>
```rust
pub fn release() -> u64 {
```

</details>

## 📂 kernel_Workspace/C_base

<details>
<summary><b>📄 kernel_Workspace/C_base/kc-abi-menager/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 2</sub>
```rust
fn main() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/C_base/kc-abi/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

## 📂 kernel_Workspace/Odin_base

<details>
<summary><b>📄 kernel_Workspace/Odin_base/odin-abi-bridge/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/Odin_base/odin-abi/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/Odin_base/odin-bin-loader/src/main.rs</b> (1 items)</summary>

#### `FN`: **main** <sub>line 2</sub>
```rust
fn main() {
```

</details>

## 📂 kernel_Workspace/base

<details>
<summary><b>📄 kernel_Workspace/base/kw-C-abi/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/base/kw-base/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/base/kw-libs/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/base/kw-odin-abi/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

## 📂 kernel_Workspace/core

<details>
<summary><b>📄 kernel_Workspace/core/kstd_alloc/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/core/kstd_base/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/core/kstd_core/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/core/kstd_data/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/core/kstd_io/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/core/linix_abi/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/core/linix_abi_com/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/core/linix_abi_driverspace/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/core/windows-com/src/lib.rs</b> (2 items)</summary>

#### `FN`: **add** <sub>line 2</sub>
```rust
pub fn add(left: u64, right: u64) -> u64 {
```

#### `FN`: **it_works** <sub>line 11</sub>
```rust
fn it_works() {
```

</details>

## 📂 kernel_Workspace/kernel

<details>
<summary><b>📄 kernel_Workspace/kernel/build.rs</b> (3 items)</summary>

#### `FN`: **collect_c_files** <sub>line 6</sub>
```rust
fn collect_c_files(dir: &Path, out: &mut Vec<PathBuf>) {
```

#### `FN`: **clang_target** <sub>line 23</sub>
```rust
fn clang_target(arch: &str) -> &'static str {
```

#### `FN`: **main** <sub>line 33</sub>
```rust
fn main() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/arch/mod.rs</b> (4 items)</summary>

#### `FN`: **init** <sub>line 13</sub>
```rust
pub fn init() {
```

#### `FN`: **hlt_loop** <sub>line 18</sub>
```rust
pub fn hlt_loop() -> ! {
```

#### `FN`: **now** <sub>line 23</sub>
```rust
pub fn now() -> u64 {
```

#### `FN`: **current_cpu** <sub>line 28</sub>
```rust
pub fn current_cpu() -> usize {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/arch/riscv64.rs</b> (10 items)</summary>

#### `STRUCT`: **HeapSpace** <sub>line 9</sub>
```rust
struct HeapSpace([u8; HEAP_SIZE]);
```

#### `STRUCT`: **Bump** <sub>line 13</sub>
```rust
struct Bump {
```

#### `FN`: **heap_init** <sub>line 20</sub>
```rust
pub fn heap_init() {
```

#### `STRUCT`: **KernelHeap** <sub>line 27</sub>
```rust
pub struct KernelHeap;
```

#### `FN`: **now** <sub>line 51</sub>
```rust
pub fn now() -> u64 {
```

#### `FN`: **riscv_panic** <sub>line 60</sub>
```rust
fn riscv_panic(info: &PanicInfo) -> ! {
```

#### `FN`: **init** <sub>line 66</sub>
```rust
pub fn init() {
```

#### `FN`: **current_cpu** <sub>line 74</sub>
```rust
pub fn current_cpu() -> usize {
```

#### `FN`: **hlt_loop** <sub>line 78</sub>
```rust
pub fn hlt_loop() -> ! {
```

#### `FN`: **early_uart** <sub>line 86</sub>
```rust
pub fn early_uart(s: &str) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/arch/x86_64.rs</b> (6 items)</summary>

#### `FN`: **x86_panic** <sub>line 6</sub>
```rust
fn x86_panic(info: &PanicInfo) -> ! {
```

#### `FN`: **init** <sub>line 10</sub>
```rust
pub fn init() {
```

#### `FN`: **hlt_loop** <sub>line 18</sub>
```rust
pub fn hlt_loop() -> ! {
```

#### `FN`: **current_cpu** <sub>line 24</sub>
```rust
pub fn current_cpu() -> usize {
```

#### `FN`: **now** <sub>line 36</sub>
```rust
pub fn now() -> u64 {
```

#### `FN`: **x86_boot** <sub>line 44</sub>
```rust
fn x86_boot(boot_info: &'static BootInfo) -> ! {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/audio/jack.rs</b> (6 items)</summary>

#### `FN`: **init** <sub>line 6</sub>
```rust
pub fn init(base: u32) -> bool {
```

#### `FN`: **poll_jack** <sub>line 10</sub>
```rust
pub fn poll_jack() -> bool {
```

#### `FN`: **query** <sub>line 23</sub>
```rust
pub fn query() -> u32 {
```

#### `FN`: **set_amp** <sub>line 29</sub>
```rust
pub fn set_amp(on: bool) {
```

#### `FN`: **play_phys** <sub>line 34</sub>
```rust
pub fn play_phys(phys: u64, len: u32) -> bool {
```

#### `FN`: **stop** <sub>line 38</sub>
```rust
pub fn stop() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/battery/aut.rs</b> (1 items)</summary>

#### `FN`: **authorize** <sub>line 4</sub>
```rust
pub fn authorize(ring: u8, op: u8) -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/battery/battery.h</b> (8 items)</summary>

#### `FUNCTION`: **battery_init** <sub>line 21</sub>
```c
bool battery_init(void);
```

#### `FUNCTION`: **battery_present** <sub>line 22</sub>
```c
bool battery_present(void);
```

#### `FUNCTION`: **battery_status** <sub>line 23</sub>
```c
bool battery_status(battery_status_t *out);
```

#### `FUNCTION`: **battery_status_packed** <sub>line 24</sub>
```c
bool battery_status_packed(uint64_t *a0, uint64_t *a1, uint64_t *a2);
```

#### `FUNCTION`: **battery_set_threshold** <sub>line 25</sub>
```c
bool battery_set_threshold(uint32_t low_pct);
```

#### `FUNCTION`: **battery_threshold** <sub>line 26</sub>
```c
uint32_t battery_threshold(void);
```

#### `FUNCTION`: **battery_low_pending** <sub>line 27</sub>
```c
bool battery_low_pending(void);
```

#### `FUNCTION`: **battery_sim_tick** <sub>line 28</sub>
```c
void battery_sim_tick(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/battery/bridge.rs</b> (3 items)</summary>

#### `FN`: **battery_status_packed** <sub>line 5</sub>
```rust
fn battery_status_packed(a0: *mut u64, a1: *mut u64, a2: *mut u64) -> bool;
```

#### `FN`: **battery_set_threshold** <sub>line 6</sub>
```rust
fn battery_set_threshold(pct: u32) -> bool;
```

#### `FN`: **batt_call** <sub>line 9</sub>
```rust
pub fn batt_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/battery/init.c</b> (1 items)</summary>

#### `FUNCTION`: **battery_init** <sub>line 6</sub>
```c
bool battery_init(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/battery/init.rs</b> (2 items)</summary>

#### `FN`: **battery_init** <sub>line 2</sub>
```rust
fn battery_init() -> bool;
```

#### `FN`: **init** <sub>line 5</sub>
```rust
pub fn init() -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/battery/operation.c</b> (10 items)</summary>

#### `FUNCTION`: **model_present** <sub>line 16</sub>
```c
static bool model_present(void)
```

#### `FUNCTION`: **model_status** <sub>line 21</sub>
```c
static bool model_status(battery_status_t *out)
```

#### `FUNCTION`: **battery_register_backend** <sub>line 38</sub>
```c
void battery_register_backend(const battery_backend_t *b)
```

#### `FUNCTION`: **battery_present** <sub>line 45</sub>
```c
bool battery_present(void)
```

#### `FUNCTION`: **battery_status** <sub>line 50</sub>
```c
bool battery_status(battery_status_t *out)
```

#### `FUNCTION`: **battery_status_packed** <sub>line 59</sub>
```c
bool battery_status_packed(uint64_t *a0, uint64_t *a1, uint64_t *a2)
```

#### `FUNCTION`: **battery_set_threshold** <sub>line 77</sub>
```c
bool battery_set_threshold(uint32_t low_pct)
```

#### `FUNCTION`: **battery_threshold** <sub>line 89</sub>
```c
uint32_t battery_threshold(void)
```

#### `FUNCTION`: **battery_low_pending** <sub>line 94</sub>
```c
bool battery_low_pending(void)
```

#### `FUNCTION`: **battery_sim_tick** <sub>line 101</sub>
```c
void battery_sim_tick(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/battery/operation.h</b> (1 items)</summary>

#### `FUNCTION`: **battery_register_backend** <sub>line 11</sub>
```c
void battery_register_backend(const battery_backend_t *b);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/bluetooth/aut.rs</b> (1 items)</summary>

#### `FN`: **authorize** <sub>line 8</sub>
```rust
pub fn authorize(ring: u8, op: u32) -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/bluetooth/bridge.rs</b> (8 items)</summary>

#### `FN`: **bt_init** <sub>line 7</sub>
```rust
fn bt_init() -> bool;
```

#### `FN`: **bt_ready** <sub>line 8</sub>
```rust
fn bt_ready() -> bool;
```

#### `FN`: **bt_info** <sub>line 9</sub>
```rust
fn bt_info(ver: *mut u8, bdaddr: *mut u8);
```

#### `FN`: **bt_hci_cmd** <sub>line 10</sub>
```rust
fn bt_hci_cmd(opcode: u16, params: *const u8, len: u8) -> bool;
```

#### `FN`: **bt_event_poll** <sub>line 11</sub>
```rust
fn bt_event_poll(buf: *mut u8, cap: u8, len: *mut u8) -> bool;
```

#### `FN`: **bt_acl_send** <sub>line 12</sub>
```rust
fn bt_acl_send(data: *const u8, len: u16) -> bool;
```

#### `FN`: **bt_acl_recv** <sub>line 13</sub>
```rust
fn bt_acl_recv(data: *mut u8, cap: u16, len: *mut u16) -> bool;
```

#### `FN`: **bt_call** <sub>line 20</sub>
```rust
pub fn bt_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/bluetooth/bt.h</b> (7 items)</summary>

#### `FUNCTION`: **bt_init** <sub>line 8</sub>
```c
bool bt_init(void);
```

#### `FUNCTION`: **bt_ready** <sub>line 9</sub>
```c
bool bt_ready(void);
```

#### `FUNCTION`: **bt_info** <sub>line 10</sub>
```c
void bt_info(uint8_t *hci_ver, uint8_t *bdaddr);
```

#### `FUNCTION`: **bt_hci_cmd** <sub>line 11</sub>
```c
bool bt_hci_cmd(uint16_t opcode, const uint8_t *params, uint8_t len);
```

#### `FUNCTION`: **bt_event_poll** <sub>line 12</sub>
```c
bool bt_event_poll(uint8_t *buf, uint8_t cap, uint8_t *len);
```

#### `FUNCTION`: **bt_acl_send** <sub>line 13</sub>
```c
bool bt_acl_send(const uint8_t *data, uint16_t len);
```

#### `FUNCTION`: **bt_acl_recv** <sub>line 14</sub>
```c
bool bt_acl_recv(uint8_t *data, uint16_t cap, uint16_t *len);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/bluetooth/init.c</b> (4 items)</summary>

#### `FUNCTION`: **wait_cmd_complete** <sub>line 9</sub>
```c
static bool wait_cmd_complete(uint16_t opcode, uint8_t *ret, uint8_t ret_len)
```

#### `FUNCTION`: **bt_init** <sub>line 45</sub>
```c
bool bt_init(void)
```

#### `FUNCTION`: **bt_ready** <sub>line 84</sub>
```c
bool bt_ready(void)
```

#### `FUNCTION`: **bt_info** <sub>line 89</sub>
```c
void bt_info(uint8_t *hci_ver, uint8_t *bdaddr)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/bluetooth/init.rs</b> (4 items)</summary>

#### `FN`: **bt_init** <sub>line 2</sub>
```rust
fn bt_init() -> bool;
```

#### `FN`: **bt_ready** <sub>line 3</sub>
```rust
fn bt_ready() -> bool;
```

#### `FN`: **init** <sub>line 6</sub>
```rust
pub fn init() -> bool {
```

#### `FN`: **ready** <sub>line 10</sub>
```rust
pub fn ready() -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/bluetooth/operation.c</b> (12 items)</summary>

#### `FUNCTION`: **evt_push** <sub>line 16</sub>
```c
static bool evt_push(const uint8_t *data, uint8_t len)
```

#### `FUNCTION`: **bt_op_model_init** <sub>line 36</sub>
```c
void bt_op_model_init(void)
```

#### `FUNCTION`: **bt_op_model_run_cmd** <sub>line 45</sub>
```c
bool bt_op_model_run_cmd(uint16_t opcode, const uint8_t *params, uint8_t len)
```

#### `FUNCTION`: **evt_push** <sub>line 62</sub>
```c
return evt_push(ev, 7);
```

#### `FUNCTION`: **evt_push** <sub>line 80</sub>
```c
return evt_push(ev, 15);
```

#### `FUNCTION`: **evt_push** <sub>line 93</sub>
```c
return evt_push(ev, 13);
```

#### `FUNCTION`: **evt_push** <sub>line 103</sub>
```c
return evt_push(ev, 7);
```

#### `FUNCTION`: **bt_hci_cmd** <sub>line 107</sub>
```c
bool bt_hci_cmd(uint16_t opcode, const uint8_t *params, uint8_t len)
```

#### `FUNCTION`: **bt_op_model_run_cmd** <sub>line 113</sub>
```c
return bt_op_model_run_cmd(opcode, params, len);
```

#### `FUNCTION`: **bt_event_poll** <sub>line 116</sub>
```c
bool bt_event_poll(uint8_t *buf, uint8_t cap, uint8_t *len)
```

#### `FUNCTION`: **bt_acl_send** <sub>line 136</sub>
```c
bool bt_acl_send(const uint8_t *data, uint16_t len)
```

#### `FUNCTION`: **bt_acl_recv** <sub>line 156</sub>
```c
bool bt_acl_recv(uint8_t *data, uint16_t cap, uint16_t *len)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/bluetooth/operation.h</b> (2 items)</summary>

#### `FUNCTION`: **bt_op_model_init** <sub>line 11</sub>
```c
void bt_op_model_init(void);
```

#### `FUNCTION`: **bt_op_model_run_cmd** <sub>line 12</sub>
```c
bool bt_op_model_run_cmd(uint16_t opcode, const uint8_t *params, uint8_t len);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/camera/aut.rs</b> (1 items)</summary>

#### `FN`: **authorize** <sub>line 6</sub>
```rust
pub fn authorize(ring: u8, op: u8) -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/camera/bridge.rs</b> (6 items)</summary>

#### `FN`: **camera_caps_get_w** <sub>line 5</sub>
```rust
fn camera_caps_get_w(w: *mut u32, h: *mut u32, fmt: *mut u32, fps: *mut u32) -> bool;
```

#### `FN`: **camera_start** <sub>line 6</sub>
```rust
fn camera_start() -> bool;
```

#### `FN`: **camera_stop** <sub>line 7</sub>
```rust
fn camera_stop() -> bool;
```

#### `FN`: **camera_frame_to_phys** <sub>line 8</sub>
```rust
fn camera_frame_to_phys(phys: u64, cap: u32, fid: *mut u64) -> bool;
```

#### `FN`: **grant_phys** <sub>line 11</sub>
```rust
fn grant_phys(va: u64) -> Option<u64> {
```

#### `FN`: **cam_call** <sub>line 15</sub>
```rust
pub fn cam_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/camera/camera.h</b> (7 items)</summary>

#### `FUNCTION`: **camera_init** <sub>line 19</sub>
```c
bool camera_init(void);
```

#### `FUNCTION`: **camera_present** <sub>line 20</sub>
```c
bool camera_present(void);
```

#### `FUNCTION`: **camera_caps_get** <sub>line 21</sub>
```c
bool camera_caps_get(camera_caps_t *out);
```

#### `FUNCTION`: **camera_start** <sub>line 22</sub>
```c
bool camera_start(void);
```

#### `FUNCTION`: **camera_stop** <sub>line 23</sub>
```c
bool camera_stop(void);
```

#### `FUNCTION`: **camera_frame** <sub>line 24</sub>
```c
bool camera_frame(void *buf, uint32_t cap, uint64_t *frame_id);
```

#### `FUNCTION`: **camera_frame_to_phys** <sub>line 25</sub>
```c
bool camera_frame_to_phys(uint64_t phys, uint32_t cap, uint64_t *frame_id);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/camera/init.c</b> (1 items)</summary>

#### `FUNCTION`: **camera_init** <sub>line 5</sub>
```c
bool camera_init(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/camera/init.rs</b> (2 items)</summary>

#### `FN`: **camera_init** <sub>line 2</sub>
```rust
fn camera_init() -> bool;
```

#### `FN`: **init** <sub>line 5</sub>
```rust
pub fn init() -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/camera/operation.c</b> (12 items)</summary>

#### `FUNCTION`: **model_present** <sub>line 15</sub>
```c
static bool model_present(void)
```

#### `FUNCTION`: **model_start** <sub>line 20</sub>
```c
static bool model_start(void)
```

#### `FUNCTION`: **model_stop** <sub>line 27</sub>
```c
static bool model_stop(void)
```

#### `FUNCTION`: **gen_frame** <sub>line 33</sub>
```c
static void gen_frame(void)
```

#### `FUNCTION`: **model_frame** <sub>line 56</sub>
```c
static bool model_frame(void *buf, uint32_t cap, uint64_t *fid)
```

#### `FUNCTION`: **camera_register_backend** <sub>line 94</sub>
```c
void camera_register_backend(const camera_backend_t *b)
```

#### `FUNCTION`: **camera_present** <sub>line 101</sub>
```c
bool camera_present(void)
```

#### `FUNCTION`: **camera_caps_get** <sub>line 106</sub>
```c
bool camera_caps_get(camera_caps_t *out)
```

#### `FUNCTION`: **camera_start** <sub>line 121</sub>
```c
bool camera_start(void)
```

#### `FUNCTION`: **camera_stop** <sub>line 130</sub>
```c
bool camera_stop(void)
```

#### `FUNCTION`: **camera_frame** <sub>line 135</sub>
```c
bool camera_frame(void *buf, uint32_t cap, uint64_t *fid)
```

#### `FUNCTION`: **camera_frame_to_phys** <sub>line 144</sub>
```c
bool camera_frame_to_phys(uint64_t phys, uint32_t cap, uint64_t *fid)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/camera/operation.h</b> (1 items)</summary>

#### `FUNCTION`: **camera_register_backend** <sub>line 13</sub>
```c
void camera_register_backend(const camera_backend_t *b);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/audit.rs</b> (17 items)</summary>

#### `ENUM`: **EventKind** <sub>line 6</sub>
```rust
pub enum EventKind {
```

#### `STRUCT`: **AuditEvent** <sub>line 16</sub>
```rust
pub struct AuditEvent {
```

#### `STRUCT`: **AuditInner** <sub>line 27</sub>
```rust
struct AuditInner {
```

#### `FN`: **init_audit_log** <sub>line 41</sub>
```rust
pub fn init_audit_log() -> Result<(), &'static str> {
```

#### `FN`: **now_tick** <sub>line 52</sub>
```rust
fn now_tick() -> u64 {
```

#### `FN`: **push** <sub>line 56</sub>
```rust
fn push(kind: EventKind, world: u32, target: u32, cap: Capability) {
```

#### `FN`: **log_check** <sub>line 73</sub>
```rust
pub fn log_check(world: u32, cap: Capability, ok: bool) {
```

#### `FN`: **log_grant** <sub>line 78</sub>
```rust
pub fn log_grant(granter: u32, target: u32, cap: Capability, ok: bool) {
```

#### `FN`: **log_revoke** <sub>line 84</sub>
```rust
pub fn log_revoke(world: u32, cap: Capability, ok: bool) {
```

#### `FN`: **log_register** <sub>line 90</sub>
```rust
pub fn log_register(world: u32) {
```

#### `FN`: **log_unregister** <sub>line 94</sub>
```rust
pub fn log_unregister(world: u32) {
```

#### `FN`: **count** <sub>line 98</sub>
```rust
pub fn count() -> usize {
```

#### `FN`: **recent** <sub>line 102</sub>
```rust
pub fn recent(n: usize) -> Vec<AuditEvent> {
```

#### `FN`: **by_world** <sub>line 119</sub>
```rust
pub fn by_world(world: u32, limit: usize) -> Vec<AuditEvent> {
```

#### `FN`: **by_kind** <sub>line 126</sub>
```rust
pub fn by_kind(kind: EventKind, limit: usize) -> Vec<AuditEvent> {
```

#### `FN`: **deny_count** <sub>line 133</sub>
```rust
pub fn deny_count() -> usize {
```

#### `FN`: **test_audit** <sub>line 142</sub>
```rust
fn test_audit() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/caps.h</b> (8 items)</summary>

#### `FUNCTION`: **caps_self_bits** <sub>line 10</sub>
```c
unsigned int caps_self_bits(void);
```

#### `FUNCTION`: **caps_world_bits** <sub>line 12</sub>
```c
unsigned int caps_world_bits(unsigned int world_id);
```

#### `FUNCTION`: **caps_has** <sub>line 14</sub>
```c
int caps_has(unsigned int world_id, unsigned char cap_id);
```

#### `FUNCTION`: **caps_name** <sub>line 16</sub>
```c
int caps_name(unsigned char cap_id, unsigned char *buf, unsigned int len);
```

#### `FUNCTION`: **caps_request** <sub>line 18</sub>
```c
int caps_request(unsigned int target, unsigned char cap_id);
```

#### `FUNCTION`: **caps_release** <sub>line 20</sub>
```c
int caps_release(unsigned int world_id, unsigned char cap_id);
```

#### `FUNCTION`: **caps_world_count** <sub>line 22</sub>
```c
unsigned int caps_world_count(void);
```

#### `FUNCTION`: **caps_audit_count** <sub>line 24</sub>
```c
unsigned long long caps_audit_count(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/check.rs</b> (21 items)</summary>

#### `FN`: **kernel_world_id** <sub>line 10</sub>
```rust
pub fn kernel_world_id() -> u32 {
```

#### `FN`: **set_kernel_world_id** <sub>line 14</sub>
```rust
pub fn set_kernel_world_id(id: u32) {
```

#### `FN`: **current_world_id_pub** <sub>line 18</sub>
```rust
pub fn current_world_id_pub() -> u32 {
```

#### `FN`: **set_current_world** <sub>line 22</sub>
```rust
pub fn set_current_world(world_id: u32) {
```

#### `FN`: **current_world_id** <sub>line 26</sub>
```rust
fn current_world_id() -> u32 {
```

#### `FN`: **has_cap** <sub>line 30</sub>
```rust
pub fn has_cap(cap: Capability) -> bool {
```

#### `FN`: **world_has_cap** <sub>line 35</sub>
```rust
pub fn world_has_cap(world_id: u32, cap: Capability) -> bool {
```

#### `FN`: **require_cap** <sub>line 39</sub>
```rust
pub fn require_cap(cap: Capability) -> CapResult<()> {
```

#### `FN`: **require_world_cap** <sub>line 54</sub>
```rust
pub fn require_world_cap(world_id: u32, cap: Capability) -> CapResult<()> {
```

#### `FN`: **require_caps** <sub>line 67</sub>
```rust
pub fn require_caps(caps: &[Capability]) -> CapResult<()> {
```

#### `FN`: **with_cap** <sub>line 74</sub>
```rust
pub fn with_cap<T, F>(cap: Capability, f: F) -> CapResult<T>
```

#### `FN`: **with_caps** <sub>line 82</sub>
```rust
pub fn with_caps<T, F>(caps: &[Capability], f: F) -> CapResult<T>
```

#### `FN`: **assert_cap** <sub>line 104</sub>
```rust
pub fn assert_cap(cap: Capability) {
```

#### `FN`: **fast_check** <sub>line 111</sub>
```rust
pub fn fast_check(cap: Capability) -> bool {
```

#### `FN`: **if_cap** <sub>line 115</sub>
```rust
pub fn if_cap<T, F>(cap: Capability, f: F, default: T) -> T
```

#### `STRUCT`: **TemporaryCaps** <sub>line 126</sub>
```rust
pub struct TemporaryCaps {
```

#### `IMPL`: **TemporaryCaps** <sub>line 131</sub>
```rust
impl TemporaryCaps {
```

#### `FN`: **enter** <sub>line 132</sub>
```rust
pub fn enter(extra: CapabilitySet) -> CapResult<Self> {
```

#### `IMPL`: **Drop** <sub>line 146</sub>
```rust
impl Drop for TemporaryCaps {
```

#### `FN`: **drop** <sub>line 147</sub>
```rust
fn drop(&mut self) {
```

#### `FN`: **test_require** <sub>line 158</sub>
```rust
fn test_require() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/defaults.rs</b> (3 items)</summary>

#### `FN`: **default_for_ring** <sub>line 6</sub>
```rust
pub fn default_for_ring(ring: u8) -> CapabilitySet {
```

#### `FN`: **install_defaults** <sub>line 15</sub>
```rust
pub fn install_defaults() -> Result<(), &'static str> {
```

#### `FN`: **register_for_ring** <sub>line 22</sub>
```rust
pub fn register_for_ring(ring: u8, parent: Option<u32>) -> Result<u32, &'static str> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/export.rs</b> (1 items)</summary>

#### `FN`: **cap_from_id** <sub>line 7</sub>
```rust
fn cap_from_id(id: u8) -> Option<Capability> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/grant.rs</b> (10 items)</summary>

#### `FN`: **grant_cap** <sub>line 18</sub>
```rust
pub fn grant_cap(granter: u32, target: u32, cap: Capability) -> CapResult<()> {
```

#### `FN`: **delegate_caps** <sub>line 31</sub>
```rust
pub fn delegate_caps(granter: u32, target: u32, caps: CapabilitySet) -> CapResult<()> {
```

#### `FN`: **inherit_set** <sub>line 38</sub>
```rust
pub fn inherit_set(parent_world: u32) -> CapabilitySet {
```

#### `FN`: **spawn_child** <sub>line 51</sub>
```rust
pub fn spawn_child(parent_world: u32) -> Result<u32, &'static str> {
```

#### `STRUCT`: **TempGrant** <sub>line 58</sub>
```rust
struct TempGrant {
```

#### `FN`: **now_tick** <sub>line 74</sub>
```rust
fn now_tick() -> u64 {
```

#### `FN`: **grant_temporary** <sub>line 78</sub>
```rust
pub fn grant_temporary(granter: u32, target: u32, cap: Capability,
```

#### `FN`: **prune_expired** <sub>line 97</sub>
```rust
pub fn prune_expired() {
```

#### `FN`: **is_temporary** <sub>line 110</sub>
```rust
pub fn is_temporary(world_id: u32, cap: Capability) -> bool {
```

#### `FN`: **test_inherit** <sub>line 121</sub>
```rust
fn test_inherit() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/hierarchy.rs</b> (10 items)</summary>

#### `FN`: **parent** <sub>line 5</sub>
```rust
pub fn parent(cap: Capability) -> Option<Capability> {
```

#### `FN`: **implies** <sub>line 47</sub>
```rust
pub fn implies(held: Capability, required: Capability) -> bool {
```

#### `FN`: **set_implies** <sub>line 62</sub>
```rust
pub fn set_implies(held: CapabilitySet, required: Capability) -> bool {
```

#### `FN`: **expand_hierarchy** <sub>line 71</sub>
```rust
pub fn expand_hierarchy(set: CapabilitySet) -> CapabilitySet {
```

#### `FN`: **path_to_root** <sub>line 85</sub>
```rust
pub fn path_to_root(cap: Capability) -> Vec<Capability> {
```

#### `FN`: **depth** <sub>line 96</sub>
```rust
pub fn depth(cap: Capability) -> usize {
```

#### `FN`: **subtree** <sub>line 106</sub>
```rust
pub fn subtree(root: Capability) -> Vec<Capability> {
```

#### `FN`: **can_delegate** <sub>line 123</sub>
```rust
pub fn can_delegate(holder: CapabilitySet, candidate: Capability) -> bool {
```

#### `FN`: **test_hierarchy** <sub>line 132</sub>
```rust
fn test_hierarchy() {
```

#### `FN`: **test_depth** <sub>line 149</sub>
```rust
fn test_depth() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/mod.rs</b> (4 items)</summary>

#### `FN`: **init** <sub>line 31</sub>
```rust
pub fn init() -> Result<(), &'static str> {
```

#### `FN`: **snapshot** <sub>line 38</sub>
```rust
pub fn snapshot() -> SystemSnapshot {
```

#### `STRUCT`: **SystemSnapshot** <sub>line 48</sub>
```rust
pub struct SystemSnapshot {
```

#### `FN`: **self_test** <sub>line 55</sub>
```rust
pub fn self_test() -> crate::testing::TestResult {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/policy.rs</b> (6 items)</summary>

#### `TYPE`: **PolicyHook** <sub>line 5</sub>
```rust
pub type PolicyHook = fn(world: u32, cap: Capability) -> bool;
```

#### `FN`: **set_hook** <sub>line 9</sub>
```rust
pub fn set_hook(h: PolicyHook) {
```

#### `FN`: **policy_allows** <sub>line 14</sub>
```rust
fn policy_allows(world: u32, cap: Capability) -> bool {
```

#### `FN`: **enforce** <sub>line 23</sub>
```rust
pub fn enforce(world: u32, cap: Capability) -> CapResult<()> {
```

#### `FN`: **enforce_self** <sub>line 35</sub>
```rust
pub fn enforce_self(cap: Capability) -> CapResult<()> {
```

#### `FN`: **allowed** <sub>line 40</sub>
```rust
pub fn allowed(world: u32, cap: Capability) -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/revoke.rs</b> (8 items)</summary>

#### `FN`: **revoke_from_world** <sub>line 7</sub>
```rust
pub fn revoke_from_world(world_id: u32, cap: Capability) -> CapResult<()> {
```

#### `FN`: **revoke_subtree** <sub>line 15</sub>
```rust
pub fn revoke_subtree(world_id: u32, cap: Capability) -> CapResult<()> {
```

#### `FN`: **revoke_global** <sub>line 23</sub>
```rust
pub fn revoke_global(cap: Capability) {
```

#### `FN`: **restore_global** <sub>line 28</sub>
```rust
pub fn restore_global(cap: Capability) {
```

#### `FN`: **revoked_list** <sub>line 32</sub>
```rust
pub fn revoked_list() -> Vec<Capability> {
```

#### `FN`: **is_globally_revoked** <sub>line 43</sub>
```rust
pub fn is_globally_revoked(cap: Capability) -> bool {
```

#### `FN`: **lockdown** <sub>line 47</sub>
```rust
pub fn lockdown(world_id: u32) {
```

#### `FN`: **test_revoke** <sub>line 68</sub>
```rust
fn test_revoke() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/sets.rs</b> (22 items)</summary>

#### `STRUCT`: **CapSetBuilder** <sub>line 4</sub>
```rust
pub struct CapSetBuilder {
```

#### `IMPL`: **CapSetBuilder** <sub>line 8</sub>
```rust
impl CapSetBuilder {
```

#### `FN`: **new** <sub>line 9</sub>
```rust
pub fn new() -> Self {
```

#### `FN`: **empty** <sub>line 13</sub>
```rust
pub fn empty() -> Self {
```

#### `FN`: **add** <sub>line 17</sub>
```rust
pub fn add(mut self, cap: Capability) -> Self {
```

#### `FN`: **add_many** <sub>line 22</sub>
```rust
pub fn add_many(mut self, caps: &[Capability]) -> Self {
```

#### `FN`: **add_category** <sub>line 29</sub>
```rust
pub fn add_category(mut self, cat: super::types::CapCategory) -> Self {
```

#### `FN`: **remove** <sub>line 38</sub>
```rust
pub fn remove(mut self, cap: Capability) -> Self {
```

#### `FN`: **with_hierarchy** <sub>line 43</sub>
```rust
pub fn with_hierarchy(self) -> Self {
```

#### `FN`: **build** <sub>line 47</sub>
```rust
pub fn build(self) -> CapabilitySet {
```

#### `FN`: **minimal_user** <sub>line 55</sub>
```rust
pub fn minimal_user() -> CapabilitySet {
```

#### `FN`: **standard_user** <sub>line 67</sub>
```rust
pub fn standard_user() -> CapabilitySet {
```

#### `FN`: **privileged_user** <sub>line 84</sub>
```rust
pub fn privileged_user() -> CapabilitySet {
```

#### `FN`: **driver** <sub>line 98</sub>
```rust
pub fn driver() -> CapabilitySet {
```

#### `FN`: **kernel** <sub>line 111</sub>
```rust
pub fn kernel() -> CapabilitySet {
```

#### `FN`: **sandbox** <sub>line 115</sub>
```rust
pub fn sandbox() -> CapabilitySet {
```

#### `FN`: **validate_hierarchy** <sub>line 125</sub>
```rust
pub fn validate_hierarchy(set: CapabilitySet) -> Result<(), &'static str> {
```

#### `FN`: **effective** <sub>line 141</sub>
```rust
pub fn effective(set: CapabilitySet) -> CapabilitySet {
```

#### `FN`: **effective_diff** <sub>line 145</sub>
```rust
pub fn effective_diff(a: CapabilitySet, b: CapabilitySet) -> CapabilitySet {
```

#### `FN`: **is_subset_with_hierarchy** <sub>line 151</sub>
```rust
pub fn is_subset_with_hierarchy(subset: CapabilitySet, superset: CapabilitySet) -> bool {
```

#### `FN`: **test_presets** <sub>line 162</sub>
```rust
fn test_presets() {
```

#### `FN`: **test_effective** <sub>line 174</sub>
```rust
fn test_effective() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/store.rs</b> (17 items)</summary>

#### `STRUCT`: **WorldCaps** <sub>line 9</sub>
```rust
pub struct WorldCaps {
```

#### `STRUCT`: **StoreInner** <sub>line 22</sub>
```rust
struct StoreInner {
```

#### `FN`: **init_store** <sub>line 28</sub>
```rust
pub fn init_store() -> Result<(), &'static str> {
```

#### `FN`: **global_caps** <sub>line 36</sub>
```rust
pub fn global_caps() -> CapabilitySet {
```

#### `FN`: **register_world** <sub>line 41</sub>
```rust
pub fn register_world(parent_world: Option<u32>, initial: CapabilitySet) -> Result<u32, &'static str> {
```

#### `FN`: **find_world_inner** <sub>line 71</sub>
```rust
fn find_world_inner(s: &StoreInner, world_id: u32) -> Result<WorldCaps, &'static str> {
```

#### `FN`: **get_world_caps** <sub>line 82</sub>
```rust
pub fn get_world_caps(world_id: u32) -> Result<CapabilitySet, &'static str> {
```

#### `FN`: **set_world_caps** <sub>line 88</sub>
```rust
pub fn set_world_caps(world_id: u32, new_caps: CapabilitySet) -> Result<(), &'static str> {
```

#### `FN`: **add_world_cap** <sub>line 115</sub>
```rust
pub fn add_world_cap(world_id: u32, cap: Capability) -> Result<(), &'static str> {
```

#### `FN`: **remove_world_cap** <sub>line 120</sub>
```rust
pub fn remove_world_cap(world_id: u32, cap: Capability) -> Result<(), &'static str> {
```

#### `FN`: **unregister_world** <sub>line 138</sub>
```rust
pub fn unregister_world(world_id: u32) -> Result<(), &'static str> {
```

#### `FN`: **world_has_cap** <sub>line 153</sub>
```rust
pub fn world_has_cap(world_id: u32, cap: Capability) -> bool {
```

#### `FN`: **world_count** <sub>line 163</sub>
```rust
pub fn world_count() -> usize {
```

#### `FN`: **iter_worlds** <sub>line 168</sub>
```rust
pub fn iter_worlds<F: FnMut(u32, CapabilitySet)>(mut f: F) {
```

#### `FN`: **add_global_revoked** <sub>line 177</sub>
```rust
pub fn add_global_revoked(cap: Capability) {
```

#### `FN`: **remove_global_revoked** <sub>line 182</sub>
```rust
pub fn remove_global_revoked(cap: Capability) {
```

#### `FN`: **test_world_registration** <sub>line 192</sub>
```rust
fn test_world_registration() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/syscalls.rs</b> (2 items)</summary>

#### `FN`: **cap_from_id** <sub>line 13</sub>
```rust
fn cap_from_id(id: u8) -> Option<Capability> {
```

#### `FN`: **cap_syscall** <sub>line 17</sub>
```rust
pub fn cap_syscall(num: u64, a0: u64, a1: u64, a2: u64) -> u64 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/tests.rs</b> (4 items)</summary>

#### `FN`: **test_full_flow** <sub>line 8</sub>
```rust
fn test_full_flow() {
```

#### `FN`: **test_hierarchy_enforcement** <sub>line 29</sub>
```rust
fn test_hierarchy_enforcement() {
```

#### `FN`: **test_global_revocation** <sub>line 40</sub>
```rust
fn test_global_revocation() {
```

#### `FN`: **test_audit_trail** <sub>line 54</sub>
```rust
fn test_audit_trail() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/caps/types.rs</b> (17 items)</summary>

#### `TYPE`: **CapID** <sub>line 3</sub>
```rust
pub type CapID = u8;
```

#### `ENUM`: **Capability** <sub>line 10</sub>
```rust
pub enum Capability {
```

#### `IMPL`: **Capability** <sub>line 49</sub>
```rust
impl Capability {
```

#### `FN`: **iter_all** <sub>line 99</sub>
```rust
pub fn iter_all() -> impl Iterator<Item = Capability> {
```

#### `FN`: **category** <sub>line 112</sub>
```rust
pub fn category(self) -> CapCategory {
```

#### `ENUM`: **CapCategory** <sub>line 127</sub>
```rust
pub enum CapCategory {
```

#### `IMPL`: **CapCategory** <sub>line 138</sub>
```rust
impl CapCategory {
```

#### `STRUCT`: **CapabilitySet** <sub>line 154</sub>
```rust
pub struct CapabilitySet {
```

#### `IMPL`: **CapabilitySet** <sub>line 158</sub>
```rust
impl CapabilitySet {
```

#### `FN`: **iter** <sub>line 215</sub>
```rust
pub fn iter(self) -> impl Iterator<Item = Capability> {
```

#### `IMPL`: **fmt** <sub>line 220</sub>
```rust
impl fmt::Debug for CapabilitySet {
```

#### `FN`: **fmt** <sub>line 221</sub>
```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

#### `STRUCT`: **CapabilityError** <sub>line 234</sub>
```rust
pub struct CapabilityError {
```

#### `IMPL`: **fmt** <sub>line 239</sub>
```rust
impl fmt::Display for CapabilityError {
```

#### `FN`: **fmt** <sub>line 240</sub>
```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

#### `TYPE`: **CapResult** <sub>line 249</sub>
```rust
pub type CapResult<T> = Result<T, CapabilityError>;
```

#### `FN`: **test_set_operations** <sub>line 256</sub>
```rust
fn test_set_operations() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/ast.c</b> (1 items)</summary>

#### `FUNCTION`: **cl_arena_init** <sub>line 3</sub>
```c
void cl_arena_init(arena_t *a, uint8_t *buf, size_t cap)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/ast.h</b> (1 items)</summary>

#### `FUNCTION`: **cl_arena_init** <sub>line 50</sub>
```c
void cl_arena_init(arena_t *a, uint8_t *buf, size_t cap);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/bridge.c</b> (18 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 3</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **k_input_key** <sub>line 4</sub>
```c
extern int32_t k_input_key(void);
```

#### `FUNCTION`: **k_fs_read** <sub>line 5</sub>
```c
extern int32_t k_fs_read(const char *path, void *buf, uint32_t cap);
```

#### `FUNCTION`: **k_getpid** <sub>line 6</sub>
```c
extern uint32_t k_getpid(void);
```

#### `FUNCTION`: **k_world_cr3** <sub>line 7</sub>
```c
extern uint64_t k_world_cr3(void);
```

#### `FUNCTION`: **k_kernel_cr3** <sub>line 8</sub>
```c
extern uint64_t k_kernel_cr3(void);
```

#### `FUNCTION`: **k_spawn** <sub>line 9</sub>
```c
extern int64_t k_spawn(const char *path, uint32_t parent, uint64_t cr3);
```

#### `FUNCTION`: **k_ipc_send** <sub>line 10</sub>
```c
extern int32_t k_ipc_send(uint32_t dst, uint64_t a0, uint64_t a1);
```

#### `FUNCTION`: **k_ipc_recv** <sub>line 11</sub>
```c
extern int32_t k_ipc_recv(uint64_t *a0, uint64_t *a1);
```

#### `FUNCTION`: **k_tick** <sub>line 12</sub>
```c
extern uint64_t k_tick(void);
```

#### `FUNCTION`: **battery_status_packed** <sub>line 16</sub>
```c
extern bool battery_status_packed(uint64_t *a0, uint64_t *a1, uint64_t *a2);
```

#### `FUNCTION`: **k_user_cstr** <sub>line 56</sub>
```c
extern bool k_user_cstr(uint64_t ptr, char *buf, uint32_t cap);
```

#### `FUNCTION`: **k_getpid** <sub>line 81</sub>
```c
return k_getpid();
```

#### `FUNCTION`: **k_tick** <sub>line 88</sub>
```c
return k_tick();
```

#### `FUNCTION`: **hdmi_submit_fill** <sub>line 157</sub>
```c
return hdmi_submit_fill((uint32_t)color,
```

#### `FUNCTION`: **cl_bridge_add** <sub>line 179</sub>
```c
int cl_bridge_add(cl_vm_t *vm, const char *name, cl_ext_fn fn)
```

#### `FUNCTION`: **cl_vm_register_extern** <sub>line 181</sub>
```c
return cl_vm_register_extern(vm, name, fn);
```

#### `FUNCTION`: **cl_bridge_init** <sub>line 184</sub>
```c
int cl_bridge_init(cl_vm_t *vm, uint8_t ring)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/bridge.h</b> (2 items)</summary>

#### `FUNCTION`: **cl_bridge_init** <sub>line 6</sub>
```c
int cl_bridge_init(cl_vm_t *vm, uint8_t ring);
```

#### `FUNCTION`: **cl_bridge_add** <sub>line 8</sub>
```c
int cl_bridge_add(cl_vm_t *vm, const char *name, cl_ext_fn fn);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/codegen.c</b> (7 items)</summary>

#### `FUNCTION`: **find_global** <sub>line 39</sub>
```c
static int find_global(cl_prog_t *P, const char *name)
```

#### `FUNCTION`: **find_local** <sub>line 57</sub>
```c
static int find_local(cg_t *g, const char *name)
```

#### `FUNCTION`: **add_local** <sub>line 75</sub>
```c
static uint16_t add_local(cg_t *g, const char *name, uint8_t bits)
```

#### `FUNCTION`: **add_const** <sub>line 98</sub>
```c
static uint32_t add_const(cg_t *g, uint64_t v, uint8_t bits, bool neg)
```

#### `FUNCTION`: **gen_expr** <sub>line 126</sub>
```c
static void gen_expr(cg_t *g, ast_node_t *n, uint8_t bits);
```

#### `FUNCTION`: **gen_expr** <sub>line 128</sub>
```c
static void gen_expr(cg_t *g, ast_node_t *n, uint8_t bits)
```

#### `FUNCTION`: **gen_stmt** <sub>line 292</sub>
```c
static void gen_stmt(cg_t *g, ast_node_t *n)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/lexer.c</b> (10 items)</summary>

#### `FUNCTION`: **peek** <sub>line 35</sub>
```c
static char peek(const lexer_t *l)
```

#### `FUNCTION`: **peek2** <sub>line 40</sub>
```c
static char peek2(const lexer_t *l)
```

#### `FUNCTION`: **take** <sub>line 45</sub>
```c
static char take(lexer_t *l)
```

#### `FUNCTION`: **skip_ws** <sub>line 72</sub>
```c
static void skip_ws(lexer_t *l)
```

#### `FUNCTION`: **is_id_start** <sub>line 104</sub>
```c
static bool is_id_start(char c)
```

#### `FUNCTION`: **is_id_char** <sub>line 109</sub>
```c
static bool is_id_char(char c)
```

#### `FUNCTION`: **is_id_start** <sub>line 111</sub>
```c
return is_id_start(c) || (c >= '0' && c <= '9');
```

#### `FUNCTION`: **is_digit** <sub>line 114</sub>
```c
static bool is_digit(char c)
```

#### `FUNCTION`: **hex_val** <sub>line 119</sub>
```c
static int hex_val(char c)
```

#### `FUNCTION`: **cl_lex_next** <sub>line 263</sub>
```c
bool cl_lex_next(lexer_t *l, token_t *t)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/lexer.h</b> (3 items)</summary>

#### `FUNCTION`: **cl_lexer_init** <sub>line 16</sub>
```c
void cl_lexer_init(lexer_t *l, const char *src, size_t len);
```

#### `FUNCTION`: **cl_lex_next** <sub>line 18</sub>
```c
bool cl_lex_next(lexer_t *l, token_t *out);
```

#### `FUNCTION`: **cl_lex_all** <sub>line 20</sub>
```c
int cl_lex_all(lexer_t *l, token_t *buf, size_t cap, size_t *count);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/loader.c</b> (10 items)</summary>

#### `FUNCTION`: **wr_u8** <sub>line 86</sub>
```c
static void wr_u8(wr_t *w, uint8_t v)
```

#### `FUNCTION`: **wr_u32** <sub>line 96</sub>
```c
static void wr_u32(wr_t *w, uint32_t v)
```

#### `FUNCTION`: **wr_bytes** <sub>line 103</sub>
```c
static void wr_bytes(wr_t *w, const void *src, size_t n)
```

#### `FUNCTION`: **rd_u8** <sub>line 110</sub>
```c
static uint8_t rd_u8(rd_t *r)
```

#### `FUNCTION`: **rd_u32** <sub>line 120</sub>
```c
static uint32_t rd_u32(rd_t *r)
```

#### `FUNCTION`: **rd_bytes** <sub>line 131</sub>
```c
static void rd_bytes(rd_t *r, void *dst, size_t n)
```

#### `FUNCTION`: **cl_save_bc** <sub>line 138</sub>
```c
size_t cl_save_bc(cl_prog_t *P, uint8_t *buf, size_t cap)
```

#### `FUNCTION`: **read_all** <sub>line 301</sub>
```c
static long read_all(const char *path, uint8_t *buf, size_t cap)
```

#### `FUNCTION`: **k_fs_read** <sub>line 316</sub>
```c
extern int32_t k_fs_read(const char *path, void *buf, uint32_t cap);
```

#### `FUNCTION`: **read_all** <sub>line 318</sub>
```c
static long read_all(const char *path, uint8_t *buf, size_t cap)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/loader.h</b> (1 items)</summary>

#### `FUNCTION`: **cl_save_bc** <sub>line 15</sub>
```c
size_t cl_save_bc(cl_prog_t *P, uint8_t *buf, size_t cap);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/native.h</b> (2 items)</summary>

#### `FUNCTION`: **uint64_t** <sub>line 9</sub>
```c
typedef uint64_t (*cl_native_fn)(uint64_t *, uint64_t *, cl_ext_fn *);
```
> signature wygenerowanej funkcji: 
> uint64_t fn(uint64_t *slots, uint64_t *globals, cl_ext_fn *externs)

#### `FUNCTION`: **cl_native_size** <sub>line 14</sub>
```c
size_t cl_native_size(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/native_x86.c</b> (13 items)</summary>

#### `FUNCTION`: **nb_u8** <sub>line 10</sub>
```c
static void nb_u8(nb_t *d, uint8_t v)
```

#### `FUNCTION`: **nb_u32** <sub>line 15</sub>
```c
static void nb_u32(nb_t *d, uint32_t v)
```

#### `FUNCTION`: **nb_u64** <sub>line 20</sub>
```c
static void nb_u64(nb_t *d, uint64_t v)
```

#### `FUNCTION`: **mov_rax_u64** <sub>line 25</sub>
```c
static void mov_rax_u64(nb_t *d, uint64_t v)
```

#### `FUNCTION`: **push_rax** <sub>line 30</sub>
```c
static void push_rax(nb_t *d) { nb_u8(d, 0x50); }
```

#### `FUNCTION`: **pop_rax** <sub>line 31</sub>
```c
static void pop_rax(nb_t *d)  { nb_u8(d, 0x58); }
```

#### `FUNCTION`: **pop_rcx** <sub>line 32</sub>
```c
static void pop_rcx(nb_t *d)  { nb_u8(d, 0x59); }
```

#### `FUNCTION`: **push_rcx** <sub>line 33</sub>
```c
static void push_rcx(nb_t *d) { nb_u8(d, 0x51); }
```

#### `FUNCTION`: **load_glob** <sub>line 47</sub>
```c
static void load_glob(nb_t *d, uint16_t a)
```

#### `FUNCTION`: **store_glob** <sub>line 53</sub>
```c
static void store_glob(nb_t *d, uint16_t a)
```

#### `FUNCTION`: **cl_native_size** <sub>line 195</sub>
```c
size_t cl_native_size(void)
```

#### `FUNCTION`: **sizeof** <sub>line 197</sub>
```c
return sizeof(g_nb_mem);
```

#### `FUNCTION`: **cl_native_size** <sub>line 208</sub>
```c
size_t cl_native_size(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/parser.c</b> (10 items)</summary>

#### `FUNCTION`: **copy_name** <sub>line 3</sub>
```c
static void copy_name(char *dst, size_t cap, const token_t *t)
```

#### `FUNCTION`: **check** <sub>line 45</sub>
```c
static bool check(parser_t *p, tok_kind_t k)
```

#### `FUNCTION`: **peek** <sub>line 47</sub>
```c
return peek(p)->kind == k;
```

#### `FUNCTION`: **match** <sub>line 50</sub>
```c
static bool match(parser_t *p, tok_kind_t k)
```

#### `FUNCTION`: **perr** <sub>line 60</sub>
```c
static void perr(parser_t *p, const char *m)
```

#### `FUNCTION`: **peek** <sub>line 73</sub>
```c
return peek(p);
```

#### `FUNCTION`: **advance** <sub>line 76</sub>
```c
return advance(p);
```

#### `FUNCTION`: **cl_node** <sub>line 81</sub>
```c
return cl_node(p->ar, kind, peek(p)->line);
```

#### `FUNCTION`: **nl_push** <sub>line 97</sub>
```c
static void nl_push(parser_t *p, nlist_t *l, ast_node_t *x)
```

#### `FUNCTION`: **nl_commit** <sub>line 106</sub>
```c
static void nl_commit(parser_t *p, nlist_t *l, ast_node_t *parent)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/run.c</b> (4 items)</summary>

#### `FUNCTION`: **cl_kernel_read** <sub>line 4</sub>
```c
extern long cl_kernel_read(const char *path, uint8_t *buf, size_t cap);
```

#### `FUNCTION`: **cl_run_script** <sub>line 7</sub>
```c
int cl_run_script(const char *path, uint8_t ring, arena_t *ar)
```

#### `FUNCTION`: **kprintf** <sub>line 23</sub>
```c
extern void kprintf(const char *, ...);
```

#### `FUNCTION`: **cl_make_exec** <sub>line 36</sub>
```c
extern void cl_make_exec(void *p, size_t len);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/sema.c</b> (12 items)</summary>

#### `FUNCTION`: **push_scope** <sub>line 75</sub>
```c
static void push_scope(sema_ctx_t *s)
```

#### `FUNCTION`: **pop_scope** <sub>line 83</sub>
```c
static void pop_scope(sema_ctx_t *s)
```

#### `FUNCTION`: **value_fits** <sub>line 165</sub>
```c
static bool value_fits(uint64_t v, bool negative, cl_type_t t)
```

#### `FUNCTION`: **check_expr** <sub>line 197</sub>
```c
static etype_t check_expr(sema_ctx_t *s, ast_node_t *n);
```

#### `FUNCTION`: **shared_validate** <sub>line 214</sub>
```c
static void shared_validate(sema_ctx_t *s)
```

#### `FUNCTION`: **check_expr** <sub>line 229</sub>
```c
static etype_t check_expr(sema_ctx_t *s, ast_node_t *n)
```

#### `FUNCTION`: **check_block** <sub>line 524</sub>
```c
static void check_block(sema_ctx_t *s, ast_node_t *n);
```

#### `FUNCTION`: **check_stmt** <sub>line 526</sub>
```c
static void check_stmt(sema_ctx_t *s, ast_node_t *n)
```

#### `FUNCTION`: **check_block** <sub>line 645</sub>
```c
static void check_block(sema_ctx_t *s, ast_node_t *n)
```

#### `FUNCTION`: **declare_top** <sub>line 657</sub>
```c
static void declare_top(sema_ctx_t *s, ast_node_t *n)
```

#### `FUNCTION`: **cl_sema_run** <sub>line 719</sub>
```c
int cl_sema_run(sema_ctx_t *s, ast_node_t *prog)
```

#### `FUNCTION`: **cl_sema_diag_count** <sub>line 750</sub>
```c
size_t cl_sema_diag_count(const sema_ctx_t *s)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/sema.h</b> (2 items)</summary>

#### `FUNCTION`: **cl_sema_run** <sub>line 16</sub>
```c
int cl_sema_run(sema_ctx_t *s, ast_node_t *prog);
```

#### `FUNCTION`: **cl_sema_diag_count** <sub>line 18</sub>
```c
size_t cl_sema_diag_count(const sema_ctx_t *s);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/vm.c</b> (13 items)</summary>

#### `FUNCTION`: **cl_wbytes** <sub>line 3</sub>
```c
int cl_wbytes(int bits)
```

#### `FUNCTION`: **cl_mask** <sub>line 14</sub>
```c
void cl_mask(uint8_t *r, int w, int bits)
```

#### `FUNCTION`: **cl_add** <sub>line 28</sub>
```c
void cl_add(uint8_t *r, const uint8_t *a, const uint8_t *b, int w)
```

#### `FUNCTION`: **cl_sub** <sub>line 40</sub>
```c
void cl_sub(uint8_t *r, const uint8_t *a, const uint8_t *b, int w)
```

#### `FUNCTION`: **cl_mul** <sub>line 52</sub>
```c
void cl_mul(uint8_t *r, const uint8_t *a, const uint8_t *b, int w)
```

#### `FUNCTION`: **bit_get** <sub>line 72</sub>
```c
static int bit_get(const uint8_t *v, int i)
```

#### `FUNCTION`: **bit_set** <sub>line 77</sub>
```c
static void bit_set(uint8_t *v, int i)
```

#### `FUNCTION`: **cmp_u** <sub>line 82</sub>
```c
static int cmp_u(const uint8_t *a, const uint8_t *b, int w)
```

#### `FUNCTION`: **cl_cmp** <sub>line 93</sub>
```c
int cl_cmp(const uint8_t *a, const uint8_t *b, int w, bool sign)
```

#### `FUNCTION`: **cmp_u** <sub>line 96</sub>
```c
return cmp_u(a, b, w);
```

#### `FUNCTION`: **cmp_u** <sub>line 113</sub>
```c
return cmp_u(x, y, w);
```

#### `FUNCTION`: **cl_shl** <sub>line 116</sub>
```c
void cl_shl(uint8_t *r, const uint8_t *a, uint32_t n, int w)
```

#### `FUNCTION`: **cl_shr** <sub>line 131</sub>
```c
void cl_shr(uint8_t *r, const uint8_t *a, uint32_t n, int w)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/core-lang/vm.h</b> (12 items)</summary>

#### `FUNCTION`: **uint64_t** <sub>line 16</sub>
```c
typedef uint64_t (*cl_ext_fn)(uint64_t, uint64_t, uint64_t,
```

#### `FUNCTION`: **cl_vm_init** <sub>line 44</sub>
```c
void cl_vm_init(cl_vm_t *vm, cl_prog_t *prog);
```

#### `FUNCTION`: **cl_vm_register_extern** <sub>line 45</sub>
```c
int cl_vm_register_extern(cl_vm_t *vm, const char *name, cl_ext_fn fn);
```

#### `FUNCTION`: **cl_vm_run** <sub>line 46</sub>
```c
cl_vm_err_t cl_vm_run(cl_vm_t *vm);
```

#### `FUNCTION`: **cl_wbytes** <sub>line 48</sub>
```c
int cl_wbytes(int bits);
```

#### `FUNCTION`: **cl_mask** <sub>line 49</sub>
```c
void cl_mask(uint8_t *r, int w, int bits);
```

#### `FUNCTION`: **cl_add** <sub>line 50</sub>
```c
void cl_add(uint8_t *r, const uint8_t *a, const uint8_t *b, int w);
```

#### `FUNCTION`: **cl_sub** <sub>line 51</sub>
```c
void cl_sub(uint8_t *r, const uint8_t *a, const uint8_t *b, int w);
```

#### `FUNCTION`: **cl_mul** <sub>line 52</sub>
```c
void cl_mul(uint8_t *r, const uint8_t *a, const uint8_t *b, int w);
```

#### `FUNCTION`: **cl_cmp** <sub>line 53</sub>
```c
int cl_cmp(const uint8_t *a, const uint8_t *b, int w, bool sign);
```

#### `FUNCTION`: **cl_shl** <sub>line 54</sub>
```c
void cl_shl(uint8_t *r, const uint8_t *a, uint32_t n, int w);
```

#### `FUNCTION`: **cl_shr** <sub>line 55</sub>
```c
void cl_shr(uint8_t *r, const uint8_t *a, uint32_t n, int w);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/acpi.rs</b> (7 items)</summary>

#### `STRUCT`: **CpuEntry** <sub>line 4</sub>
```rust
pub struct CpuEntry {
```

#### `STRUCT`: **IoApic** <sub>line 9</sub>
```rust
pub struct IoApic {
```

#### `STRUCT`: **MadtInfo** <sub>line 15</sub>
```rust
pub struct MadtInfo {
```

#### `STRUCT`: **FadtInfo** <sub>line 21</sub>
```rust
pub struct FadtInfo {
```

#### `STRUCT`: **Rsdp** <sub>line 25</sub>
```rust
pub struct Rsdp {
```

#### `FN`: **checksum** <sub>line 43</sub>
```rust
fn checksum(bytes: &[u8]) -> bool {
```

#### `FN`: **find_rsdp** <sub>line 57</sub>
```rust
pub fn find_rsdp(phys_offset: u64) -> Option<u64> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/lapic.rs</b> (13 items)</summary>

#### `FN`: **x2apic_msr** <sub>line 20</sub>
```rust
fn x2apic_msr(reg: u32) -> u32 {
```

#### `FN`: **init** <sub>line 24</sub>
```rust
pub fn init(base_phys: u64) -> bool {
```

#### `FN`: **is_x2apic** <sub>line 46</sub>
```rust
pub fn is_x2apic() -> bool {
```

#### `FN`: **read** <sub>line 50</sub>
```rust
pub fn read(reg: u32) -> u32 {
```

#### `FN`: **write** <sub>line 60</sub>
```rust
pub fn write(reg: u32, val: u32) {
```

#### `FN`: **id** <sub>line 70</sub>
```rust
pub fn id() -> u32 {
```

#### `FN`: **version** <sub>line 78</sub>
```rust
pub fn version() -> u32 {
```

#### `FN`: **enable_bsp** <sub>line 82</sub>
```rust
pub fn enable_bsp() {
```

#### `FN`: **enable_ap** <sub>line 87</sub>
```rust
pub fn enable_ap() {
```

#### `FN`: **eoi** <sub>line 93</sub>
```rust
pub fn eoi() {
```

#### `FN`: **send_ipi** <sub>line 97</sub>
```rust
pub fn send_ipi(icr: u32, dest_apic_id: u32) {
```

#### `FN`: **send_init_ipi** <sub>line 109</sub>
```rust
pub fn send_init_ipi() {
```

#### `FN`: **send_startup_ipi** <sub>line 116</sub>
```rust
pub fn send_startup_ipi(apic_id: u32, vector: u8) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/mod.rs</b> (3 items)</summary>

#### `FN`: **init_riscv** <sub>line 18</sub>
```rust
pub fn init_riscv() {
```

#### `FN`: **total_cpus** <sub>line 24</sub>
```rust
pub fn total_cpus() -> u32 {
```

#### `FN`: **total_cpus** <sub>line 29</sub>
```rust
pub fn total_cpus() -> u32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/riscv.rs</b> (5 items)</summary>

#### `FN`: **init** <sub>line 7</sub>
```rust
pub fn init() {
```

#### `FN`: **current_hart** <sub>line 12</sub>
```rust
pub fn current_hart() -> u64 {
```

#### `FN`: **poweroff** <sub>line 17</sub>
```rust
pub fn poweroff() -> ! {
```

#### `FN`: **reboot** <sub>line 22</sub>
```rust
pub fn reboot() -> ! {
```

#### `FN`: **system_reset** <sub>line 26</sub>
```rust
fn system_reset(reset_type: u32) -> ! {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/arch/risc_v/context_swich.rs</b> (12 items)</summary>

#### `FN`: **get_switch_count** <sub>line 639</sub>
```rust
pub fn get_switch_count() -> u64 {
```

#### `FN`: **get_fpu_switch_count** <sub>line 643</sub>
```rust
pub fn get_fpu_switch_count() -> u64 {
```

#### `FN`: **enable_lazy_fpu** <sub>line 647</sub>
```rust
pub fn enable_lazy_fpu() {
```

#### `FN`: **disable_lazy_fpu** <sub>line 651</sub>
```rust
pub fn disable_lazy_fpu() {
```

#### `FN`: **is_lazy_fpu_enabled** <sub>line 655</sub>
```rust
pub fn is_lazy_fpu_enabled() -> bool {
```

#### `FN`: **satp_mode_name** <sub>line 692</sub>
```rust
pub fn satp_mode_name(satp: u64) -> &'static str {
```

#### `FN`: **satp_asid** <sub>line 703</sub>
```rust
pub fn satp_asid(satp: u64) -> u64 {
```

#### `FN`: **satp_ppn** <sub>line 707</sub>
```rust
pub fn satp_ppn(satp: u64) -> u64 {
```

#### `FN`: **test_fpu_save_area_size** <sub>line 716</sub>
```rust
fn test_fpu_save_area_size() {
```

#### `FN`: **test_satp_constants** <sub>line 722</sub>
```rust
fn test_satp_constants() {
```

#### `FN`: **test_sstatus_fs_mask** <sub>line 730</sub>
```rust
fn test_sstatus_fs_mask() {
```

#### `FN`: **test_satp_mode_name** <sub>line 739</sub>
```rust
fn test_satp_mode_name() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/arch/x86_64/context_swich.rs</b> (12 items)</summary>

#### `STRUCT`: **XSaveArea** <sub>line 45</sub>
```rust
struct XSaveArea {
```

#### `IMPL`: **XSaveArea** <sub>line 49</sub>
```rust
impl XSaveArea {
```

#### `STRUCT`: **ExtendedContext** <sub>line 56</sub>
```rust
struct ExtendedContext {
```

#### `STRUCT`: **TssStruct** <sub>line 458</sub>
```rust
struct TssStruct {
```

#### `FN`: **get_switch_count** <sub>line 731</sub>
```rust
pub fn get_switch_count() -> u64 {
```

#### `FN`: **get_fpu_switch_count** <sub>line 735</sub>
```rust
pub fn get_fpu_switch_count() -> u64 {
```

#### `FN`: **enable_lazy_fpu** <sub>line 739</sub>
```rust
pub fn enable_lazy_fpu() {
```

#### `FN`: **disable_lazy_fpu** <sub>line 743</sub>
```rust
pub fn disable_lazy_fpu() {
```

#### `FN`: **is_lazy_fpu_enabled** <sub>line 747</sub>
```rust
pub fn is_lazy_fpu_enabled() -> bool {
```

#### `FN`: **test_xsave_area_alignment** <sub>line 769</sub>
```rust
fn test_xsave_area_alignment() {
```

#### `FN`: **test_msr_constants** <sub>line 774</sub>
```rust
fn test_msr_constants() {
```

#### `FN`: **test_cr3_masks** <sub>line 781</sub>
```rust
fn test_cr3_masks() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/arch_hooks.rs</b> (35 items)</summary>

#### `FN`: **register_runqueue** <sub>line 49</sub>
```rust
pub fn register_runqueue(cpu: u32, rq: *mut RunQueue) {
```

#### `FN`: **register_apic_id** <sub>line 55</sub>
```rust
pub fn register_apic_id(cpu: u32, apic_id: u32) {
```

#### `FN`: **snapshot_registry** <sub>line 61</sub>
```rust
pub fn snapshot_registry() -> [*mut RunQueue; MAX_CPUS] {
```

#### `FN`: **cpu_to_apic_id** <sub>line 69</sub>
```rust
fn cpu_to_apic_id(cpu: u32) -> Option<u32> {
```

#### `FN`: **set_tsc_frequency** <sub>line 81</sub>
```rust
pub fn set_tsc_frequency(hz: u64) {
```

#### `FN`: **tsc_frequency** <sub>line 85</sub>
```rust
pub fn tsc_frequency() -> u64 {
```

#### `FN`: **booted_cpu_count** <sub>line 89</sub>
```rust
pub fn booted_cpu_count() -> u32 {
```

#### `FN`: **ns_from_tsc** <sub>line 175</sub>
```rust
fn ns_from_tsc(tsc_delta: u64, hz: u64) -> u64 {
```

#### `FN`: **now_ns** <sub>line 183</sub>
```rust
pub fn now_ns() -> u64 {
```

#### `FN`: **timer_count_for_hz** <sub>line 217</sub>
```rust
fn timer_count_for_hz(hz: u64) -> u32 {
```

#### `FN`: **cmp_max_1** <sub>line 225</sub>
```rust
fn cmp_max_1(v: u32) -> u32 {
```

#### `FN`: **default_timer_count** <sub>line 233</sub>
```rust
fn default_timer_count() -> u32 {
```

#### `FN`: **current_cpu_id** <sub>line 270</sub>
```rust
pub fn current_cpu_id() -> u32 {
```

#### `STRUCT`: **IdtEntry** <sub>line 338</sub>
```rust
pub struct IdtEntry {
```

#### `IMPL`: **IdtEntry** <sub>line 348</sub>
```rust
impl IdtEntry {
```

#### `FN`: **set** <sub>line 353</sub>
```rust
pub fn set(&mut self, handler: usize, selector: u16, ist: u8, type_attr: u8) {
```

#### `FN`: **is_present** <sub>line 363</sub>
```rust
pub fn is_present(&self) -> bool {
```

#### `STRUCT`: **IdtDescriptor** <sub>line 369</sub>
```rust
struct IdtDescriptor {
```

#### `FN`: **arch_context_switch** <sub>line 437</sub>
```rust
fn arch_context_switch(prev_ctx: *mut CpuContext, next_ctx: *const CpuContext);
```

#### `FN`: **maybe_balance** <sub>line 462</sub>
```rust
fn maybe_balance(cpu: u32, registry: &[*mut RunQueue; MAX_CPUS]) {
```

#### `FN`: **registry_defaults_to_null_for_untouched_cpu** <sub>line 587</sub>
```rust
fn registry_defaults_to_null_for_untouched_cpu() {
```

#### `FN`: **apic_id_table_defaults_to_none_then_roundtrips** <sub>line 593</sub>
```rust
fn apic_id_table_defaults_to_none_then_roundtrips() {
```

#### `FN`: **register_runqueue_ignores_out_of_range_cpu** <sub>line 600</sub>
```rust
fn register_runqueue_ignores_out_of_range_cpu() {
```

#### `FN`: **ns_from_tsc_is_zero_without_calibration** <sub>line 605</sub>
```rust
fn ns_from_tsc_is_zero_without_calibration() {
```

#### `FN`: **ns_from_tsc_scales_correctly** <sub>line 610</sub>
```rust
fn ns_from_tsc_scales_correctly() {
```

#### `FN`: **timer_count_for_hz_has_sane_fallback** <sub>line 618</sub>
```rust
fn timer_count_for_hz_has_sane_fallback() {
```

#### `FN`: **timer_count_for_hz_never_returns_zero** <sub>line 624</sub>
```rust
fn timer_count_for_hz_never_returns_zero() {
```

#### `FN`: **tlb_pending_flag_sets_and_clears** <sub>line 629</sub>
```rust
fn tlb_pending_flag_sets_and_clears() {
```

#### `FN`: **idt_entry_encodes_and_reports_present** <sub>line 637</sub>
```rust
fn idt_entry_encodes_and_reports_present() {
```

#### `FN`: **idt_entry_missing_is_not_present** <sub>line 645</sub>
```rust
fn idt_entry_missing_is_not_present() {
```

#### `FN`: **cpuid_leaf_zero_reports_a_nonzero_max_leaf** <sub>line 651</sub>
```rust
fn cpuid_leaf_zero_reports_a_nonzero_max_leaf() {
```

#### `FN`: **rdtsc_does_not_go_backwards_across_two_reads** <sub>line 660</sub>
```rust
fn rdtsc_does_not_go_backwards_across_two_reads() {
```

#### `FN`: **cpuid_and_rdtsc_are_zero_off_x86_64** <sub>line 670</sub>
```rust
fn cpuid_and_rdtsc_are_zero_off_x86_64() {
```

#### `FN`: **set_and_read_tsc_frequency_is_stable_within_this_test** <sub>line 678</sub>
```rust
fn set_and_read_tsc_frequency_is_stable_within_this_test() {
```

#### `FN`: **booted_cpu_count_starts_at_or_above_zero** <sub>line 684</sub>
```rust
fn booted_cpu_count_starts_at_or_above_zero() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/class.rs</b> (18 items)</summary>

#### `STRUCT`: **SchedClassOps** <sub>line 12</sub>
```rust
pub struct SchedClassOps {
```

#### `FN`: **class_of** <sub>line 336</sub>
```rust
pub fn class_of(class: SchedClass) -> &'static SchedClassOps {
```

#### `FN`: **make_idle** <sub>line 514</sub>
```rust
fn make_idle(pid: TaskId) -> TaskStruct {
```

#### `FN`: **make_task** <sub>line 520</sub>
```rust
fn make_task(pid: TaskId, policy: SchedPolicy, nice: i8) -> TaskStruct {
```

#### `FN`: **ptr_of** <sub>line 526</sub>
```rust
fn ptr_of(t: &mut TaskStruct) -> *mut TaskStruct {
```

#### `FN`: **chain_is_ordered_stop_dl_rt_fair_idle** <sub>line 531</sub>
```rust
fn chain_is_ordered_stop_dl_rt_fair_idle() {
```

#### `FN`: **class_of_maps_every_variant_correctly** <sub>line 545</sub>
```rust
fn class_of_maps_every_variant_correctly() {
```

#### `FN`: **pick_next_walks_the_chain_in_priority_order** <sub>line 554</sub>
```rust
fn pick_next_walks_the_chain_in_priority_order() {
```

#### `FN`: **rt_get_rr_interval_is_nonzero_only_for_round_robin** <sub>line 596</sub>
```rust
fn rt_get_rr_interval_is_nonzero_only_for_round_robin() {
```

#### `FN`: **fair_and_deadline_and_idle_have_zero_rr_interval** <sub>line 606</sub>
```rust
fn fair_and_deadline_and_idle_have_zero_rr_interval() {
```

#### `FN`: **check_preempt_cross_class_ignores_intra_class_ops** <sub>line 618</sub>
```rust
fn check_preempt_cross_class_ignores_intra_class_ops() {
```

#### `FN`: **switched_to_reschedules_when_new_class_outranks_current** <sub>line 639</sub>
```rust
fn switched_to_reschedules_when_new_class_outranks_current() {
```

#### `FN`: **change_task_class_moves_task_between_underlying_queues** <sub>line 664</sub>
```rust
fn change_task_class_moves_task_between_underlying_queues() {
```

#### `FN`: **change_task_class_on_non_queued_task_does_not_touch_queues** <sub>line 688</sub>
```rust
fn change_task_class_on_non_queued_task_does_not_touch_queues() {
```

#### `FN`: **rt_task_tick_only_requeues_round_robin_on_slice_expiry** <sub>line 706</sub>
```rust
fn rt_task_tick_only_requeues_round_robin_on_slice_expiry() {
```

#### `FN`: **rt_task_tick_marks_resched_when_round_robin_slice_expires** <sub>line 727</sub>
```rust
fn rt_task_tick_marks_resched_when_round_robin_slice_expires() {
```

#### `FN`: **dl_task_tick_dequeues_when_throttled** <sub>line 750</sub>
```rust
fn dl_task_tick_dequeues_when_throttled() {
```

#### `FN`: **fair_charge_dispatches_to_underlying_rqfair** <sub>line 776</sub>
```rust
fn fair_charge_dispatches_to_underlying_rqfair() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/collections/bitmap.rs</b> (58 items)</summary>

#### `STRUCT`: **Bitmap** <sub>line 27</sub>
```rust
pub struct Bitmap<const WORDS: usize> {
```

#### `FN`: **set** <sub>line 43</sub>
```rust
pub fn set(&mut self, bit: usize) {
```

#### `FN`: **clear** <sub>line 50</sub>
```rust
pub fn clear(&mut self, bit: usize) {
```

#### `FN`: **toggle** <sub>line 57</sub>
```rust
pub fn toggle(&mut self, bit: usize) {
```

#### `FN`: **test** <sub>line 64</sub>
```rust
pub fn test(&self, bit: usize) -> bool {
```

#### `FN`: **set_range** <sub>line 68</sub>
```rust
pub fn set_range(&mut self, start: usize, end: usize) {
```

#### `FN`: **is_empty** <sub>line 77</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **is_full** <sub>line 81</sub>
```rust
pub fn is_full(&self) -> bool {
```

#### `FN`: **weight** <sub>line 85</sub>
```rust
pub fn weight(&self) -> u32 {
```

#### `FN`: **find_first_set** <sub>line 89</sub>
```rust
pub fn find_first_set(&self) -> Option<usize> {
```

#### `FN`: **find_first_zero** <sub>line 98</sub>
```rust
pub fn find_first_zero(&self) -> Option<usize> {
```

#### `FN`: **find_next_set** <sub>line 111</sub>
```rust
pub fn find_next_set(&self, after: usize) -> Option<usize> {
```

#### `FN`: **and** <sub>line 126</sub>
```rust
pub fn and(&self, other: &Self) -> Self {
```

#### `FN`: **or** <sub>line 134</sub>
```rust
pub fn or(&self, other: &Self) -> Self {
```

#### `FN`: **xor** <sub>line 142</sub>
```rust
pub fn xor(&self, other: &Self) -> Self {
```

#### `FN`: **andnot** <sub>line 150</sub>
```rust
pub fn andnot(&self, other: &Self) -> Self {
```

#### `FN`: **intersects** <sub>line 158</sub>
```rust
pub fn intersects(&self, other: &Self) -> bool {
```

#### `FN`: **iter** <sub>line 162</sub>
```rust
pub fn iter(&self) -> BitmapIter<'_> {
```

#### `FN`: **default** <sub>line 168</sub>
```rust
fn default() -> Self {
```

#### `STRUCT`: **BitmapIter** <sub>line 173</sub>
```rust
pub struct BitmapIter<'a> {
```

#### `TYPE`: **Item** <sub>line 180</sub>
```rust
type Item = usize;
```

#### `FN`: **next** <sub>line 182</sub>
```rust
fn next(&mut self) -> Option<usize> {
```

#### `STRUCT`: **BitmapSlice** <sub>line 200</sub>
```rust
pub struct BitmapSlice<'a> {
```

#### `FN`: **new** <sub>line 206</sub>
```rust
pub fn new(words: &'a mut [u64], nbits: usize) -> Self {
```

#### `FN`: **capacity** <sub>line 211</sub>
```rust
pub fn capacity(&self) -> usize {
```

#### `FN`: **set** <sub>line 215</sub>
```rust
pub fn set(&mut self, bit: usize) {
```

#### `FN`: **clear** <sub>line 221</sub>
```rust
pub fn clear(&mut self, bit: usize) {
```

#### `FN`: **test** <sub>line 227</sub>
```rust
pub fn test(&self, bit: usize) -> bool {
```

#### `FN`: **weight** <sub>line 231</sub>
```rust
pub fn weight(&self) -> u32 {
```

#### `FN`: **find_first_zero** <sub>line 235</sub>
```rust
pub fn find_first_zero(&self) -> Option<usize> {
```

#### `FN`: **clear_all** <sub>line 248</sub>
```rust
pub fn clear_all(&mut self) {
```

#### `STRUCT`: **AtomicBitmap** <sub>line 263</sub>
```rust
pub struct AtomicBitmap<const WORDS: usize> {
```

#### `FN`: **test** <sub>line 275</sub>
```rust
pub fn test(&self, bit: usize) -> bool {
```

#### `FN`: **test_and_set** <sub>line 280</sub>
```rust
pub fn test_and_set(&self, bit: usize) -> bool {
```
> Ustawia bit atomowo, zwraca POPRZEDNI stan (true = już był ustawiony).

#### `FN`: **test_and_clear** <sub>line 289</sub>
```rust
pub fn test_and_clear(&self, bit: usize) -> bool {
```
> Czyści bit atomowo, zwraca POPRZEDNI stan (true = był ustawiony).

#### `FN`: **weight** <sub>line 297</sub>
```rust
pub fn weight(&self) -> u32 {
```

#### `FN`: **is_full** <sub>line 301</sub>
```rust
pub fn is_full(&self) -> bool {
```

#### `FN`: **find_first_zero_and_set** <sub>line 311</sub>
```rust
pub fn find_first_zero_and_set(&self) -> Option<usize> {
```
> w tym samym słowie) — ponawiamy próbę na TYM SAMYM słowie zamiast 
> przechodzić dalej, żeby nie pominąć bitów zwolnionych w 
> międzyczasie.

#### `FN`: **default** <sub>line 340</sub>
```rust
fn default() -> Self {
```

#### `FN`: **words_for_bits_rounds_up** <sub>line 350</sub>
```rust
fn words_for_bits_rounds_up() {
```

#### `FN`: **set_clear_test_roundtrip** <sub>line 358</sub>
```rust
fn set_clear_test_roundtrip() {
```

#### `FN`: **out_of_range_access_is_a_safe_noop** <sub>line 369</sub>
```rust
fn out_of_range_access_is_a_safe_noop() {
```

#### `FN`: **weight_counts_set_bits_across_words** <sub>line 377</sub>
```rust
fn weight_counts_set_bits_across_words() {
```

#### `FN`: **find_first_set_crosses_word_boundary** <sub>line 387</sub>
```rust
fn find_first_set_crosses_word_boundary() {
```

#### `FN`: **find_first_zero_skips_full_words** <sub>line 394</sub>
```rust
fn find_first_zero_skips_full_words() {
```

#### `FN`: **find_first_zero_none_when_full** <sub>line 401</sub>
```rust
fn find_first_zero_none_when_full() {
```

#### `FN`: **find_next_set_after_given_bit** <sub>line 408</sub>
```rust
fn find_next_set_after_given_bit() {
```

#### `FN`: **set_range_sets_contiguous_span** <sub>line 417</sub>
```rust
fn set_range_sets_contiguous_span() {
```

#### `FN`: **boolean_ops_behave_as_expected** <sub>line 429</sub>
```rust
fn boolean_ops_behave_as_expected() {
```

#### `FN`: **iter_yields_set_bits_in_order** <sub>line 446</sub>
```rust
fn iter_yields_set_bits_in_order() {
```

#### `FN`: **slice_bitmap_respects_logical_bit_count_not_word_count** <sub>line 460</sub>
```rust
fn slice_bitmap_respects_logical_bit_count_not_word_count() {
```

#### `FN`: **slice_bitmap_clear_all_resets_backing_storage** <sub>line 472</sub>
```rust
fn slice_bitmap_clear_all_resets_backing_storage() {
```

#### `FN`: **atomic_bitmap_test_and_set_reports_previous_state** <sub>line 481</sub>
```rust
fn atomic_bitmap_test_and_set_reports_previous_state() {
```

#### `FN`: **atomic_bitmap_test_and_clear_reports_previous_state** <sub>line 489</sub>
```rust
fn atomic_bitmap_test_and_clear_reports_previous_state() {
```

#### `FN`: **atomic_bitmap_find_first_zero_and_set_claims_sequentially** <sub>line 498</sub>
```rust
fn atomic_bitmap_find_first_zero_and_set_claims_sequentially() {
```

#### `FN`: **atomic_bitmap_find_first_zero_and_set_returns_none_when_full** <sub>line 507</sub>
```rust
fn atomic_bitmap_find_first_zero_and_set_returns_none_when_full() {
```

#### `FN`: **atomic_bitmap_weight_matches_manual_count** <sub>line 517</sub>
```rust
fn atomic_bitmap_weight_matches_manual_count() {
```

#### `FN`: **atomic_bitmap_out_of_range_set_is_harmless** <sub>line 526</sub>
```rust
fn atomic_bitmap_out_of_range_set_is_harmless() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/collections/cpumask.rs</b> (16 items)</summary>

#### `STRUCT`: **AtomicCpuMask** <sub>line 17</sub>
```rust
pub struct AtomicCpuMask {
```

#### `IMPL`: **AtomicCpuMask** <sub>line 21</sub>
```rust
impl AtomicCpuMask {
```

#### `FN`: **set** <sub>line 29</sub>
```rust
pub fn set(&self, cpu: u32) {
```

#### `FN`: **clear** <sub>line 37</sub>
```rust
pub fn clear(&self, cpu: u32) {
```

#### `FN`: **test** <sub>line 45</sub>
```rust
pub fn test(&self, cpu: u32) -> bool {
```

#### `FN`: **snapshot** <sub>line 51</sub>
```rust
pub fn snapshot(&self) -> CpuMask {
```

#### `FN`: **mark_possible** <sub>line 74</sub>
```rust
pub fn mark_possible(cpu: u32) {
```

#### `FN`: **mark_present** <sub>line 79</sub>
```rust
pub fn mark_present(cpu: u32) {
```

#### `FN`: **mark_online** <sub>line 84</sub>
```rust
pub fn mark_online(cpu: u32) {
```

#### `FN`: **mark_offline** <sub>line 90</sub>
```rust
pub fn mark_offline(cpu: u32) {
```

#### `FN`: **is_online** <sub>line 96</sub>
```rust
pub fn is_online(cpu: u32) -> bool {
```

#### `FN`: **online_mask** <sub>line 101</sub>
```rust
pub fn online_mask() -> CpuMask {
```

#### `FN`: **active_mask** <sub>line 106</sub>
```rust
pub fn active_mask() -> CpuMask {
```

#### `FN`: **set_topology** <sub>line 111</sub>
```rust
pub fn set_topology(cpu: u32, pkg: u32, core: u32) {
```

#### `FN`: **sibling_mask** <sub>line 119</sub>
```rust
pub fn sibling_mask(cpu: u32) -> CpuMask {
```

#### `FN`: **package_mask** <sub>line 137</sub>
```rust
pub fn package_mask(cpu: u32) -> CpuMask {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/collections/plist.rs</b> (6 items)</summary>

#### `FN`: **plist_prio** <sub>line 20</sub>
```rust
fn plist_prio(node: *mut TaskStruct) -> i32 {
```

#### `STRUCT`: **PList** <sub>line 44</sub>
```rust
pub struct PList {
```

#### `IMPL`: **PList** <sub>line 48</sub>
```rust
impl PList {
```

#### `FN`: **is_empty** <sub>line 53</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **first** <sub>line 152</sub>
```rust
pub fn first(&self) -> *mut TaskStruct {
```
> Zadanie o najwyższym priorytecie (head listy poziomów), 
> pierwsze w kolejności FIFO na tym poziomie. O(1).

#### `FN`: **last** <sub>line 167</sub>
```rust
pub fn last(&self) -> *mut TaskStruct {
```
> było gwarantowane przez ówczesny `insert`). Teraz `insert` 
> utrzymuje ten inwariant jawnie: `same_prio.prev` head'a 
> zawsze wskazuje ogon FIFO, więc odczyt jest bezpośredni.

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/collections/rbtree.rs</b> (9 items)</summary>

#### `FN`: **rb_parent** <sub>line 23</sub>
```rust
fn rb_parent(node: *mut TaskStruct) -> *mut TaskStruct {
```

#### `FN`: **rb_color** <sub>line 29</sub>
```rust
fn rb_color(node: *mut TaskStruct) -> usize {
```

#### `FN`: **rb_is_red** <sub>line 35</sub>
```rust
fn rb_is_red(node: *mut TaskStruct) -> bool {
```

#### `FN`: **rb_set_parent_color** <sub>line 40</sub>
```rust
fn rb_set_parent_color(node: *mut TaskStruct, parent: *mut TaskStruct, color: usize) {
```

#### `FN`: **rb_set_parent** <sub>line 48</sub>
```rust
fn rb_set_parent(node: *mut TaskStruct, parent: *mut TaskStruct) {
```

#### `FN`: **rb_set_color** <sub>line 54</sub>
```rust
fn rb_set_color(node: *mut TaskStruct, color: usize) {
```

#### `STRUCT`: **RbTree** <sub>line 62</sub>
```rust
pub struct RbTree {
```

#### `IMPL`: **RbTree** <sub>line 67</sub>
```rust
impl RbTree {
```

#### `FN`: **rb_insert_fixup** <sub>line 116</sub>
```rust
fn rb_insert_fixup(&mut self, mut node: *mut TaskStruct) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/collections/rt_array.rs</b> (7 items)</summary>

#### `STRUCT`: **RtArray** <sub>line 4</sub>
```rust
pub struct RtArray {
```

#### `IMPL`: **RtArray** <sub>line 10</sub>
```rust
impl RtArray {
```

#### `FN`: **set_bit** <sub>line 20</sub>
```rust
pub fn set_bit(&mut self, prio: usize) {
```

#### `FN`: **clear_bit** <sub>line 27</sub>
```rust
pub fn clear_bit(&mut self, prio: usize) {
```

#### `FN`: **bit_is_set** <sub>line 34</sub>
```rust
fn bit_is_set(&self, prio: usize) -> bool {
```

#### `FN`: **highest_prio** <sub>line 41</sub>
```rust
pub fn highest_prio(&self) -> Option<usize> {
```

#### `FN`: **active_levels** <sub>line 149</sub>
```rust
pub fn active_levels(&self) -> u32 {
```
> priorytetów" bez przechodzenia całej bitmapy ręcznie za każdym 
> razem, gdy taka informacja jest potrzebna (np. w heurystykach 
> load-balancingu między CPU).

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/collections/task.rs</b> (62 items)</summary>

#### `STRUCT`: **PidAllocator** <sub>line 15</sub>
```rust
pub struct PidAllocator {
```

#### `IMPL`: **PidAllocator** <sub>line 21</sub>
```rust
impl PidAllocator {
```

#### `FN`: **alloc** <sub>line 31</sub>
```rust
pub fn alloc(&self) -> Option<TaskId> {
```

#### `FN`: **free** <sub>line 54</sub>
```rust
pub fn free(&self, pid: TaskId) {
```

#### `FN`: **reserve** <sub>line 64</sub>
```rust
pub fn reserve(&self, pid: TaskId) -> bool {
```

#### `FN`: **is_used** <sub>line 77</sub>
```rust
pub fn is_used(&self, pid: TaskId) -> bool {
```

#### `FN`: **count_allocated** <sub>line 85</sub>
```rust
pub fn count_allocated(&self) -> u32 {
```

#### `STRUCT`: **TaskTable** <sub>line 94</sub>
```rust
pub struct TaskTable {
```

#### `STRUCT`: **RtFields** <sub>line 100</sub>
```rust
pub struct RtFields {
```

#### `IMPL`: **TaskTable** <sub>line 106</sub>
```rust
impl TaskTable {
```

#### `FN`: **hash_pid** <sub>line 116</sub>
```rust
fn hash_pid(pid: TaskId) -> usize { (pid as usize) % MAX_TASKS }
```

#### `FN`: **hash_tgid** <sub>line 117</sub>
```rust
fn hash_tgid(tgid: TaskId) -> usize { (tgid as usize) % TGID_SLOTS }
```

#### `FN`: **insert** <sub>line 119</sub>
```rust
pub fn insert(&self, task: *mut TaskStruct) -> bool {
```

#### `FN`: **remove** <sub>line 154</sub>
```rust
pub fn remove(&self, pid: TaskId) -> *mut TaskStruct {
```

#### `FN`: **lookup_pid** <sub>line 185</sub>
```rust
pub fn lookup_pid(&self, pid: TaskId) -> *mut TaskStruct {
```

#### `FN`: **lookup_tgid** <sub>line 196</sub>
```rust
pub fn lookup_tgid(&self, tgid: TaskId) -> *mut TaskStruct {
```

#### `FN`: **count** <sub>line 207</sub>
```rust
pub fn count(&self) -> u32 {
```

#### `STRUCT`: **TaskIterator** <sub>line 221</sub>
```rust
pub struct TaskIterator<'a> {
```

#### `IMPL`: **TaskStruct** <sub>line 225</sub>
```rust
impl TaskStruct {
```

#### `FN`: **new** <sub>line 279</sub>
```rust
pub fn new(table: &'a TaskTable) -> Self {
```

#### `TYPE`: **Item** <sub>line 285</sub>
```rust
type Item = *mut TaskStruct;
```

#### `FN`: **next** <sub>line 287</sub>
```rust
fn next(&mut self) -> Option<Self::Item> {
```

#### `STRUCT`: **ThreadGroupIterator** <sub>line 299</sub>
```rust
pub struct ThreadGroupIterator {
```

#### `IMPL`: **ThreadGroupIterator** <sub>line 304</sub>
```rust
impl ThreadGroupIterator {
```

#### `IMPL`: **Iterator** <sub>line 316</sub>
```rust
impl Iterator for ThreadGroupIterator {
```

#### `TYPE`: **Item** <sub>line 317</sub>
```rust
type Item = *mut TaskStruct;
```

#### `FN`: **next** <sub>line 319</sub>
```rust
fn next(&mut self) -> Option<Self::Item> {
```

#### `STRUCT`: **ChildIterator** <sub>line 334</sub>
```rust
pub struct ChildIterator {
```

#### `IMPL`: **ChildIterator** <sub>line 338</sub>
```rust
impl ChildIterator {
```

#### `IMPL`: **Iterator** <sub>line 347</sub>
```rust
impl Iterator for ChildIterator {
```

#### `TYPE`: **Item** <sub>line 348</sub>
```rust
type Item = *mut TaskStruct;
```

#### `FN`: **next** <sub>line 350</sub>
```rust
fn next(&mut self) -> Option<Self::Item> {
```

#### `STRUCT`: **DescendantIterator** <sub>line 358</sub>
```rust
pub struct DescendantIterator {
```

#### `IMPL`: **DescendantIterator** <sub>line 362</sub>
```rust
impl DescendantIterator {
```

#### `IMPL`: **Iterator** <sub>line 372</sub>
```rust
impl Iterator for DescendantIterator {
```

#### `TYPE`: **Item** <sub>line 373</sub>
```rust
type Item = *mut TaskStruct;
```

#### `FN`: **next** <sub>line 375</sub>
```rust
fn next(&mut self) -> Option<Self::Item> {
```

#### `STRUCT`: **ProcessGroup** <sub>line 386</sub>
```rust
pub struct ProcessGroup {
```

#### `IMPL`: **ProcessGroup** <sub>line 396</sub>
```rust
impl ProcessGroup {
```

#### `FN`: **new** <sub>line 397</sub>
```rust
pub fn new(pgid: TaskId, leader: *mut TaskStruct) -> Self {
```

#### `FN`: **is_empty** <sub>line 427</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `STRUCT`: **Session** <sub>line 443</sub>
```rust
pub struct Session {
```

#### `IMPL`: **Session** <sub>line 452</sub>
```rust
impl Session {
```

#### `FN`: **new** <sub>line 453</sub>
```rust
pub fn new(sid: TaskId, leader: *mut TaskStruct) -> Self {
```

#### `FN`: **is_empty** <sub>line 473</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **set_foreground** <sub>line 477</sub>
```rust
pub fn set_foreground(&self, pgid: TaskId) {
```

#### `FN`: **get_foreground** <sub>line 481</sub>
```rust
pub fn get_foreground(&self) -> TaskId {
```

#### `STRUCT`: **UidTaskCount** <sub>line 488</sub>
```rust
pub struct UidTaskCount {
```

#### `IMPL`: **UidTaskCount** <sub>line 492</sub>
```rust
impl UidTaskCount {
```

#### `FN`: **inc** <sub>line 498</sub>
```rust
pub fn inc(&self, uid: u32) -> u32 {
```

#### `FN`: **dec** <sub>line 503</sub>
```rust
pub fn dec(&self, uid: u32) {
```

#### `FN`: **get** <sub>line 510</sub>
```rust
pub fn get(&self, uid: u32) -> u32 {
```

#### `ENUM`: **TaskRegistryError** <sub>line 516</sub>
```rust
pub enum TaskRegistryError {
```

#### `STRUCT`: **TaskRegistry** <sub>line 524</sub>
```rust
pub struct TaskRegistry {
```

#### `IMPL`: **TaskRegistry** <sub>line 534</sub>
```rust
impl TaskRegistry {
```

#### `FN`: **find_by_pid** <sub>line 585</sub>
```rust
pub fn find_by_pid(&self, pid: TaskId) -> *mut TaskStruct {
```

#### `FN`: **find_by_tgid** <sub>line 589</sub>
```rust
pub fn find_by_tgid(&self, tgid: TaskId) -> *mut TaskStruct {
```

#### `FN`: **iter** <sub>line 685</sub>
```rust
pub fn iter(&self) -> TaskIterator {
```

#### `FN`: **set_init_task** <sub>line 700</sub>
```rust
pub fn set_init_task(&self, task: *mut TaskStruct) {
```

#### `FN`: **stats** <sub>line 704</sub>
```rust
pub fn stats(&self) -> (u32, u64, u64) {
```

#### `STRUCT`: **PidNamespace** <sub>line 713</sub>
```rust
pub struct PidNamespace {
```

#### `IMPL`: **PidNamespace** <sub>line 721</sub>
```rust
impl PidNamespace {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/core.rs</b> (24 items)</summary>

#### `FN`: **alloc_pid** <sub>line 22</sub>
```rust
pub fn alloc_pid() -> Option<TaskId> {
```

#### `FN`: **free_pid** <sub>line 26</sub>
```rust
pub fn free_pid(pid: TaskId) {
```

#### `FN`: **reserve_pid** <sub>line 34</sub>
```rust
pub fn reserve_pid(pid: TaskId) -> bool {
```
> Rezerwuje konkretny PID (np. 0 dla `idle`/`swapper`, albo odtworzenie 
> procesu z checkpointa). Zwraca `false`, jeśli PID był już zajęty.

#### `FN`: **pid_in_use** <sub>line 41</sub>
```rust
pub fn pid_in_use(pid: TaskId) -> bool {
```

#### `FN`: **sys_sched_get_priority_max** <sub>line 219</sub>
```rust
pub fn sys_sched_get_priority_max(policy: SchedPolicy) -> i32 {
```

#### `FN`: **sys_sched_get_priority_min** <sub>line 227</sub>
```rust
pub fn sys_sched_get_priority_min(policy: SchedPolicy) -> i32 {
```

#### `FN`: **make_idle** <sub>line 240</sub>
```rust
fn make_idle(pid: TaskId) -> TaskStructT {
```

#### `FN`: **make_task** <sub>line 246</sub>
```rust
fn make_task(pid: TaskId, policy: SchedPolicy, nice: i8) -> TaskStructT {
```

#### `FN`: **ptr_of** <sub>line 252</sub>
```rust
fn ptr_of(t: &mut TaskStructT) -> *mut TaskStructT {
```

#### `FN`: **two_consecutive_allocations_are_distinct** <sub>line 257</sub>
```rust
fn two_consecutive_allocations_are_distinct() {
```

#### `FN`: **reserve_and_free_pid_roundtrip_on_a_dedicated_high_pid** <sub>line 266</sub>
```rust
fn reserve_and_free_pid_roundtrip_on_a_dedicated_high_pid() {
```

#### `FN`: **reserve_pid_rejects_out_of_range** <sub>line 278</sub>
```rust
fn reserve_pid_rejects_out_of_range() {
```

#### `FN`: **boot_bsp_registers_apic_and_runqueue** <sub>line 283</sub>
```rust
fn boot_bsp_registers_apic_and_runqueue() {
```

#### `FN`: **wake_up_new_task_enqueues_and_marks_runnable** <sub>line 294</sub>
```rust
fn wake_up_new_task_enqueues_and_marks_runnable() {
```

#### `FN`: **exit_bookkeeping_dequeues_and_marks_zombie** <sub>line 312</sub>
```rust
fn exit_bookkeeping_dequeues_and_marks_zombie() {
```

#### `FN`: **exit_bookkeeping_on_unqueued_task_returns_null_rq** <sub>line 332</sub>
```rust
fn exit_bookkeeping_on_unqueued_task_returns_null_rq() {
```

#### `FN`: **reap_zombie_destroys_and_frees_pid** <sub>line 342</sub>
```rust
fn reap_zombie_destroys_and_frees_pid() {
```

#### `FN`: **maybe_reschedule_is_noop_when_not_needed** <sub>line 358</sub>
```rust
fn maybe_reschedule_is_noop_when_not_needed() {
```

#### `FN`: **sys_nice_clamps_to_valid_range** <sub>line 369</sub>
```rust
fn sys_nice_clamps_to_valid_range() {
```

#### `FN`: **sys_nice_rejects_non_fair_policy** <sub>line 380</sub>
```rust
fn sys_nice_rejects_non_fair_policy() {
```

#### `FN`: **sys_sched_setscheduler_moves_task_to_new_class** <sub>line 388</sub>
```rust
fn sys_sched_setscheduler_moves_task_to_new_class() {
```

#### `FN`: **sys_sched_rr_get_interval_matches_class_dispatch** <sub>line 407</sub>
```rust
fn sys_sched_rr_get_interval_matches_class_dispatch() {
```

#### `FN`: **sys_sched_affinity_roundtrips** <sub>line 417</sub>
```rust
fn sys_sched_affinity_roundtrips() {
```

#### `FN`: **priority_range_helpers_match_posix_expectations** <sub>line 427</sub>
```rust
fn priority_range_helpers_match_posix_expectations() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/debug/tracepoints.rs</b> (12 items)</summary>

#### `STRUCT`: **Tracepoint** <sub>line 1</sub>
```rust
pub struct Tracepoint {}
```

#### `STRUCT`: **TracePointId** <sub>line 3</sub>
```rust
pub struct TracePointId {}
```

#### `STRUCT`: **Tracepointkind** <sub>line 5</sub>
```rust
pub struct Tracepointkind{}
```

#### `STRUCT`: **tracepoints** <sub>line 7</sub>
```rust
pub struct tracepoints {
```

#### `STRUCT`: **TraceEvent** <sub>line 11</sub>
```rust
pub struct TraceEvent{
```

#### `ENUM`: **TraceData** <sub>line 15</sub>
```rust
pub enum TraceData{
```

#### `STRUCT`: **ContextSwichTraceData** <sub>line 19</sub>
```rust
pub struct ContextSwichTraceData{
```

#### `STRUCT`: **TraceContext** <sub>line 23</sub>
```rust
pub struct TraceContext{
```

#### `STRUCT`: **TracepointRegistry** <sub>line 27</sub>
```rust
pub struct TracepointRegistry{
```

#### `STRUCT`: **RegisteredTracepoint** <sub>line 31</sub>
```rust
pub struct RegisteredTracepoint{}
```

#### `STRUCT`: **TraceBuffer** <sub>line 33</sub>
```rust
pub struct TraceBuffer{}
```

#### `STRUCT`: **PerCpuTracebuffer** <sub>line 35</sub>
```rust
pub struct PerCpuTracebuffer{}
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/entities/runqueue.rs</b> (112 items)</summary>

#### `STRUCT`: **EnqueueFlags** <sub>line 19</sub>
```rust
pub struct EnqueueFlags: u32 {
```

#### `STRUCT`: **DequeueFlags** <sub>line 30</sub>
```rust
pub struct DequeueFlags: u32 {
```

#### `TYPE`: **KeyOf** <sub>line 45</sub>
```rust
pub type KeyOf = fn(*const TaskStruct) -> u64;
```

#### `FN`: **insert_fixup** <sub>line 189</sub>
```rust
fn insert_fixup(root: &mut *mut TaskStruct, mut z: *mut TaskStruct) {
```

#### `FN`: **insert** <sub>line 240</sub>
```rust
pub fn insert(root: &mut *mut TaskStruct, leftmost: &mut *mut TaskStruct, node: *mut TaskStruct, key_of: KeyOf) {
```

#### `FN`: **delete_fixup** <sub>line 293</sub>
```rust
fn delete_fixup(root: &mut *mut TaskStruct, mut x: *mut TaskStruct, mut x_parent: *mut TaskStruct) {
```

#### `FN`: **delete** <sub>line 357</sub>
```rust
pub fn delete(root: &mut *mut TaskStruct, leftmost: &mut *mut TaskStruct, z: *mut TaskStruct) {
```

#### `FN`: **check** <sub>line 423</sub>
```rust
fn check(node: *mut TaskStruct, key_of: KeyOf) -> Result<usize, &'static str> {
```

#### `FN`: **fair_key_of** <sub>line 452</sub>
```rust
fn fair_key_of(node: *const TaskStruct) -> u64 {
```

#### `STRUCT`: **RqFair** <sub>line 457</sub>
```rust
pub struct RqFair {
```

#### `IMPL`: **Default** <sub>line 465</sub>
```rust
impl Default for RqFair {
```

#### `FN`: **default** <sub>line 466</sub>
```rust
fn default() -> Self {
```

#### `IMPL`: **RqFair** <sub>line 477</sub>
```rust
impl RqFair {
```

#### `FN`: **is_empty** <sub>line 478</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **leftmost** <sub>line 482</sub>
```rust
pub fn leftmost(&self) -> *mut TaskStruct {
```

#### `FN`: **place_entity** <sub>line 486</sub>
```rust
fn place_entity(&self, task: *mut TaskStruct, flags: EnqueueFlags) {
```

#### `FN`: **enqueue** <sub>line 499</sub>
```rust
pub fn enqueue(&mut self, task: *mut TaskStruct, flags: EnqueueFlags) {
```
> Naprawa wady #2: prawdziwe wstawienie do drzewa RB (nie 
> pojedynczy wskaźnik udający drzewo).

#### `FN`: **dequeue** <sub>line 510</sub>
```rust
pub fn dequeue(&mut self, task: *mut TaskStruct) {
```

#### `FN`: **pick_first** <sub>line 520</sub>
```rust
pub fn pick_first(&self) -> *mut TaskStruct {
```

#### `FN`: **charge_exec** <sub>line 524</sub>
```rust
pub fn charge_exec(&mut self, task: *mut TaskStruct, delta_exec: u64, now: u64) -> u64 {
```

#### `FN`: **should_preempt** <sub>line 540</sub>
```rust
pub fn should_preempt(&self, current: *const TaskStruct, candidate: *const TaskStruct) -> bool {
```

#### `FN`: **rb_invariants_ok** <sub>line 552</sub>
```rust
pub fn rb_invariants_ok(&self) -> bool {
```

#### `FN`: **rb_count** <sub>line 557</sub>
```rust
pub fn rb_count(&self) -> usize {
```

#### `FN`: **dl_key_of** <sub>line 563</sub>
```rust
fn dl_key_of(node: *const TaskStruct) -> u64 {
```

#### `STRUCT`: **RqDl** <sub>line 568</sub>
```rust
pub struct RqDl {
```

#### `IMPL`: **Default** <sub>line 578</sub>
```rust
impl Default for RqDl {
```

#### `FN`: **default** <sub>line 579</sub>
```rust
fn default() -> Self {
```

#### `IMPL`: **RqDl** <sub>line 590</sub>
```rust
impl RqDl {
```

#### `FN`: **is_empty** <sub>line 591</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **leftmost** <sub>line 595</sub>
```rust
pub fn leftmost(&self) -> *mut TaskStruct {
```

#### `FN`: **admission_control** <sub>line 599</sub>
```rust
pub fn admission_control(&self, dl_runtime: u64, dl_period: u64) -> bool {
```

#### `FN`: **task_bw** <sub>line 607</sub>
```rust
fn task_bw(task: *const TaskStruct) -> u64 {
```

#### `FN`: **enqueue** <sub>line 618</sub>
```rust
pub fn enqueue(&mut self, task: *mut TaskStruct, now: u64, flags: EnqueueFlags) {
```

#### `FN`: **dequeue** <sub>line 639</sub>
```rust
pub fn dequeue(&mut self, task: *mut TaskStruct, removing_permanently: bool) {
```

#### `FN`: **pick_first** <sub>line 649</sub>
```rust
pub fn pick_first(&self) -> *mut TaskStruct {
```

#### `FN`: **update_curr** <sub>line 653</sub>
```rust
pub fn update_curr(&mut self, task: *mut TaskStruct, delta_exec: u64, now: u64) {
```

#### `FN`: **should_preempt** <sub>line 670</sub>
```rust
pub fn should_preempt(&self, current: *const TaskStruct, candidate: *const TaskStruct) -> bool {
```

#### `FN`: **rb_invariants_ok** <sub>line 675</sub>
```rust
pub fn rb_invariants_ok(&self) -> bool {
```

#### `STRUCT`: **RqRt** <sub>line 684</sub>
```rust
pub struct RqRt {
```

#### `IMPL`: **RqRt** <sub>line 693</sub>
```rust
impl RqRt {
```

#### `FN`: **new** <sub>line 694</sub>
```rust
pub fn new() -> Self {
```

#### `FN`: **is_empty** <sub>line 710</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **set_bit** <sub>line 714</sub>
```rust
fn set_bit(&mut self, prio: usize) {
```

#### `FN`: **clear_bit** <sub>line 718</sub>
```rust
fn clear_bit(&mut self, prio: usize) {
```

#### `FN`: **sched_find_first_bit** <sub>line 722</sub>
```rust
fn sched_find_first_bit(&self) -> Option<usize> {
```

#### `FN`: **enqueue** <sub>line 735</sub>
```rust
pub fn enqueue(&mut self, task: *mut TaskStruct, flags: EnqueueFlags) {
```

#### `FN`: **dequeue** <sub>line 769</sub>
```rust
pub fn dequeue(&mut self, task: *mut TaskStruct) {
```

#### `FN`: **pick_first** <sub>line 793</sub>
```rust
pub fn pick_first(&self) -> *mut TaskStruct {
```

#### `FN`: **requeue** <sub>line 809</sub>
```rust
pub fn requeue(&mut self, task: *mut TaskStruct) {
```

#### `FN`: **highest_priority** <sub>line 820</sub>
```rust
pub fn highest_priority(&self) -> i32 {
```

#### `STRUCT`: **RqStop** <sub>line 828</sub>
```rust
pub struct RqStop {
```

#### `IMPL`: **RqStop** <sub>line 832</sub>
```rust
impl RqStop {
```

#### `FN`: **is_empty** <sub>line 833</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **enqueue** <sub>line 837</sub>
```rust
pub fn enqueue(&mut self, task: *mut TaskStruct) {
```

#### `FN`: **dequeue** <sub>line 843</sub>
```rust
pub fn dequeue(&mut self, task: *mut TaskStruct) {
```

#### `FN`: **pick_first** <sub>line 850</sub>
```rust
pub fn pick_first(&self) -> *mut TaskStruct {
```

#### `STRUCT`: **RunQueue** <sub>line 857</sub>
```rust
pub struct RunQueue {
```

#### `IMPL`: **RunQueue** <sub>line 881</sub>
```rust
impl RunQueue {
```

#### `FN`: **new** <sub>line 882</sub>
```rust
pub fn new(cpu: u32, idle_task: *mut TaskStruct) -> Self {
```

#### `FN`: **bind_idle_task** <sub>line 914</sub>
```rust
pub fn bind_idle_task(&self) {
```

#### `FN`: **now** <sub>line 920</sub>
```rust
fn now(&self) -> u64 {
```

#### `FN`: **current** <sub>line 930</sub>
```rust
pub fn current(&self) -> *mut TaskStruct {
```

#### `FN`: **set_current** <sub>line 934</sub>
```rust
fn set_current(&self, task: *mut TaskStruct) {
```

#### `FN`: **nr_running** <sub>line 938</sub>
```rust
pub fn nr_running(&self) -> u32 {
```

#### `FN`: **is_idle** <sub>line 942</sub>
```rust
pub fn is_idle(&self) -> bool {
```

#### `FN`: **nr_uninterruptible_dec_if** <sub>line 1001</sub>
```rust
fn nr_uninterruptible_dec_if(&self, task: *mut TaskStruct) {
```

#### `FN`: **fair_max_vruntime** <sub>line 1202</sub>
```rust
fn fair_max_vruntime(&self) -> Option<u64> {
```

#### `FN`: **fair_root_for_test** <sub>line 1214</sub>
```rust
pub fn fair_root_for_test(&self) -> *mut TaskStruct {
```

#### `FN`: **fair_root_for_test** <sub>line 1219</sub>
```rust
fn fair_root_for_test(&self) -> *mut TaskStruct {
```

#### `FN`: **fair_root_internal** <sub>line 1223</sub>
```rust
fn fair_root_internal(&self) -> *mut TaskStruct {
```

#### `IMPL`: **core** <sub>line 1245</sub>
```rust
impl core::fmt::Debug for RunQueue {
```

#### `FN`: **fmt** <sub>line 1246</sub>
```rust
fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
```

#### `TYPE`: **RunQueueRegistry** <sub>line 1267</sub>
```rust
pub type RunQueueRegistry<'a> = &'a [*mut RunQueue];
```

#### `FN`: **can_migrate_task** <sub>line 1294</sub>
```rust
fn can_migrate_task(task: *const TaskStruct, dst_cpu: u32) -> bool {
```

#### `FN`: **make_idle** <sub>line 1489</sub>
```rust
fn make_idle(pid: TaskId, cpu: u32) -> TaskStruct {
```

#### `FN`: **make_task** <sub>line 1496</sub>
```rust
fn make_task(pid: TaskId, policy: SchedPolicy, nice: i8) -> TaskStruct {
```

#### `FN`: **ptr_of** <sub>line 1502</sub>
```rust
fn ptr_of(t: &mut TaskStruct) -> *mut TaskStruct {
```

#### `FN`: **fair_tree_maintains_rb_invariants_under_pseudorandom_inserts** <sub>line 1507</sub>
```rust
fn fair_tree_maintains_rb_invariants_under_pseudorandom_inserts() {
```

#### `FN`: **fair_tree_inorder_traversal_is_sorted_by_vruntime** <sub>line 1537</sub>
```rust
fn fair_tree_inorder_traversal_is_sorted_by_vruntime() {
```

#### `FN`: **fair_tree_survives_random_removals_keeping_invariants** <sub>line 1566</sub>
```rust
fn fair_tree_survives_random_removals_keeping_invariants() {
```

#### `FN`: **deadline_tree_orders_by_earliest_deadline** <sub>line 1588</sub>
```rust
fn deadline_tree_orders_by_earliest_deadline() {
```

#### `FN`: **rt_queue_fifo_order_at_same_priority_without_cycles** <sub>line 1609</sub>
```rust
fn rt_queue_fifo_order_at_same_priority_without_cycles() {
```

#### `FN`: **rt_queue_picks_highest_priority_across_levels** <sub>line 1648</sub>
```rust
fn rt_queue_picks_highest_priority_across_levels() {
```

#### `FN`: **rt_queue_requeue_moves_task_to_back_of_its_level** <sub>line 1667</sub>
```rust
fn rt_queue_requeue_moves_task_to_back_of_its_level() {
```

#### `FN`: **pick_next_task_respects_stop_over_deadline_over_rt_over_fair_over_idle** <sub>line 1689</sub>
```rust
fn pick_next_task_respects_stop_over_deadline_over_rt_over_fair_over_idle() {
```

#### `FN`: **pick_next_task_never_returns_null_even_when_fully_empty** <sub>line 1739</sub>
```rust
fn pick_next_task_never_returns_null_even_when_fully_empty() {
```

#### `FN`: **deadline_task_is_picked_even_before_its_deadline_has_passed** <sub>line 1754</sub>
```rust
fn deadline_task_is_picked_even_before_its_deadline_has_passed() {
```

#### `FN`: **min_vruntime_never_decreases_across_updates** <sub>line 1783</sub>
```rust
fn min_vruntime_never_decreases_across_updates() {
```

#### `FN`: **newly_woken_task_does_not_start_before_min_vruntime** <sub>line 1797</sub>
```rust
fn newly_woken_task_does_not_start_before_min_vruntime() {
```

#### `FN`: **higher_class_always_preempts_lower_class** <sub>line 1817</sub>
```rust
fn higher_class_always_preempts_lower_class() {
```

#### `FN`: **fair_preemption_requires_minimum_granularity** <sub>line 1842</sub>
```rust
fn fair_preemption_requires_minimum_granularity() {
```

#### `FN`: **deadline_preempts_via_earliest_deadline_first** <sub>line 1860</sub>
```rust
fn deadline_preempts_via_earliest_deadline_first() {
```

#### `FN`: **deadline_task_gets_throttled_when_budget_is_exhausted** <sub>line 1874</sub>
```rust
fn deadline_task_gets_throttled_when_budget_is_exhausted() {
```

#### `FN`: **deadline_task_is_replenished_after_period_rollover** <sub>line 1892</sub>
```rust
fn deadline_task_is_replenished_after_period_rollover() {
```

#### `FN`: **deadline_admission_control_rejects_overcommitted_bandwidth** <sub>line 1909</sub>
```rust
fn deadline_admission_control_rejects_overcommitted_bandwidth() {
```

#### `FN`: **activate_and_deactivate_round_trip_updates_bookkeeping** <sub>line 1924</sub>
```rust
fn activate_and_deactivate_round_trip_updates_bookkeeping() {
```

#### `FN`: **double_enqueue_is_rejected_in_debug_builds** <sub>line 1951</sub>
```rust
fn double_enqueue_is_rejected_in_debug_builds() {
```

#### `FN`: **select_task_rq_prefers_idle_cpu_with_cache_affinity** <sub>line 1966</sub>
```rust
fn select_task_rq_prefers_idle_cpu_with_cache_affinity() {
```

#### `FN`: **select_task_rq_picks_least_loaded_when_no_affinity_hit** <sub>line 1985</sub>
```rust
fn select_task_rq_picks_least_loaded_when_no_affinity_hit() {
```

#### `FN`: **wake_up_process_transitions_state_and_enqueues_remotely** <sub>line 2011</sub>
```rust
fn wake_up_process_transitions_state_and_enqueues_remotely() {
```

#### `FN`: **wake_up_process_is_idempotent_when_already_queued** <sub>line 2030</sub>
```rust
fn wake_up_process_is_idempotent_when_already_queued() {
```

#### `FN`: **idle_balance_pulls_one_task_from_busy_cpu** <sub>line 2049</sub>
```rust
fn idle_balance_pulls_one_task_from_busy_cpu() {
```

#### `FN`: **idle_balance_does_nothing_when_balanced** <sub>line 2081</sub>
```rust
fn idle_balance_does_nothing_when_balanced() {
```

#### `FN`: **migrated_task_gets_updated_cpu_and_rq_pointer** <sub>line 2105</sub>
```rust
fn migrated_task_gets_updated_cpu_and_rq_pointer() {
```

#### `FN`: **no_migration_respects_cpus_allowed_mask** <sub>line 2140</sub>
```rust
fn no_migration_respects_cpus_allowed_mask() {
```

#### `FN`: **yield_task_pushes_vruntime_to_back_of_fair_queue** <sub>line 2169</sub>
```rust
fn yield_task_pushes_vruntime_to_back_of_fair_queue() {
```

#### `FN`: **schedule_tail_records_voluntary_vs_involuntary_switch** <sub>line 2192</sub>
```rust
fn schedule_tail_records_voluntary_vs_involuntary_switch() {
```

#### `FN`: **clock_advances_and_update_curr_charges_real_delta** <sub>line 2218</sub>
```rust
fn clock_advances_and_update_curr_charges_real_delta() {
```

#### `FN`: **update_curr_is_a_noop_on_idle_task** <sub>line 2243</sub>
```rust
fn update_curr_is_a_noop_on_idle_task() {
```

#### `FN`: **lower_nice_task_accumulates_vruntime_slower** <sub>line 2257</sub>
```rust
fn lower_nice_task_accumulates_vruntime_slower() {
```

#### `FN`: **load_balance_moves_task_between_two_uneven_queues** <sub>line 2281</sub>
```rust
fn load_balance_moves_task_between_two_uneven_queues() {
```

#### `FN`: **load_balance_is_noop_below_imbalance_threshold** <sub>line 2310</sub>
```rust
fn load_balance_is_noop_below_imbalance_threshold() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/entities/stats.rs</b> (44 items)</summary>

#### `FN`: **account_switch** <sub>line 21</sub>
```rust
pub fn account_switch() {
```

#### `FN`: **account_voluntary_switch** <sub>line 25</sub>
```rust
pub fn account_voluntary_switch(task: &mut TaskStruct) {
```

#### `FN`: **account_involuntary_switch** <sub>line 30</sub>
```rust
pub fn account_involuntary_switch(task: &mut TaskStruct) {
```

#### `FN`: **account_migration** <sub>line 35</sub>
```rust
pub fn account_migration(task: &mut TaskStruct) {
```

#### `FN`: **account_user_time** <sub>line 41</sub>
```rust
pub fn account_user_time(task: &mut TaskStruct, delta_ns: u64) {
```

#### `FN`: **account_system_time** <sub>line 48</sub>
```rust
pub fn account_system_time(task: &mut TaskStruct, delta_ns: u64) {
```

#### `FN`: **account_idle_time** <sub>line 55</sub>
```rust
pub fn account_idle_time(delta_ns: u64) {
```

#### `FN`: **account_iowait_time** <sub>line 59</sub>
```rust
pub fn account_iowait_time(delta_ns: u64) {
```

#### `FN`: **total_switches** <sub>line 64</sub>
```rust
pub fn total_switches() -> u64 { TOTAL_SWITCHES.load(Ordering::Relaxed) }
```

#### `FN`: **total_voluntary_switches** <sub>line 65</sub>
```rust
pub fn total_voluntary_switches() -> u64 { TOTAL_VOLUNTARY_SWITCHES.load(Ordering::Relaxed) }
```

#### `FN`: **total_involuntary_switches** <sub>line 66</sub>
```rust
pub fn total_involuntary_switches() -> u64 { TOTAL_INVOLUNTARY_SWITCHES.load(Ordering::Relaxed) }
```

#### `FN`: **total_migrations** <sub>line 67</sub>
```rust
pub fn total_migrations() -> u64 { TOTAL_MIGRATIONS.load(Ordering::Relaxed) }
```

#### `FN`: **total_user_time_ns** <sub>line 68</sub>
```rust
pub fn total_user_time_ns() -> u64 { TOTAL_USER_TIME_NS.load(Ordering::Relaxed) }
```

#### `FN`: **total_system_time_ns** <sub>line 69</sub>
```rust
pub fn total_system_time_ns() -> u64 { TOTAL_SYSTEM_TIME_NS.load(Ordering::Relaxed) }
```

#### `FN`: **total_idle_time_ns** <sub>line 70</sub>
```rust
pub fn total_idle_time_ns() -> u64 { TOTAL_IDLE_TIME_NS.load(Ordering::Relaxed) }
```

#### `FN`: **total_iowait_time_ns** <sub>line 71</sub>
```rust
pub fn total_iowait_time_ns() -> u64 { TOTAL_IOWAIT_TIME_NS.load(Ordering::Relaxed) }
```

#### `FN`: **total_running_time_ns** <sub>line 72</sub>
```rust
pub fn total_running_time_ns() -> u64 { TOTAL_RUNNING_TIME_NS.load(Ordering::Relaxed) }
```

#### `STRUCT`: **RqStats** <sub>line 77</sub>
```rust
pub struct RqStats {
```

#### `IMPL`: **RqStats** <sub>line 106</sub>
```rust
impl RqStats {
```

#### `FN`: **record_switch** <sub>line 126</sub>
```rust
pub fn record_switch(&mut self, voluntary: bool) {
```

#### `FN`: **record_migration** <sub>line 143</sub>
```rust
pub fn record_migration(&mut self) {
```

#### `FN`: **add_user_time** <sub>line 149</sub>
```rust
pub fn add_user_time(&mut self, delta_ns: u64) {
```

#### `FN`: **add_system_time** <sub>line 157</sub>
```rust
pub fn add_system_time(&mut self, delta_ns: u64) {
```

#### `FN`: **add_idle_time** <sub>line 165</sub>
```rust
pub fn add_idle_time(&mut self, delta_ns: u64) {
```

#### `FN`: **add_iowait_time** <sub>line 171</sub>
```rust
pub fn add_iowait_time(&mut self, delta_ns: u64) {
```

#### `FN`: **set_nr_running** <sub>line 177</sub>
```rust
pub fn set_nr_running(&mut self, nr: u32) {
```

#### `FN`: **set_nr_uninterruptible** <sub>line 182</sub>
```rust
pub fn set_nr_uninterruptible(&mut self, nr: u32) {
```

#### `FN`: **update_load_stats** <sub>line 187</sub>
```rust
pub fn update_load_stats(&mut self, sum_vruntime: u64, sum_weight: u64) {
```

#### `FN`: **load_percent** <sub>line 193</sub>
```rust
pub fn load_percent(&self) -> u32 {
```
> Oblicza przybliżone obciążenie CPU (0-100).

#### `FN`: **load_avg_1min** <sub>line 204</sub>
```rust
pub fn load_avg_1min(&self) -> f32 {
```

#### `STRUCT`: **ExecTimeHistogram** <sub>line 213</sub>
```rust
pub struct ExecTimeHistogram {
```

#### `IMPL`: **ExecTimeHistogram** <sub>line 220</sub>
```rust
impl ExecTimeHistogram {
```

#### `FN`: **add_sample** <sub>line 229</sub>
```rust
pub fn add_sample(&mut self, exec_ns: u64) {
```

#### `FN`: **average_ns** <sub>line 251</sub>
```rust
pub fn average_ns(&self) -> f32 {
```

#### `FN`: **total_samples** <sub>line 259</sub>
```rust
pub fn total_samples(&self) -> u64 {
```

#### `IMPL`: **Default** <sub>line 264</sub>
```rust
impl Default for ExecTimeHistogram {
```

#### `FN`: **default** <sub>line 265</sub>
```rust
fn default() -> Self {
```

#### `STRUCT`: **SystemStats** <sub>line 273</sub>
```rust
pub struct SystemStats {
```

#### `STRUCT`: **CpuStatsSnapshot** <sub>line 287</sub>
```rust
pub struct CpuStatsSnapshot {
```

#### `FN`: **collect_system_stats** <sub>line 298</sub>
```rust
pub fn collect_system_stats() -> SystemStats {
```

#### `IMPL`: **fmt** <sub>line 314</sub>
```rust
impl fmt::Display for RqStats {
```

#### `FN`: **fmt** <sub>line 315</sub>
```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

#### `IMPL`: **fmt** <sub>line 331</sub>
```rust
impl fmt::Display for SystemStats {
```

#### `FN`: **fmt** <sub>line 332</sub>
```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/entities/task.rs</b> (199 items)</summary>

#### `TYPE`: **TaskId** <sub>line 13</sub>
```rust
pub type TaskId = u64;
```

#### `TYPE`: **Pid** <sub>line 14</sub>
```rust
pub type Pid = TaskId;
```

#### `TYPE`: **Tgid** <sub>line 15</sub>
```rust
pub type Tgid = TaskId;
```

#### `STRUCT`: **TaskFlags** <sub>line 44</sub>
```rust
pub struct TaskFlags: u32 {
```

#### `STRUCT`: **AtomicTaskFlags** <sub>line 73</sub>
```rust
pub struct AtomicTaskFlags(AtomicU32);
```

#### `IMPL`: **AtomicTaskFlags** <sub>line 75</sub>
```rust
impl AtomicTaskFlags {
```

#### `FN`: **load** <sub>line 85</sub>
```rust
pub fn load(&self, order: Ordering) -> TaskFlags {
```

#### `FN`: **store** <sub>line 90</sub>
```rust
pub fn store(&self, flags: TaskFlags, order: Ordering) {
```

#### `FN`: **fetch_insert** <sub>line 95</sub>
```rust
pub fn fetch_insert(&self, flags: TaskFlags) -> TaskFlags {
```

#### `FN`: **fetch_remove** <sub>line 102</sub>
```rust
pub fn fetch_remove(&self, flags: TaskFlags) -> TaskFlags {
```

#### `FN`: **contains** <sub>line 108</sub>
```rust
pub fn contains(&self, flags: TaskFlags) -> bool {
```

#### `FN`: **test_and_set** <sub>line 113</sub>
```rust
pub fn test_and_set(&self, flags: TaskFlags) -> bool {
```

#### `IMPL`: **Clone** <sub>line 119</sub>
```rust
impl Clone for AtomicTaskFlags {
```

#### `FN`: **clone** <sub>line 120</sub>
```rust
fn clone(&self) -> Self {
```

#### `ENUM`: **TaskState** <sub>line 127</sub>
```rust
pub enum TaskState {
```

#### `IMPL`: **TaskState** <sub>line 138</sub>
```rust
impl TaskState {
```

#### `IMPL`: **Default** <sub>line 199</sub>
```rust
impl Default for TaskState {
```

#### `FN`: **default** <sub>line 200</sub>
```rust
fn default() -> Self {
```

#### `ENUM`: **SchedPolicy** <sub>line 208</sub>
```rust
pub enum SchedPolicy {
```

#### `IMPL`: **SchedPolicy** <sub>line 218</sub>
```rust
impl SchedPolicy {
```

#### `ENUM`: **SchedClass** <sub>line 230</sub>
```rust
pub enum SchedClass {
```

#### `IMPL`: **From** <sub>line 238</sub>
```rust
impl From<SchedPolicy> for SchedClass {
```

#### `FN`: **from** <sub>line 239</sub>
```rust
fn from(policy: SchedPolicy) -> Self {
```

#### `STRUCT`: **CpuMask** <sub>line 253</sub>
```rust
pub struct CpuMask {
```

#### `IMPL`: **CpuMask** <sub>line 257</sub>
```rust
impl CpuMask {
```

#### `FN`: **single** <sub>line 266</sub>
```rust
pub fn single(cpu: u32) -> Self {
```

#### `FN`: **first_n** <sub>line 272</sub>
```rust
pub fn first_n(n: u32) -> Self {
```

#### `FN`: **set** <sub>line 281</sub>
```rust
pub fn set(&mut self, cpu: u32) {
```

#### `FN`: **clear** <sub>line 288</sub>
```rust
pub fn clear(&mut self, cpu: u32) {
```

#### `FN`: **is_set** <sub>line 295</sub>
```rust
pub fn is_set(&self, cpu: u32) -> bool {
```

#### `FN`: **is_empty** <sub>line 300</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **count** <sub>line 304</sub>
```rust
pub fn count(&self) -> u32 {
```

#### `FN`: **intersects** <sub>line 308</sub>
```rust
pub fn intersects(&self, other: &CpuMask) -> bool {
```

#### `FN`: **and** <sub>line 312</sub>
```rust
pub fn and(&self, other: &CpuMask) -> CpuMask {
```

#### `FN`: **or** <sub>line 320</sub>
```rust
pub fn or(&self, other: &CpuMask) -> CpuMask {
```

#### `FN`: **first** <sub>line 328</sub>
```rust
pub fn first(&self) -> Option<u32> {
```

#### `FN`: **next_after** <sub>line 337</sub>
```rust
pub fn next_after(&self, after: u32) -> Option<u32> {
```

#### `FN`: **iter** <sub>line 352</sub>
```rust
pub fn iter(&self) -> CpuMaskIter {
```

#### `IMPL`: **Default** <sub>line 357</sub>
```rust
impl Default for CpuMask {
```

#### `FN`: **default** <sub>line 358</sub>
```rust
fn default() -> Self {
```

#### `STRUCT`: **CpuMaskIter** <sub>line 363</sub>
```rust
pub struct CpuMaskIter {
```

#### `IMPL`: **Iterator** <sub>line 368</sub>
```rust
impl Iterator for CpuMaskIter {
```

#### `TYPE`: **Item** <sub>line 369</sub>
```rust
type Item = u32;
```

#### `FN`: **next** <sub>line 371</sub>
```rust
fn next(&mut self) -> Option<u32> {
```

#### `FN`: **nice_to_weight** <sub>line 406</sub>
```rust
pub fn nice_to_weight(nice: i8) -> u64 {
```

#### `FN`: **nice_to_wmult** <sub>line 411</sub>
```rust
pub fn nice_to_wmult(nice: i8) -> u32 {
```

#### `FN`: **weight_to_nice** <sub>line 416</sub>
```rust
pub fn weight_to_nice(weight: u64) -> i8 {
```

#### `FN`: **calc_delta_fair** <sub>line 431</sub>
```rust
pub fn calc_delta_fair(delta_exec: u64, weight: u64, inv_weight: u32) -> u64 {
```

#### `STRUCT`: **InterruptFrame** <sub>line 442</sub>
```rust
pub struct InterruptFrame {
```

#### `STRUCT`: **FxSaveArea** <sub>line 452</sub>
```rust
pub struct FxSaveArea {
```

#### `IMPL`: **FxSaveArea** <sub>line 456</sub>
```rust
impl FxSaveArea {
```

#### `IMPL`: **Default** <sub>line 462</sub>
```rust
impl Default for FxSaveArea {
```

#### `FN`: **default** <sub>line 463</sub>
```rust
fn default() -> Self {
```

#### `IMPL`: **fmt** <sub>line 468</sub>
```rust
impl fmt::Debug for FxSaveArea {
```

#### `FN`: **fmt** <sub>line 469</sub>
```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

#### `STRUCT`: **CpuContext** <sub>line 476</sub>
```rust
pub struct CpuContext {
```

#### `IMPL`: **Default** <sub>line 492</sub>
```rust
impl Default for CpuContext {
```

#### `FN`: **default** <sub>line 493</sub>
```rust
fn default() -> Self {
```

#### `IMPL`: **CpuContext** <sub>line 512</sub>
```rust
impl CpuContext {
```

#### `FN`: **xsave_area_size** <sub>line 548</sub>
```rust
pub fn xsave_area_size() -> usize {
```

#### `FN`: **rb_parent** <sub>line 557</sub>
```rust
pub fn rb_parent(pc: usize) -> *mut TaskStruct {
```

#### `FN`: **rb_color** <sub>line 562</sub>
```rust
pub fn rb_color(pc: usize) -> usize {
```

#### `FN`: **rb_is_red** <sub>line 567</sub>
```rust
pub fn rb_is_red(pc: usize) -> bool {
```

#### `FN`: **rb_is_black** <sub>line 572</sub>
```rust
pub fn rb_is_black(pc: usize) -> bool {
```

#### `FN`: **rb_make_parent_color** <sub>line 577</sub>
```rust
pub fn rb_make_parent_color(parent: *mut TaskStruct, color: usize) -> usize {
```

#### `STRUCT`: **LoadAvg** <sub>line 583</sub>
```rust
pub struct LoadAvg {
```

#### `IMPL`: **LoadAvg** <sub>line 594</sub>
```rust
impl LoadAvg {
```

#### `FN`: **accumulate** <sub>line 595</sub>
```rust
pub fn accumulate(&mut self, now: u64, delta_ns: u64, weight: u64, running: bool) {
```

#### `STRUCT`: **SchedEntity** <sub>line 624</sub>
```rust
pub struct SchedEntity {
```

#### `IMPL`: **Default** <sub>line 646</sub>
```rust
impl Default for SchedEntity {
```

#### `FN`: **default** <sub>line 647</sub>
```rust
fn default() -> Self {
```

#### `STRUCT`: **RtSchedEntity** <sub>line 671</sub>
```rust
pub struct RtSchedEntity {
```

#### `IMPL`: **Default** <sub>line 681</sub>
```rust
impl Default for RtSchedEntity {
```

#### `FN`: **default** <sub>line 682</sub>
```rust
fn default() -> Self {
```

#### `STRUCT`: **PlistNode** <sub>line 703</sub>
```rust
pub struct PlistNode {
```

#### `IMPL`: **PlistNode** <sub>line 710</sub>
```rust
impl PlistNode {
```

#### `FN`: **init** <sub>line 711</sub>
```rust
pub fn init(&mut self, owner: *mut TaskStruct) {
```

#### `IMPL`: **Default** <sub>line 719</sub>
```rust
impl Default for PlistNode {
```

#### `FN`: **default** <sub>line 720</sub>
```rust
fn default() -> Self {
```

#### `STRUCT`: **DlSchedEntity** <sub>line 731</sub>
```rust
pub struct DlSchedEntity {
```

#### `IMPL`: **Default** <sub>line 743</sub>
```rust
impl Default for DlSchedEntity {
```

#### `FN`: **default** <sub>line 744</sub>
```rust
fn default() -> Self {
```

#### `STRUCT`: **TaskStats** <sub>line 762</sub>
```rust
pub struct TaskStats {
```

#### `STRUCT`: **Credentials** <sub>line 781</sub>
```rust
pub struct Credentials {
```

#### `IMPL`: **Credentials** <sub>line 793</sub>
```rust
impl Credentials {
```

#### `IMPL`: **Default** <sub>line 823</sub>
```rust
impl Default for Credentials {
```

#### `FN`: **default** <sub>line 824</sub>
```rust
fn default() -> Self {
```

#### `STRUCT`: **RLimit** <sub>line 831</sub>
```rust
pub struct RLimit {
```

#### `IMPL`: **RLimit** <sub>line 836</sub>
```rust
impl RLimit {
```

#### `ENUM`: **RlimitResource** <sub>line 848</sub>
```rust
pub enum RlimitResource {
```

#### `STRUCT`: **SignalState** <sub>line 872</sub>
```rust
pub struct SignalState {
```

#### `IMPL`: **SignalState** <sub>line 877</sub>
```rust
impl SignalState {
```

#### `FN`: **has_pending** <sub>line 878</sub>
```rust
pub fn has_pending(&self) -> bool {
```

#### `FN`: **raise** <sub>line 882</sub>
```rust
pub fn raise(&mut self, signum: u8) {
```

#### `FN`: **clear** <sub>line 888</sub>
```rust
pub fn clear(&mut self, signum: u8) {
```

#### `STRUCT`: **ListHead** <sub>line 898</sub>
```rust
pub struct ListHead {
```

#### `IMPL`: **ListHead** <sub>line 903</sub>
```rust
impl ListHead {
```

#### `FN`: **init** <sub>line 908</sub>
```rust
pub fn init(&mut self) {
```

#### `FN`: **is_empty** <sub>line 914</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **is_linked** <sub>line 949</sub>
```rust
pub fn is_linked(&self) -> bool {
```

#### `IMPL`: **Default** <sub>line 954</sub>
```rust
impl Default for ListHead {
```

#### `FN`: **default** <sub>line 955</sub>
```rust
fn default() -> Self {
```

#### `STRUCT`: **SpinLock** <sub>line 997</sub>
```rust
pub struct SpinLock {
```

#### `IMPL`: **SpinLock** <sub>line 1003</sub>
```rust
impl SpinLock {
```

#### `FN`: **lock** <sub>line 1012</sub>
```rust
pub fn lock(&self) {
```

#### `FN`: **unlock** <sub>line 1024</sub>
```rust
pub fn unlock(&self) {
```

#### `FN`: **try_lock** <sub>line 1028</sub>
```rust
pub fn try_lock(&self) -> bool {
```

#### `FN`: **is_locked** <sub>line 1034</sub>
```rust
pub fn is_locked(&self) -> bool {
```

#### `FN`: **lock_irqsave** <sub>line 1038</sub>
```rust
pub fn lock_irqsave(&self) -> usize {
```

#### `FN`: **unlock_irqrestore** <sub>line 1043</sub>
```rust
pub fn unlock_irqrestore(&self, flags: usize) {
```

#### `FN`: **lock_guard** <sub>line 1048</sub>
```rust
pub fn lock_guard(&self) -> SpinLockGuard<'_> {
```

#### `FN`: **lock_irqsave_guard** <sub>line 1053</sub>
```rust
pub fn lock_irqsave_guard(&self) -> SpinLockGuard<'_> {
```

#### `STRUCT`: **SpinLockGuard** <sub>line 1060</sub>
```rust
pub struct SpinLockGuard<'a> {
```

#### `FN`: **drop** <sub>line 1066</sub>
```rust
fn drop(&mut self) {
```

#### `ENUM`: **TaskError** <sub>line 1076</sub>
```rust
pub enum TaskError {
```

#### `IMPL`: **fmt** <sub>line 1088</sub>
```rust
impl fmt::Display for TaskError {
```

#### `FN`: **fmt** <sub>line 1089</sub>
```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

#### `STRUCT`: **TaskStruct** <sub>line 1112</sub>
```rust
pub struct TaskStruct {
```

#### `IMPL`: **TaskStruct** <sub>line 1164</sub>
```rust
impl TaskStruct {
```

#### `FN`: **state** <sub>line 1403</sub>
```rust
pub fn state(&self) -> TaskState {
```

#### `FN`: **set_state_unchecked** <sub>line 1407</sub>
```rust
fn set_state_unchecked(&self, new_state: TaskState) {
```

#### `FN`: **set_state** <sub>line 1410</sub>
```rust
pub fn set_state(&self, new_state: TaskState) -> Result<(), TaskError> {
```

#### `FN`: **set_state_owned** <sub>line 1429</sub>
```rust
fn set_state_owned(&mut self, new_state: TaskState) -> Result<(), TaskError> {
```

#### `FN`: **wake_up** <sub>line 1433</sub>
```rust
pub fn wake_up(&self) -> Result<(), TaskError> {
```

#### `FN`: **sleep** <sub>line 1441</sub>
```rust
pub fn sleep(&self, interruptible: bool) -> Result<(), TaskError> {
```

#### `FN`: **is_kernel_thread** <sub>line 1446</sub>
```rust
pub fn is_kernel_thread(&self) -> bool {
```

#### `FN`: **is_idle_task** <sub>line 1450</sub>
```rust
pub fn is_idle_task(&self) -> bool {
```

#### `FN`: **is_zombie** <sub>line 1454</sub>
```rust
pub fn is_zombie(&self) -> bool {
```

#### `FN`: **is_runnable** <sub>line 1458</sub>
```rust
pub fn is_runnable(&self) -> bool {
```

#### `FN`: **needs_resched** <sub>line 1462</sub>
```rust
pub fn needs_resched(&self) -> bool {
```

#### `FN`: **set_need_resched** <sub>line 1466</sub>
```rust
pub fn set_need_resched(&self) {
```

#### `FN`: **clear_need_resched** <sub>line 1470</sub>
```rust
pub fn clear_need_resched(&self) {
```

#### `FN`: **set_nice** <sub>line 1474</sub>
```rust
pub fn set_nice(&mut self, nice: i8) -> Result<(), TaskError> {
```

#### `FN`: **set_rt_priority** <sub>line 1487</sub>
```rust
pub fn set_rt_priority(&mut self, rt_priority: u8) -> Result<(), TaskError> {
```

#### `FN`: **effective_prio** <sub>line 1501</sub>
```rust
pub fn effective_prio(&self) -> i32 {
```

#### `FN`: **set_affinity** <sub>line 1505</sub>
```rust
pub fn set_affinity(&mut self, mask: CpuMask) -> Result<(), TaskError> {
```

#### `FN`: **can_run_on** <sub>line 1517</sub>
```rust
pub fn can_run_on(&self, cpu: u32) -> bool {
```

#### `FN`: **rq_ptr** <sub>line 1522</sub>
```rust
pub fn rq_ptr(&self) -> *mut core::ffi::c_void {
```
> nigdzie.

#### `FN`: **set_rq_ptr** <sub>line 1526</sub>
```rust
pub fn set_rq_ptr(&self, rq: *mut core::ffi::c_void) {
```

#### `FN`: **charge_cputime** <sub>line 1530</sub>
```rust
pub fn charge_cputime(&mut self, delta_ns: u64) {
```

#### `FN`: **record_voluntary_switch** <sub>line 1549</sub>
```rust
pub fn record_voluntary_switch(&mut self) {
```

#### `FN`: **record_involuntary_switch** <sub>line 1553</sub>
```rust
pub fn record_involuntary_switch(&mut self) {
```

#### `FN`: **record_migration** <sub>line 1557</sub>
```rust
pub fn record_migration(&mut self, new_cpu: u32) {
```

#### `FN`: **set_comm** <sub>line 1564</sub>
```rust
pub fn set_comm(&mut self, name: &str) {
```

#### `FN`: **comm_str** <sub>line 1571</sub>
```rust
pub fn comm_str(&self) -> &str {
```

#### `IMPL`: **fmt** <sub>line 1577</sub>
```rust
impl fmt::Debug for TaskStruct {
```

#### `FN`: **fmt** <sub>line 1578</sub>
```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

#### `IMPL`: **fmt** <sub>line 1591</sub>
```rust
impl fmt::Display for TaskStruct {
```

#### `FN`: **fmt** <sub>line 1592</sub>
```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

#### `STRUCT`: **ThreadInfoView** <sub>line 1608</sub>
```rust
pub struct ThreadInfoView<'a> {
```

#### `IMPL`: **TaskStruct** <sub>line 1617</sub>
```rust
impl TaskStruct {
```

#### `FN`: **thread_info** <sub>line 1618</sub>
```rust
pub fn thread_info(&mut self) -> ThreadInfoView<'_> {
```

#### `FN`: **fair_has_priority** <sub>line 1641</sub>
```rust
pub fn fair_has_priority(a: &TaskStruct, b: &TaskStruct) -> bool {
```

#### `FN`: **deadline_has_priority** <sub>line 1645</sub>
```rust
pub fn deadline_has_priority(a: &TaskStruct, b: &TaskStruct) -> bool {
```

#### `IMPL`: **TaskStruct** <sub>line 1651</sub>
```rust
impl TaskStruct {
```

#### `FN`: **blank** <sub>line 1652</sub>
```rust
pub fn blank() -> TaskStruct {
```

#### `FN`: **init_test_stub** <sub>line 1696</sub>
```rust
pub fn init_test_stub(&mut self, pid: TaskId, policy: SchedPolicy, nice: i8) {
```

#### `FN`: **nice_to_weight_is_monotonically_decreasing** <sub>line 1760</sub>
```rust
fn nice_to_weight_is_monotonically_decreasing() {
```

#### `FN`: **nice_zero_has_default_weight** <sub>line 1770</sub>
```rust
fn nice_zero_has_default_weight() {
```

#### `FN`: **nice_to_weight_clamps_out_of_range** <sub>line 1775</sub>
```rust
fn nice_to_weight_clamps_out_of_range() {
```

#### `FN`: **weight_to_nice_round_trip_is_close** <sub>line 1781</sub>
```rust
fn weight_to_nice_round_trip_is_close() {
```

#### `FN`: **calc_delta_fair_is_identity_at_nice_zero** <sub>line 1789</sub>
```rust
fn calc_delta_fair_is_identity_at_nice_zero() {
```

#### `FN`: **calc_delta_fair_grows_for_lower_weight** <sub>line 1795</sub>
```rust
fn calc_delta_fair_grows_for_lower_weight() {
```

#### `FN`: **cpumask_basic_set_clear** <sub>line 1802</sub>
```rust
fn cpumask_basic_set_clear() {
```

#### `FN`: **cpumask_all_contains_every_cpu** <sub>line 1817</sub>
```rust
fn cpumask_all_contains_every_cpu() {
```

#### `FN`: **cpumask_intersects** <sub>line 1825</sub>
```rust
fn cpumask_intersects() {
```

#### `FN`: **cpumask_and_or** <sub>line 1836</sub>
```rust
fn cpumask_and_or() {
```

#### `FN`: **cpumask_next_after_wraps** <sub>line 1849</sub>
```rust
fn cpumask_next_after_wraps() {
```

#### `FN`: **cpumask_first_n** <sub>line 1858</sub>
```rust
fn cpumask_first_n() {
```

#### `FN`: **task_state_valid_transitions** <sub>line 1866</sub>
```rust
fn task_state_valid_transitions() {
```

#### `FN`: **task_state_invalid_transitions_are_rejected** <sub>line 1874</sub>
```rust
fn task_state_invalid_transitions_are_rejected() {
```

#### `FN`: **default_rlimits_have_sane_stack_and_nofile** <sub>line 1881</sub>
```rust
fn default_rlimits_have_sane_stack_and_nofile() {
```

#### `FN`: **credentials_kernel_has_full_capabilities** <sub>line 1889</sub>
```rust
fn credentials_kernel_has_full_capabilities() {
```

#### `FN`: **credentials_user_has_no_extra_capabilities** <sub>line 1896</sub>
```rust
fn credentials_user_has_no_extra_capabilities() {
```

#### `FN`: **signal_state_pending_respects_blocked_mask** <sub>line 1903</sub>
```rust
fn signal_state_pending_respects_blocked_mask() {
```

#### `FN`: **list_head_insert_and_remove_roundtrip** <sub>line 1914</sub>
```rust
fn list_head_insert_and_remove_roundtrip() {
```

#### `FN`: **list_head_remove_never_creates_a_cycle_with_three_nodes** <sub>line 1942</sub>
```rust
fn list_head_remove_never_creates_a_cycle_with_three_nodes() {
```

#### `FN`: **spinlock_lock_unlock_cycle** <sub>line 1966</sub>
```rust
fn spinlock_lock_unlock_cycle() {
```

#### `FN`: **spinlock_try_lock_fails_when_held** <sub>line 1976</sub>
```rust
fn spinlock_try_lock_fails_when_held() {
```

#### `FN`: **spinlock_irqsave_guard_unlocks_on_drop** <sub>line 1986</sub>
```rust
fn spinlock_irqsave_guard_unlocks_on_drop() {
```

#### `FN`: **spinlock_plain_guard_unlocks_on_drop** <sub>line 1996</sub>
```rust
fn spinlock_plain_guard_unlocks_on_drop() {
```

#### `FN`: **atomic_task_flags_insert_remove_are_bitwise** <sub>line 2006</sub>
```rust
fn atomic_task_flags_insert_remove_are_bitwise() {
```

#### `FN`: **atomic_task_flags_test_and_set_is_edge_triggered** <sub>line 2019</sub>
```rust
fn atomic_task_flags_test_and_set_is_edge_triggered() {
```

#### `FN`: **rb_parent_color_encoding_roundtrips** <sub>line 2026</sub>
```rust
fn rb_parent_color_encoding_roundtrips() {
```

#### `FN`: **fx_save_area_default_is_zeroed** <sub>line 2037</sub>
```rust
fn fx_save_area_default_is_zeroed() {
```

#### `FN`: **cpu_context_default_has_no_valid_fpu_state** <sub>line 2045</sub>
```rust
fn cpu_context_default_has_no_valid_fpu_state() {
```

#### `FN`: **sched_policy_maps_to_expected_class** <sub>line 2051</sub>
```rust
fn sched_policy_maps_to_expected_class() {
```

#### `FN`: **sched_class_ordering_matches_pick_next_priority** <sub>line 2062</sub>
```rust
fn sched_class_ordering_matches_pick_next_priority() {
```

#### `FN`: **default_time_slice_matches_policy_expectations** <sub>line 2070</sub>
```rust
fn default_time_slice_matches_policy_expectations() {
```

#### `FN`: **load_avg_accumulate_grows_then_decays** <sub>line 2077</sub>
```rust
fn load_avg_accumulate_grows_then_decays() {
```

#### `FN`: **bare_task_for_state_tests** <sub>line 2086</sub>
```rust
fn bare_task_for_state_tests() -> TaskStruct {
```

#### `FN`: **set_state_via_shared_reference_does_not_need_mut** <sub>line 2131</sub>
```rust
fn set_state_via_shared_reference_does_not_need_mut() {
```

#### `FN`: **set_state_rejects_illegal_transition_via_shared_reference** <sub>line 2141</sub>
```rust
fn set_state_rejects_illegal_transition_via_shared_reference() {
```

#### `FN`: **wake_up_process_pattern_from_remote_cpu_context** <sub>line 2149</sub>
```rust
fn wake_up_process_pattern_from_remote_cpu_context() {
```

#### `FN`: **remote_wake** <sub>line 2153</sub>
```rust
fn remote_wake(t: &TaskStruct) -> Result<(), TaskError> {
```

#### `FN`: **rq_backpointer_roundtrips** <sub>line 2162</sub>
```rust
fn rq_backpointer_roundtrips() {
```

#### `FN`: **need_resched_flag_is_settable_from_shared_reference** <sub>line 2171</sub>
```rust
fn need_resched_flag_is_settable_from_shared_reference() {
```

#### `FN`: **rt_sched_entity_starts_with_initialized_list_and_no_queued_prio** <sub>line 2181</sub>
```rust
fn rt_sched_entity_starts_with_initialized_list_and_no_queued_prio() {
```

#### `IMPL`: **TaskStruct** <sub>line 2187</sub>
```rust
impl TaskStruct {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/mod.rs</b> (2 items)</summary>

#### `FN`: **self_test** <sub>line 57</sub>
```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

#### `FN`: **current_cpu_id** <sub>line 68</sub>
```rust
pub fn current_cpu_id() -> u32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/power/em.rs</b> (38 items)</summary>

#### `STRUCT`: **CapacityState** <sub>line 16</sub>
```rust
pub struct CapacityState {
```

#### `IMPL`: **CapacityState** <sub>line 27</sub>
```rust
impl CapacityState {
```

#### `FN`: **compute_cost** <sub>line 54</sub>
```rust
pub fn compute_cost(&mut self, max_cap: u32) {
```

#### `FN`: **is_valid** <sub>line 64</sub>
```rust
pub fn is_valid(&self) -> bool {
```

#### `STRUCT`: **PerformanceDomain** <sub>line 70</sub>
```rust
pub struct PerformanceDomain {
```

#### `IMPL`: **PerformanceDomain** <sub>line 93</sub>
```rust
impl PerformanceDomain {
```

#### `FN`: **init** <sub>line 120</sub>
```rust
pub fn init(&mut self, id: u32, cpus: CpuMask) {
```

#### `FN`: **add_state** <sub>line 132</sub>
```rust
pub fn add_state(&mut self, state: CapacityState) -> bool {
```

#### `FN`: **finalize** <sub>line 151</sub>
```rust
pub fn finalize(&mut self) {
```

#### `FN`: **sort_states** <sub>line 160</sub>
```rust
fn sort_states(&mut self) {
```

#### `FN`: **find_state_for_capacity** <sub>line 173</sub>
```rust
pub fn find_state_for_capacity(&self, target_cap: u32) -> Option<&CapacityState> {
```

#### `FN`: **find_state_for_frequency** <sub>line 186</sub>
```rust
pub fn find_state_for_frequency(&self, target_freq: u32) -> Option<&CapacityState> {
```

#### `FN`: **get_effective_capacity** <sub>line 203</sub>
```rust
pub fn get_effective_capacity(&self) -> u32 {
```

#### `FN`: **update_thermal_pressure** <sub>line 207</sub>
```rust
pub fn update_thermal_pressure(&self, pressure: u32) {
```

#### `FN`: **compute_power_at_state** <sub>line 218</sub>
```rust
pub fn compute_power_at_state(&self, state_idx: usize, temp_millideg: u32) -> u32 {
```

#### `FN`: **update_temperature** <sub>line 237</sub>
```rust
pub fn update_temperature(&self, power_mw: u32, delta_time_ms: u32) {
```

#### `FN`: **contains_cpu** <sub>line 252</sub>
```rust
pub fn contains_cpu(&self, cpu: u32) -> bool {
```

#### `FN`: **get_active_state_idx** <sub>line 256</sub>
```rust
pub fn get_active_state_idx(&self) -> u32 {
```

#### `FN`: **set_active_state_idx** <sub>line 260</sub>
```rust
pub fn set_active_state_idx(&self, idx: u32) {
```

#### `STRUCT`: **EnergyModel** <sub>line 268</sub>
```rust
pub struct EnergyModel {
```

#### `IMPL`: **EnergyModel** <sub>line 281</sub>
```rust
impl EnergyModel {
```

#### `FN`: **init** <sub>line 298</sub>
```rust
pub fn init(&mut self) {
```

#### `FN`: **register_domain** <sub>line 307</sub>
```rust
pub fn register_domain(&mut self, pd: PerformanceDomain) -> bool {
```

#### `FN`: **get_pd_for_cpu** <sub>line 329</sub>
```rust
pub fn get_pd_for_cpu(&self, cpu: u32) -> Option<&PerformanceDomain> {
```

#### `FN`: **get_pd_for_cpu_mut** <sub>line 340</sub>
```rust
pub fn get_pd_for_cpu_mut(&mut self, cpu: u32) -> Option<&mut PerformanceDomain> {
```

#### `FN`: **compute_system_energy** <sub>line 351</sub>
```rust
pub fn compute_system_energy(&self, delta_time_ms: u32) -> u32 {
```

#### `FN`: **compute_task_energy_on_cpu** <sub>line 365</sub>
```rust
pub fn compute_task_energy_on_cpu(&self, task_util: u32, cpu: u32) -> u32 {
```

#### `FN`: **find_best_state_for_domain** <sub>line 381</sub>
```rust
pub fn find_best_state_for_domain(&self, pd_idx: u32, target_util: u32) -> Option<usize> {
```

#### `FN`: **invert_capacity_for_thermal** <sub>line 404</sub>
```rust
pub fn invert_capacity_for_thermal(&mut self, cpu: u32, thermal_pressure: u32) {
```

#### `FN`: **clear_thermal_pressure** <sub>line 410</sub>
```rust
pub fn clear_thermal_pressure(&mut self) {
```

#### `FN`: **get_max_capacity** <sub>line 416</sub>
```rust
pub fn get_max_capacity(&self) -> u32 {
```

#### `FN`: **get_min_capacity** <sub>line 420</sub>
```rust
pub fn get_min_capacity(&self) -> u32 {
```

#### `FN`: **is_cpu_big** <sub>line 424</sub>
```rust
pub fn is_cpu_big(&self, cpu: u32) -> bool {
```

#### `FN`: **is_cpu_little** <sub>line 432</sub>
```rust
pub fn is_cpu_little(&self, cpu: u32) -> bool {
```

#### `FN`: **dump_state** <sub>line 440</sub>
```rust
pub fn dump_state(&self) {
```

#### `FN`: **em_build_synthetic_little_pd** <sub>line 484</sub>
```rust
pub fn em_build_synthetic_little_pd(cpus: CpuMask) -> PerformanceDomain {
```

#### `FN`: **em_build_synthetic_big_pd** <sub>line 502</sub>
```rust
pub fn em_build_synthetic_big_pd(cpus: CpuMask) -> PerformanceDomain {
```

#### `FN`: **em_build_synthetic_huge_pd** <sub>line 521</sub>
```rust
pub fn em_build_synthetic_huge_pd(cpus: CpuMask) -> PerformanceDomain {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/power/mod.rs</b> (26 items)</summary>

#### `STRUCT`: **PidState** <sub>line 28</sub>
```rust
pub struct PidState {
```

#### `IMPL`: **PidState** <sub>line 35</sub>
```rust
impl PidState {
```

#### `FN`: **update** <sub>line 45</sub>
```rust
pub fn update(&mut self, measurement: i64) -> i64 {
```

#### `FN`: **reset** <sub>line 54</sub>
```rust
pub fn reset(&mut self) {
```

#### `STRUCT`: **ThermalZone** <sub>line 62</sub>
```rust
pub struct ThermalZone {
```

#### `IMPL`: **ThermalZone** <sub>line 74</sub>
```rust
impl ThermalZone {
```

#### `FN`: **init** <sub>line 89</sub>
```rust
pub fn init(&mut self, pd_idx: u32) {
```

#### `FN`: **update_temp** <sub>line 99</sub>
```rust
pub fn update_temp(&self, temp_millideg: u32) {
```

#### `FN`: **get_temp** <sub>line 103</sub>
```rust
pub fn get_temp(&self) -> u32 {
```

#### `FN`: **get_level** <sub>line 107</sub>
```rust
pub fn get_level(&self) -> u32 {
```

#### `FN`: **evaluate_level** <sub>line 111</sub>
```rust
pub fn evaluate_level(&self) -> u32 {
```

#### `FN`: **compute_pid_output** <sub>line 148</sub>
```rust
pub fn compute_pid_output(&mut self) -> i64 {
```

#### `STRUCT`: **PowerScheduler** <sub>line 155</sub>
```rust
pub struct PowerScheduler {
```

#### `IMPL`: **PowerScheduler** <sub>line 168</sub>
```rust
impl PowerScheduler {
```

#### `FN`: **init** <sub>line 186</sub>
```rust
pub fn init(&mut self) {
```

#### `FN`: **register_zone** <sub>line 199</sub>
```rust
pub fn register_zone(&mut self, pd_idx: u32) -> bool {
```

#### `FN`: **get_zone_for_pd** <sub>line 209</sub>
```rust
pub fn get_zone_for_pd(&self, pd_idx: u32) -> Option<&ThermalZone> {
```

#### `FN`: **get_zone_for_pd_mut** <sub>line 218</sub>
```rust
pub fn get_zone_for_pd_mut(&mut self, pd_idx: u32) -> Option<&mut ThermalZone> {
```

#### `FN`: **update_tick** <sub>line 227</sub>
```rust
pub fn update_tick(&mut self, current_time_ns: u64) {
```

#### `FN`: **apply_mitigation** <sub>line 258</sub>
```rust
pub fn apply_mitigation(&mut self) {
```

#### `FN`: **get_max_allowed_capacity** <sub>line 308</sub>
```rust
pub fn get_max_allowed_capacity(&self, cpu: u32) -> u32 {
```

#### `FN`: **is_eas_enabled** <sub>line 315</sub>
```rust
pub fn is_eas_enabled(&self) -> bool {
```

#### `FN`: **enable_eas** <sub>line 319</sub>
```rust
pub fn enable_eas(&self) {
```

#### `FN`: **disable_eas** <sub>line 323</sub>
```rust
pub fn disable_eas(&self) {
```

#### `FN`: **is_emergency_shutdown** <sub>line 327</sub>
```rust
pub fn is_emergency_shutdown(&self) -> bool {
```

#### `FN`: **dump_thermal_state** <sub>line 331</sub>
```rust
pub fn dump_thermal_state(&self) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/power/placement.rs</b> (8 items)</summary>

#### `STRUCT`: **CpuSnapshot** <sub>line 14</sub>
```rust
pub struct CpuSnapshot {
```

#### `IMPL`: **CpuSnapshot** <sub>line 27</sub>
```rust
impl CpuSnapshot {
```

#### `STRUCT`: **DomainSnapshot** <sub>line 35</sub>
```rust
pub struct DomainSnapshot {
```

#### `IMPL`: **DomainSnapshot** <sub>line 46</sub>
```rust
impl DomainSnapshot {
```

#### `STRUCT`: **EnergyCtx** <sub>line 54</sub>
```rust
pub struct EnergyCtx {
```

#### `IMPL`: **EnergyCtx** <sub>line 68</sub>
```rust
impl EnergyCtx {
```

#### `STRUCT`: **PlacementResult** <sub>line 77</sub>
```rust
pub struct PlacementResult {
```

#### `IMPL`: **PlacementResult** <sub>line 85</sub>
```rust
impl PlacementResult {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/smp/balancing/active.rs</b> (3 items)</summary>

#### `STRUCT`: **ActiveBalanceArg** <sub>line 13</sub>
```rust
struct ActiveBalanceArg {
```

#### `FN`: **active_balance_is_noop_with_zero_imbalance** <sub>line 84</sub>
```rust
fn active_balance_is_noop_with_zero_imbalance() {
```

#### `FN`: **active_balance_is_noop_when_target_equals_busiest** <sub>line 95</sub>
```rust
fn active_balance_is_noop_when_target_equals_busiest() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/smp/balancing/calculate.rs</b> (9 items)</summary>

#### `ENUM`: **GroupType** <sub>line 17</sub>
```rust
pub enum GroupType {
```

#### `STRUCT`: **LoadCalculation** <sub>line 28</sub>
```rust
pub struct LoadCalculation {
```

#### `IMPL`: **LoadCalculation** <sub>line 40</sub>
```rust
impl LoadCalculation {
```

#### `FN`: **load_calculation_defaults** <sub>line 244</sub>
```rust
fn load_calculation_defaults() {
```

#### `FN`: **group_classification_idle** <sub>line 251</sub>
```rust
fn group_classification_idle() {
```

#### `FN`: **group_classification_has_idle_beats_overloaded** <sub>line 260</sub>
```rust
fn group_classification_has_idle_beats_overloaded() {
```

#### `FN`: **group_classification_overloaded_when_busy_and_full** <sub>line 271</sub>
```rust
fn group_classification_overloaded_when_busy_and_full() {
```

#### `FN`: **calculate_imbalance_moves_load_toward_equilibrium** <sub>line 281</sub>
```rust
fn calculate_imbalance_moves_load_toward_equilibrium() {
```

#### `FN`: **calculate_imbalance_is_zero_for_balanced_groups** <sub>line 304</sub>
```rust
fn calculate_imbalance_is_zero_for_balanced_groups() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/smp/balancing/env.rs</b> (14 items)</summary>

#### `STRUCT`: **LoadBalanceEnv** <sub>line 18</sub>
```rust
pub struct LoadBalanceEnv {
```

#### `IMPL`: **LoadBalanceEnv** <sub>line 28</sub>
```rust
impl LoadBalanceEnv {
```

#### `FN`: **set_flag** <sub>line 41</sub>
```rust
pub fn set_flag(&mut self, flag: u32) {
```

#### `FN`: **clear_flag** <sub>line 45</sub>
```rust
pub fn clear_flag(&mut self, flag: u32) {
```

#### `FN`: **has_flag** <sub>line 49</sub>
```rust
pub fn has_flag(&self, flag: u32) -> bool {
```

#### `FN`: **should_stop** <sub>line 55</sub>
```rust
pub fn should_stop(&self) -> bool {
```
> Czy przebieg powinien się zatrzymać (osiągnięto limit prób 
> przeniesienia zadań w tej iteracji `load_balance`).

#### `FN`: **record_attempt** <sub>line 59</sub>
```rust
pub fn record_attempt(&mut self) {
```

#### `FN`: **record_failure** <sub>line 63</sub>
```rust
pub fn record_failure(&mut self) {
```

#### `FN`: **reset_for_next_iteration** <sub>line 68</sub>
```rust
pub fn reset_for_next_iteration(&mut self) {
```

#### `FN`: **env** <sub>line 79</sub>
```rust
fn env() -> LoadBalanceEnv {
```

#### `FN`: **flags_set_clear_roundtrip** <sub>line 84</sub>
```rust
fn flags_set_clear_roundtrip() {
```

#### `FN`: **should_stop_once_loop_max_reached** <sub>line 94</sub>
```rust
fn should_stop_once_loop_max_reached() {
```

#### `FN`: **record_failure_sets_some_pinned** <sub>line 106</sub>
```rust
fn record_failure_sets_some_pinned() {
```

#### `FN`: **reset_clears_loop_counter_and_need_break** <sub>line 115</sub>
```rust
fn reset_clears_loop_counter_and_need_break() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/smp/ipi.rs</b> (4 items)</summary>

#### `ENUM`: **IpiType** <sub>line 8</sub>
```rust
pub enum IpiType {
```

#### `FN`: **migration_stop_ipi_increments_dedicated_counter** <sub>line 75</sub>
```rust
fn migration_stop_ipi_increments_dedicated_counter() {
```

#### `FN`: **out_of_range_cpu_is_a_safe_noop** <sub>line 85</sub>
```rust
fn out_of_range_cpu_is_a_safe_noop() {
```

#### `FN`: **handle_ipi_entry_ignores_unimplemented_types_without_panicking** <sub>line 92</sub>
```rust
fn handle_ipi_entry_ignores_unimplemented_types_without_panicking() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/smp/migration/stopper.rs</b> (22 items)</summary>

#### `ENUM`: **StopperState** <sub>line 10</sub>
```rust
pub enum StopperState {
```

#### `STRUCT`: **StopperWork** <sub>line 19</sub>
```rust
pub struct StopperWork {
```

#### `IMPL`: **StopperWork** <sub>line 29</sub>
```rust
impl StopperWork {
```

#### `FN`: **get_state** <sub>line 41</sub>
```rust
pub fn get_state(&self) -> StopperState {
```

#### `FN`: **set_state** <sub>line 52</sub>
```rust
pub fn set_state(&self, new_state: StopperState) {
```

#### `TYPE`: **StopperFn** <sub>line 57</sub>
```rust
pub type StopperFn = unsafe extern "C" fn(*mut core::ffi::c_void) -> u32;
```

#### `STRUCT`: **CpuStopper** <sub>line 60</sub>
```rust
pub struct CpuStopper {
```

#### `IMPL`: **CpuStopper** <sub>line 70</sub>
```rust
impl CpuStopper {
```

#### `FN`: **init** <sub>line 83</sub>
```rust
pub fn init(&mut self, cpu: u32) {
```

#### `FN`: **queue_work** <sub>line 88</sub>
```rust
pub fn queue_work(&self, func: StopperFn, arg: *mut core::ffi::c_void) -> bool {
```

#### `FN`: **wake_stopper_task** <sub>line 108</sub>
```rust
fn wake_stopper_task(&self) {
```

#### `FN`: **execute_work** <sub>line 120</sub>
```rust
pub fn execute_work(&self) {
```

#### `FN`: **wait_for_completion** <sub>line 146</sub>
```rust
pub fn wait_for_completion(&self) -> u32 {
```

#### `STRUCT`: **StopperRegistry** <sub>line 166</sub>
```rust
pub struct StopperRegistry {
```

#### `IMPL`: **StopperRegistry** <sub>line 170</sub>
```rust
impl StopperRegistry {
```

#### `FN`: **init_cpu** <sub>line 178</sub>
```rust
pub fn init_cpu(&mut self, cpu: u32) {
```

#### `FN`: **get** <sub>line 184</sub>
```rust
pub fn get(&self, cpu: u32) -> Option<&CpuStopper> {
```

#### `FN`: **stopper_state_transitions** <sub>line 280</sub>
```rust
fn stopper_state_transitions() {
```

#### `FN`: **stopper_registry_bounds_check** <sub>line 295</sub>
```rust
fn stopper_registry_bounds_check() {
```

#### `FN`: **queue_execute_and_wait_roundtrip_returns_value_from_function** <sub>line 302</sub>
```rust
fn queue_execute_and_wait_roundtrip_returns_value_from_function() {
```

#### `FN`: **queue_work_rejects_a_second_job_while_busy** <sub>line 317</sub>
```rust
fn queue_work_rejects_a_second_job_while_busy() {
```

#### `FN`: **execute_work_without_queued_job_is_a_noop** <sub>line 325</sub>
```rust
fn execute_work_without_queued_job_is_a_noop() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/smp/topology.rs</b> (21 items)</summary>

#### `ENUM`: **CacheLevel** <sub>line 24</sub>
```rust
pub enum CacheLevel {
```

#### `STRUCT`: **CacheTopology** <sub>line 33</sub>
```rust
pub struct CacheTopology {
```

#### `STRUCT`: **NumaNode** <sub>line 41</sub>
```rust
pub struct NumaNode {
```

#### `IMPL`: **NumaNode** <sub>line 49</sub>
```rust
impl NumaNode {
```

#### `FN`: **distance_to** <sub>line 60</sub>
```rust
pub fn distance_to(&self, other_node: u32) -> u8 {
```

#### `STRUCT`: **SchedGroup** <sub>line 67</sub>
```rust
pub struct SchedGroup {
```

#### `IMPL`: **SchedGroup** <sub>line 79</sub>
```rust
impl SchedGroup {
```

#### `STRUCT`: **SchedDomain** <sub>line 96</sub>
```rust
pub struct SchedDomain {
```

#### `IMPL`: **SchedDomain** <sub>line 115</sub>
```rust
impl SchedDomain {
```

#### `FN`: **has_flag** <sub>line 137</sub>
```rust
pub fn has_flag(&self, flag: u32) -> bool {
```

#### `FN`: **is_numa** <sub>line 141</sub>
```rust
pub fn is_numa(&self) -> bool {
```

#### `FN`: **shares_cache** <sub>line 145</sub>
```rust
pub fn shares_cache(&self) -> bool {
```

#### `FN`: **is_smt** <sub>line 149</sub>
```rust
pub fn is_smt(&self) -> bool {
```

#### `STRUCT`: **CpuTopology** <sub>line 155</sub>
```rust
pub struct CpuTopology {
```

#### `IMPL`: **CpuTopology** <sub>line 170</sub>
```rust
impl CpuTopology {
```

#### `STRUCT`: **TopologyState** <sub>line 192</sub>
```rust
pub struct TopologyState {
```

#### `IMPL`: **TopologyState** <sub>line 205</sub>
```rust
impl TopologyState {
```

#### `FN`: **alloc_domain** <sub>line 226</sub>
```rust
pub fn alloc_domain(&mut self) -> Option<&mut SchedDomain> {
```

#### `FN`: **alloc_group** <sub>line 234</sub>
```rust
pub fn alloc_group(&mut self) -> Option<&mut SchedGroup> {
```

#### `FN`: **topology_alloc_domains_respects_limits** <sub>line 393</sub>
```rust
fn topology_alloc_domains_respects_limits() {
```

#### `FN`: **sched_domain_flags_logic** <sub>line 406</sub>
```rust
fn sched_domain_flags_logic() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/tests/test_balance_math.rs</b> (13 items)</summary>

#### `FN`: **make_idle** <sub>line 5</sub>
```rust
fn make_idle(pid: u64, cpu: u32) -> TaskStruct {
```

#### `FN`: **make_task** <sub>line 12</sub>
```rust
fn make_task(pid: u64, policy: SchedPolicy, nice: i8) -> TaskStruct {
```

#### `FN`: **ptr_of** <sub>line 18</sub>
```rust
fn ptr_of(t: &mut TaskStruct) -> *mut TaskStruct {
```

#### `FN`: **select_task_rq_returns_preferred_cpu_when_idle** <sub>line 23</sub>
```rust
fn select_task_rq_returns_preferred_cpu_when_idle() {
```

#### `FN`: **select_task_rq_falls_back_to_least_loaded** <sub>line 42</sub>
```rust
fn select_task_rq_falls_back_to_least_loaded() {
```

#### `FN`: **select_task_rq_respects_cpus_allowed_mask** <sub>line 68</sub>
```rust
fn select_task_rq_respects_cpus_allowed_mask() {
```

#### `FN`: **wake_up_process_enqueues_to_target_cpu** <sub>line 88</sub>
```rust
fn wake_up_process_enqueues_to_target_cpu() {
```

#### `FN`: **idle_balance_pulls_task_from_overloaded_cpu** <sub>line 107</sub>
```rust
fn idle_balance_pulls_task_from_overloaded_cpu() {
```

#### `FN`: **load_balance_moves_task_across_uneven_queues** <sub>line 135</sub>
```rust
fn load_balance_moves_task_across_uneven_queues() {
```

#### `FN`: **no_migration_when_cpus_allowed_forbids_it** <sub>line 161</sub>
```rust
fn no_migration_when_cpus_allowed_forbids_it() {
```

#### `FN`: **migrated_task_updates_cpu_and_rq_pointer** <sub>line 189</sub>
```rust
fn migrated_task_updates_cpu_and_rq_pointer() {
```

#### `FN`: **pf_no_migrate_blocks_migration** <sub>line 224</sub>
```rust
fn pf_no_migrate_blocks_migration() {
```

#### `FN`: **load_balance_is_noop_below_threshold** <sub>line 250</sub>
```rust
fn load_balance_is_noop_below_threshold() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/tests/test_pelt.rs</b> (15 items)</summary>

#### `FN`: **default_load_avg_is_zero** <sub>line 4</sub>
```rust
fn default_load_avg_is_zero() {
```

#### `FN`: **accumulate_zero_delta_does_not_change_state_except_time** <sub>line 14</sub>
```rust
fn accumulate_zero_delta_does_not_change_state_except_time() {
```

#### `FN`: **accumulate_running_task_increases_util_and_load** <sub>line 23</sub>
```rust
fn accumulate_running_task_increases_util_and_load() {
```

#### `FN`: **accumulate_idle_task_increases_load_but_not_util** <sub>line 34</sub>
```rust
fn accumulate_idle_task_increases_load_but_not_util() {
```

#### `FN`: **decay_reduces_load_avg_over_time** <sub>line 44</sub>
```rust
fn decay_reduces_load_avg_over_time() {
```

#### `FN`: **load_avg_never_exceeds_theoretical_maximum** <sub>line 56</sub>
```rust
fn load_avg_never_exceeds_theoretical_maximum() {
```

#### `FN`: **higher_weight_produces_higher_load_avg** <sub>line 67</sub>
```rust
fn higher_weight_produces_higher_load_avg() {
```

#### `FN`: **saturating_add_prevents_overflow_on_huge_delta** <sub>line 78</sub>
```rust
fn saturating_add_prevents_overflow_on_huge_delta() {
```

#### `FN`: **alternating_running_and_idle_smooths_util_avg** <sub>line 86</sub>
```rust
fn alternating_running_and_idle_smooths_util_avg() {
```

#### `FN`: **period_contrib_increments_on_every_accumulate** <sub>line 99</sub>
```rust
fn period_contrib_increments_on_every_accumulate() {
```

#### `FN`: **util_sum_accumulates_only_when_running** <sub>line 109</sub>
```rust
fn util_sum_accumulates_only_when_running() {
```

#### `FN`: **load_sum_accumulates_regardless_of_running_state** <sub>line 118</sub>
```rust
fn load_sum_accumulates_regardless_of_running_state() {
```

#### `FN`: **pelt_precision_under_one_millisecond** <sub>line 127</sub>
```rust
fn pelt_precision_under_one_millisecond() {
```

#### `FN`: **pelt_handles_max_weight_without_panic** <sub>line 134</sub>
```rust
fn pelt_handles_max_weight_without_panic() {
```

#### `FN`: **consecutive_accumulates_update_last_time_correctly** <sub>line 141</sub>
```rust
fn consecutive_accumulates_update_last_time_correctly() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/tests/test_rbtree.rs</b> (14 items)</summary>

#### `FN`: **fair_key** <sub>line 5</sub>
```rust
fn fair_key(node: *const TaskStruct) -> u64 {
```

#### `FN`: **dl_key** <sub>line 9</sub>
```rust
fn dl_key(node: *const TaskStruct) -> u64 {
```

#### `FN`: **make_task** <sub>line 13</sub>
```rust
fn make_task(pid: u64, vruntime: u64, deadline: u64) -> TaskStruct {
```

#### `FN`: **empty_tree_has_zero_count_and_ok_invariants** <sub>line 22</sub>
```rust
fn empty_tree_has_zero_count_and_ok_invariants() {
```

#### `FN`: **single_node_tree_is_black_and_valid** <sub>line 33</sub>
```rust
fn single_node_tree_is_black_and_valid() {
```

#### `FN`: **sequential_inserts_maintain_sorted_order_and_invariants** <sub>line 46</sub>
```rust
fn sequential_inserts_maintain_sorted_order_and_invariants() {
```

#### `FN`: **reverse_inserts_maintain_invariants** <sub>line 77</sub>
```rust
fn reverse_inserts_maintain_invariants() {
```

#### `FN`: **random_inserts_and_deletes_keep_tree_valid** <sub>line 97</sub>
```rust
fn random_inserts_and_deletes_keep_tree_valid() {
```

#### `FN`: **delete_root_repeatedly_destroys_tree_cleanly** <sub>line 136</sub>
```rust
fn delete_root_repeatedly_destroys_tree_cleanly() {
```

#### `FN`: **delete_successor_maintains_inorder_traversal** <sub>line 160</sub>
```rust
fn delete_successor_maintains_inorder_traversal() {
```

#### `FN`: **duplicate_vruntime_inserts_are_placed_to_the_right** <sub>line 206</sub>
```rust
fn duplicate_vruntime_inserts_are_placed_to_the_right() {
```

#### `FN`: **deadline_key_orders_by_earliest_deadline** <sub>line 234</sub>
```rust
fn deadline_key_orders_by_earliest_deadline() {
```

#### `FN`: **subtree_min_and_max_return_extremes** <sub>line 254</sub>
```rust
fn subtree_min_and_max_return_extremes() {
```

#### `FN`: **stress_test_insert_delete_cycles** <sub>line 276</sub>
```rust
fn stress_test_insert_delete_cycles() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/time/plist.rs</b> (6 items)</summary>

#### `FN`: **plist_prio** <sub>line 20</sub>
```rust
fn plist_prio(node: *mut TaskStruct) -> i32 {
```

#### `STRUCT`: **PList** <sub>line 42</sub>
```rust
pub struct PList {
```

#### `IMPL`: **PList** <sub>line 46</sub>
```rust
impl PList {
```

#### `FN`: **is_empty** <sub>line 51</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **first** <sub>line 150</sub>
```rust
pub fn first(&self) -> *mut TaskStruct {
```
> Zadanie o najwyższym priorytecie (head listy poziomów), 
> pierwsze w kolejności FIFO na tym poziomie. O(1).

#### `FN`: **last** <sub>line 165</sub>
```rust
pub fn last(&self) -> *mut TaskStruct {
```
> było gwarantowane przez ówczesny `insert`). Teraz `insert` 
> utrzymuje ten inwariant jawnie: `same_prio.prev` head'a 
> zawsze wskazuje ogon FIFO, więc odczyt jest bezpośredni.

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/time/rbtree.rs</b> (9 items)</summary>

#### `FN`: **rb_parent** <sub>line 23</sub>
```rust
fn rb_parent(node: *mut TaskStruct) -> *mut TaskStruct {
```

#### `FN`: **rb_color** <sub>line 29</sub>
```rust
fn rb_color(node: *mut TaskStruct) -> usize {
```

#### `FN`: **rb_is_red** <sub>line 35</sub>
```rust
fn rb_is_red(node: *mut TaskStruct) -> bool {
```

#### `FN`: **rb_set_parent_color** <sub>line 40</sub>
```rust
fn rb_set_parent_color(node: *mut TaskStruct, parent: *mut TaskStruct, color: usize) {
```

#### `FN`: **rb_set_parent** <sub>line 48</sub>
```rust
fn rb_set_parent(node: *mut TaskStruct, parent: *mut TaskStruct) {
```

#### `FN`: **rb_set_color** <sub>line 54</sub>
```rust
fn rb_set_color(node: *mut TaskStruct, color: usize) {
```

#### `STRUCT`: **RbTree** <sub>line 62</sub>
```rust
pub struct RbTree {
```

#### `IMPL`: **RbTree** <sub>line 67</sub>
```rust
impl RbTree {
```

#### `FN`: **rb_insert_fixup** <sub>line 116</sub>
```rust
fn rb_insert_fixup(&mut self, mut node: *mut TaskStruct) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/time/rt_array.rs</b> (7 items)</summary>

#### `STRUCT`: **RtArray** <sub>line 4</sub>
```rust
pub struct RtArray {
```

#### `IMPL`: **RtArray** <sub>line 10</sub>
```rust
impl RtArray {
```

#### `FN`: **set_bit** <sub>line 20</sub>
```rust
pub fn set_bit(&mut self, prio: usize) {
```

#### `FN`: **clear_bit** <sub>line 27</sub>
```rust
pub fn clear_bit(&mut self, prio: usize) {
```

#### `FN`: **bit_is_set** <sub>line 34</sub>
```rust
fn bit_is_set(&self, prio: usize) -> bool {
```

#### `FN`: **highest_prio** <sub>line 41</sub>
```rust
pub fn highest_prio(&self) -> Option<usize> {
```

#### `FN`: **active_levels** <sub>line 149</sub>
```rust
pub fn active_levels(&self) -> u32 {
```
> priorytetów" bez przechodzenia całej bitmapy ręcznie za każdym 
> razem, gdy taka informacja jest potrzebna (np. w heurystykach 
> load-balancingu między CPU).

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/scheduler/time/task.rs</b> (7 items)</summary>

#### `STRUCT`: **ListHead** <sub>line 7</sub>
```rust
pub struct ListHead {
```

#### `IMPL`: **ListHead** <sub>line 12</sub>
```rust
impl ListHead {
```

#### `FN`: **is_empty** <sub>line 18</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `STRUCT`: **RtFields** <sub>line 52</sub>
```rust
pub struct RtFields {
```

#### `STRUCT`: **FairFields** <sub>line 58</sub>
```rust
pub struct FairFields {
```

#### `STRUCT`: **TaskStruct** <sub>line 63</sub>
```rust
pub struct TaskStruct {
```

#### `IMPL`: **TaskStruct** <sub>line 78</sub>
```rust
impl TaskStruct {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/smp.rs</b> (13 items)</summary>

#### `STRUCT`: **ApStack** <sub>line 32</sub>
```rust
struct ApStack([u8; AP_STACK_SIZE]);
```

#### `FN`: **stack_top_of** <sub>line 36</sub>
```rust
fn stack_top_of(i: usize) -> u64 {
```

#### `FN`: **delay** <sub>line 42</sub>
```rust
fn delay(iterations: u64) {
```

#### `FN`: **delay_ms** <sub>line 48</sub>
```rust
fn delay_ms(ms: u64) {
```

#### `FN`: **load_cpu_gdt** <sub>line 52</sub>
```rust
fn load_cpu_gdt(ist_stack_top: VirtAddr) {
```

#### `FN`: **debug_halt** <sub>line 76</sub>
```rust
fn debug_halt(msg: &str) -> ! {
```

#### `FN`: **flush_serial_no_panic** <sub>line 91</sub>
```rust
pub fn flush_serial_no_panic() {
```
> Flush the serial port's transmit FIFO (COM1) so all pending bytes actually 
> hit the log file before QEMU is killed/timeout-terminated.

#### `FN`: **wait_for** <sub>line 131</sub>
```rust
fn wait_for(flag: &AtomicBool, max_ms: u64) -> bool {
```

#### `FN`: **init** <sub>line 141</sub>
```rust
pub fn init(boot_info: &'static bootloader::BootInfo) {
```

#### `FN`: **total_cpus** <sub>line 248</sub>
```rust
pub fn total_cpus() -> u32 {
```

#### `FN`: **poweroff** <sub>line 252</sub>
```rust
pub fn poweroff() -> bool {
```

#### `FN`: **reboot** <sub>line 276</sub>
```rust
pub fn reboot() -> ! {
```

#### `FN`: **self_test** <sub>line 293</sub>
```rust
pub fn self_test() -> TestResult {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/cpu/trampoline.rs</b> (2 items)</summary>

#### `FN`: **install** <sub>line 23</sub>
```rust
pub fn install(cr3: u64, entry: u64) {
```

#### `FN`: **set_stack_and_arg** <sub>line 48</sub>
```rust
pub fn set_stack_and_arg(stack_top: u64, arg: u64) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/common.rs</b> (5 items)</summary>

#### `STRUCT`: **PackageId** <sub>line 2</sub>
```rust
pub struct PackageId {
```

#### `STRUCT`: **Version** <sub>line 8</sub>
```rust
pub struct Version {
```

#### `ENUM`: **ComponentKind** <sub>line 15</sub>
```rust
pub enum ComponentKind {
```

#### `ENUM`: **PackageStatus** <sub>line 23</sub>
```rust
pub enum PackageStatus {
```

#### `ENUM`: **CtrlInstallError** <sub>line 31</sub>
```rust
pub enum CtrlInstallError {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/init/bootstrap.rs</b> (2 items)</summary>

#### `STRUCT`: **BootstrapResult** <sub>line 1</sub>
```rust
pub struct BootstrapResult {
```

#### `FN`: **bootstrap** <sub>line 6</sub>
```rust
pub fn bootstrap() -> Result<BootstrapResult, CtrlInstallError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/init/state.rs</b> (7 items)</summary>

#### `STRUCT`: **SystemState** <sub>line 1</sub>
```rust
pub struct SystemState {
```

#### `STRUCT`: **InstalledPackage** <sub>line 6</sub>
```rust
pub struct InstalledPackage {
```

#### `IMPL`: **SystemState** <sub>line 15</sub>
```rust
impl SystemState {
```

#### `FN`: **is_installed** <sub>line 16</sub>
```rust
pub fn is_installed(&self, name: &str) -> bool { todo!() }
```

#### `FN`: **get_package** <sub>line 17</sub>
```rust
pub fn get_package(&self, name: &str) -> Option<&InstalledPackage> { todo!() }
```

#### `FN`: **register** <sub>line 18</sub>
```rust
pub fn register(&mut self, pkg: InstalledPackage) { todo!() }
```

#### `FN`: **unregister** <sub>line 19</sub>
```rust
pub fn unregister(&mut self, name: &str) -> Result<(), CtrlInstallError> { todo!() }
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/install/executor.rs</b> (2 items)</summary>

#### `STRUCT`: **InstallExecutor** <sub>line 1</sub>
```rust
pub struct InstallExecutor<'a> {
```

#### `FN`: **execute** <sub>line 6</sub>
```rust
pub fn execute(&mut self, tx: &Transaction) -> Result<(), CtrlInstallError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/install/resolver.rs</b> (2 items)</summary>

#### `STRUCT`: **ResolvedPlan** <sub>line 1</sub>
```rust
pub struct ResolvedPlan {
```

#### `FN`: **resolve** <sub>line 5</sub>
```rust
pub fn resolve(
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/install/transaction.rs</b> (2 items)</summary>

#### `ENUM`: **TransactionStep** <sub>line 1</sub>
```rust
pub enum TransactionStep {
```

#### `STRUCT`: **Transaction** <sub>line 8</sub>
```rust
pub struct Transaction {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/mod.rs</b> (6 items)</summary>

#### `STRUCT`: **CtrlInstall** <sub>line 9</sub>
```rust
pub struct CtrlInstall {
```

#### `IMPL`: **CtrlInstall** <sub>line 14</sub>
```rust
impl CtrlInstall {
```

#### `FN`: **new** <sub>line 15</sub>
```rust
pub fn new() -> Result<Self, CtrlInstallError> {
```

#### `FN`: **install** <sub>line 21</sub>
```rust
pub fn install(&mut self, name: &str) -> Result<(), CtrlInstallError> {
```

#### `FN`: **update** <sub>line 33</sub>
```rust
pub fn update(&mut self) -> Result<(), CtrlInstallError> {
```

#### `FN`: **list_installed** <sub>line 39</sub>
```rust
pub fn list_installed(&self) -> &[init::state::InstalledPackage] {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/repo/fetch.rs</b> (4 items)</summary>

#### `TRAIT`: **PackageFetcher** <sub>line 1</sub>
```rust
pub trait PackageFetcher {
```

#### `FN`: **fetch** <sub>line 2</sub>
```rust
fn fetch(&mut self, manifest: &PackageManifest) -> Result<alloc::vec::Vec<u8>, CtrlInstallError>;
```

#### `STRUCT`: **LocalFetcher** <sub>line 5</sub>
```rust
pub struct LocalFetcher {
```

#### `STRUCT`: **RemoteFetcher** <sub>line 9</sub>
```rust
pub struct RemoteFetcher {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/repo/index.rs</b> (5 items)</summary>

#### `STRUCT`: **RepositoryIndex** <sub>line 1</sub>
```rust
pub struct RepositoryIndex {
```

#### `IMPL`: **RepositoryIndex** <sub>line 5</sub>
```rust
impl RepositoryIndex {
```

#### `FN`: **find** <sub>line 6</sub>
```rust
pub fn find(&self, name: &str) -> Option<&PackageManifest> { todo!() }
```

#### `FN`: **find_by_kind** <sub>line 7</sub>
```rust
pub fn find_by_kind(&self, kind: ComponentKind) -> alloc::vec::Vec<&PackageManifest> { todo!() }
```

#### `FN`: **search** <sub>line 8</sub>
```rust
pub fn search(&self, query: &str) -> alloc::vec::Vec<&PackageManifest> { todo!() }
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/repo/manifest.rs</b> (3 items)</summary>

#### `STRUCT`: **PackageManifest** <sub>line 1</sub>
```rust
pub struct PackageManifest {
```

#### `STRUCT`: **Dependency** <sub>line 10</sub>
```rust
pub struct Dependency {
```

#### `STRUCT`: **FileEntry** <sub>line 15</sub>
```rust
pub struct FileEntry {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/update/diff.rs</b> (2 items)</summary>

#### `ENUM`: **UpdateAction** <sub>line 1</sub>
```rust
pub enum UpdateAction {
```

#### `FN`: **diff** <sub>line 7</sub>
```rust
pub fn diff(
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ctrlinstall/update/upgrade.rs</b> (3 items)</summary>

#### `STRUCT`: **UpgradePlan** <sub>line 1</sub>
```rust
pub struct UpgradePlan {
```

#### `FN`: **plan_upgrade** <sub>line 6</sub>
```rust
pub fn plan_upgrade(
```

#### `FN`: **execute_upgrade** <sub>line 13</sub>
```rust
pub fn execute_upgrade(plan: &UpgradePlan, executor: &mut InstallExecutor) -> Result<(), CtrlInstallError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/aut.rs</b> (1 items)</summary>

#### `FN`: **authorize** <sub>line 7</sub>
```rust
pub fn authorize(ring: u8, op: u8) -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/aux.c</b> (4 items)</summary>

#### `FUNCTION`: **dp_aux_sink_init** <sub>line 35</sub>
```c
void dp_aux_sink_init(void)
```

#### `FUNCTION`: **dp_aux_read** <sub>line 48</sub>
```c
bool dp_aux_read(uint32_t addr, void *buf, uint32_t len)
```

#### `FUNCTION`: **dp_aux_write** <sub>line 63</sub>
```c
bool dp_aux_write(uint32_t addr, const void *buf, uint32_t len)
```

#### `FUNCTION`: **dp_aux_read_edid** <sub>line 98</sub>
```c
bool dp_aux_read_edid(void *buf, uint32_t len)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/aux.h</b> (4 items)</summary>

#### `FUNCTION`: **dp_aux_sink_init** <sub>line 17</sub>
```c
void dp_aux_sink_init(void);
```

#### `FUNCTION`: **dp_aux_read** <sub>line 18</sub>
```c
bool dp_aux_read(uint32_t addr, void *buf, uint32_t len);
```

#### `FUNCTION`: **dp_aux_write** <sub>line 19</sub>
```c
bool dp_aux_write(uint32_t addr, const void *buf, uint32_t len);
```

#### `FUNCTION`: **dp_aux_read_edid** <sub>line 20</sub>
```c
bool dp_aux_read_edid(void *buf, uint32_t len);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/bridge.rs</b> (6 items)</summary>

#### `FN`: **dp_ready** <sub>line 5</sub>
```rust
fn dp_ready() -> bool;
```

#### `FN`: **dp_link_info** <sub>line 6</sub>
```rust
fn dp_link_info(rate: *mut u32, lanes: *mut u32);
```

#### `FN`: **dp_mode_at** <sub>line 7</sub>
```rust
fn dp_mode_at(i: u32, id: *mut u32, w: *mut u32,
```

#### `FN`: **dp_mode_set_by_id** <sub>line 9</sub>
```rust
fn dp_mode_set_by_id(id: u32) -> bool;
```

#### `FN`: **dp_submit_fill** <sub>line 10</sub>
```rust
fn dp_submit_fill(color: u32, x: u32, y: u32, w: u32, h: u32) -> u64;
```

#### `FN`: **dp_call** <sub>line 13</sub>
```rust
pub fn dp_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/dp.h</b> (7 items)</summary>

#### `FUNCTION`: **dp_init** <sub>line 10</sub>
```c
bool dp_init(void);
```

#### `FUNCTION`: **dp_ready** <sub>line 11</sub>
```c
bool dp_ready(void);
```

#### `FUNCTION`: **dp_link_info** <sub>line 13</sub>
```c
void dp_link_info(uint32_t *rate_mbps, uint32_t *lanes);
```

#### `FUNCTION`: **dp_mode_count** <sub>line 15</sub>
```c
uint32_t dp_mode_count(void);
```

#### `FUNCTION`: **dp_mode_set_by_id** <sub>line 18</sub>
```c
bool dp_mode_set_by_id(uint32_t id);
```

#### `FUNCTION`: **dp_caps** <sub>line 20</sub>
```c
void dp_caps(uint64_t *fb_phys, uint32_t *w, uint32_t *h, uint32_t *stride);
```

#### `FUNCTION`: **dp_fb_set** <sub>line 21</sub>
```c
bool dp_fb_set(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/init.c</b> (6 items)</summary>

#### `FUNCTION`: **dp_init** <sub>line 11</sub>
```c
bool dp_init(void)
```

#### `FUNCTION`: **dp_ready** <sub>line 38</sub>
```c
bool dp_ready(void)
```

#### `FUNCTION`: **dp_mode_count** <sub>line 43</sub>
```c
uint32_t dp_mode_count(void)
```

#### `FUNCTION`: **dp_mode_set_by_id** <sub>line 63</sub>
```c
bool dp_mode_set_by_id(uint32_t id)
```

#### `FUNCTION`: **dp_caps** <sub>line 75</sub>
```c
void dp_caps(uint64_t *fb_phys, uint32_t *w, uint32_t *h, uint32_t *stride)
```

#### `FUNCTION`: **dp_fb_set** <sub>line 80</sub>
```c
bool dp_fb_set(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/init.rs</b> (4 items)</summary>

#### `FN`: **dp_init** <sub>line 2</sub>
```rust
fn dp_init() -> bool;
```

#### `FN`: **dp_ready** <sub>line 3</sub>
```rust
fn dp_ready() -> bool;
```

#### `FN`: **init** <sub>line 6</sub>
```rust
pub fn init() -> bool {
```

#### `FN`: **ready** <sub>line 10</sub>
```rust
pub fn ready() -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/link.c</b> (2 items)</summary>

#### `FUNCTION`: **dp_link_train** <sub>line 7</sub>
```c
bool dp_link_train(void)
```

#### `FUNCTION`: **dp_link_info** <sub>line 70</sub>
```c
void dp_link_info(uint32_t *rate_mbps, uint32_t *lanes)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/link.h</b> (2 items)</summary>

#### `FUNCTION`: **dp_link_train** <sub>line 7</sub>
```c
bool dp_link_train(void);
```

#### `FUNCTION`: **dp_link_info** <sub>line 8</sub>
```c
void dp_link_info(uint32_t *rate_mbps, uint32_t *lanes);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/operation.c</b> (2 items)</summary>

#### `FUNCTION`: **dp_op_set_fb** <sub>line 8</sub>
```c
void dp_op_set_fb(uint64_t phys, uint32_t ww, uint32_t hh, uint32_t s)
```

#### `FUNCTION`: **dp_op_state** <sub>line 16</sub>
```c
void dp_op_state(uint64_t *phys, uint32_t *ww, uint32_t *hh, uint32_t *s)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/displayport/operation.h</b> (3 items)</summary>

#### `FUNCTION`: **dp_op_set_fb** <sub>line 6</sub>
```c
void dp_op_set_fb(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride);
```

#### `FUNCTION`: **dp_op_state** <sub>line 7</sub>
```c
void dp_op_state(uint64_t *phys, uint32_t *w, uint32_t *h, uint32_t *stride);
```

#### `FUNCTION`: **dp_op_fill** <sub>line 8</sub>
```c
void dp_op_fill(uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/pci/mod.rs</b> (17 items)</summary>

#### `STRUCT`: **PciDev** <sub>line 2</sub>
```rust
pub struct PciDev {
```

#### `FN`: **cfg_addr** <sub>line 18</sub>
```rust
fn cfg_addr(bus: u32, dev: u32, func: u32, off: u32) -> u32 {
```

#### `FN`: **read32** <sub>line 22</sub>
```rust
pub fn read32(d: PciDev, off: u32) -> u32 {
```

#### `FN`: **write32** <sub>line 29</sub>
```rust
pub fn write32(d: PciDev, off: u32, v: u32) {
```

#### `FN`: **port_out** <sub>line 36</sub>
```rust
pub fn port_out(port: u16, val: u32, size: u32) {
```

#### `FN`: **port_in** <sub>line 46</sub>
```rust
pub fn port_in(port: u16, size: u32) -> u32 {
```

#### `FN`: **read16** <sub>line 68</sub>
```rust
pub fn read16(d: PciDev, off: u32) -> u16 {
```

#### `FN`: **read8** <sub>line 72</sub>
```rust
pub fn read8(d: PciDev, off: u32) -> u8 {
```

#### `IMPL`: **PciDev** <sub>line 76</sub>
```rust
impl PciDev {
```

#### `FN`: **vendor** <sub>line 77</sub>
```rust
pub fn vendor(self) -> u16 {
```

#### `FN`: **device_id** <sub>line 81</sub>
```rust
pub fn device_id(self) -> u16 {
```

#### `FN`: **class** <sub>line 85</sub>
```rust
pub fn class(self) -> u8 {
```

#### `FN`: **subclass** <sub>line 89</sub>
```rust
pub fn subclass(self) -> u8 {
```

#### `FN`: **prog_if** <sub>line 93</sub>
```rust
pub fn prog_if(self) -> u8 {
```

#### `FN`: **bar** <sub>line 97</sub>
```rust
pub fn bar(self, idx: u32) -> u64 {
```

#### `FN`: **enable_mmio** <sub>line 108</sub>
```rust
pub fn enable_mmio(self) {
```

#### `FN`: **find_class** <sub>line 114</sub>
```rust
pub fn find_class(class: u8, subclass: u8, prog_if: u8) -> Option<PciDev> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/class/hid/keyboard.rs</b> (3 items)</summary>

#### `FN`: **key_to_ascii** <sub>line 18</sub>
```rust
pub fn key_to_ascii(key: u8, shift: bool) -> Option<u8> {
```

#### `FN`: **push_char** <sub>line 44</sub>
```rust
pub fn push_char(c: u8) {
```

#### `FN`: **take_char** <sub>line 55</sub>
```rust
pub fn take_char() -> Option<u8> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/class/hid/mod.rs</b> (4 items)</summary>

#### `STRUCT`: **HidKeyboard** <sub>line 16</sub>
```rust
pub struct HidKeyboard {
```

#### `FN`: **submit** <sub>line 26</sub>
```rust
fn submit(x: &mut Xhci, kb: &mut HidKeyboard) {
```

#### `FN`: **attach** <sub>line 31</sub>
```rust
pub fn attach(x: &mut Xhci, dev: &mut UsbDevice) -> Result<bool, UsbError> {
```

#### `FN`: **poll** <sub>line 105</sub>
```rust
pub fn poll(x: &mut Xhci) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/class/hid/report.rs</b> (2 items)</summary>

#### `STRUCT`: **BootReport** <sub>line 1</sub>
```rust
pub struct BootReport {
```

#### `FN`: **parse_boot_keyboard** <sub>line 6</sub>
```rust
pub fn parse_boot_keyboard(report: &[u8]) -> Option<BootReport> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/class/mass/blockdev.rs</b> (6 items)</summary>

#### `IMPL`: **BlockDevice** <sub>line 4</sub>
```rust
impl BlockDevice for UsbMass {
```

#### `FN`: **name** <sub>line 5</sub>
```rust
fn name(&self) -> &'static str {
```

#### `FN`: **block_size** <sub>line 9</sub>
```rust
fn block_size(&self) -> usize {
```

#### `FN`: **block_count** <sub>line 13</sub>
```rust
fn block_count(&self) -> u64 {
```

#### `FN`: **read_block** <sub>line 17</sub>
```rust
fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError> {
```

#### `FN`: **write_block** <sub>line 32</sub>
```rust
fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/class/mass/mod.rs</b> (7 items)</summary>

#### `STRUCT`: **UsbMass** <sub>line 17</sub>
```rust
pub struct UsbMass {
```

#### `IMPL`: **UsbMass** <sub>line 30</sub>
```rust
impl UsbMass {
```

#### `FN`: **bulk_out** <sub>line 31</sub>
```rust
pub fn bulk_out(&mut self, x: &mut Xhci, phys: u64, len: u32) -> Result<(), UsbError> {
```

#### `FN`: **bulk_in** <sub>line 44</sub>
```rust
pub fn bulk_in(&mut self, x: &mut Xhci, phys: u64, len: u32) -> Result<(), UsbError> {
```

#### `FN`: **scsi_cmd** <sub>line 57</sub>
```rust
pub fn scsi_cmd(&mut self, x: &mut Xhci, cdb: &[u8],
```

#### `FN`: **attach** <sub>line 101</sub>
```rust
pub fn attach(x: &mut Xhci, dev: &mut UsbDevice) -> Result<bool, UsbError> {
```

#### `FN`: **with_controller** <sub>line 186</sub>
```rust
pub fn with_controller<F: FnOnce(&mut Xhci) -> R, R>(f: F) -> R {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/class/mass/scsi.rs</b> (3 items)</summary>

#### `FN`: **read_capacity** <sub>line 5</sub>
```rust
pub fn read_capacity(x: &mut Xhci, m: &mut UsbMass) -> Result<(u32, u32), UsbError> {
```

#### `FN`: **read10** <sub>line 20</sub>
```rust
pub fn read10(x: &mut Xhci, m: &mut UsbMass,
```

#### `FN`: **write10** <sub>line 33</sub>
```rust
pub fn write10(x: &mut Xhci, m: &mut UsbMass,
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/class/mod.rs</b> (2 items)</summary>

#### `TRAIT`: **ClassDriver** <sub>line 5</sub>
```rust
pub trait ClassDriver {
```

#### `FN`: **probe** <sub>line 6</sub>
```rust
fn probe(&self, dev: &UsbDevice) -> bool;
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/core/descriptor.rs</b> (5 items)</summary>

#### `STRUCT`: **DeviceDesc** <sub>line 2</sub>
```rust
pub struct DeviceDesc {
```

#### `STRUCT`: **InterfaceDesc** <sub>line 13</sub>
```rust
pub struct InterfaceDesc {
```

#### `STRUCT`: **EndpointDesc** <sub>line 22</sub>
```rust
pub struct EndpointDesc {
```

#### `FN`: **parse_device** <sub>line 29</sub>
```rust
pub fn parse_device(buf: &[u8]) -> Option<DeviceDesc> {
```

#### `FN`: **parse_config** <sub>line 45</sub>
```rust
pub fn parse_config(buf: &[u8],
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/core/device.rs</b> (3 items)</summary>

#### `STRUCT`: **UsbDevice** <sub>line 7</sub>
```rust
pub struct UsbDevice {
```

#### `IMPL`: **UsbDevice** <sub>line 23</sub>
```rust
impl UsbDevice {
```

#### `FN`: **new** <sub>line 24</sub>
```rust
pub fn new(slot: u8, speed: u32, ctx_size: usize) -> Result<Self, UsbError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/core/enumerate.rs</b> (2 items)</summary>

#### `FN`: **kprintf** <sub>line 11</sub>
```rust
fn kprintf(fmt: *const u8, ...);
```

#### `FN`: **enumerate** <sub>line 14</sub>
```rust
pub fn enumerate(x: &mut Xhci, port: u32) -> Result<UsbDevice, UsbError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/core/speed.rs</b> (1 items)</summary>

#### `FN`: **default_ep0_mps** <sub>line 6</sub>
```rust
pub fn default_ep0_mps(speed: u32) -> u16 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/dma.rs</b> (6 items)</summary>

#### `STRUCT`: **DmaBuf** <sub>line 5</sub>
```rust
pub struct DmaBuf {
```

#### `IMPL`: **DmaBuf** <sub>line 11</sub>
```rust
impl DmaBuf {
```

#### `FN`: **new** <sub>line 12</sub>
```rust
pub fn new(len: usize) -> Result<Self, UsbError> {
```

#### `FN`: **zero** <sub>line 31</sub>
```rust
pub fn zero(&mut self) {
```

#### `IMPL`: **Drop** <sub>line 36</sub>
```rust
impl Drop for DmaBuf {
```

#### `FN`: **drop** <sub>line 37</sub>
```rust
fn drop(&mut self) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/host/xhci/context.rs</b> (11 items)</summary>

#### `STRUCT`: **Contexts** <sub>line 4</sub>
```rust
pub struct Contexts {
```

#### `IMPL`: **Contexts** <sub>line 10</sub>
```rust
impl Contexts {
```

#### `FN`: **setup_configure_ep** <sub>line 11</sub>
```rust
pub fn setup_configure_ep(&mut self, speed: u32, port: u32,
```

#### `FN`: **setup_configure_bulk_pair** <sub>line 37</sub>
```rust
pub fn setup_configure_bulk_pair(&mut self, speed: u32, port: u32,
```

#### `IMPL`: **Contexts** <sub>line 70</sub>
```rust
impl Contexts {
```

#### `FN`: **new** <sub>line 71</sub>
```rust
pub fn new(ctx_size: usize) -> Result<Self, UsbError> {
```

#### `FN`: **in_add_drop** <sub>line 80</sub>
```rust
fn in_add_drop(&mut self, add: u32, drop: u32) {
```

#### `FN`: **slot_ptr** <sub>line 89</sub>
```rust
fn slot_ptr(&mut self) -> *mut u32 {
```

#### `FN`: **ep0_ptr** <sub>line 93</sub>
```rust
fn ep0_ptr(&mut self) -> *mut u32 {
```

#### `FN`: **setup_address_device** <sub>line 97</sub>
```rust
pub fn setup_address_device(&mut self, speed: u32, port: u32,
```

#### `FN`: **setup_evaluate_mps** <sub>line 120</sub>
```rust
pub fn setup_evaluate_mps(&mut self, mps: u16) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/host/xhci/control.rs</b> (5 items)</summary>

#### `FN`: **wait_transfer** <sub>line 6</sub>
```rust
pub fn wait_transfer(x: &mut Xhci, slot: u8) -> Result<u8, UsbError> {
```

#### `FN`: **wait_transfer_ep** <sub>line 26</sub>
```rust
pub fn wait_transfer_ep(x: &mut Xhci, slot: u8, ep: u8) -> Result<u8, UsbError> {
```

#### `FN`: **control** <sub>line 46</sub>
```rust
pub fn control(x: &mut Xhci,
```

#### `FN`: **control_in** <sub>line 94</sub>
```rust
pub fn control_in(x: &mut Xhci, dev: &mut UsbDevice,
```

#### `FN`: **control_out** <sub>line 102</sub>
```rust
pub fn control_out(x: &mut Xhci, dev: &mut UsbDevice,
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/host/xhci/event.rs</b> (3 items)</summary>

#### `FN`: **kprintf** <sub>line 5</sub>
```rust
fn kprintf(fmt: *const u8, ...);
```

#### `IMPL`: **Xhci** <sub>line 8</sub>
```rust
impl Xhci {
```

#### `FN`: **drain_events** <sub>line 9</sub>
```rust
pub fn drain_events(&mut self) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/host/xhci/init.rs</b> (9 items)</summary>

#### `FN`: **kprintf** <sub>line 8</sub>
```rust
fn kprintf(fmt: *const u8, ...);
```

#### `STRUCT`: **Xhci** <sub>line 17</sub>
```rust
pub struct Xhci {
```

#### `FN`: **spin_wait** <sub>line 27</sub>
```rust
fn spin_wait<F: Fn() -> bool>(f: F) -> Result<(), UsbError> {
```

#### `FN`: **op_write64** <sub>line 39</sub>
```rust
fn op_write64(regs: &XhciRegs, off: usize, v: u64) {
```

#### `FN`: **init** <sub>line 49</sub>
```rust
pub fn init(regs: XhciRegs) -> Result<Xhci, UsbError> {
```

#### `IMPL`: **Xhci** <sub>line 102</sub>
```rust
impl Xhci {
```

#### `FN`: **command** <sub>line 103</sub>
```rust
pub fn command(&mut self, trb: Trb) -> Result<Trb, UsbError> {
```

#### `FN`: **scan_ports** <sub>line 130</sub>
```rust
pub fn scan_ports(&mut self) {
```

#### `FN`: **attach_port** <sub>line 173</sub>
```rust
fn attach_port(&mut self, p: u32) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/host/xhci/regs.rs</b> (14 items)</summary>

#### `STRUCT`: **XhciRegs** <sub>line 36</sub>
```rust
pub struct XhciRegs {
```

#### `IMPL`: **XhciRegs** <sub>line 48</sub>
```rust
impl XhciRegs {
```

#### `FN`: **new** <sub>line 49</sub>
```rust
pub fn new(phys: u64) -> Result<Self, UsbError> {
```

#### `FN`: **op** <sub>line 85</sub>
```rust
fn op(&self) -> *mut u8 {
```

#### `FN`: **rt** <sub>line 89</sub>
```rust
fn rt(&self) -> *mut u8 {
```

#### `FN`: **db** <sub>line 93</sub>
```rust
fn db(&self) -> *mut u8 {
```

#### `FN`: **op_read** <sub>line 97</sub>
```rust
pub fn op_read(&self, off: usize) -> u32 {
```

#### `FN`: **op_write** <sub>line 101</sub>
```rust
pub fn op_write(&self, off: usize, v: u32) {
```

#### `FN`: **rt_read** <sub>line 105</sub>
```rust
pub fn rt_read(&self, off: usize) -> u32 {
```

#### `FN`: **rt_write** <sub>line 109</sub>
```rust
pub fn rt_write(&self, off: usize, v: u32) {
```

#### `FN`: **doorbell** <sub>line 113</sub>
```rust
pub fn doorbell(&self, slot: u32, target: u32, task: u32) {
```

#### `FN`: **port_sc** <sub>line 120</sub>
```rust
pub fn port_sc(&self, port: u32) -> u32 {
```

#### `FN`: **port_sc_write** <sub>line 124</sub>
```rust
pub fn port_sc_write(&self, port: u32, v: u32) {
```

#### `FN`: **port_speed** <sub>line 128</sub>
```rust
pub fn port_speed(&self, port: u32) -> u32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/host/xhci/ring.rs</b> (17 items)</summary>

#### `STRUCT`: **CmdRing** <sub>line 5</sub>
```rust
pub struct CmdRing {
```

#### `IMPL`: **CmdRing** <sub>line 12</sub>
```rust
impl CmdRing {
```

#### `FN`: **new** <sub>line 13</sub>
```rust
pub fn new(count: usize) -> Result<Self, UsbError> {
```

#### `FN`: **phys** <sub>line 24</sub>
```rust
pub fn phys(&self) -> u64 {
```

#### `FN`: **enqueue** <sub>line 28</sub>
```rust
pub fn enqueue(&mut self, mut trb: Trb) {
```

#### `STRUCT`: **EventRing** <sub>line 50</sub>
```rust
pub struct EventRing {
```

#### `STRUCT`: **TransferRing** <sub>line 58</sub>
```rust
pub struct TransferRing {
```

#### `IMPL`: **TransferRing** <sub>line 65</sub>
```rust
impl TransferRing {
```

#### `FN`: **new** <sub>line 66</sub>
```rust
pub fn new(count: usize) -> Result<Self, UsbError> {
```

#### `FN`: **phys** <sub>line 77</sub>
```rust
pub fn phys(&self) -> u64 {
```

#### `FN`: **enqueue** <sub>line 81</sub>
```rust
pub fn enqueue(&mut self, mut trb: Trb) {
```

#### `IMPL`: **EventRing** <sub>line 101</sub>
```rust
impl EventRing {
```

#### `FN`: **new** <sub>line 102</sub>
```rust
pub fn new(count: usize) -> Result<Self, UsbError> {
```

#### `FN`: **erst_phys** <sub>line 115</sub>
```rust
pub fn erst_phys(&self) -> u64 {
```

#### `FN`: **pending** <sub>line 119</sub>
```rust
pub fn pending(&self) -> Option<Trb> {
```

#### `FN`: **pop** <sub>line 131</sub>
```rust
pub fn pop(&mut self) {
```

#### `FN`: **erdp** <sub>line 140</sub>
```rust
pub fn erdp(&self) -> u64 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/host/xhci/trb.rs</b> (19 items)</summary>

#### `STRUCT`: **Trb** <sub>line 29</sub>
```rust
pub struct Trb {
```

#### `IMPL`: **Trb** <sub>line 35</sub>
```rust
impl Trb {
```

#### `FN`: **typ** <sub>line 36</sub>
```rust
pub fn typ(&self) -> u32 {
```

#### `FN`: **cycle** <sub>line 40</sub>
```rust
pub fn cycle(&self) -> bool {
```

#### `FN`: **completion_code** <sub>line 44</sub>
```rust
pub fn completion_code(&self) -> u8 {
```

#### `FN`: **slot_id** <sub>line 48</sub>
```rust
pub fn slot_id(&self) -> u8 {
```

#### `FN`: **ep_id** <sub>line 52</sub>
```rust
pub fn ep_id(&self) -> u8 {
```

#### `FN`: **transfer_len** <sub>line 56</sub>
```rust
pub fn transfer_len(&self) -> u32 {
```

#### `FN`: **link** <sub>line 60</sub>
```rust
pub fn link(addr: u64) -> Self {
```

#### `FN`: **enable_slot** <sub>line 68</sub>
```rust
pub fn enable_slot() -> Self {
```

#### `FN`: **address_device** <sub>line 72</sub>
```rust
pub fn address_device(slot: u8, ctx_phys: u64) -> Self {
```

#### `FN`: **configure_ep** <sub>line 80</sub>
```rust
pub fn configure_ep(slot: u8, ctx_phys: u64) -> Self {
```

#### `FN`: **noop_cmd** <sub>line 88</sub>
```rust
pub fn noop_cmd() -> Self {
```

#### `FN`: **setup_stage** <sub>line 92</sub>
```rust
pub fn setup_stage(raw_setup: u64, trt: u32) -> Self {
```

#### `FN`: **data_stage** <sub>line 100</sub>
```rust
pub fn data_stage(addr: u64, len: u32, dir_in: bool) -> Self {
```

#### `FN`: **status_stage** <sub>line 108</sub>
```rust
pub fn status_stage(dir_in: bool) -> Self {
```

#### `FN`: **normal** <sub>line 116</sub>
```rust
pub fn normal(addr: u64, len: u32) -> Self {
```

#### `FN`: **evaluate_ctx** <sub>line 124</sub>
```rust
pub fn evaluate_ctx(slot: u8, ctx_phys: u64) -> Self {
```

#### `FN`: **pack_setup** <sub>line 133</sub>
```rust
pub fn pack_setup(bm_request: u8, b_request: u8, value: u16,
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/mod.rs</b> (4 items)</summary>

#### `ENUM`: **UsbError** <sub>line 10</sub>
```rust
pub enum UsbError {
```

#### `FN`: **init** <sub>line 24</sub>
```rust
pub fn init() -> Result<(), UsbError> {
```

#### `FN`: **poll** <sub>line 61</sub>
```rust
pub fn poll() {
```

#### `FN`: **self_test** <sub>line 67</sub>
```rust
pub fn self_test() -> TestResult {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/drivers/usb/pci_glue.rs</b> (2 items)</summary>

#### `STRUCT`: **XhciPci** <sub>line 8</sub>
```rust
pub struct XhciPci {
```

#### `FN`: **find_xhci** <sub>line 13</sub>
```rust
pub fn find_xhci() -> Result<XhciPci, UsbError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/driverspaceinit/abi/abi.rs</b> (3 items)</summary>

#### `ENUM`: **DsCmd** <sub>line 9</sub>
```rust
pub enum DsCmd {
```

#### `STRUCT`: **DsMsg** <sub>line 38</sub>
```rust
pub struct DsMsg {
```

#### `STRUCT`: **DsRing** <sub>line 53</sub>
```rust
pub struct DsRing {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/driverspaceinit/abi/src.rs</b> (7 items)</summary>

#### `FN`: **ring_bytes** <sub>line 3</sub>
```rust
pub fn ring_bytes(cap: u64) -> usize {
```

#### `STRUCT`: **RingView** <sub>line 7</sub>
```rust
pub struct RingView {
```

#### `IMPL`: **RingView** <sub>line 11</sub>
```rust
impl RingView {
```

#### `FN`: **hdr** <sub>line 16</sub>
```rust
fn hdr(&self) -> *mut DsRing {
```

#### `FN`: **init** <sub>line 20</sub>
```rust
pub fn init(&self, cap: u64) {
```

#### `FN`: **push** <sub>line 28</sub>
```rust
pub fn push(&self, msg: &DsMsg) -> bool {
```

#### `FN`: **pop** <sub>line 49</sub>
```rust
pub fn pop(&self) -> Option<DsMsg> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/driverspaceinit/init/enter.rs</b> (1 items)</summary>

#### `FN`: **enter** <sub>line 61</sub>
```rust
pub fn enter() -> Result<(), DsError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/driverspaceinit/init/init.rs</b> (10 items)</summary>

#### `STRUCT`: **Driverspace** <sub>line 9</sub>
```rust
pub struct Driverspace {
```

#### `FN`: **kv** <sub>line 20</sub>
```rust
fn kv(phys: u64) -> *mut u8 {
```

#### `FN`: **prepare** <sub>line 24</sub>
```rust
pub fn prepare() -> Result<(), DsError> {
```

#### `FN`: **self_test** <sub>line 91</sub>
```rust
pub fn self_test() -> Result<(), DsError> {
```

#### `FN`: **ready** <sub>line 134</sub>
```rust
pub fn ready() -> bool {
```

#### `FN`: **k2d_view** <sub>line 138</sub>
```rust
pub fn k2d_view() -> Option<RingView> {
```

#### `FN`: **d2k_view** <sub>line 142</sub>
```rust
pub fn d2k_view() -> Option<RingView> {
```

#### `FN`: **scratch_view** <sub>line 146</sub>
```rust
pub fn scratch_view() -> Option<*mut u8> {
```

#### `FN`: **map_into_ds** <sub>line 150</sub>
```rust
pub fn map_into_ds(va: u64, phys: u64, len: usize, prot: space::ProtFlags) -> bool {
```

#### `FN`: **unmap_from_ds** <sub>line 159</sub>
```rust
pub fn unmap_from_ds(va: u64, len: usize) -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/driverspaceinit/init/initabi.rs</b> (2 items)</summary>

#### `STRUCT`: **DsSwitch** <sub>line 7</sub>
```rust
pub struct DsSwitch {
```

#### `STRUCT`: **DsInitParams** <sub>line 47</sub>
```rust
pub struct DsInitParams {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/driverspaceinit/init/initcommand.rs</b> (5 items)</summary>

#### `ENUM`: **DsError** <sub>line 5</sub>
```rust
pub enum DsError {
```

#### `STRUCT`: **InitHandshake** <sub>line 14</sub>
```rust
pub struct InitHandshake {
```

#### `IMPL`: **InitHandshake** <sub>line 20</sub>
```rust
impl InitHandshake {
```

#### `FN`: **send** <sub>line 21</sub>
```rust
pub fn send(&mut self, cmd: DsCmd,
```

#### `FN`: **run** <sub>line 60</sub>
```rust
pub fn run(&mut self, params_va: u64) -> Result<(), DsError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/driverspaceinit/init/service.rs</b> (15 items)</summary>

#### `FN`: **kprintf** <sub>line 10</sub>
```rust
fn kprintf(fmt: *const u8, ...);
```

#### `STRUCT`: **Grant** <sub>line 17</sub>
```rust
struct Grant {
```

#### `FN`: **ac97_bars** <sub>line 33</sub>
```rust
fn ac97_bars() -> (u64, u64) {
```

#### `FN`: **grant_add** <sub>line 48</sub>
```rust
fn grant_add(va: u64, phys: u64, pages: u64, kind: u8) -> bool {
```

#### `STRUCT`: **Slot** <sub>line 65</sub>
```rust
struct Slot {
```

#### `FN`: **arch_phys_to_virt** <sub>line 73</sub>
```rust
fn arch_phys_to_virt(phys: u64) -> *mut u8;
```

#### `FN`: **ring** <sub>line 76</sub>
```rust
fn ring() -> Option<*mut Slot> {
```

#### `FN`: **vgpu_call** <sub>line 86</sub>
```rust
pub fn vgpu_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring_lvl: u8) -> i32 {
```

#### `FN`: **pci_call** <sub>line 155</sub>
```rust
pub fn pci_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8) -> i32 {
```

#### `FN`: **grant_take** <sub>line 222</sub>
```rust
fn grant_take(va: u64) -> Option<Grant> {
```

#### `FN`: **grant_phys** <sub>line 235</sub>
```rust
fn grant_phys(va: u64) -> Option<(u64, u64)> {
```

#### `FN`: **video_call** <sub>line 247</sub>
```rust
pub fn video_call(op: u32, m: &DsMsg, r: &mut DsMsg) -> i32 {
```

#### `FN`: **handle** <sub>line 271</sub>
```rust
fn handle(m: &DsMsg, r: &mut DsMsg) -> i32 {
```

#### `FN`: **poll** <sub>line 430</sub>
```rust
pub fn poll() {
```

#### `FN`: **post_event** <sub>line 459</sub>
```rust
pub fn post_event(cmd: DsCmd, a0: u64, a1: u64) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ds_ipc/buffer.rs</b> (32 items)</summary>

#### `ENUM`: **BufferTransferMode** <sub>line 24</sub>
```rust
pub enum BufferTransferMode {
```

#### `ENUM`: **BufferError** <sub>line 32</sub>
```rust
pub enum BufferError {
```

#### `IMPL`: **From** <sub>line 44</sub>
```rust
impl From<BufferError> for DsError {
```

#### `FN`: **from** <sub>line 45</sub>
```rust
fn from(e: BufferError) -> Self {
```

#### `STRUCT`: **BufferFlags** <sub>line 56</sub>
```rust
pub struct BufferFlags: u32 {
```

#### `STRUCT`: **IpcBufferDescriptor** <sub>line 66</sub>
```rust
pub struct IpcBufferDescriptor {
```

#### `STRUCT`: **IpcBufferSlot** <sub>line 74</sub>
```rust
pub struct IpcBufferSlot {
```

#### `IMPL`: **IpcBufferSlot** <sub>line 83</sub>
```rust
impl IpcBufferSlot {
```

#### `FN`: **new** <sub>line 89</sub>
```rust
pub fn new(id: u64, owner: u64) -> Self {
```

#### `FN`: **is_free** <sub>line 106</sub>
```rust
pub fn is_free(&self) -> bool {
```

#### `FN`: **mark_in_transfer** <sub>line 110</sub>
```rust
pub fn mark_in_transfer(&self) -> bool {
```

#### `FN`: **release** <sub>line 119</sub>
```rust
pub fn release(&self) {
```

#### `STRUCT`: **IpcBufferPool** <sub>line 124</sub>
```rust
pub struct IpcBufferPool {
```

#### `IMPL`: **IpcBufferPool** <sub>line 129</sub>
```rust
impl IpcBufferPool {
```

#### `FN`: **allocate_slot** <sub>line 137</sub>
```rust
pub fn allocate_slot(&self, owner_task_id: u64) -> Result<u64, BufferError> {
```

#### `FN`: **register_buffer** <sub>line 150</sub>
```rust
pub fn register_buffer(
```

#### `FN`: **get_slot** <sub>line 192</sub>
```rust
pub fn get_slot(&self, slot_id: u64) -> Option<core::sync::atomic::AtomicPtr<()>> {
```

#### `FN`: **with_slot** <sub>line 196</sub>
```rust
pub fn with_slot<F, R>(&self, slot_id: u64, f: F) -> Option<R>
```

#### `STRUCT`: **TransferContext** <sub>line 206</sub>
```rust
pub struct TransferContext<'a> {
```

#### `FN`: **determine_mode** <sub>line 216</sub>
```rust
pub fn determine_mode(&self) -> BufferTransferMode {
```

#### `FN`: **can_do_zero_copy** <sub>line 231</sub>
```rust
fn can_do_zero_copy(&self) -> bool {
```

#### `FN`: **execute_transfer** <sub>line 235</sub>
```rust
pub fn execute_transfer(&mut self) -> Result<(), BufferError> {
```

#### `FN`: **transfer_via_registers** <sub>line 243</sub>
```rust
fn transfer_via_registers(&mut self) -> Result<(), BufferError> {
```

#### `FN`: **transfer_zero_copy** <sub>line 256</sub>
```rust
fn transfer_zero_copy(&mut self) -> Result<(), BufferError> {
```

#### `FN`: **transfer_via_bounce** <sub>line 344</sub>
```rust
fn transfer_via_bounce(&mut self) -> Result<(), BufferError> {
```

#### `FN`: **revoke_ipc_buffer_capabilities** <sub>line 498</sub>
```rust
pub fn revoke_ipc_buffer_capabilities(cnode: &mut CNode, buffer_slot_id: u64) {
```

#### `FN`: **cleanup_task_ipc_buffers** <sub>line 508</sub>
```rust
pub fn cleanup_task_ipc_buffers(pool: &IpcBufferPool, task_id: u64) {
```

#### `FN`: **arch_flush_tlb_single** <sub>line 522</sub>
```rust
fn arch_flush_tlb_single(task_id: u64, vaddr: VirtAddr) {
```

#### `FN`: **arch_clean_invalidate_dcache_region** <sub>line 534</sub>
```rust
fn arch_clean_invalidate_dcache_region(vaddr: VirtAddr, size: u64) {
```

#### `FN`: **sys_ipc_buffer_register** <sub>line 551</sub>
```rust
pub fn sys_ipc_buffer_register(
```

#### `FN`: **sys_ipc_buffer_unregister** <sub>line 582</sub>
```rust
pub fn sys_ipc_buffer_unregister(
```

#### `FN`: **test_buffer_alignment_check** <sub>line 609</sub>
```rust
fn test_buffer_alignment_check() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ds_ipc/endpoint.rs</b> (8 items)</summary>

#### `STRUCT`: **Endpoint** <sub>line 8</sub>
```rust
pub struct Endpoint {
```

#### `STRUCT`: **EndpointState** <sub>line 14</sub>
```rust
struct EndpointState {
```

#### `IMPL`: **Endpoint** <sub>line 22</sub>
```rust
impl Endpoint {
```

#### `FN`: **new** <sub>line 23</sub>
```rust
pub fn new(badge: u64) -> Self {
```

#### `FN`: **enqueue_sender** <sub>line 34</sub>
```rust
pub fn enqueue_sender(&self, task: *mut Task) {
```

#### `FN`: **enqueue_receiver** <sub>line 39</sub>
```rust
pub fn enqueue_receiver(&self, task: *mut Task) {
```

#### `FN`: **dequeue_sender** <sub>line 44</sub>
```rust
pub fn dequeue_sender(&self) -> Option<*mut Task> {
```

#### `FN`: **dequeue_receiver** <sub>line 49</sub>
```rust
pub fn dequeue_receiver(&self) -> Option<*mut Task> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ds_ipc/msg.rs</b> (5 items)</summary>

#### `STRUCT`: **MessageInfo** <sub>line 5</sub>
```rust
pub struct MessageInfo {
```

#### `IMPL`: **MessageInfo** <sub>line 13</sub>
```rust
impl MessageInfo {
```

#### `STRUCT`: **MessageRegisters** <sub>line 27</sub>
```rust
pub struct MessageRegisters {
```

#### `IMPL`: **MessageRegisters** <sub>line 31</sub>
```rust
impl MessageRegisters {
```

#### `STRUCT`: **IpcMessage** <sub>line 39</sub>
```rust
pub struct IpcMessage {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/ds_ipc/syscall.rs</b> (8 items)</summary>

#### `ENUM`: **IpcSyscall** <sub>line 8</sub>
```rust
pub enum IpcSyscall {
```

#### `FN`: **sys_ipc_call** <sub>line 15</sub>
```rust
pub fn sys_ipc_call(
```

#### `FN`: **sys_ipc_recv** <sub>line 43</sub>
```rust
pub fn sys_ipc_recv(ep_cap_idx: u64) -> Result<IpcMessage, DsError> {
```

#### `FN`: **transfer_message** <sub>line 60</sub>
```rust
fn transfer_message(_target: *mut Task, _info: MessageInfo, _regs: &MessageRegisters) {
```

#### `FN`: **extract_message** <sub>line 64</sub>
```rust
fn extract_message(_sender: *mut Task) -> IpcMessage {
```

#### `FN`: **block_current_task** <sub>line 72</sub>
```rust
fn block_current_task() {}
```

#### `FN`: **wake_task** <sub>line 73</sub>
```rust
fn wake_task(_task: *mut Task) {}
```

#### `FN`: **switch_context** <sub>line 74</sub>
```rust
fn switch_context() {}
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/editor/editor.c</b> (22 items)</summary>

#### `FUNCTION`: **console_set_enabled** <sub>line 11</sub>
```c
extern void console_set_enabled(int on);
```

#### `FUNCTION`: **k_fs_read** <sub>line 12</sub>
```c
extern int32_t k_fs_read(const char *path, void *buf, uint32_t cap);
```

#### `FUNCTION`: **k_input_keycode** <sub>line 13</sub>
```c
extern uint32_t k_input_keycode(void);
```

#### `FUNCTION`: **ed_strlen** <sub>line 60</sub>
```c
static size_t ed_strlen(const char *s)
```

#### `FUNCTION`: **ed_fill** <sub>line 67</sub>
```c
static void ed_fill(uint32_t x, uint32_t y, uint32_t w, uint32_t h, uint32_t c)
```

#### `FUNCTION`: **ed_char** <sub>line 78</sub>
```c
static void ed_char(uint32_t x, uint32_t y, char ch, uint32_t color)
```

#### `FUNCTION`: **ed_text** <sub>line 98</sub>
```c
static void ed_text(uint32_t x, uint32_t y, const char *s, uint32_t color)
```

#### `FUNCTION`: **ed_num** <sub>line 107</sub>
```c
static void ed_num(char *buf, int v)
```

#### `FUNCTION`: **log_append** <sub>line 123</sub>
```c
static void log_append(const char *s)
```

#### `FUNCTION`: **tok_color** <sub>line 170</sub>
```c
static uint32_t tok_color(tok_kind_t k)
```

#### `FUNCTION`: **draw_hl** <sub>line 187</sub>
```c
static void draw_hl(int row, uint32_t x, uint32_t y)
```

#### `FUNCTION`: **draw_mouse** <sub>line 272</sub>
```c
static void draw_mouse(void)
```

#### `FUNCTION`: **render** <sub>line 294</sub>
```c
static void render(void)
```

#### `FUNCTION`: **ins_char** <sub>line 366</sub>
```c
static void ins_char(char ch)
```

#### `FUNCTION`: **do_enter** <sub>line 381</sub>
```c
static void do_enter(void)
```

#### `FUNCTION`: **do_backspace** <sub>line 409</sub>
```c
static void do_backspace(void)
```

#### `FUNCTION`: **do_delete** <sub>line 443</sub>
```c
static void do_delete(void)
```

#### `FUNCTION`: **clamp_scroll** <sub>line 455</sub>
```c
static void clamp_scroll(int rows)
```

#### `FUNCTION`: **flatten** <sub>line 464</sub>
```c
static int flatten(void)
```

#### `FUNCTION`: **ed_compile_run** <sub>line 480</sub>
```c
static void ed_compile_run(void)
```

#### `FUNCTION`: **editor_run** <sub>line 530</sub>
```c
int editor_run(const char *path)
```

#### `FUNCTION`: **volatile** <sub>line 660</sub>
```c
__asm__ volatile("pause");
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/editor/editor.h</b> (1 items)</summary>

#### `FUNCTION`: **editor_run** <sub>line 21</sub>
```c
int editor_run(const char *path);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/editor/mouse.c</b> (10 items)</summary>

#### `FUNCTION`: **outb** <sub>line 3</sub>
```c
static inline void outb(uint16_t port, uint8_t val)
```

#### `FUNCTION`: **volatile** <sub>line 5</sub>
```c
__asm__ volatile("outb %0, %1" :: "a"(val), "Nd"(port));
```

#### `FUNCTION`: **inb** <sub>line 8</sub>
```c
static inline uint8_t inb(uint16_t port)
```

#### `FUNCTION`: **volatile** <sub>line 11</sub>
```c
__asm__ volatile("inb %1, %0" : "=a"(v) : "Nd"(port));
```

#### `FUNCTION`: **wait_write** <sub>line 15</sub>
```c
static void wait_write(void)
```

#### `FUNCTION`: **wait_read** <sub>line 20</sub>
```c
static void wait_read(void)
```

#### `FUNCTION`: **mouse_cmd** <sub>line 25</sub>
```c
static void mouse_cmd(uint8_t cmd)
```

#### `FUNCTION`: **mouse_rate** <sub>line 32</sub>
```c
static void mouse_rate(uint8_t r)
```

#### `FUNCTION`: **mouse_init** <sub>line 43</sub>
```c
void mouse_init(void)
```

#### `FUNCTION`: **mouse_poll** <sub>line 56</sub>
```c
int mouse_poll(int *dx, int *dy, int *dz, int *buttons)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/editor/mouse.h</b> (2 items)</summary>

#### `FUNCTION`: **mouse_init** <sub>line 6</sub>
```c
void mouse_init(void);
```

#### `FUNCTION`: **mouse_poll** <sub>line 8</sub>
```c
int mouse_poll(int *dx, int *dy, int *dz, int *buttons);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/disc.rs</b> (11 items)</summary>

#### `ENUM`: **DiscError** <sub>line 4</sub>
```rust
pub enum DiscError {
```

#### `TYPE`: **Result** <sub>line 13</sub>
```rust
pub type Result<T> = core::result::Result<T, DiscError>;
```

#### `TRAIT`: **BlockDevice** <sub>line 17</sub>
```rust
pub trait BlockDevice {
```

#### `FN`: **block_size** <sub>line 18</sub>
```rust
fn block_size(&self) -> u64;
```

#### `FN`: **read_block** <sub>line 20</sub>
```rust
fn read_block(&mut self, lba: u64, buf: &[u8]) -> Result<()>;
```

#### `FN`: **write_block** <sub>line 22</sub>
```rust
fn write_block(&mut self, lba: u64, buf: &mut [ u8]) -> Result<()>{
```

#### `FN`: **flush** <sub>line 27</sub>
```rust
fn flush(&mut self) -> Result<()> {
```

#### `ENUM`: **PartitionTableKind** <sub>line 33</sub>
```rust
pub enum PartitionTableKind {
```

#### `STRUCT`: **Partition** <sub>line 40</sub>
```rust
pub struct Partition {
```

#### `IMPL`: **Partition** <sub>line 48</sub>
```rust
impl Partition {
```

#### `FN`: **end_lba** <sub>line 49</sub>
```rust
pub fn end_lba(&self) -> Option<u64> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/driver/ata_pio.rs</b> (22 items)</summary>

#### `STRUCT`: **AtaPio** <sub>line 26</sub>
```rust
pub struct AtaPio {
```

#### `IMPL`: **AtaPio** <sub>line 37</sub>
```rust
impl AtaPio {
```

#### `FN`: **drive_lba** <sub>line 58</sub>
```rust
fn drive_lba(&self) -> u8 {
```

#### `FN`: **drive_identify** <sub>line 62</sub>
```rust
fn drive_identify(&self) -> u8 {
```

#### `FN`: **is_present** <sub>line 66</sub>
```rust
pub fn is_present(&self) -> bool {
```

#### `FN`: **disable_irq** <sub>line 70</sub>
```rust
fn disable_irq(&self) {
```

#### `FN`: **status** <sub>line 74</sub>
```rust
fn status(&self) -> u8 {
```

#### `FN`: **wait_not_bsy** <sub>line 78</sub>
```rust
fn wait_not_bsy(&self) -> Result<(), DriverError> {
```

#### `FN`: **wait_drq** <sub>line 88</sub>
```rust
fn wait_drq(&self) -> Result<(), DriverError> {
```

#### `FN`: **wait_ready** <sub>line 104</sub>
```rust
fn wait_ready(&self) -> Result<(), DriverError> {
```

#### `FN`: **setup_lba28** <sub>line 116</sub>
```rust
fn setup_lba28(&self, lba: u32, count: u8, cmd: u8) -> Result<(), DriverError> {
```

#### `FN`: **identify** <sub>line 129</sub>
```rust
pub fn identify(&self) -> Result<[u16; 256], DriverError> {
```

#### `FN`: **read_sector** <sub>line 162</sub>
```rust
fn read_sector(&self, lba: u32, buf: &mut [u8]) -> Result<(), DriverError> {
```

#### `FN`: **write_sector** <sub>line 177</sub>
```rust
fn write_sector(&self, lba: u32, buf: &[u8]) -> Result<(), DriverError> {
```

#### `IMPL`: **BlockDevice** <sub>line 195</sub>
```rust
impl BlockDevice for AtaPio {
```

#### `FN`: **name** <sub>line 196</sub>
```rust
fn name(&self) -> &'static str {
```

#### `FN`: **block_size** <sub>line 200</sub>
```rust
fn block_size(&self) -> usize {
```

#### `FN`: **block_count** <sub>line 204</sub>
```rust
fn block_count(&self) -> u64 {
```

#### `FN`: **read_block** <sub>line 208</sub>
```rust
fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError> {
```

#### `FN`: **write_block** <sub>line 224</sub>
```rust
fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError> {
```

#### `FN`: **probe_dev** <sub>line 241</sub>
```rust
fn probe_dev(dev: &AtaPio) -> bool {
```

#### `FN`: **probe** <sub>line 253</sub>
```rust
pub fn probe() -> usize {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/driver/block.rs</b> (9 items)</summary>

#### `ENUM`: **DriverError** <sub>line 2</sub>
```rust
pub enum DriverError {
```

#### `TRAIT`: **BlockDevice** <sub>line 11</sub>
```rust
pub trait BlockDevice {
```

#### `FN`: **name** <sub>line 12</sub>
```rust
fn name(&self) -> &'static str;
```

#### `FN`: **block_size** <sub>line 13</sub>
```rust
fn block_size(&self) -> usize;
```

#### `FN`: **block_count** <sub>line 14</sub>
```rust
fn block_count(&self) -> u64;
```

#### `FN`: **read_block** <sub>line 16</sub>
```rust
fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError>;
```

#### `FN`: **write_block** <sub>line 17</sub>
```rust
fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError>;
```

#### `FN`: **read_blocks** <sub>line 19</sub>
```rust
fn read_blocks(&self, start: u64, buf: &mut [u8]) -> Result<(), DriverError> {
```

#### `FN`: **write_blocks** <sub>line 36</sub>
```rust
fn write_blocks(&self, start: u64, buf: &[u8]) -> Result<(), DriverError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/driver/lock.rs</b> (5 items)</summary>

#### `STRUCT`: **IrqGuard** <sub>line 3</sub>
```rust
pub struct IrqGuard {
```

#### `IMPL`: **IrqGuard** <sub>line 7</sub>
```rust
impl IrqGuard {
```

#### `FN`: **lock** <sub>line 8</sub>
```rust
pub fn lock() -> Self {
```

#### `IMPL`: **Drop** <sub>line 19</sub>
```rust
impl Drop for IrqGuard {
```

#### `FN`: **drop** <sub>line 20</sub>
```rust
fn drop(&mut self) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/driver/mod.rs</b> (2 items)</summary>

#### `FN`: **init** <sub>line 7</sub>
```rust
pub fn init() {
```

#### `FN`: **self_test** <sub>line 17</sub>
```rust
pub fn self_test() -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/driver/registry.rs</b> (4 items)</summary>

#### `FN`: **register** <sub>line 7</sub>
```rust
pub fn register(dev: &'static dyn BlockDevice) -> bool {
```

#### `FN`: **get** <sub>line 22</sub>
```rust
pub fn get(index: usize) -> Option<&'static dyn BlockDevice> {
```

#### `FN`: **first** <sub>line 28</sub>
```rust
pub fn first() -> Option<&'static dyn BlockDevice> {
```

#### `FN`: **count** <sub>line 32</sub>
```rust
pub fn count() -> usize {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/ext4/mod.rs</b> (20 items)</summary>

#### `ENUM`: **ExtError** <sub>line 6</sub>
```rust
pub enum ExtError {
```

#### `STRUCT`: **Superblock** <sub>line 15</sub>
```rust
pub struct Superblock {
```

#### `STRUCT`: **RawInode** <sub>line 27</sub>
```rust
pub struct RawInode {
```

#### `STRUCT`: **DirEntry** <sub>line 33</sub>
```rust
pub struct DirEntry {
```

#### `STRUCT`: **Ext4** <sub>line 39</sub>
```rust
pub struct Ext4 {
```

#### `FN`: **u16le** <sub>line 49</sub>
```rust
fn u16le(b: &[u8], o: usize) -> u16 {
```

#### `FN`: **u32le** <sub>line 53</sub>
```rust
fn u32le(b: &[u8], o: usize) -> u32 {
```

#### `FN`: **u64le** <sub>line 57</sub>
```rust
fn u64le(b: &[u8], o: usize) -> u64 {
```

#### `IMPL`: **Ext4** <sub>line 64</sub>
```rust
impl Ext4 {
```

#### `FN`: **mount** <sub>line 65</sub>
```rust
pub fn mount(disk: &'static dyn BlockDevice) -> Result<Self, ExtError> {
```

#### `FN`: **read_at** <sub>line 107</sub>
```rust
fn read_at(disk: &'static dyn BlockDevice,
```

#### `FN`: **read_blk** <sub>line 129</sub>
```rust
pub fn read_blk(&self, blk: u64) -> Result<Vec<u8>, ExtError> {
```

#### `FN`: **inode_table** <sub>line 138</sub>
```rust
fn inode_table(&self, ino: u32) -> Result<u64, ExtError> {
```

#### `FN`: **read_inode** <sub>line 156</sub>
```rust
pub fn read_inode(&self, ino: u32) -> Result<RawInode, ExtError> {
```

#### `FN`: **block_map** <sub>line 173</sub>
```rust
fn block_map(&self, inode: &RawInode, file_blk: u32) -> Option<u64> {
```

#### `FN`: **read_file** <sub>line 227</sub>
```rust
pub fn read_file(&self, inode: &RawInode,
```

#### `FN`: **read_dir** <sub>line 262</sub>
```rust
pub fn read_dir(&self, inode: &RawInode) -> Result<Vec<DirEntry>, ExtError> {
```

#### `FN`: **resolve** <sub>line 297</sub>
```rust
pub fn resolve(&self, path: &str) -> Result<RawInode, ExtError> {
```

#### `FN`: **read_path** <sub>line 332</sub>
```rust
pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Result<usize, ExtError> {
```

#### `FN`: **list_path** <sub>line 337</sub>
```rust
pub fn list_path(&self, path: &str) -> Result<Vec<DirEntry>, ExtError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/fat32/mod.rs</b> (18 items)</summary>

#### `ENUM`: **FatError** <sub>line 6</sub>
```rust
pub enum FatError {
```

#### `STRUCT`: **Fat32** <sub>line 14</sub>
```rust
pub struct Fat32 {
```

#### `STRUCT`: **FatEntry** <sub>line 23</sub>
```rust
pub struct FatEntry {
```

#### `FN`: **u16le** <sub>line 30</sub>
```rust
fn u16le(b: &[u8], o: usize) -> u16 {
```

#### `FN`: **u32le** <sub>line 34</sub>
```rust
fn u32le(b: &[u8], o: usize) -> u32 {
```

#### `IMPL`: **Fat32** <sub>line 38</sub>
```rust
impl Fat32 {
```

#### `FN`: **mount** <sub>line 39</sub>
```rust
pub fn mount(disk: &'static dyn BlockDevice) -> Result<Self, FatError> {
```

#### `FN`: **read_at** <sub>line 65</sub>
```rust
fn read_at(&self, off: u64, buf: &mut [u8]) -> Result<(), FatError> {
```

#### `FN`: **cluster_off** <sub>line 85</sub>
```rust
fn cluster_off(&self, clus: u32) -> u64 {
```

#### `FN`: **read_cluster** <sub>line 89</sub>
```rust
fn read_cluster(&self, clus: u32) -> Result<Vec<u8>, FatError> {
```

#### `FN`: **next_cluster** <sub>line 95</sub>
```rust
fn next_cluster(&self, clus: u32) -> Option<u32> {
```

#### `FN`: **short_name** <sub>line 113</sub>
```rust
fn short_name(raw: &[u8]) -> Vec<u8> {
```

#### `FN`: **lfn_char** <sub>line 138</sub>
```rust
fn lfn_char(c: u16) -> u8 {
```

#### `FN`: **read_dir_cluster** <sub>line 142</sub>
```rust
pub fn read_dir_cluster(&self, start: u32) -> Result<Vec<FatEntry>, FatError> {
```

#### `FN`: **resolve** <sub>line 226</sub>
```rust
pub fn resolve(&self, path: &str) -> Result<FatEntry, FatError> {
```

#### `FN`: **read_file** <sub>line 264</sub>
```rust
pub fn read_file(&self, entry: &FatEntry,
```

#### `FN`: **read_path** <sub>line 290</sub>
```rust
pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Result<usize, FatError> {
```

#### `FN`: **list_path** <sub>line 295</sub>
```rust
pub fn list_path(&self, path: &str) -> Result<Vec<FatEntry>, FatError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/mbr.rs</b> (12 items)</summary>

#### `FN`: **kprintf** <sub>line 5</sub>
```rust
fn kprintf(fmt: *const u8, ...);
```

#### `STRUCT`: **MbrEntry** <sub>line 9</sub>
```rust
pub struct MbrEntry {
```

#### `STRUCT`: **Partition** <sub>line 16</sub>
```rust
pub struct Partition {
```

#### `IMPL`: **BlockDevice** <sub>line 27</sub>
```rust
impl BlockDevice for Partition {
```

#### `FN`: **name** <sub>line 28</sub>
```rust
fn name(&self) -> &'static str {
```

#### `FN`: **block_size** <sub>line 32</sub>
```rust
fn block_size(&self) -> usize {
```

#### `FN`: **block_count** <sub>line 36</sub>
```rust
fn block_count(&self) -> u64 {
```

#### `FN`: **read_block** <sub>line 40</sub>
```rust
fn read_block(&self, block: u64, buf: &mut [u8]) -> Result<(), DriverError> {
```

#### `FN`: **write_block** <sub>line 48</sub>
```rust
fn write_block(&self, block: u64, buf: &[u8]) -> Result<(), DriverError> {
```

#### `FN`: **parse_mbr** <sub>line 57</sub>
```rust
pub fn parse_mbr(buf: &[u8]) -> Option<[MbrEntry; 4]> {
```

#### `FN`: **probe_disk** <sub>line 80</sub>
```rust
pub fn probe_disk(d: &'static dyn BlockDevice) -> usize {
```

#### `FN`: **init** <sub>line 138</sub>
```rust
pub fn init() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/mod.rs</b> (4 items)</summary>

#### `FN`: **init** <sub>line 9</sub>
```rust
pub fn init() {
```

#### `FN`: **self_test** <sub>line 14</sub>
```rust
pub fn self_test() -> TestResult {
```

#### `FN`: **ensure_formatted** <sub>line 57</sub>
```rust
pub fn ensure_formatted(dev: &dyn BlockDevice) -> Result<()> {
```

#### `FN`: **root_device** <sub>line 113</sub>
```rust
pub fn root_device() -> Option<&'static dyn driver::block::BlockDevice> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/tangfs/btree.rs</b> (6 items)</summary>

#### `STRUCT`: **BtreeNode** <sub>line 9</sub>
```rust
pub struct BtreeNode {
```

#### `FN`: **lookup_dir_entry** <sub>line 21</sub>
```rust
pub fn lookup_dir_entry(fs: &TangFs, dir_ino: u64, name: &str) -> Result<u64> {
```

#### `FN`: **search_leaf_node** <sub>line 46</sub>
```rust
fn search_leaf_node(node: &BtreeNode, name: &str) -> Result<u64> {
```

#### `FN`: **find_child_in_internal** <sub>line 67</sub>
```rust
fn find_child_in_internal(node: &BtreeNode, name: &str) -> Result<u64> {
```

#### `FN`: **read_dir_entries** <sub>line 91</sub>
```rust
pub fn read_dir_entries(fs: &TangFs, dir_ino: u64) -> Result<Vec<DirEntry>> {
```

#### `FN`: **collect_leaf_entries** <sub>line 134</sub>
```rust
fn collect_leaf_entries(node: &BtreeNode, entries: &mut Vec<DirEntry>) -> Result<()> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/tangfs/dir.rs</b> (1 items)</summary>

#### `STRUCT`: **DirEntry** <sub>line 2</sub>
```rust
pub struct DirEntry {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/tangfs/extern.rs</b> (2 items)</summary>

#### `FN`: **allocate_block** <sub>line 4</sub>
```rust
pub fn allocate_block(fs: &TangFs) -> Result<u64> {
```

#### `FN`: **free_block** <sub>line 35</sub>
```rust
pub fn free_block(fs: &TangFs, block: u64) -> Result<()> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/tangfs/file.rs</b> (3 items)</summary>

#### `FN`: **read_file** <sub>line 4</sub>
```rust
pub fn read_file(fs: &TangFs, inode: &Inode, mut offset: u64, buf: &mut [u8]) -> Result<usize> {
```

#### `FN`: **write_file** <sub>line 50</sub>
```rust
pub fn write_file(fs: &TangFs, inode: &mut Inode, mut offset: u64, data: &[u8]) -> Result<usize> {
```

#### `FN`: **allocate_block_for_inode** <sub>line 85</sub>
```rust
fn allocate_block_for_inode(fs: &TangFs, inode: &mut Inode, logical_block: u64) -> Result<u64> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/tangfs/inode.rs</b> (12 items)</summary>

#### `STRUCT`: **Inode** <sub>line 7</sub>
```rust
pub struct Inode {
```

#### `STRUCT`: **Extent** <sub>line 28</sub>
```rust
pub struct Extent {
```

#### `STRUCT`: **InodeHandle** <sub>line 35</sub>
```rust
pub struct InodeHandle {
```

#### `IMPL`: **InodeHandle** <sub>line 40</sub>
```rust
impl InodeHandle {
```

#### `FN`: **load_inode** <sub>line 41</sub>
```rust
fn load_inode(&self) -> Result<Inode> {
```

#### `FN`: **save_inode** <sub>line 55</sub>
```rust
fn save_inode(&self, inode: &Inode) -> Result<()> {
```

#### `IMPL`: **VfsInode** <sub>line 75</sub>
```rust
impl VfsInode for InodeHandle {
```

#### `FN`: **lookup** <sub>line 76</sub>
```rust
fn lookup(&self, name: &str) -> Result<Box<dyn VfsInode>> {
```

#### `FN`: **readdir** <sub>line 95</sub>
```rust
fn readdir(&self) -> Result<Vec<DirEntry>> {
```

#### `FN`: **read** <sub>line 105</sub>
```rust
fn read(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
```

#### `FN`: **write** <sub>line 109</sub>
```rust
fn write(&self, offset: u64, data: &[u8]) -> Result<usize> {
```

#### `FN`: **stat** <sub>line 116</sub>
```rust
fn stat(&self) -> Result<crate::fs::vfs::Stat> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/tangfs/journal.rs</b> (5 items)</summary>

#### `STRUCT`: **Journal** <sub>line 5</sub>
```rust
pub struct Journal {
```

#### `IMPL`: **Journal** <sub>line 12</sub>
```rust
impl Journal {
```

#### `FN`: **open** <sub>line 13</sub>
```rust
pub fn open(device: &'static dyn crate::fs::driver::BlockDevice, sb: &super::Superblock) -> Result<Self> {
```

#### `FN`: **write_block** <sub>line 22</sub>
```rust
pub fn write_block(&mut self, block: u64, data: &[u8]) -> Result<()> {
```

#### `FN`: **replay** <sub>line 39</sub>
```rust
pub fn replay(&self) -> Result<()> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/tangfs/mod.rs</b> (8 items)</summary>

#### `STRUCT`: **TangFs** <sub>line 18</sub>
```rust
pub struct TangFs {
```

#### `IMPL`: **TangFs** <sub>line 25</sub>
```rust
impl TangFs {
```

#### `FN`: **mount** <sub>line 26</sub>
```rust
pub fn mount(device: &'static dyn crate::fs::driver::BlockDevice) -> Result<Self> {
```

#### `FN`: **read_block** <sub>line 49</sub>
```rust
pub fn read_block(&self, block: u64) -> Result<Vec<u8>> {
```

#### `FN`: **write_block** <sub>line 63</sub>
```rust
pub fn write_block(&self, block: u64, data: &[u8]) -> Result<()> {
```

#### `IMPL`: **FileSystem** <sub>line 80</sub>
```rust
impl FileSystem for TangFs {
```

#### `FN`: **root_inode** <sub>line 81</sub>
```rust
fn root_inode(&self) -> Box<dyn VfsInode> {
```

#### `FN`: **statfs** <sub>line 89</sub>
```rust
fn statfs(&self) -> Result<crate::fs::vfs::StatFs> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/tangfs/superblock.rs</b> (8 items)</summary>

#### `STRUCT`: **Superblock** <sub>line 5</sub>
```rust
pub struct Superblock {
```

#### `IMPL`: **Superblock** <sub>line 25</sub>
```rust
impl Superblock {
```

#### `FN`: **read** <sub>line 26</sub>
```rust
pub fn read(device: &dyn BlockDevice) -> Result<Self> {
```

#### `FN`: **write** <sub>line 45</sub>
```rust
pub fn write(&self, device: &dyn BlockDevice) -> Result<()> {
```

#### `FN`: **calculate_checksum** <sub>line 55</sub>
```rust
fn calculate_checksum(sb: &Superblock) -> u32 {
```

#### `FN`: **generate_uuid** <sub>line 77</sub>
```rust
pub fn generate_uuid() -> [u8; 16] {
```

#### `IMPL`: **Clone** <sub>line 88</sub>
```rust
impl Clone for Superblock {
```

#### `FN`: **clone** <sub>line 89</sub>
```rust
fn clone(&self) -> Self {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/tfs.rs</b> (20 items)</summary>

#### `ENUM`: **FsError** <sub>line 19</sub>
```rust
pub enum FsError {
```

#### `TYPE`: **Result** <sub>line 30</sub>
```rust
pub type Result<T> = core::result::Result<T, FsError>;
```

#### `STRUCT`: **Superblock** <sub>line 33</sub>
```rust
pub struct Superblock {
```

#### `FN`: **rd32** <sub>line 39</sub>
```rust
fn rd32(b: &[u8], off: usize) -> u32 {
```

#### `FN`: **wr32** <sub>line 43</sub>
```rust
fn wr32(b: &mut [u8], off: usize, v: u32) {
```

#### `FN`: **format** <sub>line 48</sub>
```rust
pub fn format(dev: &dyn BlockDevice) -> Result<()> {
```

#### `FN`: **read_superblock** <sub>line 70</sub>
```rust
pub fn read_superblock(dev: &dyn BlockDevice) -> Result<Superblock> {
```

#### `FN`: **write_superblock** <sub>line 85</sub>
```rust
pub fn write_superblock(dev: &dyn BlockDevice, sb: &Superblock) -> Result<()> {
```

#### `STRUCT`: **DirEntry** <sub>line 98</sub>
```rust
struct DirEntry {
```

#### `FN`: **read_entry** <sub>line 112</sub>
```rust
fn read_entry(block: &[u8], idx: usize) -> DirEntry {
```

#### `FN`: **write_entry** <sub>line 124</sub>
```rust
fn write_entry(block: &mut [u8], idx: usize, e: &DirEntry) {
```

#### `FN`: **entry_name** <sub>line 132</sub>
```rust
fn entry_name(e: &DirEntry) -> &str {
```

#### `FN`: **find_entry** <sub>line 137</sub>
```rust
fn find_entry(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<(usize, DirEntry)> {
```

#### `FN`: **list_dir** <sub>line 156</sub>
```rust
pub fn list_dir(dev: &dyn BlockDevice, dir: u32, out: &mut impl Write) -> Result<()> {
```

#### `FN`: **write_file** <sub>line 186</sub>
```rust
pub fn write_file(dev: &dyn BlockDevice, dir: u32, name: &str, data: &[u8]) -> Result<()> {
```

#### `FN`: **remove** <sub>line 246</sub>
```rust
pub fn remove(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<()> {
```

#### `FN`: **mkdir** <sub>line 274</sub>
```rust
pub fn mkdir(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<()> {
```

#### `FN`: **find_dir** <sub>line 325</sub>
```rust
pub fn find_dir(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<u32> {
```

#### `FN`: **entries** <sub>line 333</sub>
```rust
pub fn entries(dev: &dyn BlockDevice, dir: u32) -> Result<alloc::vec::Vec<(alloc::string::String, u32, u8)>> {
```

#### `FN`: **read_file** <sub>line 352</sub>
```rust
pub fn read_file(dev: &dyn BlockDevice, dir: u32, name: &str) -> Result<alloc::vec::Vec<u8>> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/fs/vfs.rs</b> (7 items)</summary>

#### `STRUCT`: **FsEntry** <sub>line 6</sub>
```rust
pub struct FsEntry {
```

#### `ENUM`: **Mounted** <sub>line 12</sub>
```rust
pub enum Mounted {
```

#### `IMPL`: **Mounted** <sub>line 18</sub>
```rust
impl Mounted {
```

#### `FN`: **read_path** <sub>line 19</sub>
```rust
pub fn read_path(&self, path: &str, buf: &mut [u8]) -> Option<usize> {
```

#### `FN`: **list_path** <sub>line 27</sub>
```rust
pub fn list_path(&self, path: &str) -> Option<Vec<FsEntry>> {
```

#### `FN`: **mount_all** <sub>line 56</sub>
```rust
pub fn mount_all() {
```

#### `FN`: **root** <sub>line 74</sub>
```rust
pub fn root() -> Option<&'static Mounted> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/gdt.rs</b> (2 items)</summary>

#### `STRUCT`: **Selectors** <sub>line 18</sub>
```rust
struct Selectors {
```

#### `FN`: **init** <sub>line 52</sub>
```rust
pub fn init() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/gfx/console.rs</b> (15 items)</summary>

#### `FN`: **fb** <sub>line 28</sub>
```rust
fn fb() -> &'static mut Framebuffer {
```

#### `FN`: **cols** <sub>line 32</sub>
```rust
pub fn cols() -> usize {
```

#### `FN`: **rows** <sub>line 36</sub>
```rust
pub fn rows() -> usize {
```

#### `FN`: **fb_info** <sub>line 40</sub>
```rust
pub fn fb_info() -> (u32, u32, u32, u64) {
```

#### `FN`: **framebuffer_info** <sub>line 54</sub>
```rust
fn framebuffer_info() -> Option<(u32, u32, u32, u64, bool)> {
```

#### `FN`: **set_enabled** <sub>line 79</sub>
```rust
pub fn set_enabled(enabled: bool) {
```

#### `FN`: **set_palette_rgb332** <sub>line 90</sub>
```rust
fn set_palette_rgb332() {
```

#### `FN`: **set_palette16** <sub>line 111</sub>
```rust
fn set_palette16() {
```

#### `FN`: **disable_text_cursor** <sub>line 127</sub>
```rust
fn disable_text_cursor() {
```

#### `FN`: **delay** <sub>line 139</sub>
```rust
fn delay() {
```

#### `FN`: **test_fill** <sub>line 145</sub>
```rust
pub fn test_fill(r: u32, g: u32, b: u32) -> bool {
```

#### `FN`: **resync_background** <sub>line 164</sub>
```rust
pub fn resync_background() -> bool {
```

#### `FN`: **init** <sub>line 188</sub>
```rust
pub fn init(fb_addr: u64, width: u32, height: u32, stride: u32, format: PixelFormat) -> bool {
```

#### `FN`: **refresh** <sub>line 313</sub>
```rust
pub fn refresh() {
```

#### `FN`: **draw_cell** <sub>line 363</sub>
```rust
fn draw_cell(row: usize, col: usize, ch: u8, attr: u8) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/gfx/framebuffer.rs</b> (16 items)</summary>

#### `ENUM`: **PixelFormat** <sub>line 8</sub>
```rust
pub enum PixelFormat {
```

#### `STRUCT`: **Framebuffer** <sub>line 21</sub>
```rust
pub struct Framebuffer {
```

#### `FN`: **rgb332_index** <sub>line 29</sub>
```rust
fn rgb332_index(r: u32, g: u32, b: u32) -> u8 {
```

#### `FN`: **rgb332_from_index** <sub>line 33</sub>
```rust
fn rgb332_from_index(idx: u8) -> (u32, u32, u32) {
```

#### `FN`: **rgb_to_index4** <sub>line 40</sub>
```rust
fn rgb_to_index4(r: u32, g: u32, b: u32) -> u8 {
```

#### `FN`: **index4_to_rgb** <sub>line 56</sub>
```rust
fn index4_to_rgb(idx: u8) -> (u32, u32, u32) {
```

#### `IMPL`: **Framebuffer** <sub>line 60</sub>
```rust
impl Framebuffer {
```

#### `FN`: **ry** <sub>line 61</sub>
```rust
fn ry(&self, y: usize) -> usize {
```

#### `FN`: **rx** <sub>line 65</sub>
```rust
fn rx(&self, x: usize) -> usize {
```

#### `FN`: **get** <sub>line 69</sub>
```rust
pub fn get(&self, x: usize, y: usize) -> u32 {
```

#### `FN`: **set** <sub>line 85</sub>
```rust
pub fn set(&mut self, x: usize, y: usize, c: u32) {
```

#### `FN`: **offset** <sub>line 107</sub>
```rust
pub fn offset(&self, x: usize, y: usize) -> usize {
```

#### `FN`: **add** <sub>line 111</sub>
```rust
pub fn add(&mut self, x: usize, y: usize, r: u32, g: u32, b: u32) {
```

#### `FN`: **planar_set** <sub>line 119</sub>
```rust
fn planar_set(&self, x: usize, y: usize, color: u8) {
```

#### `FN`: **planar_get** <sub>line 134</sub>
```rust
fn planar_get(&self, x: usize, y: usize) -> u8 {
```

#### `FN`: **rgb** <sub>line 153</sub>
```rust
pub fn rgb(r: u32, g: u32, b: u32) -> u32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/gfx/galaxy.rs</b> (7 items)</summary>

#### `FN`: **hash2** <sub>line 6</sub>
```rust
fn hash2(x: i64, y: i64) -> u32 {
```

#### `FN`: **vnoise** <sub>line 14</sub>
```rust
fn vnoise(xq: i64, yq: i64) -> i64 {
```

#### `FN`: **fbm** <sub>line 34</sub>
```rust
fn fbm(xq: i64, yq: i64) -> i64 {
```

#### `FN`: **band_fall** <sub>line 42</sub>
```rust
fn band_fall(x: i64, y: i64, w: i64, h: i64) -> i64 {
```

#### `FN`: **green_fall** <sub>line 60</sub>
```rust
fn green_fall(x: i64, y: i64, w: i64, _h: i64) -> i64 {
```

#### `FN`: **px_add** <sub>line 83</sub>
```rust
fn px_add(fb: &mut Framebuffer, x: i64, y: i64,
```

#### `FN`: **render** <sub>line 96</sub>
```rust
pub fn render(fb: &mut Framebuffer, t: u32) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/gfx/mod.rs</b> (7 items)</summary>

#### `FN`: **init** <sub>line 16</sub>
```rust
pub fn init() -> bool {
```

#### `FN`: **init_mode** <sub>line 25</sub>
```rust
pub fn init_mode(fb_phys: u64, width: u32, height: u32, stride: u32) -> bool {
```

#### `FN`: **set_resolution** <sub>line 30</sub>
```rust
pub fn set_resolution(mode: vga::VideoMode) -> bool {
```

#### `FN`: **set_resolution_w_h** <sub>line 47</sub>
```rust
pub fn set_resolution_w_h(width: u32, height: u32) -> bool {
```

#### `FN`: **current_resolution** <sub>line 73</sub>
```rust
pub fn current_resolution() -> (u32, u32) {
```

#### `FN`: **refresh** <sub>line 77</sub>
```rust
pub fn refresh() {
```

#### `FN`: **self_test** <sub>line 81</sub>
```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/gfx/panic_screen.rs</b> (1 items)</summary>

#### `FN`: **show** <sub>line 11</sub>
```rust
pub fn show(info: &PanicInfo) -> ! {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/gfx/vga.rs</b> (9 items)</summary>

#### `ENUM`: **VideoMode** <sub>line 4</sub>
```rust
pub enum VideoMode {
```

#### `FN`: **write_regs** <sub>line 9</sub>
```rust
fn write_regs(misc: u8, seq: &[u8], crtc: &[u8], gfx: &[u8], attr: &[u8]) {
```

#### `FN`: **set_mode** <sub>line 47</sub>
```rust
pub fn set_mode(mode: VideoMode) {
```

#### `FN`: **dispi_write** <sub>line 91</sub>
```rust
fn dispi_write(index: u16, value: u16) {
```

#### `FN`: **dispi_read** <sub>line 98</sub>
```rust
fn dispi_read(index: u16) -> u16 {
```

#### `FN`: **bochs_version** <sub>line 105</sub>
```rust
pub fn bochs_version() -> Option<u16> {
```

#### `FN`: **bochs_disable** <sub>line 114</sub>
```rust
pub fn bochs_disable() {
```

#### `FN`: **bochs_set_mode** <sub>line 120</sub>
```rust
pub fn bochs_set_mode(width: u32, height: u32, bpp: u32) -> bool {
```

#### `FN`: **bochs_lfb_base** <sub>line 139</sub>
```rust
pub fn bochs_lfb_base() -> Option<u64> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/aut.rs</b> (2 items)</summary>

#### `FN`: **authorize** <sub>line 18</sub>
```rust
pub fn authorize(ring: u8, op: u32) -> bool {
```

#### `FN`: **tick_reset** <sub>line 46</sub>
```rust
pub fn tick_reset() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/bridge.rs</b> (13 items)</summary>

#### `FN`: **hdmi_iface_acquire** <sub>line 5</sub>
```rust
fn hdmi_iface_acquire(owner: u32) -> bool;
```

#### `FN`: **hdmi_iface_release** <sub>line 6</sub>
```rust
fn hdmi_iface_release(owner: u32) -> bool;
```

#### `FN`: **hdmi_iface_init** <sub>line 8</sub>
```rust
fn hdmi_iface_init(owner: u32, fb_phys: u64, w: u32, h: u32, stride: u32) -> bool;
```

#### `FN`: **hdmi_iface_ready** <sub>line 9</sub>
```rust
fn hdmi_iface_ready() -> bool;
```

#### `FN`: **hdmi_iface_mode_set** <sub>line 11</sub>
```rust
fn hdmi_iface_mode_set(owner: u32, id: u32) -> bool;
```

#### `FN`: **hdmi_iface_mode_current** <sub>line 12</sub>
```rust
fn hdmi_iface_mode_current(id: *mut u32, w: *mut u32, h: *mut u32, r: *mut u32) -> bool;
```

#### `FN`: **hdmi_iface_mode_at** <sub>line 13</sub>
```rust
fn hdmi_iface_mode_at(i: u32, id: *mut u32, w: *mut u32, h: *mut u32, r: *mut u32) -> bool;
```

#### `FN`: **hdmi_iface_submit_fill** <sub>line 15</sub>
```rust
fn hdmi_iface_submit_fill(owner: u32, color: u32, x: u32, y: u32, w: u32, h: u32) -> u64;
```

#### `FN`: **hdmi_iface_poll** <sub>line 16</sub>
```rust
fn hdmi_iface_poll(owner: u32, out: *mut u64) -> bool;
```

#### `FN`: **hdmi_iface_caps** <sub>line 18</sub>
```rust
fn hdmi_iface_caps(w: *mut u32, h: *mut u32, s: *mut u32, phys: *mut u64);
```

#### `FN`: **hdmi_iface_fb_grant** <sub>line 20</sub>
```rust
fn hdmi_iface_fb_grant(owner: u32, phys: *mut u64, w: *mut u32, h: *mut u32, s: *mut u32) -> bool;
```

#### `FN`: **hdmi_iface_fb_revoke** <sub>line 21</sub>
```rust
fn hdmi_iface_fb_revoke(owner: u32) -> bool;
```

#### `FN`: **hdmi_call** <sub>line 24</sub>
```rust
pub fn hdmi_call(op: u32, m: &DsMsg, r: &mut DsMsg, ring: u8, owner: u32) -> i32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/hdmi.h</b> (14 items)</summary>

#### `FUNCTION`: **hdmi_init_with** <sub>line 41</sub>
```c
bool hdmi_init_with(uint64_t fb_phys, uint32_t w, uint32_t h, uint32_t stride);
```

#### `FUNCTION`: **hdmi_ready** <sub>line 42</sub>
```c
bool hdmi_ready(void);
```

#### `FUNCTION`: **hdmi_caps** <sub>line 43</sub>
```c
void hdmi_caps(hdmi_caps_t *out);
```

#### `FUNCTION`: **hdmi_fb_set** <sub>line 44</sub>
```c
bool hdmi_fb_set(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride);
```

#### `FUNCTION`: **hdmi_mode_set_by_id** <sub>line 45</sub>
```c
bool hdmi_mode_set_by_id(uint32_t id);
```

#### `FUNCTION`: **hdmi_mode_current_raw** <sub>line 46</sub>
```c
bool hdmi_mode_current_raw(uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r);
```

#### `FUNCTION`: **hdmi_submit** <sub>line 47</sub>
```c
uint64_t hdmi_submit(const hdmi_transfer_t *t);
```

#### `FUNCTION`: **hdmi_poll** <sub>line 48</sub>
```c
bool hdmi_poll(uint64_t *out_seq);
```

#### `FUNCTION`: **hdmi_pending** <sub>line 49</sub>
```c
uint32_t hdmi_pending(void);
```

#### `FUNCTION`: **hdmi_submit_fill** <sub>line 50</sub>
```c
uint64_t hdmi_submit_fill(uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h);
```

#### `FUNCTION`: **hdmi_caps_raw** <sub>line 51</sub>
```c
void hdmi_caps_raw(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys);
```

#### `FUNCTION`: **hdmi_fb_grant** <sub>line 53</sub>
```c
bool hdmi_fb_grant(void);
```

#### `FUNCTION`: **hdmi_fb_revoke** <sub>line 54</sub>
```c
void hdmi_fb_revoke(void);
```

#### `FUNCTION`: **hdmi_fb_granted** <sub>line 55</sub>
```c
bool hdmi_fb_granted(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/init.c</b> (7 items)</summary>

#### `FUNCTION`: **hdmi_init_with** <sub>line 6</sub>
```c
bool hdmi_init_with(uint64_t fb_phys, uint32_t w, uint32_t h, uint32_t stride)
```

#### `FUNCTION`: **hdmi_ready** <sub>line 30</sub>
```c
bool hdmi_ready(void)
```

#### `FUNCTION`: **hdmi_caps** <sub>line 35</sub>
```c
void hdmi_caps(hdmi_caps_t *out)
```

#### `FUNCTION`: **hdmi_fb_set** <sub>line 40</sub>
```c
bool hdmi_fb_set(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride)
```

#### `FUNCTION`: **hdmi_mode_set_by_id** <sub>line 51</sub>
```c
bool hdmi_mode_set_by_id(uint32_t id)
```

#### `FUNCTION`: **hdmi_mode_apply** <sub>line 64</sub>
```c
return hdmi_mode_apply(mode);
```

#### `FUNCTION`: **hdmi_mode_current_raw** <sub>line 67</sub>
```c
bool hdmi_mode_current_raw(uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/init.rs</b> (4 items)</summary>

#### `FN`: **hdmi_init_with** <sub>line 2</sub>
```rust
fn hdmi_init_with(fb_phys: u64, w: u32, h: u32, stride: u32) -> bool;
```

#### `FN`: **hdmi_ready** <sub>line 3</sub>
```rust
fn hdmi_ready() -> bool;
```

#### `FN`: **init** <sub>line 6</sub>
```rust
pub fn init() -> bool {
```

#### `FN`: **ready** <sub>line 11</sub>
```rust
pub fn ready() -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/interface.c</b> (22 items)</summary>

#### `FUNCTION`: **hdmi_iface_acquire** <sub>line 7</sub>
```c
bool hdmi_iface_acquire(uint32_t owner)
```

#### `FUNCTION`: **hdmi_iface_release** <sub>line 21</sub>
```c
bool hdmi_iface_release(uint32_t owner)
```

#### `FUNCTION`: **hdmi_iface_owner** <sub>line 27</sub>
```c
uint32_t hdmi_iface_owner(void)
```

#### `FUNCTION`: **atomic_load** <sub>line 29</sub>
```c
return atomic_load(&port_owner);
```

#### `FUNCTION`: **hdmi_iface_check** <sub>line 32</sub>
```c
bool hdmi_iface_check(uint32_t owner)
```

#### `FUNCTION`: **hdmi_iface_init** <sub>line 37</sub>
```c
bool hdmi_iface_init(uint32_t owner, uint64_t fb_phys, uint32_t w, uint32_t h, uint32_t stride)
```

#### `FUNCTION`: **hdmi_init_with** <sub>line 43</sub>
```c
return hdmi_init_with(fb_phys, w, h, stride);
```

#### `FUNCTION`: **hdmi_iface_ready** <sub>line 46</sub>
```c
bool hdmi_iface_ready(void)
```

#### `FUNCTION`: **hdmi_ready** <sub>line 48</sub>
```c
return hdmi_ready();
```

#### `FUNCTION`: **hdmi_iface_mode_set** <sub>line 51</sub>
```c
bool hdmi_iface_mode_set(uint32_t owner, uint32_t id)
```

#### `FUNCTION`: **hdmi_mode_set_by_id** <sub>line 57</sub>
```c
return hdmi_mode_set_by_id(id);
```

#### `FUNCTION`: **hdmi_iface_mode_current** <sub>line 60</sub>
```c
bool hdmi_iface_mode_current(uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r)
```

#### `FUNCTION`: **hdmi_mode_current_raw** <sub>line 62</sub>
```c
return hdmi_mode_current_raw(id, w, h, r);
```

#### `FUNCTION`: **hdmi_iface_mode_at** <sub>line 65</sub>
```c
bool hdmi_iface_mode_at(uint32_t i, uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r)
```

#### `FUNCTION`: **hdmi_mode_at_raw** <sub>line 67</sub>
```c
return hdmi_mode_at_raw(i, id, w, h, r);
```

#### `FUNCTION`: **hdmi_iface_submit_fill** <sub>line 70</sub>
```c
uint64_t hdmi_iface_submit_fill(uint32_t owner, uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h)
```

#### `FUNCTION`: **hdmi_submit_fill** <sub>line 76</sub>
```c
return hdmi_submit_fill(color, x, y, w, h);
```

#### `FUNCTION`: **hdmi_iface_poll** <sub>line 79</sub>
```c
bool hdmi_iface_poll(uint32_t owner, uint64_t *out_seq)
```

#### `FUNCTION`: **hdmi_poll** <sub>line 85</sub>
```c
return hdmi_poll(out_seq);
```

#### `FUNCTION`: **hdmi_iface_caps** <sub>line 88</sub>
```c
void hdmi_iface_caps(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys)
```

#### `FUNCTION`: **hdmi_iface_fb_grant** <sub>line 93</sub>
```c
bool hdmi_iface_fb_grant(uint32_t owner, uint64_t *phys, uint32_t *w, uint32_t *h, uint32_t *s)
```

#### `FUNCTION`: **hdmi_iface_fb_revoke** <sub>line 114</sub>
```c
bool hdmi_iface_fb_revoke(uint32_t owner)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/interface.h</b> (14 items)</summary>

#### `FUNCTION`: **hdmi_iface_acquire** <sub>line 9</sub>
```c
bool hdmi_iface_acquire(uint32_t owner);
```

#### `FUNCTION`: **hdmi_iface_release** <sub>line 10</sub>
```c
bool hdmi_iface_release(uint32_t owner);
```

#### `FUNCTION`: **hdmi_iface_owner** <sub>line 11</sub>
```c
uint32_t hdmi_iface_owner(void);
```

#### `FUNCTION`: **hdmi_iface_check** <sub>line 12</sub>
```c
bool hdmi_iface_check(uint32_t owner);
```

#### `FUNCTION`: **hdmi_iface_init** <sub>line 14</sub>
```c
bool hdmi_iface_init(uint32_t owner, uint64_t fb_phys, uint32_t w, uint32_t h, uint32_t stride);
```

#### `FUNCTION`: **hdmi_iface_ready** <sub>line 15</sub>
```c
bool hdmi_iface_ready(void);
```

#### `FUNCTION`: **hdmi_iface_mode_set** <sub>line 17</sub>
```c
bool hdmi_iface_mode_set(uint32_t owner, uint32_t id);
```

#### `FUNCTION`: **hdmi_iface_mode_current** <sub>line 18</sub>
```c
bool hdmi_iface_mode_current(uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r);
```

#### `FUNCTION`: **hdmi_iface_mode_at** <sub>line 19</sub>
```c
bool hdmi_iface_mode_at(uint32_t i, uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r);
```

#### `FUNCTION`: **hdmi_iface_submit_fill** <sub>line 21</sub>
```c
uint64_t hdmi_iface_submit_fill(uint32_t owner, uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h);
```

#### `FUNCTION`: **hdmi_iface_poll** <sub>line 22</sub>
```c
bool hdmi_iface_poll(uint32_t owner, uint64_t *out_seq);
```

#### `FUNCTION`: **hdmi_iface_caps** <sub>line 24</sub>
```c
void hdmi_iface_caps(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys);
```

#### `FUNCTION`: **hdmi_iface_fb_grant** <sub>line 26</sub>
```c
bool hdmi_iface_fb_grant(uint32_t owner, uint64_t *phys, uint32_t *w, uint32_t *h, uint32_t *s);
```

#### `FUNCTION`: **hdmi_iface_fb_revoke** <sub>line 27</sub>
```c
bool hdmi_iface_fb_revoke(uint32_t owner);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/mode.c</b> (4 items)</summary>

#### `FUNCTION`: **hdmi_mode_count** <sub>line 15</sub>
```c
uint32_t hdmi_mode_count(void)
```

#### `FUNCTION`: **hdmi_mode_valid** <sub>line 51</sub>
```c
bool hdmi_mode_valid(const hdmi_mode_t *m)
```

#### `FUNCTION`: **hdmi_mode_apply** <sub>line 62</sub>
```c
bool hdmi_mode_apply(const hdmi_mode_t *m)
```

#### `FUNCTION`: **hdmi_mode_at_raw** <sub>line 78</sub>
```c
bool hdmi_mode_at_raw(uint32_t i, uint32_t *id, uint32_t *w, uint32_t *h, uint32_t *r)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/mode.h</b> (3 items)</summary>

#### `FUNCTION`: **hdmi_mode_count** <sub>line 23</sub>
```c
uint32_t hdmi_mode_count(void);
```

#### `FUNCTION`: **hdmi_mode_valid** <sub>line 27</sub>
```c
bool hdmi_mode_valid(const hdmi_mode_t *m);
```

#### `FUNCTION`: **hdmi_mode_apply** <sub>line 28</sub>
```c
bool hdmi_mode_apply(const hdmi_mode_t *m);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/operation.c</b> (14 items)</summary>

#### `FUNCTION`: **hdmi_op_state** <sub>line 20</sub>
```c
void hdmi_op_state(hdmi_caps_t *out)
```

#### `FUNCTION`: **hdmi_op_ready** <sub>line 27</sub>
```c
bool hdmi_op_ready(void)
```

#### `FUNCTION`: **hdmi_op_set_fb** <sub>line 32</sub>
```c
void hdmi_op_set_fb(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride)
```

#### `FUNCTION`: **hdmi_op_exec** <sub>line 42</sub>
```c
void hdmi_op_exec(hdmi_transfer_t *t)
```

#### `FUNCTION`: **hdmi_submit** <sub>line 96</sub>
```c
uint64_t hdmi_submit(const hdmi_transfer_t *t)
```

#### `FUNCTION`: **hdmi_poll** <sub>line 125</sub>
```c
bool hdmi_poll(uint64_t *out_seq)
```

#### `FUNCTION`: **hdmi_pending** <sub>line 144</sub>
```c
uint32_t hdmi_pending(void)
```

#### `FUNCTION`: **hdmi_submit_fill** <sub>line 149</sub>
```c
uint64_t hdmi_submit_fill(uint32_t color, uint32_t x, uint32_t y, uint32_t w, uint32_t h)
```

#### `FUNCTION`: **hdmi_submit** <sub>line 159</sub>
```c
return hdmi_submit(&t);
```

#### `FUNCTION`: **hdmi_op_set_mode** <sub>line 162</sub>
```c
void hdmi_op_set_mode(const hdmi_mode_t *m)
```

#### `FUNCTION`: **hdmi_fb_grant** <sub>line 176</sub>
```c
bool hdmi_fb_grant(void)
```

#### `FUNCTION`: **hdmi_fb_revoke** <sub>line 186</sub>
```c
void hdmi_fb_revoke(void)
```

#### `FUNCTION`: **hdmi_fb_granted** <sub>line 191</sub>
```c
bool hdmi_fb_granted(void)
```

#### `FUNCTION`: **hdmi_caps_raw** <sub>line 196</sub>
```c
void hdmi_caps_raw(uint32_t *w, uint32_t *h, uint32_t *s, uint64_t *phys)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/hdmi/operation.h</b> (5 items)</summary>

#### `FUNCTION`: **hdmi_op_set_mode** <sub>line 7</sub>
```c
void hdmi_op_set_mode(const hdmi_mode_t *m);
```

#### `FUNCTION`: **hdmi_op_exec** <sub>line 8</sub>
```c
void hdmi_op_exec(hdmi_transfer_t *t);
```

#### `FUNCTION`: **hdmi_op_state** <sub>line 9</sub>
```c
void hdmi_op_state(hdmi_caps_t *out);
```

#### `FUNCTION`: **hdmi_op_set_fb** <sub>line 10</sub>
```c
void hdmi_op_set_fb(uint64_t phys, uint32_t w, uint32_t h, uint32_t stride);
```

#### `FUNCTION`: **hdmi_op_ready** <sub>line 11</sub>
```c
bool hdmi_op_ready(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/interrupts.rs</b> (5 items)</summary>

#### `ENUM`: **InterruptIndex** <sub>line 50</sub>
```rust
pub enum InterruptIndex {
```

#### `IMPL`: **InterruptIndex** <sub>line 55</sub>
```rust
impl InterruptIndex {
```

#### `FN`: **as_u8** <sub>line 56</sub>
```rust
fn as_u8(self) -> u8 {
```

#### `FN`: **breakpoint_handler** <sub>line 61</sub>
```rust
fn breakpoint_handler(stack_frame: InterruptStackFrame) {
```

#### `FN`: **init_idt** <sub>line 83</sub>
```rust
pub fn init_idt() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/kernel_glue.rs</b> (2 items)</summary>

#### `FN`: **cstr_to_str** <sub>line 3</sub>
```rust
fn cstr_to_str<'a>(p: *const u8) -> Option<&'a str> {
```

#### `FN`: **tfs_read_path** <sub>line 96</sub>
```rust
fn tfs_read_path(dev: &dyn crate::fs::driver::block::BlockDevice, path: &str) -> Option<alloc::vec::Vec<u8>> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/kstd_glue.rs</b> (1 items)</summary>

#### `FN`: **cstr_to_str** <sub>line 3</sub>
```rust
fn cstr_to_str<'a>(p: *const u8) -> Option<&'a str> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/main.rs</b> (5 items)</summary>

#### `FN`: **init** <sub>line 141</sub>
```rust
pub fn init() {
```

#### `FN`: **hlt_loop** <sub>line 145</sub>
```rust
pub fn hlt_loop() -> ! {
```

#### `FN`: **init_permissions** <sub>line 149</sub>
```rust
pub fn init_permissions() {
```

#### `FN`: **kernel_main** <sub>line 162</sub>
```rust
pub fn kernel_main(boot_info: &'static bootloader::BootInfo) -> ! {
```

#### `FN`: **kernel_main_riscv** <sub>line 221</sub>
```rust
pub fn kernel_main_riscv() -> ! {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/api/alloc.c</b> (17 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 8</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **k_memset** <sub>line 10</sub>
```c
static void k_memset(const void *dst, const uint8 value, const size_t n) {
```

#### `FUNCTION`: **k_memcpy** <sub>line 15</sub>
```c
static void k_memcpy(const void *dst, const void *src, const size_t n) {
```

#### `FUNCTION`: **k_strlen** <sub>line 21</sub>
```c
static size_t k_strlen(const char *s) {
```

#### `FUNCTION`: **k_usable_size** <sub>line 27</sub>
```c
static size_t k_usable_size(void *ptr) {
```

#### `FUNCTION`: **heap_usable_size** <sub>line 32</sub>
```c
return heap_usable_size(ptr);
```

#### `FUNCTION`: **dbg_alloc** <sub>line 38</sub>
```c
return dbg_alloc(size);
```

#### `FUNCTION`: **heap_alloc** <sub>line 40</sub>
```c
return heap_alloc(size);
```

#### `FUNCTION`: **kfree** <sub>line 44</sub>
```c
void kfree(void *ptr) {
```

#### `FUNCTION`: **kzalloc** <sub>line 63</sub>
```c
return kzalloc(count * size);
```

#### `FUNCTION`: **heap_alloc_aligned** <sub>line 70</sub>
```c
return heap_alloc_aligned(size, align);
```

#### `FUNCTION`: **kmalloc_usable_size** <sub>line 74</sub>
```c
size_t kmalloc_usable_size(void *ptr) { return k_usable_size(ptr); }
```

#### `FUNCTION`: **heap_alloc** <sub>line 95</sub>
```c
return heap_alloc(pages * ARCH_PAGE_SIZE);
```

#### `FUNCTION`: **kfree_pages** <sub>line 98</sub>
```c
void kfree_pages(void *ptr, const size_t pages) {
```

#### `FUNCTION`: **kvirt_to_phys** <sub>line 118</sub>
```c
uint64_t kvirt_to_phys(void *ptr) {
```

#### `FUNCTION`: **arch_virt_to_phys** <sub>line 123</sub>
```c
return arch_virt_to_phys(ptr);
```

#### `FUNCTION`: **kalloc_dump** <sub>line 126</sub>
```c
void kalloc_dump(void) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/api/alloc.h</b> (5 items)</summary>

#### `FUNCTION`: **kfree** <sub>line 13</sub>
```c
void kfree(void *ptr);
```

#### `FUNCTION`: **kmalloc_usable_size** <sub>line 15</sub>
```c
size_t kmalloc_usable_size(void *ptr);
```

#### `FUNCTION`: **kfree_pages** <sub>line 18</sub>
```c
void kfree_pages(void *ptr, size_t pages);
```

#### `FUNCTION`: **kvirt_to_phys** <sub>line 22</sub>
```c
uint64_t kvirt_to_phys(void *ptr);
```

#### `FUNCTION`: **kalloc_dump** <sub>line 24</sub>
```c
void kalloc_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/debug/alloc_debug.c</b> (6 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 11</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **dbg_tail_canary** <sub>line 25</sub>
```c
static uint64_t dbg_tail_canary(size_t size) { return ~((uint64_t)size) ^ DBG_MAGIC; }
```

#### `FUNCTION`: **dbg_free** <sub>line 56</sub>
```c
void dbg_free(void *ptr) {
```

#### `FUNCTION`: **dbg_usable_size** <sub>line 90</sub>
```c
size_t dbg_usable_size(void *ptr) {
```

#### `FUNCTION`: **dbg_verify** <sub>line 97</sub>
```c
bool dbg_verify(void *ptr) {
```

#### `FUNCTION`: **mm_debug_dump** <sub>line 104</sub>
```c
void mm_debug_dump(void) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/debug/alloc_debug.h</b> (4 items)</summary>

#### `FUNCTION`: **dbg_free** <sub>line 9</sub>
```c
void dbg_free(void *ptr);
```

#### `FUNCTION`: **dbg_verify** <sub>line 11</sub>
```c
bool dbg_verify(void *ptr);
```

#### `FUNCTION`: **dbg_usable_size** <sub>line 12</sub>
```c
size_t dbg_usable_size(void *ptr);
```

#### `FUNCTION`: **mm_debug_dump** <sub>line 13</sub>
```c
void mm_debug_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/debug/leak.c</b> (7 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 4</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **leak_track_id** <sub>line 22</sub>
```c
bool leak_track_id(void *ptr, size_t size, uint64_t caller, uint64_t *out_id) {
```

#### `FUNCTION`: **leak_track** <sub>line 41</sub>
```c
bool leak_track(void *ptr, size_t size, uint64_t caller) { return leak_track_id(ptr, size, caller, NULL); }
```

#### `FUNCTION`: **leak_untrack** <sub>line 42</sub>
```c
bool leak_untrack(void *ptr, size_t *out_size, uint64_t *out_caller) {
```

#### `FUNCTION`: **leak_contains** <sub>line 56</sub>
```c
bool leak_contains(void *ptr) {
```

#### `FUNCTION`: **leak_count** <sub>line 67</sub>
```c
size_t leak_count(void) {
```

#### `FUNCTION`: **leak_dump** <sub>line 74</sub>
```c
void leak_dump(void) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/debug/leak.h</b> (6 items)</summary>

#### `FUNCTION`: **leak_track** <sub>line 8</sub>
```c
bool leak_track(void *ptr, size_t size, uint64_t caller);
```

#### `FUNCTION`: **leak_track_id** <sub>line 10</sub>
```c
bool leak_track_id(void *ptr, size_t size, uint64_t caller, uint64_t *out_id);
```

#### `FUNCTION`: **leak_untrack** <sub>line 12</sub>
```c
bool leak_untrack(void *ptr, size_t *out_size, uint64_t *out_caller);
```

#### `FUNCTION`: **leak_contains** <sub>line 13</sub>
```c
bool leak_contains(void *ptr);
```

#### `FUNCTION`: **leak_count** <sub>line 14</sub>
```c
size_t leak_count(void);
```

#### `FUNCTION`: **leak_dump** <sub>line 16</sub>
```c
void leak_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/debug/stats.c</b> (9 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 4</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **alloc_stats_note_alloc** <sub>line 15</sub>
```c
void alloc_stats_note_alloc(size_t size) {
```

#### `FUNCTION`: **alloc_stats_note_free** <sub>line 23</sub>
```c
void alloc_stats_note_free(size_t size) {
```

#### `FUNCTION`: **alloc_stats_live_count** <sub>line 31</sub>
```c
size_t alloc_stats_live_count(void) {
```

#### `FUNCTION`: **alloc_stats_live_bytes** <sub>line 37</sub>
```c
size_t alloc_stats_live_bytes(void) {
```

#### `FUNCTION`: **alloc_stats_peak_bytes** <sub>line 43</sub>
```c
size_t alloc_stats_peak_bytes(void) {
```

#### `FUNCTION`: **alloc_stats_total_allocs** <sub>line 49</sub>
```c
size_t alloc_stats_total_allocs(void) {
```

#### `FUNCTION`: **alloc_stats_total_frees** <sub>line 55</sub>
```c
size_t alloc_stats_total_frees(void) {
```

#### `FUNCTION`: **alloc_stats_dump** <sub>line 61</sub>
```c
void alloc_stats_dump(void) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/debug/stats.h</b> (8 items)</summary>

#### `FUNCTION`: **alloc_stats_note_alloc** <sub>line 8</sub>
```c
void alloc_stats_note_alloc(size_t size);
```

#### `FUNCTION`: **alloc_stats_note_free** <sub>line 9</sub>
```c
void alloc_stats_note_free(size_t size);
```

#### `FUNCTION`: **alloc_stats_live_count** <sub>line 11</sub>
```c
size_t alloc_stats_live_count(void);
```

#### `FUNCTION`: **alloc_stats_live_bytes** <sub>line 12</sub>
```c
size_t alloc_stats_live_bytes(void);
```

#### `FUNCTION`: **alloc_stats_peak_bytes** <sub>line 13</sub>
```c
size_t alloc_stats_peak_bytes(void);
```

#### `FUNCTION`: **alloc_stats_total_allocs** <sub>line 14</sub>
```c
size_t alloc_stats_total_allocs(void);
```

#### `FUNCTION`: **alloc_stats_total_frees** <sub>line 15</sub>
```c
size_t alloc_stats_total_frees(void);
```

#### `FUNCTION`: **alloc_stats_dump** <sub>line 17</sub>
```c
void alloc_stats_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/heap/buddy.c</b> (16 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 6</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **order_size** <sub>line 25</sub>
```c
static inline uint64_t order_size(size_t o) { return ARCH_PAGE_SIZE << o; }
```

#### `FUNCTION`: **page_of** <sub>line 26</sub>
```c
static size_t page_of(uint64_t va) { return (size_t)((va - buddy_base) / ARCH_PAGE_SIZE); }
```

#### `FUNCTION`: **va_of** <sub>line 27</sub>
```c
static uint64_t va_of(size_t page) { return buddy_base + (uint64_t)page * ARCH_PAGE_SIZE; }
```

#### `FUNCTION`: **size_to_order** <sub>line 28</sub>
```c
static size_t size_to_order(size_t size) {
```

#### `FUNCTION`: **fl_push** <sub>line 36</sub>
```c
static void fl_push(size_t o, size_t idx) {
```

#### `FUNCTION`: **fl_pop** <sub>line 40</sub>
```c
static size_t fl_pop(size_t o) {
```

#### `FUNCTION`: **fl_remove** <sub>line 46</sub>
```c
static void fl_remove(size_t o, size_t idx) {
```

#### `FUNCTION`: **buddy_init** <sub>line 56</sub>
```c
bool buddy_init(uint64_t base, size_t size, buddy_map_cb map_cb, buddy_unmap_cb unmap_cb) {
```

#### `FUNCTION`: **buddy_ready** <sub>line 95</sub>
```c
bool buddy_ready(void) { return buddy_initialized; }
```

#### `FUNCTION`: **buddy_alloc** <sub>line 133</sub>
```c
return buddy_alloc(p);
```

#### `FUNCTION`: **buddy_free** <sub>line 136</sub>
```c
void buddy_free(void *ptr) {
```

#### `FUNCTION`: **buddy_block_size** <sub>line 163</sub>
```c
size_t buddy_block_size(void *ptr) {
```

#### `FUNCTION`: **buddy_stat_used_bytes** <sub>line 171</sub>
```c
size_t buddy_stat_used_bytes(void) { return buddy_used_bytes; }
```

#### `FUNCTION`: **buddy_stat_free_bytes** <sub>line 172</sub>
```c
size_t buddy_stat_free_bytes(void) { return buddy_free_bytes; }
```

#### `FUNCTION`: **buddy_dump** <sub>line 173</sub>
```c
void buddy_dump(void) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/heap/buddy.h</b> (8 items)</summary>

#### `FUNCTION`: **bool** <sub>line 8</sub>
```c
typedef bool (*buddy_map_cb)(uint64_t virt, size_t size);
```

#### `FUNCTION`: **void** <sub>line 9</sub>
```c
typedef void (*buddy_unmap_cb)(uint64_t virt, size_t size);
```

#### `FUNCTION`: **buddy_ready** <sub>line 16</sub>
```c
bool buddy_ready(void);
```

#### `FUNCTION`: **buddy_free** <sub>line 20</sub>
```c
void buddy_free(void *ptr);
```

#### `FUNCTION`: **buddy_block_size** <sub>line 22</sub>
```c
size_t buddy_block_size(void *ptr);
```

#### `FUNCTION`: **buddy_stat_used_bytes** <sub>line 24</sub>
```c
size_t buddy_stat_used_bytes(void);
```

#### `FUNCTION`: **buddy_stat_free_bytes** <sub>line 25</sub>
```c
size_t buddy_stat_free_bytes(void);
```

#### `FUNCTION`: **buddy_dump** <sub>line 27</sub>
```c
void buddy_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/heap/heap.c</b> (16 items)</summary>

#### `FUNCTION`: **heap_map_cb** <sub>line 25</sub>
```c
static bool heap_map_cb(uint64_t virt, size_t size)
```

#### `FUNCTION`: **heap_unmap_cb** <sub>line 72</sub>
```c
static void heap_unmap_cb(uint64_t virt, size_t size)
```

#### `FUNCTION`: **heap_memset** <sub>line 87</sub>
```c
static void heap_memset(void *dst, uint8_t value, size_t n)
```

#### `FUNCTION`: **heap_in_slab** <sub>line 96</sub>
```c
static bool heap_in_slab(uint64_t va)
```

#### `FUNCTION`: **heap_in_buddy** <sub>line 101</sub>
```c
static bool heap_in_buddy(uint64_t va)
```

#### `FUNCTION`: **heap_init** <sub>line 106</sub>
```c
bool heap_init(void)
```

#### `FUNCTION`: **heap_ready** <sub>line 134</sub>
```c
bool heap_ready(void)
```

#### `FUNCTION`: **slab_alloc** <sub>line 147</sub>
```c
return slab_alloc(size);
```

#### `FUNCTION`: **buddy_alloc** <sub>line 151</sub>
```c
return buddy_alloc(size);
```

#### `FUNCTION`: **slab_alloc** <sub>line 167</sub>
```c
return slab_alloc(size);
```

#### `FUNCTION`: **buddy_alloc_aligned** <sub>line 171</sub>
```c
return buddy_alloc_aligned(size, align);
```

#### `FUNCTION`: **heap_free** <sub>line 187</sub>
```c
void heap_free(void *ptr)
```

#### `FUNCTION`: **heap_usable_size** <sub>line 207</sub>
```c
size_t heap_usable_size(void *ptr)
```

#### `FUNCTION`: **slab_usable_size** <sub>line 217</sub>
```c
return slab_usable_size(ptr);
```

#### `FUNCTION`: **buddy_block_size** <sub>line 222</sub>
```c
return buddy_block_size(ptr);
```

#### `FUNCTION`: **heap_dump** <sub>line 228</sub>
```c
void heap_dump(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/heap/heap.h</b> (5 items)</summary>

#### `FUNCTION`: **heap_init** <sub>line 8</sub>
```c
bool heap_init(void);
```

#### `FUNCTION`: **heap_ready** <sub>line 9</sub>
```c
bool heap_ready(void);
```

#### `FUNCTION`: **heap_free** <sub>line 15</sub>
```c
void heap_free(void *ptr);
```

#### `FUNCTION`: **heap_usable_size** <sub>line 17</sub>
```c
size_t heap_usable_size(void *ptr);
```

#### `FUNCTION`: **heap_dump** <sub>line 19</sub>
```c
void heap_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/heap/slab.c</b> (17 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 9</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **slab_lock** <sub>line 50</sub>
```c
static void slab_lock(void)
```

#### `FUNCTION`: **slab_unlock** <sub>line 55</sub>
```c
static void slab_unlock(void)
```

#### `FUNCTION`: **slab_bit_test** <sub>line 60</sub>
```c
static inline bool slab_bit_test(const slab_desc_t *d, uint32_t idx)
```

#### `FUNCTION`: **slab_bit_set** <sub>line 65</sub>
```c
static inline void slab_bit_set(slab_desc_t *d, uint32_t idx)
```

#### `FUNCTION`: **slab_bit_clear** <sub>line 70</sub>
```c
static inline void slab_bit_clear(slab_desc_t *d, uint32_t idx)
```

#### `FUNCTION`: **slab_bitmap_clear_all** <sub>line 75</sub>
```c
static inline void slab_bitmap_clear_all(slab_desc_t *d)
```

#### `FUNCTION`: **slab_page_va** <sub>line 82</sub>
```c
static uint64_t slab_page_va(size_t page)
```

#### `FUNCTION`: **partial_remove** <sub>line 87</sub>
```c
static void partial_remove(slab_cache_t *c, uint32_t page)
```

#### `FUNCTION`: **slab_grow** <sub>line 101</sub>
```c
static bool slab_grow(uint32_t cache_id)
```

#### `FUNCTION`: **slab_init** <sub>line 155</sub>
```c
bool slab_init(uint64_t base, size_t size)
```

#### `FUNCTION`: **slab_ready** <sub>line 228</sub>
```c
bool slab_ready(void)
```

#### `FUNCTION`: **slab_free** <sub>line 286</sub>
```c
void slab_free(void *ptr)
```

#### `FUNCTION`: **slab_usable_size** <sub>line 365</sub>
```c
size_t slab_usable_size(void *ptr)
```

#### `FUNCTION`: **slab_stat_used_bytes** <sub>line 389</sub>
```c
size_t slab_stat_used_bytes(void)
```

#### `FUNCTION`: **slab_stat_free_bytes** <sub>line 413</sub>
```c
size_t slab_stat_free_bytes(void)
```

#### `FUNCTION`: **slab_dump** <sub>line 436</sub>
```c
void slab_dump(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/heap/slab.h</b> (7 items)</summary>

#### `FUNCTION`: **slab_init** <sub>line 10</sub>
```c
bool slab_init(uint64_t base, size_t size);
```

#### `FUNCTION`: **slab_ready** <sub>line 11</sub>
```c
bool slab_ready(void);
```

#### `FUNCTION`: **slab_free** <sub>line 14</sub>
```c
void slab_free(void *ptr);
```

#### `FUNCTION`: **slab_usable_size** <sub>line 16</sub>
```c
size_t slab_usable_size(void *ptr);
```

#### `FUNCTION`: **slab_stat_used_bytes** <sub>line 18</sub>
```c
size_t slab_stat_used_bytes(void);
```

#### `FUNCTION`: **slab_stat_free_bytes** <sub>line 19</sub>
```c
size_t slab_stat_free_bytes(void);
```

#### `FUNCTION`: **slab_dump** <sub>line 21</sub>
```c
void slab_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/physical/bitmap.c</b> (27 items)</summary>

#### `FUNCTION`: **size_min** <sub>line 7</sub>
```c
static inline size_t size_min(size_t a, size_t b)
```

#### `FUNCTION`: **align_up_size** <sub>line 12</sub>
```c
static inline size_t align_up_size(size_t value, size_t align)
```

#### `FUNCTION`: **mask_from_bit** <sub>line 27</sub>
```c
static inline uint64_t mask_from_bit(size_t bit)
```

#### `FUNCTION`: **mask_to_bit** <sub>line 36</sub>
```c
static inline uint64_t mask_to_bit(size_t bit)
```

#### `FUNCTION`: **lower_bits_mask** <sub>line 45</sub>
```c
static inline uint64_t lower_bits_mask(size_t bit)
```

#### `FUNCTION`: **bitmap_words_for_bits** <sub>line 54</sub>
```c
size_t bitmap_words_for_bits(size_t bit_count)
```

#### `FUNCTION`: **bitmap_bytes_for_bits** <sub>line 63</sub>
```c
size_t bitmap_bytes_for_bits(size_t bit_count)
```

#### `FUNCTION`: **bitmap_words_for_bits** <sub>line 65</sub>
```c
return bitmap_words_for_bits(bit_count) * sizeof(uint64_t);
```

#### `FUNCTION`: **bitmap_mark_tail_used** <sub>line 68</sub>
```c
static void bitmap_mark_tail_used(bitmap_t *bm)
```

#### `FUNCTION`: **bitmap_init_virt** <sub>line 84</sub>
```c
void bitmap_init_virt(bitmap_t *bm, void *storage, size_t bit_count)
```

#### `FUNCTION`: **bitmap_init_phys** <sub>line 102</sub>
```c
bool bitmap_init_phys(bitmap_t *bm, uint64_t storage_phys, size_t bit_count)
```

#### `FUNCTION`: **bitmap_fill** <sub>line 119</sub>
```c
void bitmap_fill(bitmap_t *bm, bool value)
```

#### `FUNCTION`: **bitmap_set** <sub>line 137</sub>
```c
void bitmap_set(bitmap_t *bm, size_t bit)
```

#### `FUNCTION`: **bitmap_clear** <sub>line 149</sub>
```c
void bitmap_clear(bitmap_t *bm, size_t bit)
```

#### `FUNCTION`: **bitmap_test** <sub>line 161</sub>
```c
bool bitmap_test(const bitmap_t *bm, size_t bit)
```

#### `FUNCTION`: **bitmap_set_range** <sub>line 173</sub>
```c
void bitmap_set_range(bitmap_t *bm, size_t start, size_t count)
```

#### `FUNCTION`: **bitmap_clear_range** <sub>line 216</sub>
```c
void bitmap_clear_range(bitmap_t *bm, size_t start, size_t count)
```

#### `FUNCTION`: **bitmap_test_range_free** <sub>line 259</sub>
```c
bool bitmap_test_range_free(const bitmap_t *bm, size_t start, size_t count)
```

#### `FUNCTION`: **find_next_zero_bit** <sub>line 310</sub>
```c
static size_t find_next_zero_bit(const bitmap_t *bm, size_t start)
```

#### `FUNCTION`: **find_next_used_bit** <sub>line 354</sub>
```c
static size_t find_next_used_bit(const bitmap_t *bm, size_t start, size_t limit)
```

#### `FUNCTION`: **bitmap_alloc** <sub>line 398</sub>
```c
size_t bitmap_alloc(bitmap_t *bm)
```

#### `FUNCTION`: **bitmap_alloc_from** <sub>line 426</sub>
```c
size_t bitmap_alloc_from(bitmap_t *bm, size_t min_bit)
```

#### `FUNCTION`: **bitmap_alloc_range** <sub>line 452</sub>
```c
size_t bitmap_alloc_range(bitmap_t *bm, size_t count, size_t align)
```

#### `FUNCTION`: **bitmap_alloc** <sub>line 474</sub>
```c
return bitmap_alloc(bm);
```

#### `FUNCTION`: **bitmap_free** <sub>line 519</sub>
```c
void bitmap_free(bitmap_t *bm, size_t bit)
```

#### `FUNCTION`: **bitmap_free_range** <sub>line 534</sub>
```c
void bitmap_free_range(bitmap_t *bm, size_t start, size_t count)
```

#### `FUNCTION`: **bitmap_count_free** <sub>line 557</sub>
```c
size_t bitmap_count_free(const bitmap_t *bm)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/physical/bitmap.h</b> (21 items)</summary>

#### `FUNCTION`: **bitmap_words_for_bits** <sub>line 30</sub>
```c
size_t bitmap_words_for_bits(size_t bit_count);
```

#### `FUNCTION`: **bitmap_bytes_for_bits** <sub>line 31</sub>
```c
size_t bitmap_bytes_for_bits(size_t bit_count);
```

#### `FUNCTION`: **bitmap_init_virt** <sub>line 33</sub>
```c
void bitmap_init_virt(bitmap_t *bm, void *storage, size_t bit_count);
```

#### `FUNCTION`: **bitmap_init_phys** <sub>line 35</sub>
```c
bool bitmap_init_phys(bitmap_t *bm, uint64_t storage_phys, size_t bit_count);
```

#### `FUNCTION`: **bitmap_fill** <sub>line 37</sub>
```c
void bitmap_fill(bitmap_t *bm, bool value);
```

#### `FUNCTION`: **bitmap_set** <sub>line 39</sub>
```c
void bitmap_set(bitmap_t *bm, size_t bit);
```

#### `FUNCTION`: **bitmap_clear** <sub>line 40</sub>
```c
void bitmap_clear(bitmap_t *bm, size_t bit);
```

#### `FUNCTION`: **bitmap_test** <sub>line 41</sub>
```c
bool bitmap_test(const bitmap_t *bm, size_t bit);
```

#### `FUNCTION`: **bitmap_fill** <sub>line 43</sub>
```c
void bitmap_fill(bitmap_t *bm, bool value);
```

#### `FUNCTION`: **bitmap_set** <sub>line 45</sub>
```c
void bitmap_set(bitmap_t *bm, size_t bit);
```

#### `FUNCTION`: **bitmap_clear** <sub>line 46</sub>
```c
void bitmap_clear(bitmap_t *bm, size_t bit);
```

#### `FUNCTION`: **bitmap_test** <sub>line 47</sub>
```c
bool bitmap_test(const bitmap_t *bm, size_t bit);
```

#### `FUNCTION`: **bitmap_set_range** <sub>line 49</sub>
```c
void bitmap_set_range(bitmap_t *bm, size_t start, size_t count);
```

#### `FUNCTION`: **bitmap_clear_range** <sub>line 50</sub>
```c
void bitmap_clear_range(bitmap_t *bm, size_t start, size_t count);
```

#### `FUNCTION`: **bitmap_test_range_free** <sub>line 52</sub>
```c
bool bitmap_test_range_free(const bitmap_t *bm, size_t start, size_t count);
```

#### `FUNCTION`: **bitmap_alloc** <sub>line 54</sub>
```c
size_t bitmap_alloc(bitmap_t *bm);
```

#### `FUNCTION`: **bitmap_alloc_from** <sub>line 57</sub>
```c
size_t bitmap_alloc_from(bitmap_t *bm, size_t min_bit);
```

#### `FUNCTION`: **bitmap_alloc_range** <sub>line 59</sub>
```c
size_t bitmap_alloc_range(bitmap_t *bm, size_t count, size_t align);
```

#### `FUNCTION`: **bitmap_free** <sub>line 61</sub>
```c
void bitmap_free(bitmap_t *bm, size_t bit);
```

#### `FUNCTION`: **bitmap_free_range** <sub>line 62</sub>
```c
void bitmap_free_range(bitmap_t *bm, size_t start, size_t count);
```

#### `FUNCTION`: **bitmap_count_free** <sub>line 64</sub>
```c
size_t bitmap_count_free(const bitmap_t *bm);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/physical/frame.c</b> (24 items)</summary>

#### `FUNCTION`: **count_dma32_in_range** <sub>line 20</sub>
```c
static inline size_t count_dma32_in_range(size_t start_pfn, size_t count)
```

#### `FUNCTION`: **dma32_account_alloc** <sub>line 31</sub>
```c
static inline void dma32_account_alloc(size_t start_pfn, size_t count)
```

#### `FUNCTION`: **dma32_account_free** <sub>line 36</sub>
```c
static inline void dma32_account_free(size_t start_pfn, size_t count)
```

#### `FUNCTION`: **frame_to_pfn** <sub>line 47</sub>
```c
size_t frame_to_pfn(frame_t frame)
```

#### `FUNCTION`: **frame_from_pfn** <sub>line 52</sub>
```c
frame_t frame_from_pfn(size_t pfn)
```

#### `FUNCTION`: **frame_phys** <sub>line 57</sub>
```c
uint64_t frame_phys(frame_t frame)
```

#### `FUNCTION`: **arch_phys_to_virt** <sub>line 64</sub>
```c
return arch_phys_to_virt(frame);
```

#### `FUNCTION`: **frame_is_valid** <sub>line 67</sub>
```c
bool frame_is_valid(frame_t frame)
```

#### `FUNCTION`: **frame_init** <sub>line 82</sub>
```c
bool frame_init(uint64_t bitmap_phys, size_t bit_count)
```

#### `FUNCTION`: **frame_init_from_memory** <sub>line 118</sub>
```c
bool frame_init_from_memory(void)
```

#### `FUNCTION`: **frame_ready** <sub>line 189</sub>
```c
bool frame_ready(void)
```

#### `FUNCTION`: **frame_alloc** <sub>line 194</sub>
```c
bool frame_alloc(frame_t *out)
```

#### `FUNCTION`: **frame_alloc_zero** <sub>line 221</sub>
```c
bool frame_alloc_zero(frame_t *out)
```

#### `FUNCTION`: **frame_free** <sub>line 271</sub>
```c
bool frame_free(frame_t frame)
```

#### `FUNCTION`: **frame_free_contiguous** <sub>line 295</sub>
```c
bool frame_free_contiguous(frame_t start, size_t count)
```

#### `FUNCTION`: **frame_zero** <sub>line 333</sub>
```c
bool frame_zero(frame_t frame)
```

#### `FUNCTION`: **volatile** <sub>line 345</sub>
```c
__asm__ volatile("" ::: "memory");
```

#### `FUNCTION`: **frame_total** <sub>line 398</sub>
```c
size_t frame_total(void)
```

#### `FUNCTION`: **frame_allocated** <sub>line 407</sub>
```c
size_t frame_allocated(void)
```

#### `FUNCTION`: **frame_free_count** <sub>line 416</sub>
```c
size_t frame_free_count(void)
```

#### `FUNCTION`: **frame_zone_dma32_total** <sub>line 429</sub>
```c
size_t frame_zone_dma32_total(void)
```

#### `FUNCTION`: **frame_zone_dma32_free** <sub>line 438</sub>
```c
size_t frame_zone_dma32_free(void)
```

#### `FUNCTION`: **frame_zone_normal_total** <sub>line 451</sub>
```c
size_t frame_zone_normal_total(void)
```

#### `FUNCTION`: **frame_zone_normal_free** <sub>line 460</sub>
```c
size_t frame_zone_normal_free(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/physical/frame.h</b> (19 items)</summary>

#### `FUNCTION`: **frame_init** <sub>line 12</sub>
```c
bool frame_init(uint64_t bitmap_phys, size_t bit_count);
```

#### `FUNCTION`: **frame_init_from_memory** <sub>line 14</sub>
```c
bool frame_init_from_memory(void);
```

#### `FUNCTION`: **frame_ready** <sub>line 16</sub>
```c
bool frame_ready(void);
```

#### `FUNCTION`: **frame_alloc** <sub>line 18</sub>
```c
bool frame_alloc(frame_t *out);
```

#### `FUNCTION`: **frame_alloc_zero** <sub>line 19</sub>
```c
bool frame_alloc_zero(frame_t *out);
```

#### `FUNCTION`: **frame_free** <sub>line 29</sub>
```c
bool frame_free(frame_t frame);
```

#### `FUNCTION`: **frame_free_contiguous** <sub>line 30</sub>
```c
bool frame_free_contiguous(frame_t start, size_t count);
```

#### `FUNCTION`: **frame_is_valid** <sub>line 32</sub>
```c
bool frame_is_valid(frame_t frame);
```

#### `FUNCTION`: **frame_zero** <sub>line 33</sub>
```c
bool frame_zero(frame_t frame);
```

#### `FUNCTION`: **frame_phys** <sub>line 36</sub>
```c
uint64_t frame_phys(frame_t frame);
```

#### `FUNCTION`: **frame_to_pfn** <sub>line 38</sub>
```c
size_t frame_to_pfn(frame_t frame);
```

#### `FUNCTION`: **frame_from_pfn** <sub>line 39</sub>
```c
frame_t frame_from_pfn(size_t pfn);
```

#### `FUNCTION`: **frame_total** <sub>line 41</sub>
```c
size_t frame_total(void);
```

#### `FUNCTION`: **frame_allocated** <sub>line 42</sub>
```c
size_t frame_allocated(void);
```

#### `FUNCTION`: **frame_free_count** <sub>line 43</sub>
```c
size_t frame_free_count(void);
```

#### `FUNCTION`: **frame_zone_dma32_total** <sub>line 46</sub>
```c
size_t frame_zone_dma32_total(void);
```

#### `FUNCTION`: **frame_zone_dma32_free** <sub>line 47</sub>
```c
size_t frame_zone_dma32_free(void);
```

#### `FUNCTION`: **frame_zone_normal_total** <sub>line 48</sub>
```c
size_t frame_zone_normal_total(void);
```

#### `FUNCTION`: **frame_zone_normal_free** <sub>line 49</sub>
```c
size_t frame_zone_normal_free(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/physical/pmm.c</b> (39 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 6</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **pmm_panic** <sub>line 8</sub>
```c
static void pmm_panic(const char *msg) __attribute__((noreturn));
```

#### `FUNCTION`: **pmm_panic** <sub>line 9</sub>
```c
static void pmm_panic(const char *msg)
```

#### `FUNCTION`: **pmm_lock** <sub>line 23</sub>
```c
static void pmm_lock(void)
```

#### `FUNCTION`: **pmm_unlock** <sub>line 28</sub>
```c
static void pmm_unlock(void)
```

#### `FUNCTION`: **pmm_bytes_to_frames** <sub>line 37</sub>
```c
static size_t pmm_bytes_to_frames(size_t bytes)
```

#### `FUNCTION`: **pmm_align_bytes_to_frames** <sub>line 50</sub>
```c
static size_t pmm_align_bytes_to_frames(size_t align_bytes)
```

#### `FUNCTION`: **pmm_zero_frames** <sub>line 75</sub>
```c
static bool pmm_zero_frames(uint64_t start_phys, size_t count)
```

#### `FUNCTION`: **pmm_init** <sub>line 88</sub>
```c
bool pmm_init(void)
```

#### `FUNCTION`: **pmm_ready** <sub>line 116</sub>
```c
bool pmm_ready(void)
```

#### `FUNCTION`: **pmm_alloc_frame** <sub>line 121</sub>
```c
bool pmm_alloc_frame(uint64_t *out_phys)
```

#### `FUNCTION`: **pmm_alloc_zero_frame** <sub>line 143</sub>
```c
bool pmm_alloc_zero_frame(uint64_t *out_phys)
```

#### `FUNCTION`: **pmm_alloc_frames** <sub>line 165</sub>
```c
bool pmm_alloc_frames(size_t count, uint64_t *out_phys)
```

#### `FUNCTION`: **pmm_alloc_bytes** <sub>line 212</sub>
```c
bool pmm_alloc_bytes(size_t bytes, uint64_t *out_phys)
```

#### `FUNCTION`: **pmm_alloc_frames** <sub>line 220</sub>
```c
return pmm_alloc_frames(frames, out_phys);
```

#### `FUNCTION`: **pmm_alloc_zero_bytes** <sub>line 223</sub>
```c
bool pmm_alloc_zero_bytes(size_t bytes, uint64_t *out_phys)
```

#### `FUNCTION`: **pmm_alloc_zero_frame** <sub>line 236</sub>
```c
return pmm_alloc_zero_frame(out_phys);
```

#### `FUNCTION`: **pmm_alloc_frames_aligned** <sub>line 275</sub>
```c
return pmm_alloc_frames_aligned(frames, align_frames, out_phys);
```

#### `FUNCTION`: **pmm_free_frame** <sub>line 278</sub>
```c
bool pmm_free_frame(uint64_t phys)
```

#### `FUNCTION`: **pmm_free_frames** <sub>line 293</sub>
```c
bool pmm_free_frames(uint64_t phys, size_t count)
```

#### `FUNCTION`: **pmm_free_bytes** <sub>line 312</sub>
```c
bool pmm_free_bytes(uint64_t phys, size_t bytes)
```

#### `FUNCTION`: **pmm_free_frames** <sub>line 328</sub>
```c
return pmm_free_frames(phys, frames);
```

#### `FUNCTION`: **pmm_stat_total_frames** <sub>line 331</sub>
```c
size_t pmm_stat_total_frames(void)
```

#### `FUNCTION`: **frame_total** <sub>line 337</sub>
```c
return frame_total();
```

#### `FUNCTION`: **pmm_stat_free_frames** <sub>line 340</sub>
```c
size_t pmm_stat_free_frames(void)
```

#### `FUNCTION`: **frame_free_count** <sub>line 346</sub>
```c
return frame_free_count();
```

#### `FUNCTION`: **pmm_stat_allocated_frames** <sub>line 349</sub>
```c
size_t pmm_stat_allocated_frames(void)
```

#### `FUNCTION`: **frame_allocated** <sub>line 355</sub>
```c
return frame_allocated();
```

#### `FUNCTION`: **pmm_stat_total_bytes** <sub>line 358</sub>
```c
uint64_t pmm_stat_total_bytes(void)
```

#### `FUNCTION`: **pmm_stat_free_bytes** <sub>line 373</sub>
```c
uint64_t pmm_stat_free_bytes(void)
```

#### `FUNCTION`: **pmm_stat_dma32_total_frames** <sub>line 388</sub>
```c
size_t pmm_stat_dma32_total_frames(void)
```

#### `FUNCTION`: **frame_zone_dma32_total** <sub>line 394</sub>
```c
return frame_zone_dma32_total();
```

#### `FUNCTION`: **pmm_stat_dma32_free_frames** <sub>line 397</sub>
```c
size_t pmm_stat_dma32_free_frames(void)
```

#### `FUNCTION`: **frame_zone_dma32_free** <sub>line 403</sub>
```c
return frame_zone_dma32_free();
```

#### `FUNCTION`: **pmm_stat_normal_total_frames** <sub>line 406</sub>
```c
size_t pmm_stat_normal_total_frames(void)
```

#### `FUNCTION`: **frame_zone_normal_total** <sub>line 412</sub>
```c
return frame_zone_normal_total();
```

#### `FUNCTION`: **pmm_stat_normal_free_frames** <sub>line 415</sub>
```c
size_t pmm_stat_normal_free_frames(void)
```

#### `FUNCTION`: **frame_zone_normal_free** <sub>line 421</sub>
```c
return frame_zone_normal_free();
```

#### `FUNCTION`: **pmm_dump** <sub>line 449</sub>
```c
void pmm_dump(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/physical/pmm.h</b> (21 items)</summary>

#### `FUNCTION`: **pmm_init** <sub>line 10</sub>
```c
bool pmm_init(void);
```

#### `FUNCTION`: **pmm_ready** <sub>line 12</sub>
```c
bool pmm_ready(void);
```

#### `FUNCTION`: **pmm_alloc_frame** <sub>line 14</sub>
```c
bool pmm_alloc_frame(uint64_t *out_phys);
```

#### `FUNCTION`: **pmm_alloc_zero_frame** <sub>line 16</sub>
```c
bool pmm_alloc_zero_frame(uint64_t *out_phys);
```

#### `FUNCTION`: **pmm_alloc_frames** <sub>line 18</sub>
```c
bool pmm_alloc_frames(size_t count, uint64_t *out_phys);
```

#### `FUNCTION`: **pmm_alloc_bytes** <sub>line 23</sub>
```c
bool pmm_alloc_bytes(size_t bytes, uint64_t *out_phys);
```

#### `FUNCTION`: **pmm_alloc_zero_bytes** <sub>line 24</sub>
```c
bool pmm_alloc_zero_bytes(size_t bytes, uint64_t *out_phys);
```

#### `FUNCTION`: **pmm_free_frame** <sub>line 28</sub>
```c
bool pmm_free_frame(uint64_t phys);
```

#### `FUNCTION`: **pmm_free_frames** <sub>line 33</sub>
```c
bool pmm_free_frames(uint64_t phys, size_t count);
```

#### `FUNCTION`: **pmm_free_bytes** <sub>line 34</sub>
```c
bool pmm_free_bytes(uint64_t phys, size_t bytes);
```

#### `FUNCTION`: **pmm_stat_total_frames** <sub>line 35</sub>
```c
size_t pmm_stat_total_frames(void);
```

#### `FUNCTION`: **pmm_stat_free_frames** <sub>line 36</sub>
```c
size_t pmm_stat_free_frames(void);
```

#### `FUNCTION`: **pmm_stat_allocated_frames** <sub>line 37</sub>
```c
size_t pmm_stat_allocated_frames(void);
```

#### `FUNCTION`: **pmm_stat_total_bytes** <sub>line 39</sub>
```c
uint64_t pmm_stat_total_bytes(void);
```

#### `FUNCTION`: **pmm_stat_free_bytes** <sub>line 40</sub>
```c
uint64_t pmm_stat_free_bytes(void);
```

#### `FUNCTION`: **pmm_stat_dma32_total_frames** <sub>line 43</sub>
```c
size_t pmm_stat_dma32_total_frames(void);
```

#### `FUNCTION`: **pmm_stat_dma32_free_frames** <sub>line 44</sub>
```c
size_t pmm_stat_dma32_free_frames(void);
```

#### `FUNCTION`: **pmm_stat_normal_total_frames** <sub>line 45</sub>
```c
size_t pmm_stat_normal_total_frames(void);
```

#### `FUNCTION`: **pmm_stat_normal_free_frames** <sub>line 46</sub>
```c
size_t pmm_stat_normal_free_frames(void);
```

#### `FUNCTION`: **pmm_dump** <sub>line 48</sub>
```c
void pmm_dump(void);
```

#### `FUNCTION`: **pmm_self_test** <sub>line 51</sub>
```c
bool pmm_self_test(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/special/contiguous.c</b> (3 items)</summary>

#### `FUNCTION`: **contig_bytes_to_frames** <sub>line 7</sub>
```c
static bool contig_bytes_to_frames(size_t bytes, size_t *out_frames)
```

#### `FUNCTION`: **size_bytes_to_pages_checked** <sub>line 9</sub>
```c
return size_bytes_to_pages_checked(bytes, ARCH_PAGE_SIZE, out_frames);
```

#### `FUNCTION`: **contig_free** <sub>line 56</sub>
```c
void contig_free(uint64_t phys, size_t bytes)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/special/contiguous.h</b> (1 items)</summary>

#### `FUNCTION`: **contig_free** <sub>line 13</sub>
```c
void contig_free(uint64_t phys, size_t bytes);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/special/dma.c</b> (8 items)</summary>

#### `FUNCTION`: **dma_bytes_to_frames** <sub>line 8</sub>
```c
static bool dma_bytes_to_frames(size_t bytes, size_t *out_frames)
```

#### `FUNCTION`: **size_bytes_to_pages_checked** <sub>line 10</sub>
```c
return size_bytes_to_pages_checked(bytes, ARCH_PAGE_SIZE, out_frames);
```

#### `FUNCTION`: **dma_free_coherent** <sub>line 57</sub>
```c
void dma_free_coherent(uint64_t phys, void *virt, size_t bytes)
```

#### `FUNCTION`: **clflush_range** <sub>line 82</sub>
```c
static void clflush_range(void *addr, size_t len)
```

#### `FUNCTION`: **volatile** <sub>line 88</sub>
```c
__asm__ volatile("clflush (%0)" :: "r"(a) : "memory");
```

#### `FUNCTION`: **volatile** <sub>line 91</sub>
```c
__asm__ volatile("" ::: "memory");
```

#### `FUNCTION`: **dma_sync_for_device** <sub>line 94</sub>
```c
void dma_sync_for_device(void *virt, size_t len)
```

#### `FUNCTION`: **dma_sync_for_cpu** <sub>line 103</sub>
```c
void dma_sync_for_cpu(void *virt, size_t len)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/special/dma.h</b> (3 items)</summary>

#### `FUNCTION`: **dma_free_coherent** <sub>line 16</sub>
```c
void dma_free_coherent(uint64_t phys, void *virt, size_t bytes);
```

#### `FUNCTION`: **dma_sync_for_device** <sub>line 18</sub>
```c
void dma_sync_for_device(void *virt, size_t len);
```

#### `FUNCTION`: **dma_sync_for_cpu** <sub>line 19</sub>
```c
void dma_sync_for_cpu(void *virt, size_t len);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/virtual/mapping.c</b> (10 items)</summary>

#### `FUNCTION`: **resolve_space** <sub>line 9</sub>
```c
static uint64_t resolve_space(mapping_space_t space)
```

#### `FUNCTION`: **mapping_init** <sub>line 18</sub>
```c
void mapping_init(void)
```

#### `FUNCTION`: **mapping_kernel_space** <sub>line 24</sub>
```c
mapping_space_t mapping_kernel_space(void)
```

#### `FUNCTION`: **page_range_ok** <sub>line 34</sub>
```c
static bool page_range_ok(uint64_t addr, size_t len)
```

#### `FUNCTION`: **mapping_translate** <sub>line 145</sub>
```c
uint64_t mapping_translate(mapping_space_t space, uint64_t virt)
```

#### `FUNCTION`: **paging_translate_in** <sub>line 147</sub>
```c
return paging_translate_in(resolve_space(space), virt);
```

#### `FUNCTION`: **mapping_is_mapped** <sub>line 150</sub>
```c
bool mapping_is_mapped(mapping_space_t space, uint64_t virt)
```

#### `FUNCTION`: **paging_is_mapped_in** <sub>line 152</sub>
```c
return paging_is_mapped_in(resolve_space(space), virt);
```

#### `FUNCTION`: **copy_page_by_phys** <sub>line 155</sub>
```c
static void copy_page_by_phys(uint64_t dst_phys, uint64_t src_phys)
```

#### `FUNCTION`: **volatile** <sub>line 164</sub>
```c
__asm__ volatile("" ::: "memory");
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/virtual/mapping.h</b> (4 items)</summary>

#### `FUNCTION`: **mapping_init** <sub>line 12</sub>
```c
void mapping_init(void);
```

#### `FUNCTION`: **mapping_kernel_space** <sub>line 13</sub>
```c
mapping_space_t mapping_kernel_space(void);
```

#### `FUNCTION`: **mapping_translate** <sub>line 30</sub>
```c
uint64_t mapping_translate(mapping_space_t space, uint64_t virt);
```

#### `FUNCTION`: **mapping_is_mapped** <sub>line 31</sub>
```c
bool mapping_is_mapped(mapping_space_t space, uint64_t virt);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/virtual/page.c</b> (20 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 5</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **page_lock** <sub>line 14</sub>
```c
static void page_lock(void)
```

#### `FUNCTION`: **page_unlock** <sub>line 34</sub>
```c
static void page_unlock(void)
```

#### `FUNCTION`: **page_phys_to_pfn** <sub>line 55</sub>
```c
static size_t page_phys_to_pfn(uint64_t phys)
```

#### `FUNCTION`: **page_ptr_valid** <sub>line 60</sub>
```c
static bool page_ptr_valid(const page_t *page)
```

#### `FUNCTION`: **page_init** <sub>line 71</sub>
```c
bool page_init(void)
```

#### `FUNCTION`: **page_ready** <sub>line 125</sub>
```c
bool page_ready(void)
```

#### `FUNCTION`: **page_attach** <sub>line 184</sub>
```c
bool page_attach(uint64_t phys, uint32_t type)
```

#### `FUNCTION`: **page_get** <sub>line 217</sub>
```c
bool page_get(page_t *page)
```

#### `FUNCTION`: **page_put** <sub>line 237</sub>
```c
bool page_put(page_t *page)
```

#### `FUNCTION`: **page_phys** <sub>line 270</sub>
```c
uint64_t page_phys(const page_t *page)
```

#### `FUNCTION`: **arch_phys_to_virt** <sub>line 289</sub>
```c
return arch_phys_to_virt(phys);
```

#### `FUNCTION`: **page_pfn** <sub>line 292</sub>
```c
size_t page_pfn(const page_t *page)
```

#### `FUNCTION`: **page_from_phys** <sub>line 331</sub>
```c
return page_from_phys(arch_virt_to_phys(va));
```

#### `FUNCTION`: **page_refcount** <sub>line 334</sub>
```c
uint32_t page_refcount(const page_t *page)
```

#### `FUNCTION`: **page_type** <sub>line 343</sub>
```c
uint32_t page_type(const page_t *page)
```

#### `FUNCTION`: **page_set_type** <sub>line 352</sub>
```c
void page_set_type(page_t *page, uint32_t type)
```

#### `FUNCTION`: **page_stat_total** <sub>line 361</sub>
```c
size_t page_stat_total(void)
```

#### `FUNCTION`: **page_stat_referenced** <sub>line 370</sub>
```c
size_t page_stat_referenced(void)
```

#### `FUNCTION`: **page_dump** <sub>line 387</sub>
```c
void page_dump(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/virtual/page.h</b> (13 items)</summary>

#### `FUNCTION`: **page_init** <sub>line 22</sub>
```c
bool page_init(void);
```

#### `FUNCTION`: **page_ready** <sub>line 23</sub>
```c
bool page_ready(void);
```

#### `FUNCTION`: **page_attach** <sub>line 28</sub>
```c
bool page_attach(uint64_t phys, uint32_t type);
```

#### `FUNCTION`: **page_get** <sub>line 30</sub>
```c
bool page_get(page_t *page);
```

#### `FUNCTION`: **page_put** <sub>line 31</sub>
```c
bool page_put(page_t *page);
```

#### `FUNCTION`: **page_phys** <sub>line 33</sub>
```c
uint64_t page_phys(const page_t *page);
```

#### `FUNCTION`: **page_pfn** <sub>line 35</sub>
```c
size_t page_pfn(const page_t *page);
```

#### `FUNCTION`: **page_refcount** <sub>line 41</sub>
```c
uint32_t page_refcount(const page_t *page);
```

#### `FUNCTION`: **page_type** <sub>line 42</sub>
```c
uint32_t page_type(const page_t *page);
```

#### `FUNCTION`: **page_set_type** <sub>line 43</sub>
```c
void page_set_type(page_t *page, uint32_t type);
```

#### `FUNCTION`: **page_stat_total** <sub>line 45</sub>
```c
size_t page_stat_total(void);
```

#### `FUNCTION`: **page_stat_referenced** <sub>line 46</sub>
```c
size_t page_stat_referenced(void);
```

#### `FUNCTION`: **page_dump** <sub>line 48</sub>
```c
void page_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/virtual/vmm.c</b> (24 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 7</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **vmm_lock** <sub>line 21</sub>
```c
static void vmm_lock(void)
```

#### `FUNCTION`: **vmm_unlock** <sub>line 41</sub>
```c
static void vmm_unlock(void)
```

#### `FUNCTION`: **vmm_bytes_to_pages** <sub>line 62</sub>
```c
static size_t vmm_bytes_to_pages(size_t bytes)
```

#### `FUNCTION`: **vmm_align_to_pages** <sub>line 75</sub>
```c
static size_t vmm_align_to_pages(size_t align)
```

#### `FUNCTION`: **vmm_page_virt** <sub>line 100</sub>
```c
static uint64_t vmm_page_virt(size_t page)
```

#### `FUNCTION`: **vmm_virt_page** <sub>line 105</sub>
```c
static size_t vmm_virt_page(uint64_t virt)
```

#### `FUNCTION`: **vmm_in_region** <sub>line 110</sub>
```c
static bool vmm_in_region(uint64_t virt)
```

#### `FUNCTION`: **vmm_flags_to_pte** <sub>line 116</sub>
```c
static uint64_t vmm_flags_to_pte(uint32_t flags)
```

#### `FUNCTION`: **vmm_init** <sub>line 139</sub>
```c
bool vmm_init(void)
```

#### `FUNCTION`: **vmm_ready** <sub>line 173</sub>
```c
bool vmm_ready(void)
```

#### `FUNCTION`: **vmm_alloc** <sub>line 256</sub>
```c
bool vmm_alloc(size_t bytes, uint32_t flags, uint64_t *out_virt)
```

#### `FUNCTION`: **vmm_alloc_aligned** <sub>line 258</sub>
```c
return vmm_alloc_aligned(bytes, ARCH_PAGE_SIZE, flags, out_virt);
```

#### `FUNCTION`: **vmm_map_device** <sub>line 261</sub>
```c
bool vmm_map_device(uint64_t phys, size_t len, uint64_t *out_virt)
```

#### `FUNCTION`: **vmm_free** <sub>line 323</sub>
```c
bool vmm_free(uint64_t virt, size_t bytes)
```

#### `FUNCTION`: **vmm_unmap_device** <sub>line 369</sub>
```c
bool vmm_unmap_device(uint64_t virt, size_t len)
```

#### `FUNCTION`: **vmm_translate** <sub>line 413</sub>
```c
uint64_t vmm_translate(uint64_t virt)
```

#### `FUNCTION`: **paging_translate** <sub>line 419</sub>
```c
return paging_translate(virt);
```

#### `FUNCTION`: **vmm_stat_total_pages** <sub>line 422</sub>
```c
size_t vmm_stat_total_pages(void)
```

#### `FUNCTION`: **vmm_stat_free_pages** <sub>line 431</sub>
```c
size_t vmm_stat_free_pages(void)
```

#### `FUNCTION`: **vmm_stat_allocated_pages** <sub>line 444</sub>
```c
size_t vmm_stat_allocated_pages(void)
```

#### `FUNCTION`: **vmm_stat_total_bytes** <sub>line 453</sub>
```c
uint64_t vmm_stat_total_bytes(void)
```

#### `FUNCTION`: **vmm_stat_free_bytes** <sub>line 458</sub>
```c
uint64_t vmm_stat_free_bytes(void)
```

#### `FUNCTION`: **vmm_dump** <sub>line 463</sub>
```c
void vmm_dump(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/alloc/virtual/vmm.h</b> (14 items)</summary>

#### `FUNCTION`: **vmm_init** <sub>line 18</sub>
```c
bool vmm_init(void);
```

#### `FUNCTION`: **vmm_ready** <sub>line 19</sub>
```c
bool vmm_ready(void);
```

#### `FUNCTION`: **vmm_alloc** <sub>line 21</sub>
```c
bool vmm_alloc(size_t bytes, uint32_t flags, uint64_t *out_virt);
```

#### `FUNCTION`: **vmm_alloc_aligned** <sub>line 22</sub>
```c
bool vmm_alloc_aligned(size_t bytes, size_t align, uint32_t flags, uint64_t *out_virt);
```

#### `FUNCTION`: **vmm_map_device** <sub>line 23</sub>
```c
bool vmm_map_device(uint64_t phys, size_t len, uint64_t *out_virt);
```

#### `FUNCTION`: **vmm_free** <sub>line 25</sub>
```c
bool vmm_free(uint64_t virt, size_t bytes);
```

#### `FUNCTION`: **vmm_unmap_device** <sub>line 26</sub>
```c
bool vmm_unmap_device(uint64_t virt, size_t len);
```

#### `FUNCTION`: **vmm_translate** <sub>line 28</sub>
```c
uint64_t vmm_translate(uint64_t virt);
```

#### `FUNCTION`: **vmm_stat_total_pages** <sub>line 30</sub>
```c
size_t vmm_stat_total_pages(void);
```

#### `FUNCTION`: **vmm_stat_free_pages** <sub>line 31</sub>
```c
size_t vmm_stat_free_pages(void);
```

#### `FUNCTION`: **vmm_stat_allocated_pages** <sub>line 32</sub>
```c
size_t vmm_stat_allocated_pages(void);
```

#### `FUNCTION`: **vmm_stat_total_bytes** <sub>line 33</sub>
```c
uint64_t vmm_stat_total_bytes(void);
```

#### `FUNCTION`: **vmm_stat_free_bytes** <sub>line 34</sub>
```c
uint64_t vmm_stat_free_bytes(void);
```

#### `FUNCTION`: **vmm_dump** <sub>line 36</sub>
```c
void vmm_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/api.rs</b> (9 items)</summary>

#### `FN`: **kmalloc** <sub>line 5</sub>
```rust
pub fn kmalloc(size: usize) -> Option<*mut u8> {
```

#### `FN`: **kzalloc** <sub>line 11</sub>
```rust
pub fn kzalloc(size: usize) -> Option<*mut u8> {
```

#### `FN`: **krealloc** <sub>line 17</sub>
```rust
pub fn krealloc(ptr: *mut u8, size: usize) -> Option<*mut u8> {
```

#### `FN`: **kfree** <sub>line 23</sub>
```rust
pub fn kfree(ptr: *mut u8) {
```

#### `FN`: **kalloc_pages** <sub>line 27</sub>
```rust
pub fn kalloc_pages(pages: usize) -> Option<*mut u8> {
```

#### `FN`: **kfree_pages** <sub>line 33</sub>
```rust
pub fn kfree_pages(ptr: *mut u8, _pages: usize) {
```

#### `FN`: **virt_to_phys** <sub>line 37</sub>
```rust
pub fn virt_to_phys(ptr: *mut u8) -> u64 {
```

#### `STRUCT`: **KernelAlloc** <sub>line 41</sub>
```rust
pub struct KernelAlloc;
```

#### `FN`: **self_test** <sub>line 72</sub>
```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/aarch64/memory.h</b> (9 items)</summary>

#### `FUNCTION`: **arch_memory_ready** <sub>line 25</sub>
```c
bool arch_memory_ready(void);
```

#### `FUNCTION`: **arch_memory_boot_alloc** <sub>line 27</sub>
```c
bool arch_memory_boot_alloc(uint64_t len, uint64_t align, uint64_t *out);
```

#### `FUNCTION`: **arch_memory_reserve_range** <sub>line 28</sub>
```c
void arch_memory_reserve_range(uint64_t base, uint64_t len);
```

#### `FUNCTION`: **arch_is_page_aligned** <sub>line 30</sub>
```c
bool arch_is_page_aligned(uint64_t a);
```

#### `FUNCTION`: **arch_page_align_up** <sub>line 31</sub>
```c
uint64_t arch_page_align_up(uint64_t a);
```

#### `FUNCTION`: **arch_page_align_down** <sub>line 32</sub>
```c
uint64_t arch_page_align_down(uint64_t a);
```

#### `FUNCTION`: **arch_virt_to_phys** <sub>line 35</sub>
```c
uint64_t arch_virt_to_phys(void *virt);
```

#### `FUNCTION`: **arch_memory_max_phys** <sub>line 37</sub>
```c
uint64_t arch_memory_max_phys(void);
```

#### `FUNCTION`: **arch_memory_dump** <sub>line 38</sub>
```c
void arch_memory_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/aarch64/paging.c</b> (26 items)</summary>

#### `FUNCTION`: **hw_bits** <sub>line 47</sub>
```c
static uint64_t hw_bits(uint64_t flags, int level)
```

#### `FUNCTION`: **alloc_table_early** <sub>line 89</sub>
```c
static uint64_t alloc_table_early(void)
```

#### `FUNCTION`: **ensure_table** <sub>line 104</sub>
```c
static uint64_t ensure_table(uint64_t *entry, bool early)
```

#### `FUNCTION`: **paging_map_page_in** <sub>line 129</sub>
```c
bool paging_map_page_in(uint64_t pml4, uint64_t virt, uint64_t phys, uint64_t flags)
```

#### `FUNCTION`: **paging_unmap_page_in** <sub>line 156</sub>
```c
bool paging_unmap_page_in(uint64_t pml4, uint64_t virt)
```

#### `FUNCTION`: **paging_translate_in** <sub>line 200</sub>
```c
uint64_t paging_translate_in(uint64_t pml4, uint64_t virt)
```

#### `FUNCTION`: **paging_is_mapped_in** <sub>line 240</sub>
```c
bool paging_is_mapped_in(uint64_t pml4, uint64_t virt)
```

#### `FUNCTION`: **paging_translate_in** <sub>line 242</sub>
```c
return paging_translate_in(pml4, virt) != UINT64_MAX;
```

#### `FUNCTION`: **paging_set_flags_in** <sub>line 245</sub>
```c
bool paging_set_flags_in(uint64_t pml4, uint64_t virt, uint64_t flags)
```

#### `FUNCTION`: **paging_init** <sub>line 267</sub>
```c
void paging_init(uint64_t boot_phys_offset)
```

#### `FUNCTION`: **volatile** <sub>line 273</sub>
```c
__asm__ volatile("mrs %0, ttbr1_el1" : "=r"(cur));
```

#### `FUNCTION`: **volatile** <sub>line 306</sub>
```c
__asm__ volatile("msr mair_el1, %0" :: "r"(mair));
```

#### `FUNCTION`: **volatile** <sub>line 320</sub>
```c
__asm__ volatile("msr tcr_el1, %0" :: "r"(tcr));
```

#### `FUNCTION`: **volatile** <sub>line 322</sub>
```c
__asm__ volatile("msr ttbr0_el1, %0" :: "r"(kernel_l0));
```

#### `FUNCTION`: **volatile** <sub>line 323</sub>
```c
__asm__ volatile("msr ttbr1_el1, %0" :: "r"(kernel_l0));
```

#### `FUNCTION`: **volatile** <sub>line 325</sub>
```c
__asm__ volatile("isb");
```

#### `FUNCTION`: **volatile** <sub>line 326</sub>
```c
__asm__ volatile("tlbi vmalle1");
```

#### `FUNCTION`: **volatile** <sub>line 327</sub>
```c
__asm__ volatile("dsb sy");
```

#### `FUNCTION`: **volatile** <sub>line 328</sub>
```c
__asm__ volatile("isb");
```

#### `FUNCTION`: **paging_enable_nx** <sub>line 331</sub>
```c
bool paging_enable_nx(void)
```

#### `FUNCTION`: **paging_read_cr3** <sub>line 336</sub>
```c
uint64_t paging_read_cr3(void)
```

#### `FUNCTION`: **volatile** <sub>line 340</sub>
```c
__asm__ volatile("mrs %0, ttbr1_el1" : "=r"(v));
```

#### `FUNCTION`: **paging_write_cr3** <sub>line 345</sub>
```c
void paging_write_cr3(uint64_t pml4_phys)
```

#### `FUNCTION`: **volatile** <sub>line 347</sub>
```c
__asm__ volatile("msr ttbr0_el1, %0" :: "r"(pml4_phys));
```

#### `FUNCTION`: **volatile** <sub>line 348</sub>
```c
__asm__ volatile("msr ttbr1_el1, %0" :: "r"(pml4_phys));
```

#### `FUNCTION`: **volatile** <sub>line 349</sub>
```c
__asm__ volatile("isb");
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/aarch64/paging.h</b> (9 items)</summary>

#### `FUNCTION`: **paging_init** <sub>line 17</sub>
```c
void paging_init(uint64_t boot_phys_offset);
```

#### `FUNCTION`: **paging_enable_nx** <sub>line 19</sub>
```c
bool paging_enable_nx(void);
```

#### `FUNCTION`: **paging_read_cr3** <sub>line 21</sub>
```c
uint64_t paging_read_cr3(void);
```

#### `FUNCTION`: **paging_write_cr3** <sub>line 22</sub>
```c
void paging_write_cr3(uint64_t pml4_phys);
```

#### `FUNCTION`: **paging_map_page_in** <sub>line 24</sub>
```c
bool paging_map_page_in(uint64_t pml4, uint64_t virt, uint64_t phys, uint64_t flags);
```

#### `FUNCTION`: **paging_unmap_page_in** <sub>line 25</sub>
```c
bool paging_unmap_page_in(uint64_t pml4, uint64_t virt);
```

#### `FUNCTION`: **paging_set_flags_in** <sub>line 26</sub>
```c
bool paging_set_flags_in(uint64_t pml4, uint64_t virt, uint64_t flags);
```

#### `FUNCTION`: **paging_is_mapped_in** <sub>line 27</sub>
```c
bool paging_is_mapped_in(uint64_t pml4, uint64_t virt);
```

#### `FUNCTION`: **paging_translate_in** <sub>line 28</sub>
```c
uint64_t paging_translate_in(uint64_t pml4, uint64_t virt);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/aarch64/tlb.c</b> (16 items)</summary>

#### `FUNCTION`: **tlb_init** <sub>line 5</sub>
```c
bool tlb_init(void)
```

#### `FUNCTION`: **tlb_ready** <sub>line 11</sub>
```c
bool tlb_ready(void)
```

#### `FUNCTION`: **tlb_flush_all** <sub>line 16</sub>
```c
void tlb_flush_all(void)
```

#### `FUNCTION`: **volatile** <sub>line 18</sub>
```c
__asm__ volatile("dsb sy");
```

#### `FUNCTION`: **volatile** <sub>line 19</sub>
```c
__asm__ volatile("tlbi vmalle1");
```

#### `FUNCTION`: **volatile** <sub>line 20</sub>
```c
__asm__ volatile("dsb sy");
```

#### `FUNCTION`: **volatile** <sub>line 21</sub>
```c
__asm__ volatile("isb");
```

#### `FUNCTION`: **tlb_flush_all_including_global** <sub>line 24</sub>
```c
void tlb_flush_all_including_global(void)
```

#### `FUNCTION`: **tlb_flush_page_addr** <sub>line 29</sub>
```c
void tlb_flush_page_addr(uint64_t addr)
```

#### `FUNCTION`: **volatile** <sub>line 31</sub>
```c
__asm__ volatile("dsb ishst");
```

#### `FUNCTION`: **volatile** <sub>line 32</sub>
```c
__asm__ volatile("tlbi vaae1is, %0" :: "r"(addr >> 12));
```

#### `FUNCTION`: **volatile** <sub>line 33</sub>
```c
__asm__ volatile("dsb ish");
```

#### `FUNCTION`: **volatile** <sub>line 34</sub>
```c
__asm__ volatile("isb");
```

#### `FUNCTION`: **tlb_flush_page** <sub>line 37</sub>
```c
void tlb_flush_page(const void *addr)
```

#### `FUNCTION`: **tlb_flush_range_addr** <sub>line 42</sub>
```c
void tlb_flush_range_addr(uint64_t addr, size_t pages)
```

#### `FUNCTION`: **tlb_flush_range** <sub>line 54</sub>
```c
void tlb_flush_range(const void *addr, size_t pages)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/aarch64/tlb.h</b> (8 items)</summary>

#### `FUNCTION`: **tlb_init** <sub>line 8</sub>
```c
bool tlb_init(void);
```

#### `FUNCTION`: **tlb_ready** <sub>line 9</sub>
```c
bool tlb_ready(void);
```

#### `FUNCTION`: **tlb_flush_all** <sub>line 11</sub>
```c
void tlb_flush_all(void);
```

#### `FUNCTION`: **tlb_flush_all_including_global** <sub>line 12</sub>
```c
void tlb_flush_all_including_global(void);
```

#### `FUNCTION`: **tlb_flush_page** <sub>line 14</sub>
```c
void tlb_flush_page(const void *addr);
```

#### `FUNCTION`: **tlb_flush_page_addr** <sub>line 15</sub>
```c
void tlb_flush_page_addr(uint64_t addr);
```

#### `FUNCTION`: **tlb_flush_range** <sub>line 17</sub>
```c
void tlb_flush_range(const void *addr, size_t pages);
```

#### `FUNCTION`: **tlb_flush_range_addr** <sub>line 18</sub>
```c
void tlb_flush_range_addr(uint64_t addr, size_t pages);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/x86_64/memory.c</b> (21 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 106</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **arch_mem_panic** <sub>line 108</sub>
```c
static void arch_mem_panic(const char *msg) __attribute__((noreturn));
```

#### `FUNCTION`: **arch_mem_panic** <sub>line 109</sub>
```c
static void arch_mem_panic(const char *msg)
```

#### `FUNCTION`: **arch_memory_boot_alloc** <sub>line 127</sub>
```c
bool arch_memory_boot_alloc(uint64_t len, uint64_t align, uint64_t *out_base)
```

#### `FUNCTION`: **safe_end** <sub>line 145</sub>
```c
static uint64_t safe_end(uint64_t base, uint64_t len)
```

#### `FUNCTION`: **u64_max** <sub>line 154</sub>
```c
static uint64_t u64_max(uint64_t a, uint64_t b)
```

#### `FUNCTION`: **u64_min** <sub>line 159</sub>
```c
static uint64_t u64_min(uint64_t a, uint64_t b)
```

#### `FUNCTION`: **align_up_page_safe** <sub>line 164</sub>
```c
static uint64_t align_up_page_safe(uint64_t v)
```

#### `FUNCTION`: **align_down_page** <sub>line 173</sub>
```c
static uint64_t align_down_page(uint64_t v)
```

#### `FUNCTION`: **align_up_generic** <sub>line 178</sub>
```c
static uint64_t align_up_generic(uint64_t v, uint64_t align)
```

#### `FUNCTION`: **raw_type_to_arch** <sub>line 193</sub>
```c
static arch_mem_type_t raw_type_to_arch(uint32_t raw_type)
```

#### `FUNCTION`: **sort_raw_entries** <sub>line 215</sub>
```c
static void sort_raw_entries(arch_raw_mem_entry_t *entries, size_t count)
```

#### `FUNCTION`: **map_reserve_range** <sub>line 270</sub>
```c
static void map_reserve_range(uint64_t base, uint64_t len)
```

#### `FUNCTION`: **map_align_usable_regions** <sub>line 343</sub>
```c
static void map_align_usable_regions(void)
```

#### `FUNCTION`: **recalc_stats** <sub>line 411</sub>
```c
static void recalc_stats(void)
```

#### `FUNCTION`: **arch_memory_ready** <sub>line 524</sub>
```c
bool arch_memory_ready(void)
```

#### `FUNCTION`: **arch_memory_regions** <sub>line 538</sub>
```c
size_t arch_memory_regions(const arch_mem_region_t **out)
```

#### `FUNCTION`: **arch_memory_total_usable** <sub>line 547</sub>
```c
uint64_t arch_memory_total_usable(void)
```

#### `FUNCTION`: **arch_memory_range_is_usable** <sub>line 552</sub>
```c
bool arch_memory_range_is_usable(uint64_t base, uint64_t len)
```

#### `FUNCTION`: **arch_memory_reserve_range** <sub>line 585</sub>
```c
void arch_memory_reserve_range(uint64_t base, uint64_t len)
```

#### `FUNCTION`: **arch_memory_dump** <sub>line 659</sub>
```c
void arch_memory_dump(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/x86_64/memory.h</b> (11 items)</summary>

#### `FUNCTION`: **arch_memory_ready** <sub>line 71</sub>
```c
bool arch_memory_ready(void);
```

#### `FUNCTION`: **arch_memory_boot_alloc** <sub>line 73</sub>
```c
bool arch_memory_boot_alloc(uint64_t len, uint64_t align, uint64_t *out_base);
```

#### `FUNCTION`: **arch_memory_regions** <sub>line 76</sub>
```c
size_t arch_memory_regions(const arch_mem_region_t **out);
```

#### `FUNCTION`: **arch_memory_total_usable** <sub>line 77</sub>
```c
uint64_t arch_memory_total_usable(void);
```

#### `FUNCTION`: **arch_memory_range_is_usable** <sub>line 78</sub>
```c
bool arch_memory_range_is_usable(uint64_t base, uint64_t len);
```

#### `FUNCTION`: **arch_memory_reserve_range** <sub>line 79</sub>
```c
void arch_memory_reserve_range(uint64_t base, uint64_t len);
```

#### `FUNCTION`: **arch_memory_dump** <sub>line 83</sub>
```c
void arch_memory_dump(void);
```

#### `FUNCTION`: **arch_page_align_down** <sub>line 85</sub>
```c
static inline uint64_t arch_page_align_down(uint64_t v)
```

#### `FUNCTION`: **arch_page_align_up** <sub>line 90</sub>
```c
static inline uint64_t arch_page_align_up(uint64_t v)
```

#### `FUNCTION`: **arch_is_page_aligned** <sub>line 99</sub>
```c
static inline bool arch_is_page_aligned(uint64_t v)
```

#### `FUNCTION`: **arch_virt_to_phys** <sub>line 110</sub>
```c
static inline uint64_t arch_virt_to_phys(const void *virt)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/x86_64/paging.c</b> (25 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 5</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **paging_panic** <sub>line 7</sub>
```c
static void paging_panic(const char *msg) __attribute__((noreturn));
```

#### `FUNCTION`: **paging_panic** <sub>line 8</sub>
```c
static void paging_panic(const char *msg)
```

#### `FUNCTION`: **zero_page_phys** <sub>line 31</sub>
```c
static void zero_page_phys(uint64_t phys)
```

#### `FUNCTION`: **volatile** <sub>line 39</sub>
```c
__asm__ volatile("" ::: "memory");
```

#### `FUNCTION`: **pml4_index** <sub>line 42</sub>
```c
static inline size_t pml4_index(uint64_t virt)
```

#### `FUNCTION`: **pdpt_index** <sub>line 47</sub>
```c
static inline size_t pdpt_index(uint64_t virt)
```

#### `FUNCTION`: **pd_index** <sub>line 52</sub>
```c
static inline size_t pd_index(uint64_t virt)
```

#### `FUNCTION`: **pt_index** <sub>line 57</sub>
```c
static inline size_t pt_index(uint64_t virt)
```

#### `FUNCTION`: **paging_read_cr3** <sub>line 62</sub>
```c
uint64_t paging_read_cr3(void)
```

#### `FUNCTION`: **volatile** <sub>line 66</sub>
```c
__asm__ volatile("mov %%cr3, %0" : "=r"(cr3));
```

#### `FUNCTION`: **paging_flush_tlb_all** <sub>line 71</sub>
```c
void paging_flush_tlb_all(void)
```

#### `FUNCTION`: **paging_flush_page** <sub>line 76</sub>
```c
void paging_flush_page(uint64_t addr)
```

#### `FUNCTION`: **alloc_zeroed_table_page** <sub>line 81</sub>
```c
static uint64_t alloc_zeroed_table_page(void)
```

#### `FUNCTION`: **paging_map_page** <sub>line 163</sub>
```c
bool paging_map_page(uint64_t virt, uint64_t phys, uint64_t flags)
```

#### `FUNCTION`: **map_page_2m** <sub>line 203</sub>
```c
static bool map_page_2m(uint64_t virt, uint64_t phys, uint64_t flags)
```

#### `FUNCTION`: **paging_map_range** <sub>line 246</sub>
```c
bool paging_map_range(uint64_t virt, uint64_t phys, uint64_t len, uint64_t flags)
```

#### `FUNCTION`: **paging_set_boot_phys_offset** <sub>line 299</sub>
```c
void paging_set_boot_phys_offset(uint64_t phys_offset)
```

#### `FUNCTION`: **paging_boot_phys_offset_valid** <sub>line 309</sub>
```c
bool paging_boot_phys_offset_valid(void)
```

#### `FUNCTION`: **paging_init_direct_map** <sub>line 314</sub>
```c
void paging_init_direct_map(void)
```

#### `FUNCTION`: **paging_init** <sub>line 354</sub>
```c
void paging_init(uint64_t phys_offset)
```

#### `FUNCTION`: **volatile** <sub>line 540</sub>
```c
__asm__ volatile("wrmsr" : : "A"(efer), "c"(0xC0000080));
```
> } 
> __asm__ volatile("rdmsr" : "=A"(efer) : "c"(0xC0000080)); 
> efer |= (1ULL << 11); /* NXE bit

#### `FUNCTION`: **paging_nx_enabled** <sub>line 544</sub>
```c
bool paging_nx_enabled(void)
```

#### `FUNCTION`: **paging_write_cr3** <sub>line 549</sub>
```c
void paging_write_cr3(uint64_t pml4_phys)
```

#### `FUNCTION`: **volatile** <sub>line 551</sub>
```c
__asm__ volatile("mov %0, %%cr3" : : "r"(pml4_phys) : "memory");
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/x86_64/paging.h</b> (39 items)</summary>

#### `FUNCTION`: **paging_set_boot_phys_offset** <sub>line 35</sub>
```c
void paging_set_boot_phys_offset(uint64_t phys_offset);
```

#### `FUNCTION`: **paging_boot_phys_offset_valid** <sub>line 37</sub>
```c
bool paging_boot_phys_offset_valid(void);
```

#### `FUNCTION`: **paging_init_direct_map** <sub>line 39</sub>
```c
void paging_init_direct_map(void);
```

#### `FUNCTION`: **paging_init** <sub>line 41</sub>
```c
void paging_init(uint64_t boot_phys_offset);
```

#### `FUNCTION`: **paging_read_cr3** <sub>line 43</sub>
```c
uint64_t paging_read_cr3(void);
```

#### `FUNCTION`: **paging_flush_tlb_all** <sub>line 44</sub>
```c
void paging_flush_tlb_all(void);
```

#### `FUNCTION`: **paging_flush_page** <sub>line 45</sub>
```c
void paging_flush_page(uint64_t addr);
```

#### `FUNCTION`: **paging_map_page** <sub>line 47</sub>
```c
bool paging_map_page(uint64_t virt, uint64_t phys, uint64_t flags);
```

#### `FUNCTION`: **paging_map_range** <sub>line 49</sub>
```c
bool paging_map_range(uint64_t virt, uint64_t phys, uint64_t len, uint64_t flags);
```

#### `FUNCTION`: **paging_translate** <sub>line 51</sub>
```c
uint64_t paging_translate(uint64_t virt);
```

#### `FUNCTION`: **paging_is_mapped** <sub>line 52</sub>
```c
bool paging_is_mapped(uint64_t virt);
```

#### `FUNCTION`: **paging_unmap_page** <sub>line 53</sub>
```c
bool paging_unmap_page(uint64_t virt);
```

#### `FUNCTION`: **paging_set_flags** <sub>line 54</sub>
```c
bool paging_set_flags(uint64_t virt, uint64_t flags);
```

#### `FUNCTION`: **paging_get_flags** <sub>line 55</sub>
```c
bool paging_get_flags(uint64_t virt, uint64_t *out_flags);
```

#### `FUNCTION`: **paging_enable_nx** <sub>line 56</sub>
```c
void paging_enable_nx(void);
```

#### `FUNCTION`: **paging_nx_enabled** <sub>line 57</sub>
```c
bool paging_nx_enabled(void);
```

#### `FUNCTION`: **paging_unmap_page_in** <sub>line 63</sub>
```c
bool paging_unmap_page_in(uint64_t pml4, uint64_t virt);
```

#### `FUNCTION`: **paging_translate_in** <sub>line 64</sub>
```c
uint64_t paging_translate_in(uint64_t pml4, uint64_t virt);
```

#### `FUNCTION`: **paging_is_mapped_in** <sub>line 65</sub>
```c
bool paging_is_mapped_in(uint64_t pml4, uint64_t virt);
```

#### `FUNCTION`: **paging_set_flags_in** <sub>line 66</sub>
```c
bool paging_set_flags_in(uint64_t pml4, uint64_t virt, uint64_t flags);
```

#### `FUNCTION`: **paging_get_flags_in** <sub>line 67</sub>
```c
bool paging_get_flags_in(uint64_t pml4, uint64_t virt, uint64_t *out_flags);
```

#### `FUNCTION`: **paging_create_pml4** <sub>line 69</sub>
```c
uint64_t paging_create_pml4(void);
```

#### `FUNCTION`: **paging_destroy_pml4** <sub>line 70</sub>
```c
void paging_destroy_pml4(uint64_t pml4_phys);
```

#### `FUNCTION`: **paging_switch_pml4** <sub>line 71</sub>
```c
void paging_switch_pml4(uint64_t pml4_phys);
```

#### `FUNCTION`: **paging_map_mmio** <sub>line 73</sub>
```c
bool paging_map_mmio(uint64_t virt, uint64_t phys, uint64_t len);
```

#### `FUNCTION`: **paging_enable_write_protect** <sub>line 75</sub>
```c
void paging_enable_write_protect(void);
```

#### `FUNCTION`: **paging_disable_write_protect** <sub>line 76</sub>
```c
void paging_disable_write_protect(void);
```

#### `FUNCTION`: **paging_write_protect_enabled** <sub>line 77</sub>
```c
bool paging_write_protect_enabled(void);
```

#### `FUNCTION`: **paging_enable_smep** <sub>line 79</sub>
```c
void paging_enable_smep(void);
```

#### `FUNCTION`: **paging_enable_smap** <sub>line 80</sub>
```c
void paging_enable_smap(void);
```

#### `FUNCTION`: **paging_smep_enabled** <sub>line 81</sub>
```c
bool paging_smep_enabled(void);
```

#### `FUNCTION`: **paging_smap_enabled** <sub>line 82</sub>
```c
bool paging_smap_enabled(void);
```

#### `FUNCTION`: **paging_pcid_supported** <sub>line 84</sub>
```c
bool paging_pcid_supported(void);
```

#### `FUNCTION`: **paging_enable_pcid** <sub>line 85</sub>
```c
void paging_enable_pcid(void);
```

#### `FUNCTION`: **paging_pcid_enabled** <sub>line 86</sub>
```c
bool paging_pcid_enabled(void);
```

#### `FUNCTION`: **paging_invpcid** <sub>line 87</sub>
```c
void paging_invpcid(uint64_t type, uint64_t pcid, uint64_t addr);
```

#### `FUNCTION`: **paging_la57_supported** <sub>line 90</sub>
```c
bool paging_la57_supported(void);
```

#### `FUNCTION`: **paging_assert_4level_paging** <sub>line 91</sub>
```c
void paging_assert_4level_paging(void);
```

#### `FUNCTION`: **paging_write_cr3** <sub>line 92</sub>
```c
void paging_write_cr3(uint64_t pml4_phys);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/x86_64/tlb.c</b> (14 items)</summary>

#### `FUNCTION`: **tlb_init** <sub>line 39</sub>
```c
bool tlb_init(void)
```

#### `FUNCTION`: **tlb_ready** <sub>line 58</sub>
```c
bool tlb_ready(void)
```

#### `FUNCTION`: **tlb_has_pcid** <sub>line 63</sub>
```c
bool tlb_has_pcid(void)
```

#### `FUNCTION`: **tlb_has_invpcid** <sub>line 68</sub>
```c
bool tlb_has_invpcid(void)
```

#### `FUNCTION`: **tlb_flush_all** <sub>line 73</sub>
```c
void tlb_flush_all(void)
```

#### `FUNCTION`: **tlb_flush_all_including_global** <sub>line 84</sub>
```c
void tlb_flush_all_including_global(void)
```

#### `FUNCTION`: **tlb_flush_page_addr** <sub>line 101</sub>
```c
void tlb_flush_page_addr(uint64_t addr)
```

#### `FUNCTION`: **tlb_flush_page** <sub>line 106</sub>
```c
void tlb_flush_page(const void *addr)
```

#### `FUNCTION`: **tlb_flush_range_addr** <sub>line 116</sub>
```c
void tlb_flush_range_addr(uint64_t addr, size_t pages)
```

#### `FUNCTION`: **tlb_flush_range** <sub>line 140</sub>
```c
void tlb_flush_range(const void *addr, size_t pages)
```

#### `FUNCTION`: **tlb_flush_pcid** <sub>line 150</sub>
```c
void tlb_flush_pcid(uint16_t pcid)
```

#### `FUNCTION`: **tlb_flush_pcid_addr** <sub>line 163</sub>
```c
void tlb_flush_pcid_addr(uint16_t pcid, uint64_t addr)
```

#### `FUNCTION`: **tlb_wbinvd** <sub>line 174</sub>
```c
void tlb_wbinvd(void)
```

#### `FUNCTION`: **tlb_clflush** <sub>line 179</sub>
```c
void tlb_clflush(uint64_t addr)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/x86_64/tlb.h</b> (14 items)</summary>

#### `FUNCTION`: **tlb_init** <sub>line 14</sub>
```c
bool tlb_init(void);
```

#### `FUNCTION`: **tlb_ready** <sub>line 15</sub>
```c
bool tlb_ready(void);
```

#### `FUNCTION`: **tlb_has_pcid** <sub>line 16</sub>
```c
bool tlb_has_pcid(void);
```

#### `FUNCTION`: **tlb_has_invpcid** <sub>line 17</sub>
```c
bool tlb_has_invpcid(void);
```

#### `FUNCTION`: **tlb_flush_all** <sub>line 19</sub>
```c
void tlb_flush_all(void);
```

#### `FUNCTION`: **tlb_flush_all_including_global** <sub>line 20</sub>
```c
void tlb_flush_all_including_global(void);
```

#### `FUNCTION`: **tlb_flush_page** <sub>line 22</sub>
```c
void tlb_flush_page(const void *addr);
```

#### `FUNCTION`: **tlb_flush_page_addr** <sub>line 23</sub>
```c
void tlb_flush_page_addr(uint64_t addr);
```

#### `FUNCTION`: **tlb_flush_range** <sub>line 25</sub>
```c
void tlb_flush_range(const void *addr, size_t pages);
```

#### `FUNCTION`: **tlb_flush_range_addr** <sub>line 26</sub>
```c
void tlb_flush_range_addr(uint64_t addr, size_t pages);
```

#### `FUNCTION`: **tlb_flush_pcid** <sub>line 28</sub>
```c
void tlb_flush_pcid(uint16_t pcid);
```

#### `FUNCTION`: **tlb_flush_pcid_addr** <sub>line 29</sub>
```c
void tlb_flush_pcid_addr(uint16_t pcid, uint64_t addr);
```

#### `FUNCTION`: **tlb_wbinvd** <sub>line 31</sub>
```c
void tlb_wbinvd(void);
```

#### `FUNCTION`: **tlb_clflush** <sub>line 32</sub>
```c
void tlb_clflush(uint64_t addr);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/arch/x86_64/tlb_asm.h</b> (8 items)</summary>

#### `FUNCTION`: **tlb_asm_invlpg** <sub>line 6</sub>
```c
void tlb_asm_invlpg(uint64_t addr);
```

#### `FUNCTION`: **tlb_asm_read_cr3** <sub>line 8</sub>
```c
uint64_t tlb_asm_read_cr3(void);
```

#### `FUNCTION`: **tlb_asm_write_cr3** <sub>line 9</sub>
```c
void tlb_asm_write_cr3(uint64_t value);
```

#### `FUNCTION`: **tlb_asm_read_cr4** <sub>line 11</sub>
```c
uint64_t tlb_asm_read_cr4(void);
```

#### `FUNCTION`: **tlb_asm_write_cr4** <sub>line 12</sub>
```c
void tlb_asm_write_cr4(uint64_t value);
```

#### `FUNCTION`: **tlb_asm_invpcid** <sub>line 14</sub>
```c
void tlb_asm_invpcid(uint64_t type, const void *desc);
```

#### `FUNCTION`: **tlb_asm_wbinvd** <sub>line 16</sub>
```c
void tlb_asm_wbinvd(void);
```

#### `FUNCTION`: **tlb_asm_clflush** <sub>line 17</sub>
```c
void tlb_asm_clflush(uint64_t addr);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/cache/cache.c</b> (5 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 3</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **cache_init** <sub>line 8</sub>
```c
bool cache_init(void)
```

#### `FUNCTION`: **cache_ready** <sub>line 22</sub>
```c
bool cache_ready(void)
```

#### `FUNCTION`: **cache_register** <sub>line 27</sub>
```c
void cache_register(kcache_t *c)
```

#### `FUNCTION`: **cache_dump** <sub>line 60</sub>
```c
void cache_dump(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/cache/cache.h</b> (4 items)</summary>

#### `FUNCTION`: **cache_init** <sub>line 7</sub>
```c
bool cache_init(void);
```

#### `FUNCTION`: **cache_ready** <sub>line 8</sub>
```c
bool cache_ready(void);
```

#### `FUNCTION`: **cache_register** <sub>line 10</sub>
```c
void cache_register(kcache_t *c);
```

#### `FUNCTION`: **cache_dump** <sub>line 13</sub>
```c
void cache_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/cache/object_cache.c</b> (10 items)</summary>

#### `FUNCTION`: **cache_lock** <sub>line 9</sub>
```c
static void cache_lock(void)
```

#### `FUNCTION`: **cache_unlock** <sub>line 29</sub>
```c
static void cache_unlock(void)
```

#### `FUNCTION`: **cache_memset** <sub>line 50</sub>
```c
static void cache_memset(void *dst, uint8_t value, size_t n)
```

#### `FUNCTION`: **cache_strlen** <sub>line 59</sub>
```c
static size_t cache_strlen(const char *s)
```

#### `FUNCTION`: **kcache_grow** <sub>line 70</sub>
```c
static bool kcache_grow(kcache_t *c)
```

#### `FUNCTION`: **kcache_destroy** <sub>line 167</sub>
```c
void kcache_destroy(kcache_t *c)
```

#### `FUNCTION`: **kcache_free** <sub>line 241</sub>
```c
void kcache_free(kcache_t *c, void *ptr)
```

#### `FUNCTION`: **kcache_object_size** <sub>line 264</sub>
```c
size_t kcache_object_size(kcache_t *c)
```

#### `FUNCTION`: **kcache_free_count** <sub>line 273</sub>
```c
size_t kcache_free_count(kcache_t *c)
```

#### `FUNCTION`: **kcache_total_count** <sub>line 282</sub>
```c
size_t kcache_total_count(kcache_t *c)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/cache/object_cashe.h</b> (6 items)</summary>

#### `FUNCTION`: **void** <sub>line 12</sub>
```c
typedef void (*kcache_ctor_t)(void *obj);
```

#### `FUNCTION`: **kcache_destroy** <sub>line 47</sub>
```c
void kcache_destroy(kcache_t *c);
```

#### `FUNCTION`: **kcache_free** <sub>line 51</sub>
```c
void kcache_free(kcache_t *c, void *ptr);
```

#### `FUNCTION`: **kcache_object_size** <sub>line 53</sub>
```c
size_t kcache_object_size(kcache_t *c);
```

#### `FUNCTION`: **kcache_free_count** <sub>line 54</sub>
```c
size_t kcache_free_count(kcache_t *c);
```

#### `FUNCTION`: **kcache_total_count** <sub>line 55</sub>
```c
size_t kcache_total_count(kcache_t *c);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/cache/per_cpu.c</b> (3 items)</summary>

#### `FUNCTION`: **per_cpu_init** <sub>line 5</sub>
```c
bool per_cpu_init(void)
```

#### `FUNCTION`: **per_cpu_count** <sub>line 11</sub>
```c
size_t per_cpu_count(void)
```

#### `FUNCTION`: **per_cpu_id** <sub>line 16</sub>
```c
size_t per_cpu_id(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/cache/per_cpu.h</b> (3 items)</summary>

#### `FUNCTION`: **per_cpu_init** <sub>line 12</sub>
```c
bool per_cpu_init(void);
```

#### `FUNCTION`: **per_cpu_count** <sub>line 13</sub>
```c
size_t per_cpu_count(void);
```

#### `FUNCTION`: **per_cpu_id** <sub>line 14</sub>
```c
size_t per_cpu_id(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/address.c</b> (10 items)</summary>

#### `FUNCTION`: **addr_is_canonical** <sub>line 4</sub>
```c
bool addr_is_canonical(uint64_t a)
```

#### `FUNCTION`: **addr_is_user** <sub>line 11</sub>
```c
bool addr_is_user(uint64_t a)
```

#### `FUNCTION`: **addr_is_canonical** <sub>line 13</sub>
```c
return addr_is_canonical(a) && a < ADDR_USER_MAX;
```

#### `FUNCTION`: **addr_is_kernel** <sub>line 16</sub>
```c
bool addr_is_kernel(uint64_t a)
```

#### `FUNCTION`: **addr_is_canonical** <sub>line 18</sub>
```c
return addr_is_canonical(a) && a >= ADDR_KERNEL_MIN;
```

#### `FUNCTION`: **addr_is_direct_map** <sub>line 21</sub>
```c
bool addr_is_direct_map(uint64_t a)
```

#### `FUNCTION`: **addr_align_up** <sub>line 26</sub>
```c
uint64_t addr_align_up(uint64_t a, uint64_t align)
```

#### `FUNCTION`: **addr_align_down** <sub>line 41</sub>
```c
uint64_t addr_align_down(uint64_t a, uint64_t align)
```

#### `FUNCTION`: **addr_phys_to_direct** <sub>line 50</sub>
```c
uint64_t addr_phys_to_direct(uint64_t phys)
```

#### `FUNCTION`: **addr_direct_to_phys** <sub>line 55</sub>
```c
uint64_t addr_direct_to_phys(uint64_t va)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/address.h</b> (8 items)</summary>

#### `FUNCTION`: **addr_is_canonical** <sub>line 10</sub>
```c
bool addr_is_canonical(uint64_t a);
```

#### `FUNCTION`: **addr_is_user** <sub>line 11</sub>
```c
bool addr_is_user(uint64_t a);
```

#### `FUNCTION`: **addr_is_kernel** <sub>line 12</sub>
```c
bool addr_is_kernel(uint64_t a);
```

#### `FUNCTION`: **addr_is_direct_map** <sub>line 13</sub>
```c
bool addr_is_direct_map(uint64_t a);
```

#### `FUNCTION`: **addr_align_up** <sub>line 15</sub>
```c
uint64_t addr_align_up(uint64_t a, uint64_t align);
```

#### `FUNCTION`: **addr_align_down** <sub>line 16</sub>
```c
uint64_t addr_align_down(uint64_t a, uint64_t align);
```

#### `FUNCTION`: **addr_phys_to_direct** <sub>line 18</sub>
```c
uint64_t addr_phys_to_direct(uint64_t phys);
```

#### `FUNCTION`: **addr_direct_to_phys** <sub>line 19</sub>
```c
uint64_t addr_direct_to_phys(uint64_t va);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/mm.c</b> (8 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 15</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **mm_init** <sub>line 20</sub>
```c
bool mm_init(const mm_boot_params_t *params)
```

#### `FUNCTION`: **mm_ready** <sub>line 97</sub>
```c
bool mm_ready(void)
```

#### `FUNCTION`: **mm_total_ram** <sub>line 102</sub>
```c
uint64_t mm_total_ram(void)
```

#### `FUNCTION`: **pmm_stat_total_bytes** <sub>line 104</sub>
```c
return pmm_stat_total_bytes();
```

#### `FUNCTION`: **mm_free_ram** <sub>line 107</sub>
```c
uint64_t mm_free_ram(void)
```

#### `FUNCTION`: **pmm_stat_free_bytes** <sub>line 109</sub>
```c
return pmm_stat_free_bytes();
```

#### `FUNCTION`: **mm_dump** <sub>line 112</sub>
```c
void mm_dump(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/mm.h</b> (5 items)</summary>

#### `FUNCTION`: **mm_init** <sub>line 26</sub>
```c
bool mm_init(const mm_boot_params_t *params);
```

#### `FUNCTION`: **mm_ready** <sub>line 27</sub>
```c
bool mm_ready(void);
```

#### `FUNCTION`: **mm_total_ram** <sub>line 29</sub>
```c
uint64_t mm_total_ram(void);
```

#### `FUNCTION`: **mm_free_ram** <sub>line 30</sub>
```c
uint64_t mm_free_ram(void);
```

#### `FUNCTION`: **mm_dump** <sub>line 32</sub>
```c
void mm_dump(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/range.c</b> (2 items)</summary>

#### `FUNCTION`: **range_is_canonical** <sub>line 3</sub>
```c
bool range_is_canonical(uint64_t addr)
```

#### `FUNCTION`: **is_pow2** <sub>line 11</sub>
```c
static bool is_pow2(uint64_t v)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/range.h</b> (1 items)</summary>

#### `FUNCTION`: **range_is_canonical** <sub>line 18</sub>
```c
bool range_is_canonical(uint64_t addr);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/region.c</b> (6 items)</summary>

#### `FUNCTION`: **region_make** <sub>line 3</sub>
```c
region_t region_make(uint64_t start, uint64_t len)
```

#### `FUNCTION`: **region_valid** <sub>line 13</sub>
```c
bool region_valid(region_t r)
```

#### `FUNCTION`: **region_len** <sub>line 18</sub>
```c
uint64_t region_len(region_t r)
```

#### `FUNCTION`: **region_contains** <sub>line 27</sub>
```c
bool region_contains(region_t r, uint64_t addr)
```

#### `FUNCTION`: **region_overlaps** <sub>line 32</sub>
```c
bool region_overlaps(region_t a, region_t b)
```

#### `FUNCTION`: **region_intersect** <sub>line 37</sub>
```c
region_t region_intersect(region_t a, region_t b)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/region.h</b> (6 items)</summary>

#### `FUNCTION`: **region_make** <sub>line 13</sub>
```c
region_t region_make(uint64_t start, uint64_t len);
```

#### `FUNCTION`: **region_valid** <sub>line 14</sub>
```c
bool region_valid(region_t r);
```

#### `FUNCTION`: **region_len** <sub>line 15</sub>
```c
uint64_t region_len(region_t r);
```

#### `FUNCTION`: **region_contains** <sub>line 17</sub>
```c
bool region_contains(region_t r, uint64_t addr);
```

#### `FUNCTION`: **region_overlaps** <sub>line 18</sub>
```c
bool region_overlaps(region_t a, region_t b);
```

#### `FUNCTION`: **region_intersect** <sub>line 19</sub>
```c
region_t region_intersect(region_t a, region_t b);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/sizeutil.c</b> (2 items)</summary>

#### `FUNCTION`: **size_is_pow2** <sub>line 3</sub>
```c
bool size_is_pow2(size_t v)
```

#### `FUNCTION`: **size_round_up_pow2** <sub>line 8</sub>
```c
bool size_round_up_pow2(size_t v, size_t *out)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/sizeutil.h</b> (2 items)</summary>

#### `FUNCTION`: **size_is_pow2** <sub>line 8</sub>
```c
bool size_is_pow2(size_t v);
```

#### `FUNCTION`: **size_round_up_pow2** <sub>line 11</sub>
```c
bool size_round_up_pow2(size_t v, size_t *out);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/smp_lock.c</b> (8 items)</summary>

#### `FUNCTION`: **cpu_pause** <sub>line 3</sub>
```c
static inline void cpu_pause(void)
```

#### `FUNCTION`: **volatile** <sub>line 5</sub>
```c
__asm__ volatile("pause" ::: "memory");
```

#### `FUNCTION`: **irq_save_disable** <sub>line 8</sub>
```c
static inline uint64_t irq_save_disable(void)
```

#### `FUNCTION`: **irq_restore** <sub>line 24</sub>
```c
static inline void irq_restore(uint64_t flags)
```

#### `FUNCTION`: **smp_current_cpu_id** <sub>line 35</sub>
```c
int64_t smp_current_cpu_id(void)
```

#### `FUNCTION`: **smp_lock_init** <sub>line 49</sub>
```c
void smp_lock_init(smp_ticket_lock_t *lock)
```

#### `FUNCTION`: **smp_lock_acquire** <sub>line 58</sub>
```c
void smp_lock_acquire(smp_ticket_lock_t *lock)
```

#### `FUNCTION`: **smp_lock_release** <sub>line 80</sub>
```c
bool smp_lock_release(smp_ticket_lock_t *lock)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/core/smp_lock.h</b> (4 items)</summary>

#### `FUNCTION`: **smp_lock_init** <sub>line 21</sub>
```c
void smp_lock_init(smp_ticket_lock_t *lock);
```

#### `FUNCTION`: **smp_lock_acquire** <sub>line 24</sub>
```c
void smp_lock_acquire(smp_ticket_lock_t *lock);
```

#### `FUNCTION`: **smp_lock_release** <sub>line 27</sub>
```c
bool smp_lock_release(smp_ticket_lock_t *lock);
```

#### `FUNCTION`: **smp_current_cpu_id** <sub>line 30</sub>
```c
int64_t smp_current_cpu_id(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/ffi.rs</b> (53 items)</summary>

#### `STRUCT`: **RawMemEntry** <sub>line 5</sub>
```rust
pub struct RawMemEntry {
```

#### `STRUCT`: **MmBootParams** <sub>line 14</sub>
```rust
pub struct MmBootParams {
```

#### `FN`: **mm_init** <sub>line 25</sub>
```rust
pub fn mm_init(params: *const MmBootParams) -> bool;
```

#### `FN`: **mm_ready** <sub>line 26</sub>
```rust
pub fn mm_ready() -> bool;
```

#### `FN`: **mm_total_ram** <sub>line 27</sub>
```rust
pub fn mm_total_ram() -> u64;
```

#### `FN`: **mm_free_ram** <sub>line 28</sub>
```rust
pub fn mm_free_ram() -> u64;
```

#### `FN`: **mm_dump** <sub>line 29</sub>
```rust
pub fn mm_dump();
```

#### `FN`: **arch_memory_ready** <sub>line 31</sub>
```rust
pub fn arch_memory_ready() -> bool;
```

#### `FN`: **arch_memory_reserve_range** <sub>line 32</sub>
```rust
pub fn arch_memory_reserve_range(base: u64, len: u64);
```

#### `FN`: **arch_memory_boot_alloc** <sub>line 33</sub>
```rust
pub fn arch_memory_boot_alloc(len: u64, align: u64, out: *mut u64) -> bool;
```

#### `FN`: **paging_init** <sub>line 35</sub>
```rust
pub fn paging_init(boot_phys_offset: u64);
```

#### `FN`: **paging_enable_nx** <sub>line 36</sub>
```rust
pub fn paging_enable_nx();
```

#### `FN`: **paging_read_cr3** <sub>line 37</sub>
```rust
pub fn paging_read_cr3() -> u64;
```

#### `FN`: **paging_is_mapped** <sub>line 38</sub>
```rust
pub fn paging_is_mapped(virt: u64) -> bool;
```

#### `FN`: **paging_aspace_switch** <sub>line 39</sub>
```rust
pub fn paging_aspace_switch(aspace: *mut c_void);
```

#### `FN`: **paging_map_page** <sub>line 40</sub>
```rust
pub fn paging_map_page(virt: u64, phys: u64, flags: u64) -> bool;
```

#### `FN`: **pmm_init** <sub>line 42</sub>
```rust
pub fn pmm_init() -> bool;
```

#### `FN`: **pmm_alloc_frame** <sub>line 43</sub>
```rust
pub fn pmm_alloc_frame(out: *mut u64) -> bool;
```

#### `FN`: **pmm_alloc_zero_frame** <sub>line 44</sub>
```rust
pub fn pmm_alloc_zero_frame(out: *mut u64) -> bool;
```

#### `FN`: **pmm_alloc_frames** <sub>line 45</sub>
```rust
pub fn pmm_alloc_frames(count: usize, out: *mut u64) -> bool;
```

#### `FN`: **pmm_alloc_frames_aligned** <sub>line 46</sub>
```rust
pub fn pmm_alloc_frames_aligned(count: usize,
```

#### `FN`: **pmm_free_frame** <sub>line 49</sub>
```rust
pub fn pmm_free_frame(phys: u64) -> bool;
```

#### `FN`: **pmm_free_frames** <sub>line 50</sub>
```rust
pub fn pmm_free_frames(phys: u64, count: usize) -> bool;
```

#### `FN`: **vmm_init** <sub>line 53</sub>
```rust
pub fn vmm_init() -> bool;
```

#### `FN`: **vmm_alloc** <sub>line 54</sub>
```rust
pub fn vmm_alloc(bytes: usize, flags: u32, out: *mut u64) -> bool;
```

#### `FN`: **vmm_free** <sub>line 55</sub>
```rust
pub fn vmm_free(virt: u64, bytes: usize) -> bool;
```

#### `FN`: **vmm_map_device** <sub>line 56</sub>
```rust
pub fn vmm_map_device(phys: u64, len: usize, out: *mut u64) -> bool;
```

#### `FN`: **vmm_unmap_device** <sub>line 57</sub>
```rust
pub fn vmm_unmap_device(virt: u64, len: usize) -> bool;
```

#### `FN`: **kmalloc** <sub>line 59</sub>
```rust
pub fn kmalloc(size: usize) -> *mut c_void;
```

#### `FN`: **kzalloc** <sub>line 60</sub>
```rust
pub fn kzalloc(size: usize) -> *mut c_void;
```

#### `FN`: **kcalloc** <sub>line 61</sub>
```rust
pub fn kcalloc(count: usize, size: usize) -> *mut c_void;
```

#### `FN`: **krealloc** <sub>line 62</sub>
```rust
pub fn krealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
```

#### `FN`: **kmalloc_aligned** <sub>line 63</sub>
```rust
pub fn kmalloc_aligned(size: usize, align: usize) -> *mut c_void;
```

#### `FN`: **kfree** <sub>line 64</sub>
```rust
pub fn kfree(ptr: *mut c_void);
```

#### `FN`: **kalloc_pages** <sub>line 65</sub>
```rust
pub fn kalloc_pages(pages: usize) -> *mut c_void;
```

#### `FN`: **kfree_pages** <sub>line 66</sub>
```rust
pub fn kfree_pages(ptr: *mut c_void, pages: usize);
```

#### `FN`: **kvirt_to_phys** <sub>line 67</sub>
```rust
pub fn kvirt_to_phys(ptr: *mut c_void) -> u64;
```

#### `FN`: **contig_alloc** <sub>line 69</sub>
```rust
pub fn contig_alloc(bytes: usize,
```

#### `FN`: **contig_free** <sub>line 73</sub>
```rust
pub fn contig_free(phys: u64, bytes: usize);
```

#### `FN`: **dma_alloc_coherent** <sub>line 74</sub>
```rust
pub fn dma_alloc_coherent(bytes: usize,
```

#### `FN`: **dma_free_coherent** <sub>line 78</sub>
```rust
pub fn dma_free_coherent(phys: u64, virt: *mut c_void, bytes: usize);
```

#### `FN`: **aspace_subsystem_init** <sub>line 81</sub>
```rust
pub fn aspace_subsystem_init() -> bool;
```

#### `FN`: **aspace_create** <sub>line 82</sub>
```rust
pub fn aspace_create() -> *mut c_void;
```

#### `FN`: **aspace_destroy** <sub>line 83</sub>
```rust
pub fn aspace_destroy(pa: *mut c_void);
```

#### `FN`: **aspace_paging_handle** <sub>line 84</sub>
```rust
pub fn aspace_paging_handle(pa: *mut c_void) -> *mut c_void;
```

#### `FN`: **aspace_map_anon** <sub>line 85</sub>
```rust
pub fn aspace_map_anon(pa: *mut c_void,
```

#### `FN`: **aspace_unmap** <sub>line 89</sub>
```rust
pub fn aspace_unmap(pa: *mut c_void, addr: u64, len: usize) -> bool;
```

#### `FN`: **aspace_protect** <sub>line 90</sub>
```rust
pub fn aspace_protect(pa: *mut c_void,
```

#### `FN`: **aspace_brk** <sub>line 94</sub>
```rust
pub fn aspace_brk(pa: *mut c_void, new_brk: u64) -> u64;
```

#### `FN`: **mmap** <sub>line 95</sub>
```rust
pub fn mmap(pa: *mut c_void,
```

#### `FN`: **paging_aspace_map** <sub>line 100</sub>
```rust
pub fn paging_aspace_map(aspace: *mut c_void,
```

#### `FN`: **munmap** <sub>line 105</sub>
```rust
pub fn munmap(pa: *mut c_void, addr: u64, len: usize) -> bool;
```

#### `FN`: **paging_aspace_cr3** <sub>line 106</sub>
```rust
pub fn paging_aspace_cr3(aspace: *mut c_void) -> u64;
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/init.rs</b> (10 items)</summary>

#### `STRUCT`: **Driverspace** <sub>line 9</sub>
```rust
pub struct Driverspace {
```

#### `FN`: **kv** <sub>line 20</sub>
```rust
fn kv(phys: u64) -> *mut u8 {
```

#### `FN`: **prepare** <sub>line 24</sub>
```rust
pub fn prepare() -> Result<(), DsError> {
```

#### `FN`: **self_test** <sub>line 91</sub>
```rust
pub fn self_test() -> Result<(), DsError> {
```

#### `FN`: **ready** <sub>line 134</sub>
```rust
pub fn ready() -> bool {
```

#### `FN`: **k2d_view** <sub>line 138</sub>
```rust
pub fn k2d_view() -> Option<RingView> {
```

#### `FN`: **d2k_view** <sub>line 142</sub>
```rust
pub fn d2k_view() -> Option<RingView> {
```

#### `FN`: **scratch_view** <sub>line 146</sub>
```rust
pub fn scratch_view() -> Option<*mut u8> {
```

#### `FN`: **map_into_ds** <sub>line 150</sub>
```rust
pub fn map_into_ds(va: u64, phys: u64, len: usize, prot: space::ProtFlags) -> bool {
```

#### `FN`: **unmap_from_ds** <sub>line 159</sub>
```rust
pub fn unmap_from_ds(va: u64, len: usize) -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/kprintf.c</b> (10 items)</summary>

#### `FUNCTION`: **inb** <sub>line 14</sub>
```c
static inline uint8_t inb(uint16_t port)
```

#### `FUNCTION`: **volatile** <sub>line 18</sub>
```c
__asm__ volatile("inb %%dx, %0" : "=a"(v) : "d"(port));
```

#### `FUNCTION`: **outb** <sub>line 23</sub>
```c
static inline void outb(uint16_t port, uint8_t val)
```

#### `FUNCTION`: **volatile** <sub>line 25</sub>
```c
__asm__ volatile("outb %0, %%dx" : : "a"(val), "d"(port));
```

#### `FUNCTION`: **serial_init** <sub>line 30</sub>
```c
static void serial_init(void)
```

#### `FUNCTION`: **serial_putc** <sub>line 47</sub>
```c
static void serial_putc(char c)
```

#### `FUNCTION`: **serial_puts** <sub>line 55</sub>
```c
static void serial_puts(const char *s)
```

#### `FUNCTION`: **serial_put_uint** <sub>line 62</sub>
```c
static void serial_put_uint(uint64_t v, unsigned base, bool upper)
```

#### `FUNCTION`: **kprintf** <sub>line 84</sub>
```c
void kprintf(const char *fmt, ...)
```

#### `FUNCTION`: **serial_write_str** <sub>line 184</sub>
```c
void serial_write_str(const char *s)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/mm_bridge.rs</b> (4 items)</summary>

#### `STRUCT`: **RawMemEntry** <sub>line 14</sub>
```rust
pub struct RawMemEntry {
```

#### `IMPL`: **RawMemEntry** <sub>line 21</sub>
```rust
impl RawMemEntry {
```

#### `FN`: **arch_memory_init** <sub>line 33</sub>
```rust
fn arch_memory_init(
```

#### `FN`: **arch_memory_dump** <sub>line 42</sub>
```rust
fn arch_memory_dump();
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/mod.rs</b> (7 items)</summary>

#### `FN`: **init** <sub>line 20</sub>
```rust
pub fn init(params: &ffi::MmBootParams) -> bool {
```

#### `FN`: **ready** <sub>line 25</sub>
```rust
pub fn ready() -> bool {
```

#### `FN`: **dump** <sub>line 30</sub>
```rust
pub fn dump() {
```

#### `FN`: **init_riscv** <sub>line 35</sub>
```rust
pub fn init_riscv() -> bool {
```

#### `STRUCT`: **EntryStorage** <sub>line 51</sub>
```rust
struct EntryStorage(UnsafeCell<[ffi::RawMemEntry; MAX_RAW_ENTRIES]>);
```

#### `FN`: **init_from_boot_info** <sub>line 57</sub>
```rust
pub fn init_from_boot_info(boot_info: &'static bootloader::BootInfo) -> bool {
```

#### `FN`: **self_test** <sub>line 110</sub>
```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/paging/paging.c</b> (15 items)</summary>

#### `FUNCTION`: **prot_to_pte** <sub>line 13</sub>
```c
static uint64_t prot_to_pte(uint32_t prot)
```

#### `FUNCTION`: **page_range_ok** <sub>line 36</sub>
```c
static bool page_range_ok(uint64_t addr, size_t len)
```

#### `FUNCTION`: **paging_subsystem_init** <sub>line 48</sub>
```c
bool paging_subsystem_init(void)
```

#### `FUNCTION`: **free_tables** <sub>line 60</sub>
```c
static void free_tables(uint64_t table_phys, int level)
```

#### `FUNCTION`: **paging_aspace_destroy** <sub>line 109</sub>
```c
void paging_aspace_destroy(address_space_t *as)
```

#### `FUNCTION`: **paging_aspace_switch** <sub>line 130</sub>
```c
void paging_aspace_switch(address_space_t *as)
```

#### `FUNCTION`: **paging_aspace_cr3** <sub>line 140</sub>
```c
uint64_t paging_aspace_cr3(const address_space_t *as)
```

#### `FUNCTION`: **paging_aspace_unmap** <sub>line 187</sub>
```c
bool paging_aspace_unmap(address_space_t *as, uint64_t virt, size_t len)
```

#### `FUNCTION`: **paging_aspace_translate** <sub>line 262</sub>
```c
uint64_t paging_aspace_translate(address_space_t *as, uint64_t virt)
```

#### `FUNCTION`: **paging_translate_in** <sub>line 265</sub>
```c
return paging_translate_in(paging_kernel_pml4, virt);
```

#### `FUNCTION`: **paging_translate_in** <sub>line 268</sub>
```c
return paging_translate_in(as->pml4_phys, virt);
```

#### `FUNCTION`: **paging_kernel_map** <sub>line 271</sub>
```c
bool paging_kernel_map(uint64_t virt, uint64_t phys, size_t len, uint32_t prot)
```

#### `FUNCTION`: **paging_aspace_map** <sub>line 277</sub>
```c
return paging_aspace_map((address_space_t *)&(address_space_t){
```

#### `FUNCTION`: **paging_kernel_unmap** <sub>line 284</sub>
```c
bool paging_kernel_unmap(uint64_t virt, size_t len)
```

#### `FUNCTION`: **paging_aspace_unmap** <sub>line 290</sub>
```c
return paging_aspace_unmap((address_space_t *)&(address_space_t){
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/paging/paging.h</b> (8 items)</summary>

#### `FUNCTION`: **paging_subsystem_init** <sub>line 19</sub>
```c
bool paging_subsystem_init(void);
```

#### `FUNCTION`: **paging_aspace_destroy** <sub>line 22</sub>
```c
void paging_aspace_destroy(address_space_t *as);
```

#### `FUNCTION`: **paging_aspace_switch** <sub>line 23</sub>
```c
void paging_aspace_switch(address_space_t *as);
```

#### `FUNCTION`: **paging_aspace_cr3** <sub>line 24</sub>
```c
uint64_t paging_aspace_cr3(const address_space_t *as);
```

#### `FUNCTION`: **paging_aspace_unmap** <sub>line 32</sub>
```c
bool paging_aspace_unmap(address_space_t *as, uint64_t virt, size_t len);
```

#### `FUNCTION`: **paging_aspace_translate** <sub>line 39</sub>
```c
uint64_t paging_aspace_translate(address_space_t *as, uint64_t virt);
```

#### `FUNCTION`: **paging_kernel_map** <sub>line 41</sub>
```c
bool paging_kernel_map(uint64_t virt, uint64_t phys, size_t len, uint32_t prot);
```

#### `FUNCTION`: **paging_kernel_unmap** <sub>line 42</sub>
```c
bool paging_kernel_unmap(uint64_t virt, size_t len);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/paging/pml.c</b> (5 items)</summary>

#### `FUNCTION`: **pml_level_shift** <sub>line 5</sub>
```c
uint64_t pml_level_shift(int level)
```

#### `FUNCTION`: **pml_index** <sub>line 19</sub>
```c
uint64_t pml_index(uint64_t virt, int level)
```

#### `FUNCTION`: **pml_entry_present** <sub>line 24</sub>
```c
bool pml_entry_present(uint64_t entry)
```

#### `FUNCTION`: **pml_entry_large** <sub>line 29</sub>
```c
bool pml_entry_large(uint64_t entry)
```

#### `FUNCTION`: **pml_entry_addr** <sub>line 34</sub>
```c
uint64_t pml_entry_addr(uint64_t entry)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/paging/pml.h</b> (5 items)</summary>

#### `FUNCTION`: **pml_level_shift** <sub>line 14</sub>
```c
uint64_t pml_level_shift(int level);
```

#### `FUNCTION`: **pml_index** <sub>line 15</sub>
```c
uint64_t pml_index(uint64_t virt, int level);
```

#### `FUNCTION`: **pml_entry_present** <sub>line 17</sub>
```c
bool pml_entry_present(uint64_t entry);
```

#### `FUNCTION`: **pml_entry_large** <sub>line 18</sub>
```c
bool pml_entry_large(uint64_t entry);
```

#### `FUNCTION`: **pml_entry_addr** <sub>line 19</sub>
```c
uint64_t pml_entry_addr(uint64_t entry);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/paging/tlb.c</b> (4 items)</summary>

#### `FUNCTION`: **tlb_batch_begin** <sub>line 4</sub>
```c
void tlb_batch_begin(tlb_batch_t *batch)
```

#### `FUNCTION`: **tlb_batch_full** <sub>line 10</sub>
```c
void tlb_batch_full(tlb_batch_t *batch)
```

#### `FUNCTION`: **tlb_batch_add** <sub>line 15</sub>
```c
void tlb_batch_add(tlb_batch_t *batch, uint64_t virt)
```

#### `FUNCTION`: **tlb_batch_commit** <sub>line 30</sub>
```c
void tlb_batch_commit(tlb_batch_t *batch)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/paging/tlb.h</b> (4 items)</summary>

#### `FUNCTION`: **tlb_batch_begin** <sub>line 16</sub>
```c
void tlb_batch_begin(tlb_batch_t *batch);
```

#### `FUNCTION`: **tlb_batch_add** <sub>line 17</sub>
```c
void tlb_batch_add(tlb_batch_t *batch, uint64_t virt);
```

#### `FUNCTION`: **tlb_batch_full** <sub>line 18</sub>
```c
void tlb_batch_full(tlb_batch_t *batch);
```

#### `FUNCTION`: **tlb_batch_commit** <sub>line 19</sub>
```c
void tlb_batch_commit(tlb_batch_t *batch);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/phys.rs</b> (11 items)</summary>

#### `FN`: **alloc_frame** <sub>line 3</sub>
```rust
pub fn alloc_frame() -> Option<u64> {
```

#### `FN`: **alloc_zero_frame** <sub>line 13</sub>
```rust
pub fn alloc_zero_frame() -> Option<u64> {
```

#### `FN`: **alloc_frames** <sub>line 23</sub>
```rust
pub fn alloc_frames(count: usize) -> Option<u64> {
```

#### `FN`: **alloc_frames_aligned** <sub>line 33</sub>
```rust
pub fn alloc_frames_aligned(count: usize, align: usize) -> Option<u64> {
```

#### `FN`: **free_frame** <sub>line 43</sub>
```rust
pub fn free_frame(phys: u64) -> bool {
```

#### `FN`: **free_frames** <sub>line 47</sub>
```rust
pub fn free_frames(phys: u64, count: usize) -> bool {
```

#### `FN`: **reserve** <sub>line 51</sub>
```rust
pub fn reserve(base: u64, len: u64) {
```

#### `FN`: **total_bytes** <sub>line 55</sub>
```rust
pub fn total_bytes() -> u64 {
```

#### `FN`: **free_bytes** <sub>line 59</sub>
```rust
pub fn free_bytes() -> u64 {
```

#### `FN`: **phys_to_virt** <sub>line 65</sub>
```rust
pub fn phys_to_virt(phys: u64) -> *mut u8 {
```

#### `FN`: **self_test** <sub>line 69</sub>
```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/process/address_space.c</b> (15 items)</summary>

#### `FUNCTION`: **as_lock** <sub>line 20</sub>
```c
static void as_lock(void)
```

#### `FUNCTION`: **as_unlock** <sub>line 25</sub>
```c
static void as_unlock(void)
```

#### `FUNCTION`: **aspace_subsystem_init** <sub>line 30</sub>
```c
bool aspace_subsystem_init(void)
```

#### `FUNCTION`: **unmap_pages_free** <sub>line 92</sub>
```c
static void unmap_pages_free(proc_aspace_t *pa, uint64_t start, uint64_t end)
```

#### `FUNCTION`: **vma_insert** <sub>line 105</sub>
```c
static void vma_insert(proc_aspace_t *pa, vma_t *v)
```

#### `FUNCTION`: **vma_remove** <sub>line 117</sub>
```c
static void vma_remove(proc_aspace_t *pa, vma_t *v)
```

#### `FUNCTION`: **find_gap** <sub>line 146</sub>
```c
static uint64_t find_gap(proc_aspace_t *pa, uint64_t len, uint64_t hint)
```

#### `FUNCTION`: **aspace_destroy** <sub>line 235</sub>
```c
void aspace_destroy(proc_aspace_t *pa)
```

#### `FUNCTION`: **aspace_map_anon** <sub>line 268</sub>
```c
uint64_t aspace_map_anon(proc_aspace_t *pa, uint64_t hint, size_t len, uint32_t prot)
```

#### `FUNCTION`: **aspace_map_at** <sub>line 311</sub>
```c
uint64_t aspace_map_at(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot)
```

#### `FUNCTION`: **aspace_unmap** <sub>line 353</sub>
```c
bool aspace_unmap(proc_aspace_t *pa, uint64_t addr, size_t len)
```

#### `FUNCTION`: **aspace_protect** <sub>line 466</sub>
```c
bool aspace_protect(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot)
```

#### `FUNCTION`: **aspace_protect_checked** <sub>line 468</sub>
```c
return aspace_protect_checked(pa, addr, len, prot, prot, NULL);
```

#### `FUNCTION`: **aspace_stack_base** <sub>line 471</sub>
```c
uint64_t aspace_stack_base(void)
```

#### `FUNCTION`: **aspace_brk** <sub>line 523</sub>
```c
uint64_t aspace_brk(proc_aspace_t *pa, uint64_t new_brk)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/process/address_space.h</b> (9 items)</summary>

#### `FUNCTION`: **aspace_subsystem_init** <sub>line 30</sub>
```c
bool aspace_subsystem_init(void);
```

#### `FUNCTION`: **aspace_destroy** <sub>line 33</sub>
```c
void aspace_destroy(proc_aspace_t *pa);
```

#### `FUNCTION`: **aspace_map_anon** <sub>line 37</sub>
```c
uint64_t aspace_map_anon(proc_aspace_t *pa, uint64_t hint, size_t len, uint32_t prot);
```

#### `FUNCTION`: **aspace_map_at** <sub>line 38</sub>
```c
uint64_t aspace_map_at(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot);
```

#### `FUNCTION`: **aspace_reserve_at** <sub>line 39</sub>
```c
uint64_t aspace_reserve_at(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t flags);
```

#### `FUNCTION`: **aspace_unmap** <sub>line 40</sub>
```c
bool aspace_unmap(proc_aspace_t *pa, uint64_t addr, size_t len);
```

#### `FUNCTION`: **aspace_protect** <sub>line 41</sub>
```c
bool aspace_protect(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot);
```

#### `FUNCTION`: **aspace_stack_base** <sub>line 51</sub>
```c
uint64_t aspace_stack_base(void);
```

#### `FUNCTION`: **aspace_brk** <sub>line 53</sub>
```c
uint64_t aspace_brk(proc_aspace_t *pa, uint64_t new_brk);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/process/mmap.c</b> (5 items)</summary>

#### `FUNCTION`: **aspace_map_at** <sub>line 17</sub>
```c
return aspace_map_at(pa, addr, len, prot);
```

#### `FUNCTION`: **aspace_map_anon** <sub>line 20</sub>
```c
return aspace_map_anon(pa, addr, len, prot);
```

#### `FUNCTION`: **munmap** <sub>line 23</sub>
```c
bool munmap(proc_aspace_t *pa, uint64_t addr, size_t len)
```

#### `FUNCTION`: **aspace_unmap** <sub>line 25</sub>
```c
return aspace_unmap(pa, addr, len);
```

#### `FUNCTION`: **mprotect** <sub>line 28</sub>
```c
bool mprotect(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/process/mmap.h</b> (2 items)</summary>

#### `FUNCTION`: **munmap** <sub>line 20</sub>
```c
bool munmap(proc_aspace_t *pa, uint64_t addr, size_t len);
```

#### `FUNCTION`: **mprotect** <sub>line 21</sub>
```c
bool mprotect(proc_aspace_t *pa, uint64_t addr, size_t len, uint32_t prot);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/protection/guard.c</b> (3 items)</summary>

#### `FUNCTION`: **guard_install** <sub>line 4</sub>
```c
uint64_t guard_install(proc_aspace_t *pa, uint64_t addr, size_t len)
```

#### `FUNCTION`: **aspace_reserve_at** <sub>line 6</sub>
```c
return aspace_reserve_at(pa, addr, len, VMA_FLAG_GUARD);
```

#### `FUNCTION`: **guard_user_stack** <sub>line 9</sub>
```c
bool guard_user_stack(proc_aspace_t *pa)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/protection/guard.h</b> (2 items)</summary>

#### `FUNCTION`: **guard_install** <sub>line 9</sub>
```c
uint64_t guard_install(proc_aspace_t *pa, uint64_t addr, size_t len);
```

#### `FUNCTION`: **guard_user_stack** <sub>line 10</sub>
```c
bool guard_user_stack(proc_aspace_t *pa);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/protection/isolation.c</b> (12 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 5</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **read_cr4** <sub>line 20</sub>
```c
static uint64_t read_cr4(void)
```

#### `FUNCTION`: **volatile** <sub>line 24</sub>
```c
__asm__ volatile("mov %%cr4, %0" : "=r"(v));
```

#### `FUNCTION`: **write_cr4** <sub>line 29</sub>
```c
static void write_cr4(uint64_t v)
```

#### `FUNCTION`: **volatile** <sub>line 31</sub>
```c
__asm__ volatile("mov %0, %%cr4" :: "r"(v) : "memory");
```

#### `FUNCTION`: **isolation_has_smep** <sub>line 34</sub>
```c
bool isolation_has_smep(void)
```

#### `FUNCTION`: **isolation_has_smap** <sub>line 43</sub>
```c
bool isolation_has_smap(void)
```

#### `FUNCTION`: **isolation_enable_smep** <sub>line 52</sub>
```c
bool isolation_enable_smep(void)
```

#### `FUNCTION`: **isolation_enable_smap** <sub>line 63</sub>
```c
bool isolation_enable_smap(void)
```

#### `FUNCTION`: **audit_tables** <sub>line 74</sub>
```c
static size_t audit_tables(uint64_t table_phys, int level, size_t violations)
```

#### `FUNCTION`: **isolation_audit_kernel** <sub>line 98</sub>
```c
size_t isolation_audit_kernel(void)
```

#### `FUNCTION`: **isolation_init** <sub>line 125</sub>
```c
bool isolation_init(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/protection/isolation.h</b> (6 items)</summary>

#### `FUNCTION`: **isolation_init** <sub>line 8</sub>
```c
bool isolation_init(void);
```

#### `FUNCTION`: **isolation_has_smep** <sub>line 10</sub>
```c
bool isolation_has_smep(void);
```

#### `FUNCTION`: **isolation_has_smap** <sub>line 11</sub>
```c
bool isolation_has_smap(void);
```

#### `FUNCTION`: **isolation_enable_smep** <sub>line 12</sub>
```c
bool isolation_enable_smep(void);
```

#### `FUNCTION`: **isolation_enable_smap** <sub>line 13</sub>
```c
bool isolation_enable_smap(void);
```

#### `FUNCTION`: **isolation_audit_kernel** <sub>line 15</sub>
```c
size_t isolation_audit_kernel(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/protection/permissions.c</b> (6 items)</summary>

#### `FUNCTION`: **perm_is_wx** <sub>line 5</sub>
```c
bool perm_is_wx(uint32_t prot)
```

#### `FUNCTION`: **perm_sanitize** <sub>line 10</sub>
```c
uint32_t perm_sanitize(uint32_t prot)
```

#### `FUNCTION`: **perm_mprotect_allowed** <sub>line 19</sub>
```c
bool perm_mprotect_allowed(uint32_t old_prot, uint32_t new_prot)
```

#### `FUNCTION`: **perm_set_strict_wx** <sub>line 34</sub>
```c
void perm_set_strict_wx(bool on)
```

#### `FUNCTION`: **perm_kernel_default** <sub>line 39</sub>
```c
uint32_t perm_kernel_default(void)
```

#### `FUNCTION`: **perm_user_default** <sub>line 44</sub>
```c
uint32_t perm_user_default(void)
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/protection/permissions.h</b> (6 items)</summary>

#### `FUNCTION`: **perm_sanitize** <sub>line 8</sub>
```c
uint32_t perm_sanitize(uint32_t prot);
```

#### `FUNCTION`: **perm_is_wx** <sub>line 9</sub>
```c
bool perm_is_wx(uint32_t prot);
```

#### `FUNCTION`: **perm_mprotect_allowed** <sub>line 10</sub>
```c
bool perm_mprotect_allowed(uint32_t old_prot, uint32_t new_prot);
```

#### `FUNCTION`: **perm_set_strict_wx** <sub>line 11</sub>
```c
void perm_set_strict_wx(bool on);
```

#### `FUNCTION`: **perm_kernel_default** <sub>line 13</sub>
```c
uint32_t perm_kernel_default(void);
```

#### `FUNCTION`: **perm_user_default** <sub>line 14</sub>
```c
uint32_t perm_user_default(void);
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/riscv.rs</b> (85 items)</summary>

#### `STRUCT`: **FramePool** <sub>line 10</sub>
```rust
struct FramePool([u8; FRAME_POOL_BYTES]);
```

#### `STRUCT`: **State** <sub>line 14</sub>
```rust
struct State {
```

#### `FN`: **pool_base** <sub>line 24</sub>
```rust
fn pool_base() -> u64 {
```

#### `FN`: **init** <sub>line 29</sub>
```rust
pub fn init() {
```

#### `FN`: **bit_set** <sub>line 35</sub>
```rust
fn bit_set(b: &mut [u64; BITMAP_WORDS], idx: usize) {
```

#### `FN`: **bit_clear** <sub>line 39</sub>
```rust
fn bit_clear(b: &mut [u64; BITMAP_WORDS], idx: usize) {
```

#### `FN`: **bit_get** <sub>line 43</sub>
```rust
fn bit_get(b: &[u64; BITMAP_WORDS], idx: usize) -> bool {
```

#### `FN`: **reserve** <sub>line 47</sub>
```rust
pub fn reserve(base: u64, len: u64) {
```

#### `FN`: **scan_run** <sub>line 66</sub>
```rust
fn scan_run(s: &mut State, count: usize, align_frames: usize) -> Option<usize> {
```

#### `FN`: **alloc_frame** <sub>line 91</sub>
```rust
pub fn alloc_frame() -> Option<u64> {
```

#### `FN`: **alloc_zero_frame** <sub>line 95</sub>
```rust
pub fn alloc_zero_frame() -> Option<u64> {
```

#### `FN`: **alloc_frames** <sub>line 103</sub>
```rust
pub fn alloc_frames(count: usize) -> Option<u64> {
```

#### `FN`: **alloc_frames_aligned** <sub>line 108</sub>
```rust
pub fn alloc_frames_aligned(count: usize, align_frames: usize) -> Option<u64> {
```

#### `FN`: **free_frame** <sub>line 121</sub>
```rust
pub fn free_frame(pa: u64) -> bool {
```

#### `FN`: **free_frames** <sub>line 125</sub>
```rust
pub fn free_frames(pa: u64, count: usize) -> bool {
```

#### `FN`: **total_bytes** <sub>line 152</sub>
```rust
pub fn total_bytes() -> u64 {
```

#### `FN`: **free_bytes** <sub>line 156</sub>
```rust
pub fn free_bytes() -> u64 {
```

#### `FN`: **phys_to_virt** <sub>line 165</sub>
```rust
pub fn phys_to_virt(phys: u64) -> *mut u8 {
```

#### `STRUCT`: **HeapPool** <sub>line 180</sub>
```rust
struct HeapPool([u8; HEAP_BYTES]);
```

#### `STRUCT`: **HeapState** <sub>line 184</sub>
```rust
struct HeapState {
```

#### `FN`: **pool_base** <sub>line 190</sub>
```rust
fn pool_base() -> usize {
```

#### `FN`: **init** <sub>line 195</sub>
```rust
pub fn init() {
```

#### `FN`: **pool_bytes** <sub>line 206</sub>
```rust
pub fn pool_bytes() -> u64 {
```

#### `FN`: **heap_free_bytes** <sub>line 210</sub>
```rust
pub fn heap_free_bytes() -> u64 {
```

#### `FN`: **push_free** <sub>line 224</sub>
```rust
fn push_free(s: &mut HeapState, block: usize) {
```

#### `FN`: **raw_alloc** <sub>line 231</sub>
```rust
fn raw_alloc(size: usize, align: usize) -> *mut u8 {
```

#### `FN`: **kmalloc** <sub>line 334</sub>
```rust
pub fn kmalloc(size: usize) -> Option<*mut u8> {
```

#### `FN`: **kmalloc_aligned** <sub>line 342</sub>
```rust
pub fn kmalloc_aligned(size: usize, align: usize) -> Option<*mut u8> {
```

#### `FN`: **kzalloc** <sub>line 350</sub>
```rust
pub fn kzalloc(size: usize) -> Option<*mut u8> {
```

#### `FN`: **kcalloc** <sub>line 356</sub>
```rust
pub fn kcalloc(count: usize, size: usize) -> Option<*mut u8> {
```

#### `FN`: **kfree** <sub>line 360</sub>
```rust
pub fn kfree(ptr: *mut u8) {
```

#### `FN`: **krealloc** <sub>line 378</sub>
```rust
pub fn krealloc(ptr: *mut u8, size: usize) -> Option<*mut u8> {
```

#### `FN`: **kalloc_pages** <sub>line 395</sub>
```rust
pub fn kalloc_pages(pages: usize) -> Option<*mut u8> {
```

#### `FN`: **kfree_pages** <sub>line 399</sub>
```rust
pub fn kfree_pages(ptr: *mut u8, _pages: usize) {
```

#### `FN`: **kvirt_to_phys** <sub>line 403</sub>
```rust
pub fn kvirt_to_phys(ptr: *mut u8) -> u64 {
```

#### `STRUCT`: **VmmFlags** <sub>line 420</sub>
```rust
pub struct VmmFlags(u32);
```

#### `IMPL`: **VmmFlags** <sub>line 422</sub>
```rust
impl VmmFlags {
```

#### `IMPL`: **BitOr** <sub>line 435</sub>
```rust
impl BitOr for VmmFlags {
```

#### `TYPE`: **Output** <sub>line 436</sub>
```rust
type Output = VmmFlags;
```

#### `FN`: **bitor** <sub>line 438</sub>
```rust
fn bitor(self, rhs: VmmFlags) -> VmmFlags {
```

#### `STRUCT`: **DeviceMap** <sub>line 452</sub>
```rust
struct DeviceMap {
```

#### `STRUCT`: **State** <sub>line 458</sub>
```rust
struct State {
```

#### `FN`: **init** <sub>line 470</sub>
```rust
pub fn init() {
```

#### `FN`: **overlaps** <sub>line 477</sub>
```rust
fn overlaps(s: &State, va: u64, len: u64) -> bool {
```

#### `FN`: **take_range** <sub>line 483</sub>
```rust
fn take_range(s: &mut State, len: u64) -> Option<u64> {
```

#### `FN`: **alloc** <sub>line 504</sub>
```rust
pub fn alloc(bytes: usize, _flags: VmmFlags) -> Option<u64> {
```

#### `FN`: **free** <sub>line 513</sub>
```rust
pub fn free(va: u64, bytes: usize) -> bool {
```

#### `FN`: **map_device** <sub>line 525</sub>
```rust
pub fn map_device(phys: u64, len: usize) -> Option<u64> {
```

#### `FN`: **unmap_device** <sub>line 536</sub>
```rust
pub fn unmap_device(va: u64, len: usize) -> bool {
```

#### `STRUCT`: **ProtFlags** <sub>line 570</sub>
```rust
pub struct ProtFlags(u32);
```

#### `IMPL`: **ProtFlags** <sub>line 572</sub>
```rust
impl ProtFlags {
```

#### `IMPL`: **BitOr** <sub>line 585</sub>
```rust
impl BitOr for ProtFlags {
```

#### `TYPE`: **Output** <sub>line 586</sub>
```rust
type Output = ProtFlags;
```

#### `FN`: **bitor** <sub>line 588</sub>
```rust
fn bitor(self, rhs: ProtFlags) -> ProtFlags {
```

#### `STRUCT`: **MapFlags** <sub>line 601</sub>
```rust
pub struct MapFlags(u32);
```

#### `IMPL`: **MapFlags** <sub>line 603</sub>
```rust
impl MapFlags {
```

#### `IMPL`: **BitOr** <sub>line 614</sub>
```rust
impl BitOr for MapFlags {
```

#### `TYPE`: **Output** <sub>line 615</sub>
```rust
type Output = MapFlags;
```

#### `FN`: **bitor** <sub>line 617</sub>
```rust
fn bitor(self, rhs: MapFlags) -> MapFlags {
```

#### `FN`: **sfence_all** <sub>line 626</sub>
```rust
fn sfence_all() {
```

#### `FN`: **pte_for_prot** <sub>line 632</sub>
```rust
fn pte_for_prot(prot: ProtFlags) -> u64 {
```

#### `STRUCT`: **Range** <sub>line 650</sub>
```rust
struct Range {
```

#### `STRUCT`: **Inner** <sub>line 656</sub>
```rust
struct Inner {
```

#### `STRUCT`: **AddressSpace** <sub>line 663</sub>
```rust
pub struct AddressSpace {
```

#### `FN`: **alloc_table** <sub>line 668</sub>
```rust
fn alloc_table() -> Option<usize> {
```

#### `IMPL`: **AddressSpace** <sub>line 673</sub>
```rust
impl AddressSpace {
```

#### `FN`: **new** <sub>line 674</sub>
```rust
pub fn new() -> Option<Self> {
```

#### `FN`: **handle** <sub>line 687</sub>
```rust
pub fn handle(&self) -> *mut c_void {
```

#### `FN`: **cr3** <sub>line 691</sub>
```rust
pub fn cr3(&self) -> u64 {
```

#### `FN`: **ensure_table** <sub>line 695</sub>
```rust
fn ensure_table(entry: u64, tables: &mut Vec<u64>) -> Option<usize> {
```

#### `FN`: **map_page** <sub>line 704</sub>
```rust
fn map_page(
```

#### `FN`: **translate** <sub>line 732</sub>
```rust
pub fn translate(&self, va: u64) -> Option<u64> {
```

#### `IMPL`: **AddressSpace** <sub>line 757</sub>
```rust
impl AddressSpace {
```

#### `FN`: **map_phys** <sub>line 758</sub>
```rust
pub fn map_phys(&self, virt: u64, phys: u64, len: usize, prot: ProtFlags) -> bool {
```

#### `FN`: **map_anon** <sub>line 773</sub>
```rust
pub fn map_anon(&self, hint: u64, len: usize, prot: ProtFlags) -> Option<u64> {
```

#### `FN`: **mmap** <sub>line 807</sub>
```rust
pub fn mmap(
```

#### `FN`: **munmap** <sub>line 821</sub>
```rust
pub fn munmap(&self, addr: u64, len: usize) -> bool {
```

#### `FN`: **protect** <sub>line 840</sub>
```rust
pub fn protect(&self, addr: u64, len: usize, prot: ProtFlags) -> bool {
```

#### `FN`: **set_pte_flags** <sub>line 852</sub>
```rust
fn set_pte_flags(&self, va: u64, flags: u64) -> bool {
```

#### `FN`: **brk** <sub>line 878</sub>
```rust
pub fn brk(&self, new_brk: u64) -> u64 {
```

#### `FN`: **switch** <sub>line 886</sub>
```rust
pub fn switch(&self) {
```

#### `FN`: **unmap_page_root** <sub>line 891</sub>
```rust
fn unmap_page_root(root: usize, va: u64) -> bool {
```

#### `IMPL`: **Drop** <sub>line 915</sub>
```rust
impl Drop for AddressSpace {
```

#### `FN`: **drop** <sub>line 916</sub>
```rust
fn drop(&mut self) {
```

#### `FN`: **init** <sub>line 931</sub>
```rust
pub fn init() -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/space.rs</b> (25 items)</summary>

#### `STRUCT`: **ProtFlags** <sub>line 7</sub>
```rust
pub struct ProtFlags(u32);
```

#### `IMPL`: **ProtFlags** <sub>line 9</sub>
```rust
impl ProtFlags {
```

#### `IMPL`: **BitOr** <sub>line 23</sub>
```rust
impl BitOr for ProtFlags {
```

#### `TYPE`: **Output** <sub>line 24</sub>
```rust
type Output = ProtFlags;
```

#### `FN`: **bitor** <sub>line 26</sub>
```rust
fn bitor(self, rhs: ProtFlags) -> ProtFlags {
```

#### `STRUCT`: **MapFlags** <sub>line 39</sub>
```rust
pub struct MapFlags(u32);
```

#### `IMPL`: **MapFlags** <sub>line 41</sub>
```rust
impl MapFlags {
```

#### `IMPL`: **BitOr** <sub>line 52</sub>
```rust
impl BitOr for MapFlags {
```

#### `TYPE`: **Output** <sub>line 53</sub>
```rust
type Output = MapFlags;
```

#### `FN`: **bitor** <sub>line 55</sub>
```rust
fn bitor(self, rhs: MapFlags) -> MapFlags {
```

#### `STRUCT`: **AddressSpace** <sub>line 64</sub>
```rust
pub struct AddressSpace {
```

#### `IMPL`: **AddressSpace** <sub>line 68</sub>
```rust
impl AddressSpace {
```

#### `FN`: **new** <sub>line 69</sub>
```rust
pub fn new() -> Option<Self> {
```

#### `FN`: **handle** <sub>line 79</sub>
```rust
pub fn handle(&self) -> *mut c_void {
```

#### `FN`: **cr3** <sub>line 83</sub>
```rust
pub fn cr3(&self) -> u64 {
```

#### `FN`: **map_phys** <sub>line 86</sub>
```rust
pub fn map_phys(&self, virt: u64, phys: u64, len: usize, prot: ProtFlags) -> bool {
```

#### `FN`: **map_anon** <sub>line 90</sub>
```rust
pub fn map_anon(&self, hint: u64, len: usize, prot: ProtFlags) -> Option<u64> {
```

#### `FN`: **mmap** <sub>line 96</sub>
```rust
pub fn mmap(&self, addr: u64, len: usize, prot: ProtFlags, flags: MapFlags) -> Option<u64> {
```

#### `FN`: **munmap** <sub>line 102</sub>
```rust
pub fn munmap(&self, addr: u64, len: usize) -> bool {
```

#### `FN`: **protect** <sub>line 106</sub>
```rust
pub fn protect(&self, addr: u64, len: usize, prot: ProtFlags) -> bool {
```

#### `FN`: **brk** <sub>line 110</sub>
```rust
pub fn brk(&self, new_brk: u64) -> u64 {
```

#### `FN`: **switch** <sub>line 114</sub>
```rust
pub fn switch(&self) {
```

#### `IMPL`: **Drop** <sub>line 120</sub>
```rust
impl Drop for AddressSpace {
```

#### `FN`: **drop** <sub>line 121</sub>
```rust
fn drop(&mut self) {
```

#### `FN`: **self_test** <sub>line 126</sub>
```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/mm/virt.rs</b> (10 items)</summary>

#### `STRUCT`: **VmmFlags** <sub>line 6</sub>
```rust
pub struct VmmFlags(u32);
```

#### `IMPL`: **VmmFlags** <sub>line 8</sub>
```rust
impl VmmFlags {
```

#### `IMPL`: **BitOr** <sub>line 21</sub>
```rust
impl BitOr for VmmFlags {
```

#### `TYPE`: **Output** <sub>line 22</sub>
```rust
type Output = VmmFlags;
```

#### `FN`: **bitor** <sub>line 24</sub>
```rust
fn bitor(self, rhs: VmmFlags) -> VmmFlags {
```

#### `FN`: **alloc** <sub>line 35</sub>
```rust
pub fn alloc(bytes: usize, flags: VmmFlags) -> Option<u64> {
```

#### `FN`: **free** <sub>line 45</sub>
```rust
pub fn free(virt: u64, bytes: usize) -> bool {
```

#### `FN`: **map_device** <sub>line 49</sub>
```rust
pub fn map_device(phys: u64, len: usize) -> Option<u64> {
```

#### `FN`: **unmap_device** <sub>line 59</sub>
```rust
pub fn unmap_device(virt: u64, len: usize) -> bool {
```

#### `FN`: **self_test** <sub>line 63</sub>
```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/arp.rs</b> (10 items)</summary>

#### `STRUCT`: **ArpPacket** <sub>line 13</sub>
```rust
pub struct ArpPacket {
```

#### `IMPL`: **ArpPacket** <sub>line 21</sub>
```rust
impl ArpPacket {
```

#### `FN`: **parse** <sub>line 22</sub>
```rust
pub fn parse(bytes: &[u8]) -> Result<Self, PacketError> {
```

#### `FN`: **write_to** <sub>line 62</sub>
```rust
pub fn write_to(&self, out: &mut [u8]) -> Result<usize, PacketError> {
```

#### `STRUCT`: **CacheEntry** <sub>line 106</sub>
```rust
struct CacheEntry {
```

#### `STRUCT`: **ArpCache** <sub>line 112</sub>
```rust
pub struct ArpCache<const N: usize> {
```

#### `FN`: **lookup** <sub>line 121</sub>
```rust
pub fn lookup(&mut self, ip: Ipv4Address, now_ms: u64) -> Option<MacAddress> {
```

#### `FN`: **insert** <sub>line 134</sub>
```rust
pub fn insert(&mut self, ip: Ipv4Address, mac: MacAddress, expires_at_ms: u64) {
```

#### `FN`: **default** <sub>line 173</sub>
```rust
fn default() -> Self {
```

#### `FN`: **arp_round_trip_and_cache_expiry** <sub>line 183</sub>
```rust
fn arp_round_trip_and_cache_expiry() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/checksum.rs</b> (5 items)</summary>

#### `FN`: **ones_complement_sum** <sub>line 2</sub>
```rust
pub fn ones_complement_sum(bytes: &[u8]) -> u32 {
```

#### `FN`: **fold** <sub>line 18</sub>
```rust
pub fn fold(sum: u32) -> u16 {
```

#### `FN`: **checksum** <sub>line 27</sub>
```rust
pub fn checksum(bytes: &[u8]) -> u16 {
```

#### `FN`: **is_valid** <sub>line 32</sub>
```rust
pub fn is_valid(bytes: &[u8]) -> bool {
```

#### `FN`: **checksum_round_trip** <sub>line 47</sub>
```rust
fn checksum_round_trip() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/command.rs</b> (7 items)</summary>

#### `STRUCT`: **NetworkCommandRunner** <sub>line 7</sub>
```rust
pub struct NetworkCommandRunner<const ARP_ENTRIES: usize> {
```

#### `FN`: **start_ping** <sub>line 24</sub>
```rust
pub fn start_ping(
```

#### `FN`: **poll** <sub>line 49</sub>
```rust
pub fn poll(
```

#### `FN`: **parse_ipv4** <sub>line 69</sub>
```rust
pub fn parse_ipv4(input: &str) -> Option<Ipv4Address> {
```

#### `FN`: **parse_octet** <sub>line 85</sub>
```rust
fn parse_octet(input: &str) -> Option<u8> {
```

#### `FN`: **parses_ipv4** <sub>line 107</sub>
```rust
fn parses_ipv4() {
```

#### `FN`: **rejects_invalid_ipv4** <sub>line 112</sub>
```rust
fn rejects_invalid_ipv4() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/device.rs</b> (11 items)</summary>

#### `STRUCT`: **PollResult** <sub>line 4</sub>
```rust
pub struct PollResult {
```

#### `STRUCT`: **TxFrame** <sub>line 11</sub>
```rust
pub struct TxFrame<'a> {
```

#### `STRUCT`: **RxFrame** <sub>line 23</sub>
```rust
pub struct RxFrame<'a> {
```

#### `TRAIT`: **NetworkDevice** <sub>line 28</sub>
```rust
pub trait NetworkDevice {
```

#### `FN`: **init** <sub>line 29</sub>
```rust
fn init(&mut self) -> Result<(), NetworkError>;
```

#### `FN`: **mac_address** <sub>line 30</sub>
```rust
fn mac_address(&self) -> MacAddress;
```

#### `FN`: **mtu** <sub>line 31</sub>
```rust
fn mtu(&self) -> usize;
```

#### `FN`: **submit_tx** <sub>line 32</sub>
```rust
fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError>;
```

#### `FN`: **poll** <sub>line 33</sub>
```rust
fn poll(&mut self) -> Result<PollResult, NetworkError>;
```

#### `FN`: **take_rx** <sub>line 34</sub>
```rust
fn take_rx(&mut self) -> Option<RxFrame<'_>>;
```

#### `FN`: **recycle_rx** <sub>line 35</sub>
```rust
fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError>;
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/driver.rs</b> (7 items)</summary>

#### `STRUCT`: **DriverRegistry** <sub>line 5</sub>
```rust
pub struct DriverRegistry<'a> {
```

#### `FN`: **new** <sub>line 11</sub>
```rust
pub fn new() -> Self {
```

#### `FN`: **register** <sub>line 18</sub>
```rust
pub fn register(&mut self, device: &'a mut dyn NetworkDevice) -> bool {
```

#### `FN`: **device** <sub>line 27</sub>
```rust
pub fn device(&mut self, index: usize) -> Option<&mut (dyn NetworkDevice + 'a)> {
```

#### `FN`: **len** <sub>line 31</sub>
```rust
pub fn len(&self) -> usize {
```

#### `FN`: **is_empty** <sub>line 35</sub>
```rust
pub fn is_empty(&self) -> bool {
```

#### `FN`: **default** <sub>line 41</sub>
```rust
fn default() -> Self {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/error.rs</b> (2 items)</summary>

#### `ENUM`: **NetworkError** <sub>line 2</sub>
```rust
pub enum NetworkError {
```

#### `ENUM`: **PacketError** <sub>line 18</sub>
```rust
pub enum PacketError {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/ethernet.rs</b> (8 items)</summary>

#### `STRUCT`: **EthernetHeader** <sub>line 11</sub>
```rust
pub struct EthernetHeader {
```

#### `STRUCT`: **EthernetFrame** <sub>line 18</sub>
```rust
pub struct EthernetFrame<'a> {
```

#### `FN`: **parse** <sub>line 24</sub>
```rust
pub fn parse(bytes: &'a [u8]) -> Result<Self, PacketError> {
```

#### `FN`: **is_for** <sub>line 46</sub>
```rust
pub fn is_for(&self, local: MacAddress) -> bool {
```

#### `FN`: **build** <sub>line 51</sub>
```rust
pub fn build(
```

#### `FN`: **pad_to_minimum** <sub>line 66</sub>
```rust
pub fn pad_to_minimum(frame: &mut [u8], logical_len: usize) -> Result<usize, PacketError> {
```

#### `FN`: **self_test** <sub>line 78</sub>
```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

#### `FN`: **ethernet_round_trip_and_padding** <sub>line 97</sub>
```rust
fn ethernet_round_trip_and_padding() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/icmp.rs</b> (6 items)</summary>

#### `STRUCT`: **IcmpPacket** <sub>line 8</sub>
```rust
pub struct IcmpPacket<'a> {
```

#### `FN`: **parse** <sub>line 17</sub>
```rust
pub fn parse(bytes: &'a [u8]) -> Result<Self, PacketError> {
```

#### `FN`: **is_reply_for** <sub>line 37</sub>
```rust
pub fn is_reply_for(&self, identifier: u16, sequence: u16) -> bool {
```

#### `FN`: **write_echo_request** <sub>line 44</sub>
```rust
pub fn write_echo_request(
```

#### `FN`: **write_echo_reply** <sub>line 66</sub>
```rust
pub fn write_echo_reply(out: &mut [u8], request: IcmpPacket<'_>) -> Result<usize, PacketError> {
```

#### `FN`: **echo_request_and_reply_keep_identity_and_payload** <sub>line 83</sub>
```rust
fn echo_request_and_reply_keep_identity_and_payload() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/ipv4.rs</b> (6 items)</summary>

#### `STRUCT`: **Ipv4Packet** <sub>line 9</sub>
```rust
pub struct Ipv4Packet<'a> {
```

#### `FN`: **parse** <sub>line 19</sub>
```rust
pub fn parse(bytes: &'a [u8]) -> Result<Self, PacketError> {
```

#### `STRUCT`: **Ipv4Header** <sub>line 64</sub>
```rust
pub struct Ipv4Header {
```

#### `IMPL`: **Ipv4Header** <sub>line 72</sub>
```rust
impl Ipv4Header {
```

#### `FN`: **write** <sub>line 73</sub>
```rust
pub fn write<'a>(
```

#### `FN`: **ipv4_build_then_parse** <sub>line 105</sub>
```rust
fn ipv4_build_then_parse() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/packet.rs</b> (4 items)</summary>

#### `STRUCT`: **IcmpEchoRequest** <sub>line 8</sub>
```rust
pub struct IcmpEchoRequest<'a> {
```

#### `FN`: **build_icmp_echo** <sub>line 19</sub>
```rust
pub fn build_icmp_echo(
```

#### `FN`: **self_test** <sub>line 43</sub>
```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

#### `FN`: **self_test_passes** <sub>line 69</sub>
```rust
fn self_test_passes() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/ping.rs</b> (6 items)</summary>

#### `ENUM`: **PingResult** <sub>line 9</sub>
```rust
pub enum PingResult {
```

#### `STRUCT`: **PingClient** <sub>line 16</sub>
```rust
pub struct PingClient<const ARP_ENTRIES: usize> {
```

#### `FN`: **start** <sub>line 37</sub>
```rust
pub fn start(
```

#### `FN`: **poll** <sub>line 51</sub>
```rust
pub fn poll(
```

#### `FN`: **send_pending** <sub>line 82</sub>
```rust
fn send_pending(
```

#### `FN`: **map_packet_error** <sub>line 115</sub>
```rust
fn map_packet_error(error: PacketError) -> NetworkError {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/protocols.rs</b> (7 items)</summary>

#### `STRUCT`: **MacAddress** <sub>line 3</sub>
```rust
pub struct MacAddress(pub [u8; 6]);
```

#### `IMPL`: **MacAddress** <sub>line 5</sub>
```rust
impl MacAddress {
```

#### `FN`: **is_broadcast** <sub>line 9</sub>
```rust
pub fn is_broadcast(&self) -> bool {
```

#### `ENUM`: **EtherType** <sub>line 16</sub>
```rust
pub enum EtherType {
```

#### `STRUCT`: **EthernetHeader** <sub>line 24</sub>
```rust
pub struct EthernetHeader {
```

#### `STRUCT`: **Ipv4Header** <sub>line 32</sub>
```rust
pub struct Ipv4Header {
```

#### `STRUCT`: **UdpHeader** <sub>line 47</sub>
```rust
pub struct UdpHeader {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/runtime.rs</b> (6 items)</summary>

#### `STRUCT`: **NetworkRuntime** <sub>line 23</sub>
```rust
struct NetworkRuntime {
```

#### `FN`: **init** <sub>line 30</sub>
```rust
pub fn init() -> Result<(), NetworkError> {
```

#### `FN`: **start_ping** <sub>line 46</sub>
```rust
pub fn start_ping(destination: Ipv4Address, now_ms: u64) -> Result<PingResult, NetworkError> {
```

#### `FN`: **poll** <sub>line 54</sub>
```rust
pub fn poll(now_ms: u64) -> Result<Option<PingResult>, NetworkError> {
```

#### `FN`: **is_ready** <sub>line 63</sub>
```rust
pub fn is_ready() -> bool {
```

#### `FN`: **legacy_virtio_device** <sub>line 67</sub>
```rust
fn legacy_virtio_device() -> Option<PciDevice> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/stack.rs</b> (14 items)</summary>

#### `STRUCT`: **NetworkConfig** <sub>line 11</sub>
```rust
pub struct NetworkConfig {
```

#### `IMPL`: **NetworkConfig** <sub>line 19</sub>
```rust
impl NetworkConfig {
```

#### `ENUM`: **StackEvent** <sub>line 31</sub>
```rust
pub enum StackEvent {
```

#### `STRUCT`: **PingRequest** <sub>line 54</sub>
```rust
pub struct PingRequest<'a> {
```

#### `STRUCT`: **NetworkStack** <sub>line 62</sub>
```rust
pub struct NetworkStack<const ARP_ENTRIES: usize> {
```

#### `FN`: **next_hop_mac** <sub>line 83</sub>
```rust
pub fn next_hop_mac(&mut self, destination: Ipv4Address, now_ms: u64) -> Option<MacAddress> {
```

#### `FN`: **build_arp_request** <sub>line 87</sub>
```rust
pub fn build_arp_request(
```

#### `FN`: **build_ping** <sub>line 103</sub>
```rust
pub fn build_ping(
```

#### `FN`: **process_rx** <sub>line 132</sub>
```rust
pub fn process_rx(
```

#### `FN`: **process_arp** <sub>line 149</sub>
```rust
fn process_arp(
```

#### `FN`: **process_ipv4** <sub>line 182</sub>
```rust
fn process_ipv4(&mut self, frame: EthernetFrame<'_>) -> Result<StackEvent, PacketError> {
```

#### `FN`: **stack** <sub>line 215</sub>
```rust
fn stack() -> NetworkStack<4> {
```

#### `FN`: **outside_subnet_uses_gateway** <sub>line 226</sub>
```rust
fn outside_subnet_uses_gateway() {
```

#### `FN`: **arp_request_and_ping_are_ethernet_padded** <sub>line 232</sub>
```rust
fn arp_request_and_ping_are_ethernet_padded() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/types.rs</b> (9 items)</summary>

#### `STRUCT`: **MacAddress** <sub>line 5</sub>
```rust
pub struct MacAddress(pub [u8; 6]);
```

#### `IMPL`: **MacAddress** <sub>line 7</sub>
```rust
impl MacAddress {
```

#### `IMPL`: **fmt** <sub>line 37</sub>
```rust
impl fmt::Debug for MacAddress {
```

#### `FN`: **fmt** <sub>line 38</sub>
```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

#### `STRUCT`: **Ipv4Address** <sub>line 49</sub>
```rust
pub struct Ipv4Address(pub [u8; 4]);
```

#### `IMPL`: **Ipv4Address** <sub>line 51</sub>
```rust
impl Ipv4Address {
```

#### `IMPL`: **fmt** <sub>line 81</sub>
```rust
impl fmt::Debug for Ipv4Address {
```

#### `FN`: **fmt** <sub>line 82</sub>
```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
```

#### `FN`: **subnet_check_uses_mask** <sub>line 92</sub>
```rust
fn subnet_check_uses_mask() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/virtio/descriptor.rs</b> (1 items)</summary>

#### `STRUCT`: **VirtqDescriptor** <sub>line 7</sub>
```rust
pub struct VirtqDescriptor {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/virtio/device.rs</b> (7 items)</summary>

#### `STRUCT`: **Capabilities** <sub>line 5</sub>
```rust
pub struct Capabilities {
```

#### `TRAIT`: **NetworkDevice** <sub>line 11</sub>
```rust
pub trait NetworkDevice {
```

#### `FN`: **mac_address** <sub>line 12</sub>
```rust
fn mac_address(&self) -> MacAddress;
```

#### `FN`: **mtu** <sub>line 13</sub>
```rust
fn mtu(&self) -> usize;
```

#### `FN`: **capabilities** <sub>line 14</sub>
```rust
fn capabilities(&self) -> Capabilities;
```

#### `FN`: **transmit** <sub>line 15</sub>
```rust
fn transmit(&mut self, packet: &[u8]) -> Result<(), NetworkError>;
```

#### `FN`: **receive** <sub>line 16</sub>
```rust
fn receive(&mut self) -> Option<&[u8]>;
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/virtio/net.rs</b> (27 items)</summary>

#### `STRUCT`: **VirtioNetHeader** <sub>line 48</sub>
```rust
pub struct VirtioNetHeader {
```

#### `STRUCT`: **LegacyQueue** <sub>line 58</sub>
```rust
struct LegacyQueue {
```

#### `IMPL`: **LegacyQueue** <sub>line 65</sub>
```rust
impl LegacyQueue {
```

#### `FN`: **allocate** <sub>line 75</sub>
```rust
fn allocate(size: u16) -> Result<Self, NetworkError> {
```

#### `FN`: **descriptor_offset** <sub>line 101</sub>
```rust
fn descriptor_offset(&self, index: u16) -> usize {
```

#### `FN`: **avail_offset** <sub>line 105</sub>
```rust
fn avail_offset(&self) -> usize {
```

#### `FN`: **used_offset** <sub>line 109</sub>
```rust
fn used_offset(&self) -> usize {
```

#### `STRUCT`: **UsedElement** <sub>line 164</sub>
```rust
struct UsedElement {
```

#### `STRUCT`: **VirtioNetDevice** <sub>line 169</sub>
```rust
pub struct VirtioNetDevice {
```

#### `IMPL`: **VirtioNetDevice** <sub>line 181</sub>
```rust
impl VirtioNetDevice {
```

#### `FN`: **mmio_base** <sub>line 196</sub>
```rust
pub fn mmio_base(&self) -> usize {
```

#### `FN`: **initialize** <sub>line 200</sub>
```rust
fn initialize(&mut self) -> Result<(), NetworkError> {
```

#### `FN`: **configure_queue** <sub>line 246</sub>
```rust
fn configure_queue(&mut self, index: u16, queue: LegacyQueue) -> Result<(), NetworkError> {
```

#### `FN`: **post_rx** <sub>line 260</sub>
```rust
fn post_rx(&mut self, index: u16) -> Result<(), NetworkError> {
```

#### `FN`: **reclaim_tx** <sub>line 282</sub>
```rust
fn reclaim_tx(&mut self) -> u16 {
```

#### `FN`: **notify** <sub>line 291</sub>
```rust
fn notify(&self, queue: u16) {
```

#### `FN`: **fail** <sub>line 295</sub>
```rust
fn fail(&mut self) {
```

#### `FN`: **mmio_read** <sub>line 300</sub>
```rust
fn mmio_read(&self, offset: usize) -> u32 {
```

#### `FN`: **mmio_write** <sub>line 304</sub>
```rust
fn mmio_write(&self, offset: usize, value: u32) {
```

#### `IMPL`: **NetworkDevice** <sub>line 311</sub>
```rust
impl NetworkDevice for VirtioNetDevice {
```

#### `FN`: **init** <sub>line 312</sub>
```rust
fn init(&mut self) -> Result<(), NetworkError> {
```

#### `FN`: **mac_address** <sub>line 316</sub>
```rust
fn mac_address(&self) -> MacAddress {
```

#### `FN`: **mtu** <sub>line 320</sub>
```rust
fn mtu(&self) -> usize {
```

#### `FN`: **submit_tx** <sub>line 324</sub>
```rust
fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError> {
```

#### `FN`: **poll** <sub>line 358</sub>
```rust
fn poll(&mut self) -> Result<PollResult, NetworkError> {
```

#### `FN`: **take_rx** <sub>line 375</sub>
```rust
fn take_rx(&mut self) -> Option<RxFrame<'_>> {
```

#### `FN`: **recycle_rx** <sub>line 393</sub>
```rust
fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/virtio/pci_legacy.rs</b> (30 items)</summary>

#### `STRUCT`: **VirtioNetHeader** <sub>line 38</sub>
```rust
struct VirtioNetHeader {
```

#### `STRUCT`: **LegacyQueue** <sub>line 48</sub>
```rust
struct LegacyQueue {
```

#### `IMPL`: **LegacyQueue** <sub>line 55</sub>
```rust
impl LegacyQueue {
```

#### `FN`: **allocate** <sub>line 65</sub>
```rust
fn allocate(size: u16) -> Result<Self, NetworkError> {
```

#### `FN`: **descriptor_offset** <sub>line 91</sub>
```rust
fn descriptor_offset(&self, index: u16) -> usize {
```

#### `FN`: **avail_offset** <sub>line 95</sub>
```rust
fn avail_offset(&self) -> usize {
```

#### `FN`: **used_offset** <sub>line 99</sub>
```rust
fn used_offset(&self) -> usize {
```

#### `STRUCT`: **UsedElement** <sub>line 154</sub>
```rust
struct UsedElement {
```

#### `STRUCT`: **VirtioPciLegacyNetDevice** <sub>line 159</sub>
```rust
pub struct VirtioPciLegacyNetDevice {
```

#### `IMPL`: **VirtioPciLegacyNetDevice** <sub>line 173</sub>
```rust
impl VirtioPciLegacyNetDevice {
```

#### `FN`: **initialize** <sub>line 192</sub>
```rust
fn initialize(&mut self) -> Result<(), NetworkError> {
```

#### `FN`: **configure_queue** <sub>line 233</sub>
```rust
fn configure_queue(&mut self, index: u16, queue: LegacyQueue) -> Result<(), NetworkError> {
```

#### `FN`: **post_rx** <sub>line 245</sub>
```rust
fn post_rx(&mut self, index: u16) -> Result<(), NetworkError> {
```

#### `FN`: **reclaim_tx** <sub>line 267</sub>
```rust
fn reclaim_tx(&mut self) -> u16 {
```

#### `FN`: **notify** <sub>line 276</sub>
```rust
fn notify(&self, queue: u16) {
```

#### `FN`: **fail** <sub>line 280</sub>
```rust
fn fail(&mut self) {
```

#### `FN`: **read_u8** <sub>line 285</sub>
```rust
fn read_u8(&self, offset: u16) -> u8 {
```

#### `FN`: **read_u16** <sub>line 290</sub>
```rust
fn read_u16(&self, offset: u16) -> u16 {
```

#### `FN`: **read_u32** <sub>line 295</sub>
```rust
fn read_u32(&self, offset: u16) -> u32 {
```

#### `FN`: **write_u8** <sub>line 300</sub>
```rust
fn write_u8(&self, offset: u16, value: u8) {
```

#### `FN`: **write_u16** <sub>line 307</sub>
```rust
fn write_u16(&self, offset: u16, value: u16) {
```

#### `FN`: **write_u32** <sub>line 314</sub>
```rust
fn write_u32(&self, offset: u16, value: u32) {
```

#### `IMPL`: **NetworkDevice** <sub>line 322</sub>
```rust
impl NetworkDevice for VirtioPciLegacyNetDevice {
```

#### `FN`: **init** <sub>line 323</sub>
```rust
fn init(&mut self) -> Result<(), NetworkError> {
```

#### `FN`: **mac_address** <sub>line 327</sub>
```rust
fn mac_address(&self) -> MacAddress {
```

#### `FN`: **mtu** <sub>line 331</sub>
```rust
fn mtu(&self) -> usize {
```

#### `FN`: **submit_tx** <sub>line 335</sub>
```rust
fn submit_tx(&mut self, frame: TxFrame<'_>) -> Result<(), NetworkError> {
```

#### `FN`: **poll** <sub>line 369</sub>
```rust
fn poll(&mut self) -> Result<PollResult, NetworkError> {
```

#### `FN`: **take_rx** <sub>line 383</sub>
```rust
fn take_rx(&mut self) -> Option<RxFrame<'_>> {
```

#### `FN`: **recycle_rx** <sub>line 401</sub>
```rust
fn recycle_rx(&mut self, buffer_id: u16) -> Result<(), NetworkError> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/virtio/queue.rs</b> (13 items)</summary>

#### `STRUCT`: **Descriptor** <sub>line 8</sub>
```rust
pub struct Descriptor {
```

#### `IMPL`: **Descriptor** <sub>line 15</sub>
```rust
impl Descriptor {
```

#### `STRUCT`: **DescriptorId** <sub>line 26</sub>
```rust
pub struct DescriptorId(pub u16);
```

#### `STRUCT`: **DescriptorPool** <sub>line 28</sub>
```rust
pub struct DescriptorPool<const N: usize> {
```

#### `FN`: **allocate** <sub>line 59</sub>
```rust
pub fn allocate(&mut self) -> Result<DescriptorId, NetworkError> {
```

#### `FN`: **configure** <sub>line 75</sub>
```rust
pub fn configure(
```

#### `FN`: **descriptor** <sub>line 101</sub>
```rust
pub fn descriptor(&self, id: DescriptorId) -> Result<&Descriptor, NetworkError> {
```

#### `FN`: **release_chain** <sub>line 107</sub>
```rust
pub fn release_chain(&mut self, head: DescriptorId) -> Result<(), NetworkError> {
```

#### `FN`: **default** <sub>line 135</sub>
```rust
fn default() -> Self {
```

#### `STRUCT`: **QueueMemory** <sub>line 141</sub>
```rust
pub struct QueueMemory {
```

#### `FN`: **self_test** <sub>line 148</sub>
```rust
pub fn self_test() -> Result<&'static str, &'static str> {
```

#### `FN`: **pool_allocates_and_releases_chain_once** <sub>line 173</sub>
```rust
fn pool_allocates_and_releases_chain_once() {
```

#### `FN`: **zero_sized_pool_fails_cleanly** <sub>line 186</sub>
```rust
fn zero_sized_pool_fails_cleanly() {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nic/virtio/transport.rs</b> (11 items)</summary>

#### `STRUCT`: **QueueSetup** <sub>line 4</sub>
```rust
pub struct QueueSetup {
```

#### `TRAIT`: **VirtioTransport** <sub>line 11</sub>
```rust
pub trait VirtioTransport {
```

#### `FN`: **reset** <sub>line 12</sub>
```rust
fn reset(&mut self) -> Result<(), NetworkError>;
```

#### `FN`: **status** <sub>line 13</sub>
```rust
fn status(&self) -> u8;
```

#### `FN`: **set_status** <sub>line 14</sub>
```rust
fn set_status(&mut self, status: u8);
```

#### `FN`: **device_features** <sub>line 16</sub>
```rust
fn device_features(&self) -> u64;
```

#### `FN`: **set_driver_features** <sub>line 17</sub>
```rust
fn set_driver_features(&mut self, features: u64);
```

#### `FN`: **queue_max_size** <sub>line 19</sub>
```rust
fn queue_max_size(&self, queue_index: u16) -> u16;
```

#### `FN`: **configure_queue** <sub>line 20</sub>
```rust
fn configure_queue(&mut self, queue_index: u16, setup: QueueSetup) -> Result<(), NetworkError>;
```

#### `FN`: **notify_queue** <sub>line 22</sub>
```rust
fn notify_queue(&mut self, queue_index: u16);
```

#### `FN`: **read_config** <sub>line 24</sub>
```rust
fn read_config(&self, offset: u16, out: &mut [u8]) -> Result<(), NetworkError>;
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nimcore.rs</b> (14 items)</summary>

#### `FN`: **nim_u64_to_str** <sub>line 2</sub>
```rust
fn nim_u64_to_str(v: u64, base: u8, buf: *mut u8, cap: u32) -> u32;
```

#### `FN`: **nim_parse_u64** <sub>line 3</sub>
```rust
fn nim_parse_u64(s: *const u8, len: u32, base: u8, out: *mut u64) -> u8;
```

#### `FN`: **nim_rb_push** <sub>line 4</sub>
```rust
fn nim_rb_push(b: u8) -> u8;
```

#### `FN`: **nim_rb_pop** <sub>line 5</sub>
```rust
fn nim_rb_pop() -> i32;
```

#### `FN`: **nim_shell_register** <sub>line 6</sub>
```rust
fn nim_shell_register(name: *const u8, nlen: u32,
```

#### `FN`: **nim_shell_run** <sub>line 8</sub>
```rust
fn nim_shell_run(line: *const u8, len: u32) -> i32;
```

#### `FN`: **nim_banner** <sub>line 9</sub>
```rust
fn nim_banner(buf: *mut u8, cap: u32) -> u32;
```

#### `TYPE`: **ShellHandler** <sub>line 12</sub>
```rust
pub type ShellHandler = extern "C" fn(*mut u8, u32) -> i32;
```

#### `FN`: **banner** <sub>line 14</sub>
```rust
pub fn banner(buf: &mut [u8]) -> usize {
```

#### `FN`: **shell_register** <sub>line 18</sub>
```rust
pub fn shell_register(name: &str, h: ShellHandler) -> bool {
```

#### `FN`: **shell_run** <sub>line 22</sub>
```rust
pub fn shell_run(line: &str) -> i32 {
```

#### `FN`: **key_push** <sub>line 26</sub>
```rust
pub fn key_push(c: u8) {
```

#### `FN`: **key_pop** <sub>line 30</sub>
```rust
pub fn key_pop() -> Option<u8> {
```

#### `FN`: **parse_u64** <sub>line 35</sub>
```rust
pub fn parse_u64(s: &str, base: u8) -> Option<u64> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nimcore/banner.nim</b> (3 items)</summary>

#### `CONST`: **ART** <sub>line 1</sub>
```nim
const ART = """
```

#### `PROC`: **nim_banner** <sub>line 13</sub>
```nim
proc nim_banner(buf: ptr uint8, cap: uint32): uint32 {.exportc, cdecl.} =
```

#### `VAR`: **o** <sub>line 14</sub>
```nim
var o = 0u32
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nimcore/format.nim</b> (12 items)</summary>

#### `PROC`: **nim_u64_to_str** <sub>line 1</sub>
```nim
proc nim_u64_to_str(v: uint64, base: uint8, buf: ptr uint8, cap: uint32): uint32
```

#### `VAR`: **tmp** <sub>line 3</sub>
```nim
var tmp: array[64, uint8]
```

#### `VAR`: **n** <sub>line 4</sub>
```nim
var n = 0u32
```

#### `VAR`: **x** <sub>line 5</sub>
```nim
var x = v
```

#### `LET`: **b** <sub>line 7</sub>
```nim
let b = if base < 2: 10u64 else: uint64(base)
```

#### `LET`: **d** <sub>line 14</sub>
```nim
let d = x mod b
```

#### `VAR`: **i** <sub>line 22</sub>
```nim
var i = 0u32
```

#### `PROC`: **nim_hex_dump** <sub>line 29</sub>
```nim
proc nim_hex_dump(src: ptr uint8, len: uint32, buf: ptr uint8, cap: uint32): uint32
```

#### `CONST`: **hexd** <sub>line 31</sub>
```nim
const hexd = "0123456789abcdef"
```

#### `VAR`: **o** <sub>line 32</sub>
```nim
var o = 0u32
```

#### `VAR`: **i** <sub>line 33</sub>
```nim
var i = 0u32
```

#### `LET`: **b** <sub>line 36</sub>
```nim
let b = src[i]
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nimcore/parse.nim</b> (6 items)</summary>

#### `PROC`: **nim_parse_u64** <sub>line 1</sub>
```nim
proc nim_parse_u64(s: ptr uint8, len: uint32, base: uint8, out_v: ptr uint64): uint8
```

#### `VAR`: **v** <sub>line 3</sub>
```nim
var v = 0u64
```

#### `VAR`: **i** <sub>line 4</sub>
```nim
var i = 0u32
```

#### `LET`: **b** <sub>line 5</sub>
```nim
let b = uint64(base)
```

#### `LET`: **c** <sub>line 15</sub>
```nim
let c = s[i]
```

#### `LET`: **d** <sub>line 16</sub>
```nim
let d =
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nimcore/ringbuf.nim</b> (7 items)</summary>

#### `TYPE`: **Rb** <sub>line 1</sub>
```nim
type Rb = object
```

#### `VAR`: **key_rb** <sub>line 6</sub>
```nim
var key_rb: Rb
```

#### `PROC`: **nim_rb_push** <sub>line 8</sub>
```nim
proc nim_rb_push(b: uint8): uint8 {.exportc, cdecl.} =
```

#### `LET`: **next** <sub>line 9</sub>
```nim
let next = (key_rb.head + 1) mod 1024
```

#### `PROC`: **nim_rb_pop** <sub>line 15</sub>
```nim
proc nim_rb_pop(): int32 {.exportc, cdecl.} =
```

#### `LET`: **b** <sub>line 17</sub>
```nim
let b = key_rb.data[key_rb.tail]
```

#### `PROC`: **nim_rb_len** <sub>line 21</sub>
```nim
proc nim_rb_len(): uint32 {.exportc, cdecl.} =
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/nimcore/shell.nim</b> (11 items)</summary>

#### `TYPE`: **Handler** <sub>line 1</sub>
```nim
type Handler = proc(arg: ptr uint8, len: uint32): int32 {.cdecl.}
```

#### `TYPE`: **Cmd** <sub>line 3</sub>
```nim
type Cmd = object
```

#### `VAR`: **cmds** <sub>line 9</sub>
```nim
var cmds: array[32, Cmd]
```

#### `PROC`: **nim_shell_register** <sub>line 11</sub>
```nim
proc nim_shell_register(name: ptr uint8, nlen: uint32, h: Handler): uint8
```

#### `VAR`: **k** <sub>line 17</sub>
```nim
var k = 0u32
```

#### `PROC`: **eq_name** <sub>line 28</sub>
```nim
proc eq_name(c: var Cmd, line: ptr uint8, nlen: uint32): bool =
```

#### `VAR`: **k** <sub>line 30</sub>
```nim
var k = 0u32
```

#### `PROC`: **nim_shell_run** <sub>line 36</sub>
```nim
proc nim_shell_run(line: ptr uint8, len: uint32): int32
```

#### `VAR`: **sp** <sub>line 38</sub>
```nim
var sp = 0u32
```

#### `VAR`: **arg** <sub>line 44</sub>
```nim
var arg: ptr uint8 = nil
```

#### `VAR`: **alen** <sub>line 45</sub>
```nim
var alen = 0u32
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/pci.rs</b> (17 items)</summary>

#### `STRUCT`: **PciAddress** <sub>line 11</sub>
```rust
pub struct PciAddress {
```

#### `STRUCT`: **PciDevice** <sub>line 18</sub>
```rust
pub struct PciDevice {
```

#### `FN`: **config_address** <sub>line 28</sub>
```rust
fn config_address(addr: PciAddress, offset: u8) -> u32 {
```

#### `FN`: **config_read_u32** <sub>line 36</sub>
```rust
pub fn config_read_u32(addr: PciAddress, offset: u8) -> u32 {
```

#### `FN`: **config_write_u32** <sub>line 45</sub>
```rust
pub fn config_write_u32(addr: PciAddress, offset: u8, value: u32) {
```

#### `FN`: **config_read_u16** <sub>line 54</sub>
```rust
pub fn config_read_u16(addr: PciAddress, offset: u8) -> u16 {
```

#### `FN`: **config_read_u8** <sub>line 60</sub>
```rust
pub fn config_read_u8(addr: PciAddress, offset: u8) -> u8 {
```

#### `FN`: **probe_function** <sub>line 66</sub>
```rust
fn probe_function(bus: u8, device: u8, function: u8) -> Option<PciDevice> {
```

#### `FN`: **enumerate** <sub>line 88</sub>
```rust
pub fn enumerate() -> Vec<PciDevice> {
```

#### `ENUM`: **NicKind** <sub>line 121</sub>
```rust
pub enum NicKind {
```

#### `FN`: **find_nic** <sub>line 126</sub>
```rust
pub fn find_nic(devices: &[PciDevice]) -> Option<(PciDevice, NicKind)> {
```

#### `FN`: **bar** <sub>line 142</sub>
```rust
pub fn bar(addr: PciAddress, bar_index: u8) -> u32 {
```

#### `FN`: **io_base_from_bar** <sub>line 147</sub>
```rust
pub fn io_base_from_bar(bar_value: u32) -> Option<u16> {
```

#### `FN`: **mem_base_from_bar** <sub>line 155</sub>
```rust
pub fn mem_base_from_bar(bar_value: u32) -> Option<u32> {
```

#### `FN`: **enable_bus_mastering** <sub>line 163</sub>
```rust
pub fn enable_bus_mastering(addr: PciAddress) {
```

#### `FN`: **init** <sub>line 172</sub>
```rust
pub fn init() {
```

#### `FN`: **self_test** <sub>line 186</sub>
```rust
pub fn self_test() -> TestResult {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/policy/ada/policy.adb</b> (2 items)</summary>

#### `PACKAGE`: **body** <sub>line 1</sub>
```ada
package body Policy
```

#### `FUNCTION`: **Evaluate** <sub>line 5</sub>
```ada
function Evaluate (Ring  : Ring_Id;
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/policy/ada/policy.ads</b> (6 items)</summary>

#### `PACKAGE`: **Policy** <sub>line 3</sub>
```ada
package Policy
```

#### `TYPE`: **Ring_Id** <sub>line 7</sub>
```ada
type Ring_Id is (Ring_Kernel, Ring_Driver, Ring_User)
```

#### `TYPE`: **Call_Class** <sub>line 10</sub>
```ada
type Call_Class is (Cls_Sys, Cls_Video, Cls_Audio,
```

#### `TYPE`: **Decision** <sub>line 14</sub>
```ada
type Decision is (Allow, Deny)
```

#### `TYPE`: **U64** <sub>line 17</sub>
```ada
type U64 is mod 2**64;
```

#### `FUNCTION`: **Evaluate** <sub>line 21</sub>
```ada
function Evaluate (Ring  : Ring_Id;
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/policy/ada/policy_c.adb</b> (4 items)</summary>

#### `PACKAGE`: **body** <sub>line 3</sub>
```ada
package body Policy_C is
```

#### `FUNCTION`: **To_Ring** <sub>line 5</sub>
```ada
function To_Ring (V : Unsigned_8) return Ring_Id is
```

#### `FUNCTION`: **To_Class** <sub>line 11</sub>
```ada
function To_Class (V : Unsigned_8) return Call_Class is
```

#### `FUNCTION`: **policy_evaluate** <sub>line 20</sub>
```ada
function policy_evaluate (ring : Unsigned_8;
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/policy/ada/policy_c.ads</b> (2 items)</summary>

#### `PACKAGE`: **Policy_C** <sub>line 4</sub>
```ada
package Policy_C is
```

#### `FUNCTION`: **policy_evaluate** <sub>line 6</sub>
```ada
function policy_evaluate (ring : Unsigned_8;
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/policy/bridge.rs</b> (3 items)</summary>

#### `FN`: **policy_evaluate** <sub>line 2</sub>
```rust
fn policy_evaluate(ring: u8, cls: u8, op: u8, arg: u64) -> u8;
```

#### `FN`: **nim_policy_log** <sub>line 3</sub>
```rust
fn nim_policy_log(ring: u8, cls: u8, op: u8, dec: u8);
```

#### `FN`: **check** <sub>line 9</sub>
```rust
pub fn check(ring: u8, cmd: u32, arg: u64) -> bool {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/policy/mod.rs</b> (13 items)</summary>

#### `FN`: **evaluate** <sub>line 34</sub>
```rust
pub fn evaluate(ring: u8, class: u8, op: u8, _arg: u64) -> u8 {
```

#### `STRUCT`: **PolicyEntry** <sub>line 59</sub>
```rust
pub struct PolicyEntry {
```

#### `STRUCT`: **LogInner** <sub>line 68</sub>
```rust
struct LogInner {
```

#### `FN`: **policy_log** <sub>line 82</sub>
```rust
pub fn policy_log(ring: u8, cls: u8, op: u8, dec: u8) {
```

#### `FN`: **denies** <sub>line 94</sub>
```rust
pub fn denies() -> u64 {
```

#### `FN`: **total** <sub>line 98</sub>
```rust
pub fn total() -> u64 {
```

#### `FN`: **entry** <sub>line 103</sub>
```rust
pub fn entry(idx: usize) -> Option<PolicyEntry> {
```

#### `FN`: **ring_of** <sub>line 112</sub>
```rust
pub fn ring_of(world: u32) -> u8 {
```

#### `FN`: **required_caps** <sub>line 125</sub>
```rust
pub fn required_caps(class: u8) -> &'static [Capability] {
```

#### `FN`: **hook** <sub>line 136</sub>
```rust
pub fn hook(world: u32, cap: Capability) -> bool {
```

#### `FN`: **install** <sub>line 149</sub>
```rust
pub fn install() {
```

#### `FN`: **decide** <sub>line 153</sub>
```rust
pub fn decide(world: u32, cmd: u32, arg: u64) -> Result<(), &'static str> {
```

#### `FN`: **decide_current** <sub>line 185</sub>
```rust
pub fn decide_current(cmd: u32, arg: u64) -> Result<(), &'static str> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/policy/nim/policynim.nim</b> (4 items)</summary>

#### `PROC`: **nim_policy_log** <sub>line 10</sub>
```nim
proc nim_policy_log(ring, cls, op, dec: uint8) {.exportc, cdecl.} =
```

#### `PROC`: **nim_policy_denies** <sub>line 17</sub>
```nim
proc nim_policy_denies(): uint64 {.exportc, cdecl.} =
```

#### `PROC`: **nim_policy_get** <sub>line 20</sub>
```nim
proc nim_policy_get(idx: uint32,
```

#### `LET`: **e** <sub>line 26</sub>
```nim
let e = logbuf[idx]
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/process/fd.rs</b> (10 items)</summary>

#### `ENUM`: **FdKind** <sub>line 9</sub>
```rust
pub enum FdKind {
```

#### `STRUCT`: **Fd** <sub>line 16</sub>
```rust
pub struct Fd {
```

#### `IMPL`: **Fd** <sub>line 23</sub>
```rust
impl Fd {
```

#### `FN`: **kprintf** <sub>line 32</sub>
```rust
fn kprintf(fmt: *const u8, ...);
```

#### `FN`: **pool_take** <sub>line 35</sub>
```rust
fn pool_take() -> Option<i16> {
```

#### `FN`: **pool_drop** <sub>line 48</sub>
```rust
fn pool_drop(slot: i16) {
```

#### `FN`: **open_for** <sub>line 54</sub>
```rust
pub fn open_for(pid: u32, path: &str) -> i32 {
```

#### `FN`: **read_for** <sub>line 105</sub>
```rust
pub fn read_for(pid: u32, fd: i32, out: &mut [u8]) -> i32 {
```

#### `FN`: **write_for** <sub>line 157</sub>
```rust
pub fn write_for(pid: u32, fd: i32, data: &[u8]) -> i32 {
```

#### `FN`: **close_for** <sub>line 187</sub>
```rust
pub fn close_for(pid: u32, fd: i32) -> i32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/process/proc.rs</b> (1 items)</summary>

#### `STRUCT`: **Process** <sub>line 3</sub>
```rust
pub struct Process {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/serial.rs</b> (13 items)</summary>

#### `FN`: **init** <sub>line 9</sub>
```rust
pub fn init() {
```

#### `FN`: **tx_empty** <sub>line 27</sub>
```rust
fn tx_empty() -> bool {
```

#### `FN`: **write_byte** <sub>line 32</sub>
```rust
pub fn write_byte(byte: u8) {
```

#### `FN`: **init** <sub>line 47</sub>
```rust
pub fn init() {
```

#### `FN`: **tx_empty** <sub>line 50</sub>
```rust
fn tx_empty() -> bool {
```

#### `FN`: **write_byte** <sub>line 54</sub>
```rust
pub fn write_byte(byte: u8) {
```

#### `FN`: **init** <sub>line 60</sub>
```rust
pub fn init() {
```

#### `FN`: **write_byte** <sub>line 67</sub>
```rust
pub fn write_byte(byte: u8) {
```

#### `FN`: **write_str** <sub>line 74</sub>
```rust
pub fn write_str(s: &str) {
```

#### `STRUCT`: **SerialWriter** <sub>line 80</sub>
```rust
pub struct SerialWriter;
```

#### `IMPL`: **fmt** <sub>line 82</sub>
```rust
impl fmt::Write for SerialWriter {
```

#### `FN`: **write_str** <sub>line 83</sub>
```rust
fn write_str(&mut self, s: &str) -> fmt::Result {
```

#### `FN`: **print_args** <sub>line 89</sub>
```rust
pub fn print_args(args: fmt::Arguments) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/terminal/mod.rs</b> (26 items)</summary>

#### `FN`: **editor_run** <sub>line 7</sub>
```rust
fn editor_run(path: *const u8) -> i32;
```

#### `FN`: **kbuf_push** <sub>line 24</sub>
```rust
fn kbuf_push(c: u8) {
```

#### `FN`: **kbuf_pop** <sub>line 34</sub>
```rust
fn kbuf_pop() -> Option<u8> {
```

#### `FN`: **scancode_to_char** <sub>line 45</sub>
```rust
fn scancode_to_char(code: u8) -> Option<char> {
```

#### `FN`: **push_scancode** <sub>line 116</sub>
```rust
pub fn push_scancode(code: u8) {
```

#### `FN`: **scancode_to_keycode** <sub>line 133</sub>
```rust
fn scancode_to_keycode(code: u8) -> Option<u32> {
```

#### `FN`: **keycode_push** <sub>line 152</sub>
```rust
fn keycode_push(k: u32) {
```

#### `FN`: **pop_keycode** <sub>line 164</sub>
```rust
pub fn pop_keycode() -> Option<u32> {
```

#### `FN`: **set_keycode_capture** <sub>line 177</sub>
```rust
pub fn set_keycode_capture(on: bool) {
```

#### `FN`: **line_as_str** <sub>line 195</sub>
```rust
fn line_as_str(buf: &mut [u8]) -> &str {
```

#### `FN`: **line_clear** <sub>line 203</sub>
```rust
fn line_clear() {
```

#### `FN`: **draw_prompt** <sub>line 207</sub>
```rust
fn draw_prompt() {
```

#### `FN`: **redraw_line** <sub>line 226</sub>
```rust
fn redraw_line() {
```

#### `FN`: **handle_char** <sub>line 247</sub>
```rust
fn handle_char(c: char) {
```

#### `FN`: **split_once_space** <sub>line 296</sub>
```rust
fn split_once_space(s: &str) -> (&str, &str) {
```

#### `FN`: **parse_resolution** <sub>line 304</sub>
```rust
fn parse_resolution(s: &str) -> Option<(u32, u32)> {
```

#### `FN`: **dev** <sub>line 315</sub>
```rust
fn dev() -> Option<&'static dyn BlockDevice> {
```

#### `FN`: **execute** <sub>line 319</sub>
```rust
fn execute(line: &str) {
```

#### `FN`: **print_ping_result** <sub>line 528</sub>
```rust
fn print_ping_result(result: crate::nic::PingResult) {
```

#### `FN`: **poll_network** <sub>line 544</sub>
```rust
fn poll_network() {
```

#### `STRUCT`: **Sink** <sub>line 553</sub>
```rust
struct Sink;
```

#### `IMPL`: **core** <sub>line 555</sub>
```rust
impl core::fmt::Write for Sink {
```

#### `FN`: **write_str** <sub>line 556</sub>
```rust
fn write_str(&mut self, s: &str) -> core::fmt::Result {
```

#### `FN`: **init** <sub>line 564</sub>
```rust
pub fn init() {
```

#### `FN`: **run** <sub>line 569</sub>
```rust
pub fn run() -> ! {
```

#### `FN`: **self_test** <sub>line 583</sub>
```rust
pub fn self_test() -> crate::testing::TestResult {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/testing.rs</b> (5 items)</summary>

#### `TYPE`: **TestResult** <sub>line 4</sub>
```rust
pub type TestResult = Result<&'static str, &'static str>;
```

#### `STRUCT`: **Test** <sub>line 6</sub>
```rust
pub struct Test {
```

#### `FN`: **self_test** <sub>line 14</sub>
```rust
pub fn self_test() -> $crate::testing::TestResult {
```

#### `FN`: **run_test** <sub>line 20</sub>
```rust
pub fn run_test(test: &Test) -> bool {
```

#### `FN`: **run_all** <sub>line 41</sub>
```rust
pub fn run_all(tests: &[Test]) {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/trampoline_rings/arch/aarch64/trampoline_rings.rs</b> (9 items)</summary>

#### `STRUCT`: **CpuCtx** <sub>line 15</sub>
```rust
pub struct CpuCtx {
```

#### `FN`: **tr_init** <sub>line 23</sub>
```rust
fn tr_init();
```

#### `FN`: **tr_restore_ctx** <sub>line 24</sub>
```rust
pub fn tr_restore_ctx(ctx: *mut CpuCtx) -> !;
```

#### `STRUCT`: **World** <sub>line 27</sub>
```rust
pub struct World {
```

#### `FN`: **init** <sub>line 40</sub>
```rust
pub fn init() {
```

#### `FN`: **add_world** <sub>line 46</sub>
```rust
pub fn add_world(ring: u8, ttbr0: u64, entry: u64,
```

#### `FN`: **write_ttbr0** <sub>line 68</sub>
```rust
fn write_ttbr0(v: u64) {
```

#### `FN`: **pick_next** <sub>line 74</sub>
```rust
fn pick_next(from: Option<usize>) -> usize {
```

#### `FN`: **start** <sub>line 90</sub>
```rust
pub fn start() -> ! {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/trampoline_rings/arch/risc-v/trampoline_rings.rs</b> (9 items)</summary>

#### `STRUCT`: **CpuCtx** <sub>line 16</sub>
```rust
pub struct CpuCtx {
```

#### `FN`: **tr_init** <sub>line 25</sub>
```rust
fn tr_init(kernel_stack_top: u64);
```

#### `FN`: **tr_restore_ctx** <sub>line 26</sub>
```rust
pub fn tr_restore_ctx(ctx: *mut CpuCtx) -> !;
```

#### `STRUCT`: **World** <sub>line 29</sub>
```rust
pub struct World {
```

#### `FN`: **init** <sub>line 44</sub>
```rust
pub fn init() {
```

#### `FN`: **add_world** <sub>line 51</sub>
```rust
pub fn add_world(ring: u8, satp: u64, entry: u64,
```

#### `FN`: **write_satp** <sub>line 73</sub>
```rust
fn write_satp(v: u64) {
```

#### `FN`: **pick_next** <sub>line 79</sub>
```rust
fn pick_next(from: Option<usize>) -> usize {
```

#### `FN`: **start** <sub>line 95</sub>
```rust
pub fn start() -> ! {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/trampoline_rings/arch/x86_64/trampoline_rings.rs</b> (15 items)</summary>

#### `STRUCT`: **CpuCtx** <sub>line 20</sub>
```rust
pub struct CpuCtx {
```

#### `FN`: **tr_init** <sub>line 29</sub>
```rust
fn tr_init(rsp0: u64);
```

#### `FN`: **tr_restore_ctx** <sub>line 30</sub>
```rust
pub fn tr_restore_ctx(ctx: *mut CpuCtx) -> !;
```

#### `STRUCT`: **IdtGate** <sub>line 37</sub>
```rust
struct IdtGate {
```

#### `FN`: **set_gate** <sub>line 51</sub>
```rust
fn set_gate(i: usize, handler: u64, dpl: u8) {
```

#### `STRUCT`: **World** <sub>line 65</sub>
```rust
pub struct World {
```

#### `FN`: **sel_for** <sub>line 80</sub>
```rust
fn sel_for(ring: u8) -> (u64, u64) {
```

#### `FN`: **init** <sub>line 87</sub>
```rust
pub fn init() {
```

#### `STRUCT`: **IdtPtr** <sub>line 122</sub>
```rust
struct IdtPtr {
```

#### `FN`: **add_world** <sub>line 129</sub>
```rust
pub fn add_world(ring: u8, cr3: u64, entry: u64,
```

#### `FN`: **write_cr3** <sub>line 154</sub>
```rust
fn write_cr3(v: u64) {
```

#### `FN`: **pick_next** <sub>line 158</sub>
```rust
fn pick_next(from: Option<usize>) -> usize {
```

#### `FN`: **start** <sub>line 174</sub>
```rust
pub fn start() -> ! {
```

#### `IMPL`: **World** <sub>line 226</sub>
```rust
impl World {
```

#### `FN`: **ctx_mut** <sub>line 227</sub>
```rust
fn ctx_mut(&mut self) -> &mut CpuCtx {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/userspace/process/proc.rs</b> (7 items)</summary>

#### `STRUCT`: **IpcMsg** <sub>line 5</sub>
```rust
pub struct IpcMsg {
```

#### `STRUCT`: **Process** <sub>line 13</sub>
```rust
pub struct Process {
```

#### `FN`: **register** <sub>line 29</sub>
```rust
pub fn register(world: usize, parent: u32) -> Option<u32> {
```

#### `FN`: **by_pid** <sub>line 55</sub>
```rust
pub fn by_pid(pid: u32) -> Option<&'static mut Process> {
```

#### `FN`: **by_world** <sub>line 69</sub>
```rust
pub fn by_world(world: usize) -> Option<&'static mut Process> {
```

#### `FN`: **send** <sub>line 83</sub>
```rust
pub fn send(dst: u32, msg: IpcMsg) -> bool {
```

#### `FN`: **recv** <sub>line 101</sub>
```rust
pub fn recv(pid: u32) -> Option<IpcMsg> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/userspace/process/runcl.rs</b> (9 items)</summary>

#### `FN`: **k_fs_read** <sub>line 2</sub>
```rust
fn k_fs_read(path: *const u8, buf: *mut u8, cap: u32) -> i32;
```

#### `FN`: **cl_compile_source** <sub>line 4</sub>
```rust
fn cl_compile_source(src: *const u8, len: usize, ar: *mut u8,
```

#### `FN`: **cl_vm_init** <sub>line 8</sub>
```rust
fn cl_vm_init(vm: *mut u8, prog: *mut u8);
```

#### `FN`: **cl_bridge_init** <sub>line 9</sub>
```rust
fn cl_bridge_init(vm: *mut u8, ring: u8) -> i32;
```

#### `FN`: **cl_vm_run** <sub>line 10</sub>
```rust
fn cl_vm_run(vm: *mut u8) -> i32;
```

#### `STRUCT`: **ArenaBuf** <sub>line 16</sub>
```rust
struct ArenaBuf([u8; 196608]);
```

#### `STRUCT`: **ProgBuf** <sub>line 20</sub>
```rust
struct ProgBuf([u8; 65536]);
```

#### `STRUCT`: **VmBuf** <sub>line 24</sub>
```rust
struct VmBuf([u8; 98304]);
```

#### `FN`: **run** <sub>line 27</sub>
```rust
pub fn run(path: &str) -> i32 {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/userspace/process/spawn.rs</b> (1 items)</summary>

#### `FN`: **spawn_init** <sub>line 6</sub>
```rust
pub fn spawn_init() -> Result<(), &'static str> {
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/userspace/process/syscall.rs</b> (9 items)</summary>

#### `FN`: **paging_translate_in** <sub>line 17</sub>
```rust
fn paging_translate_in(pml4: u64, virt: u64) -> u64;
```

#### `FN`: **kprintf** <sub>line 18</sub>
```rust
fn kprintf(fmt: *const u8, ...);
```

#### `FN`: **user_copy_in** <sub>line 23</sub>
```rust
fn user_copy_in(cr3: u64, src: u64, dst: &mut [u8]) -> bool {
```

#### `FN`: **user_cstr** <sub>line 50</sub>
```rust
fn user_cstr(cr3: u64, ptr: u64, buf: &mut [u8]) -> Option<&str> {
```

#### `FN`: **do_spawn** <sub>line 70</sub>
```rust
fn do_spawn(cr3: u64, path_ptr: u64, parent_pid: u32) -> i64 {
```

#### `FN`: **handle** <sub>line 121</sub>
```rust
pub fn handle(world: usize, c: &mut tr::CpuCtx) {
```

#### `FN`: **hdmi_caps_raw** <sub>line 207</sub>
```rust
fn hdmi_caps_raw(w: *mut u32, h: *mut u32,
```

#### `FN`: **kvirt_to_phys** <sub>line 209</sub>
```rust
fn kvirt_to_phys(p: *const u8) -> u64;
```

#### `FN`: **paging_map_page_in** <sub>line 210</sub>
```rust
fn paging_map_page_in(pml4: u64, virt: u64,
```

</details>

<details>
<summary><b>📄 kernel_Workspace/kernel/src/vga_buffer.rs</b> (23 items)</summary>

#### `ENUM`: **Color** <sub>line 6</sub>
```rust
pub enum Color {
```

#### `STRUCT`: **ColorCode** <sub>line 34</sub>
```rust
struct ColorCode(u8);
```

#### `IMPL`: **ColorCode** <sub>line 36</sub>
```rust
impl ColorCode {
```

#### `FN`: **new** <sub>line 37</sub>
```rust
fn new(foreground: Color, background: Color) -> ColorCode {
```

#### `STRUCT`: **ScreenChar** <sub>line 44</sub>
```rust
struct ScreenChar {
```

#### `STRUCT`: **Buffer** <sub>line 53</sub>
```rust
struct Buffer {
```

#### `STRUCT`: **Writer** <sub>line 57</sub>
```rust
pub struct Writer {
```

#### `IMPL`: **Writer** <sub>line 63</sub>
```rust
impl Writer {
```

#### `FN`: **write_byte** <sub>line 64</sub>
```rust
pub fn write_byte(&mut self, byte: u8) {
```

#### `FN`: **new_line** <sub>line 83</sub>
```rust
fn new_line(&mut self) {
```

#### `FN`: **clear_row** <sub>line 94</sub>
```rust
fn clear_row(&mut self, row: usize) {
```

#### `FN`: **write_string** <sub>line 104</sub>
```rust
pub fn write_string(&mut self, s: &str) {
```

#### `FN`: **set_color** <sub>line 113</sub>
```rust
pub fn set_color(&mut self, foreground: Color) {
```

#### `FN`: **set_column** <sub>line 117</sub>
```rust
pub fn set_column(&mut self, col: usize) {
```

#### `FN`: **clear_to_end** <sub>line 121</sub>
```rust
pub fn clear_to_end(&mut self) {
```

#### `FN`: **write_byte_colored** <sub>line 133</sub>
```rust
pub fn write_byte_colored(&mut self, byte: u8, fg: Color) {
```

#### `FN`: **column** <sub>line 140</sub>
```rust
pub fn column(&self) -> usize {
```

#### `FN`: **clear_screen** <sub>line 144</sub>
```rust
pub fn clear_screen(&mut self) {
```

#### `IMPL`: **fmt** <sub>line 152</sub>
```rust
impl fmt::Write for Writer {
```

#### `FN`: **write_str** <sub>line 153</sub>
```rust
fn write_str(&mut self, s: &str) -> fmt::Result {
```

#### `FN`: **text_cell** <sub>line 176</sub>
```rust
pub fn text_cell(row: usize, col: usize) -> (u8, u8) {
```

#### `FN`: **_print** <sub>line 222</sub>
```rust
pub fn _print(args: fmt::Arguments) {
```

#### `FN`: **_print_colored** <sub>line 229</sub>
```rust
pub fn _print_colored(_color: Color, args: fmt::Arguments) {
```

</details>

## 📂 kernel_Workspace/kernel-bin

<details>
<summary><b>📄 kernel_Workspace/kernel-bin/src/main.rs</b> (1 items)</summary>

#### `FN`: **kernel_main** <sub>line 8</sub>
```rust
fn kernel_main(boot_info: &'static BootInfo) -> ! {
```

</details>

## 📂 kstd/include

<details>
<summary><b>📄 kstd/include/kstd.h</b> (1 items)</summary>

#### `FUNCTION`: **tr_log** <sub>line 12</sub>
```c
void tr_log(const char *s);
```

</details>

<details>
<summary><b>📄 kstd/include/kstd_audio.h</b> (2 items)</summary>

#### `FUNCTION`: **tr_free** <sub>line 7</sub>
```c
void tr_free(void *ptr, uint32_t bytes);
```

#### `FUNCTION`: **tr_map_mmio** <sub>line 8</sub>
```c
tr_status_t tr_map_mmio(uint64_t phys, uint32_t len, void **out_va);
```

</details>

<details>
<summary><b>📄 kstd/include/kstd_bt.h</b> (5 items)</summary>

#### `FUNCTION`: **tr_bt_info** <sub>line 6</sub>
```c
tr_status_t tr_bt_info(uint8_t *hci_ver, uint8_t *addr6);
```

#### `FUNCTION`: **tr_bt_cmd** <sub>line 7</sub>
```c
tr_status_t tr_bt_cmd(uint16_t opcode, const void *params, uint8_t len);
```

#### `FUNCTION`: **tr_bt_evt** <sub>line 8</sub>
```c
tr_status_t tr_bt_evt(void *buf, uint8_t *len);
```

#### `FUNCTION`: **tr_bt_acl_send** <sub>line 9</sub>
```c
tr_status_t tr_bt_acl_send(const void *data, uint16_t len);
```

#### `FUNCTION`: **tr_bt_acl_recv** <sub>line 10</sub>
```c
tr_status_t tr_bt_acl_recv(void *buf, uint16_t *len);
```

</details>

<details>
<summary><b>📄 kstd/include/kstd_fs.h</b> (2 items)</summary>

#### `FUNCTION`: **tr_fs_read** <sub>line 6</sub>
```c
tr_status_t tr_fs_read(const char *path, void *buf, uint32_t cap, uint32_t *out);
```

#### `FUNCTION`: **tr_fs_exists** <sub>line 7</sub>
```c
tr_status_t tr_fs_exists(const char *path);
```

</details>

<details>
<summary><b>📄 kstd/include/kstd_input.h</b> (1 items)</summary>

#### `FUNCTION`: **tr_input_key** <sub>line 6</sub>
```c
int32_t tr_input_key(void);
```

</details>

<details>
<summary><b>📄 kstd/include/kstd_mem.h</b> (2 items)</summary>

#### `FUNCTION`: **tr_free** <sub>line 7</sub>
```c
void tr_free(void *ptr, uint32_t bytes);
```

#### `FUNCTION`: **tr_map_mmio** <sub>line 8</sub>
```c
tr_status_t tr_map_mmio(uint64_t phys, uint32_t len, void **out_va);
```

</details>

## 📂 kstd/src

<details>
<summary><b>📄 kstd/src/kernel/audio.c</b> (10 items)</summary>

#### `FUNCTION`: **k_audio_play** <sub>line 3</sub>
```c
extern int32_t k_audio_play(uint64_t phys, uint32_t len);
```

#### `FUNCTION`: **k_audio_stop** <sub>line 4</sub>
```c
extern int32_t k_audio_stop(void);
```

#### `FUNCTION`: **k_audio_jack** <sub>line 5</sub>
```c
extern int32_t k_audio_jack(void);
```

#### `FUNCTION`: **k_audio_amp** <sub>line 6</sub>
```c
extern int32_t k_audio_amp(int32_t on);
```

#### `FUNCTION`: **kvirt_to_phys** <sub>line 7</sub>
```c
extern uint64_t kvirt_to_phys(void *ptr);
```

#### `FUNCTION`: **tr_audio_play** <sub>line 9</sub>
```c
tr_status_t tr_audio_play(const void *data, uint32_t len)
```

#### `FUNCTION`: **k_audio_play** <sub>line 17</sub>
```c
return k_audio_play(phys, len) == 0 ? TR_OK : TR_ERR_IO;
```

#### `FUNCTION`: **tr_audio_stop** <sub>line 20</sub>
```c
tr_status_t tr_audio_stop(void)
```

#### `FUNCTION`: **tr_audio_jack** <sub>line 26</sub>
```c
tr_status_t tr_audio_jack(bool *present)
```

#### `FUNCTION`: **tr_audio_amp** <sub>line 36</sub>
```c
tr_status_t tr_audio_amp(bool on)
```

</details>

<details>
<summary><b>📄 kstd/src/kernel/bt.c</b> (15 items)</summary>

#### `FUNCTION`: **bt_ready** <sub>line 3</sub>
```c
extern bool bt_ready(void);
```

#### `FUNCTION`: **bt_info** <sub>line 4</sub>
```c
extern void bt_info(uint8_t *ver, uint8_t *addr);
```

#### `FUNCTION`: **bt_hci_cmd** <sub>line 5</sub>
```c
extern bool bt_hci_cmd(uint16_t op, const uint8_t *p, uint8_t len);
```

#### `FUNCTION`: **bt_event_poll** <sub>line 6</sub>
```c
extern bool bt_event_poll(uint8_t *buf, uint8_t *len);
```

#### `FUNCTION`: **bt_acl_send** <sub>line 7</sub>
```c
extern bool bt_acl_send(const uint8_t *d, uint16_t len);
```

#### `FUNCTION`: **bt_acl_recv** <sub>line 8</sub>
```c
extern bool bt_acl_recv(uint8_t *d, uint16_t *len);
```

#### `FUNCTION`: **tr_bt_info** <sub>line 10</sub>
```c
tr_status_t tr_bt_info(uint8_t *hci_ver, uint8_t *addr6)
```

#### `FUNCTION`: **tr_bt_cmd** <sub>line 20</sub>
```c
tr_status_t tr_bt_cmd(uint16_t opcode, const void *params, uint8_t len)
```

#### `FUNCTION`: **bt_hci_cmd** <sub>line 22</sub>
```c
return bt_hci_cmd(opcode, params, len) ? TR_OK : TR_ERR_IO;
```

#### `FUNCTION`: **tr_bt_evt** <sub>line 25</sub>
```c
tr_status_t tr_bt_evt(void *buf, uint8_t *len)
```

#### `FUNCTION`: **bt_event_poll** <sub>line 27</sub>
```c
return bt_event_poll(buf, len) ? TR_OK : TR_ERR_TIMEOUT;
```

#### `FUNCTION`: **tr_bt_acl_send** <sub>line 30</sub>
```c
tr_status_t tr_bt_acl_send(const void *data, uint16_t len)
```

#### `FUNCTION`: **bt_acl_send** <sub>line 32</sub>
```c
return bt_acl_send(data, len) ? TR_OK : TR_ERR_BUSY;
```

#### `FUNCTION`: **tr_bt_acl_recv** <sub>line 35</sub>
```c
tr_status_t tr_bt_acl_recv(void *buf, uint16_t *len)
```

#### `FUNCTION`: **bt_acl_recv** <sub>line 37</sub>
```c
return bt_acl_recv(buf, len) ? TR_OK : TR_ERR_TIMEOUT;
```

</details>

<details>
<summary><b>📄 kstd/src/kernel/fs.c</b> (5 items)</summary>

#### `FUNCTION`: **k_fs_read** <sub>line 3</sub>
```c
extern int32_t k_fs_read(const char *path, void *buf, uint32_t cap);
```

#### `FUNCTION`: **k_fs_exists** <sub>line 4</sub>
```c
extern int32_t k_fs_exists(const char *path);
```

#### `FUNCTION`: **tr_fs_read** <sub>line 6</sub>
```c
tr_status_t tr_fs_read(const char *path, void *buf, uint32_t cap, uint32_t *out)
```

#### `FUNCTION`: **tr_fs_exists** <sub>line 21</sub>
```c
tr_status_t tr_fs_exists(const char *path)
```

#### `FUNCTION`: **k_fs_exists** <sub>line 23</sub>
```c
return k_fs_exists(path) == 1 ? TR_OK : TR_ERR_NOTFOUND;
```

</details>

<details>
<summary><b>📄 kstd/src/kernel/input.c</b> (3 items)</summary>

#### `FUNCTION`: **k_input_key** <sub>line 3</sub>
```c
extern int32_t k_input_key(void);
```

#### `FUNCTION`: **tr_input_key** <sub>line 5</sub>
```c
int32_t tr_input_key(void)
```

#### `FUNCTION`: **k_input_key** <sub>line 7</sub>
```c
return k_input_key();
```

</details>

<details>
<summary><b>📄 kstd/src/kernel/log.c</b> (2 items)</summary>

#### `FUNCTION`: **kprintf** <sub>line 3</sub>
```c
extern void kprintf(const char *fmt, ...);
```

#### `FUNCTION`: **tr_log** <sub>line 5</sub>
```c
void tr_log(const char *s)
```

</details>

<details>
<summary><b>📄 kstd/src/kernel/mem.c</b> (6 items)</summary>

#### `FUNCTION`: **kfree** <sub>line 4</sub>
```c
extern void kfree(void *ptr);
```

#### `FUNCTION`: **vmm_map_device** <sub>line 5</sub>
```c
extern bool vmm_map_device(uint64_t phys, unsigned long len, uint64_t *virt);
```

#### `FUNCTION`: **kvirt_to_phys** <sub>line 6</sub>
```c
extern uint64_t kvirt_to_phys(void *ptr);
```

#### `FUNCTION`: **kmalloc** <sub>line 10</sub>
```c
return kmalloc(bytes);
```

#### `FUNCTION`: **tr_free** <sub>line 13</sub>
```c
void tr_free(void *ptr, uint32_t bytes)
```

#### `FUNCTION`: **tr_map_mmio** <sub>line 19</sub>
```c
tr_status_t tr_map_mmio(uint64_t phys, uint32_t len, void **out_va)
```

</details>

<details>
<summary><b>📄 kstd/src/user/audio.c</b> (6 items)</summary>

#### `FUNCTION`: **ds_poll** <sub>line 7</sub>
```c
extern void ds_poll(void);
```

#### `FUNCTION`: **ds_take** <sub>line 8</sub>
```c
extern int ds_take(uint64_t id, ds_msg_t *out);
```

#### `FUNCTION`: **tr_audio_play** <sub>line 13</sub>
```c
tr_status_t tr_audio_play(const void *data, uint32_t len)
```

#### `FUNCTION`: **tr_audio_stop** <sub>line 57</sub>
```c
tr_status_t tr_audio_stop(void)
```

#### `FUNCTION`: **tr_audio_jack** <sub>line 63</sub>
```c
tr_status_t tr_audio_jack(bool *present)
```

#### `FUNCTION`: **tr_audio_amp** <sub>line 82</sub>
```c
tr_status_t tr_audio_amp(bool on)
```

</details>

<details>
<summary><b>📄 kstd/src/user/bt.c</b> (7 items)</summary>

#### `FUNCTION`: **ds_poll** <sub>line 6</sub>
```c
extern void ds_poll(void);
```

#### `FUNCTION`: **ds_take** <sub>line 7</sub>
```c
extern int ds_take(uint64_t id, ds_msg_t *out);
```

#### `FUNCTION`: **tr_bt_info** <sub>line 11</sub>
```c
tr_status_t tr_bt_info(uint8_t *hci_ver, uint8_t *addr6)
```

#### `FUNCTION`: **tr_bt_cmd** <sub>line 31</sub>
```c
tr_status_t tr_bt_cmd(uint16_t opcode, const void *params, uint8_t len)
```

#### `FUNCTION`: **tr_bt_evt** <sub>line 51</sub>
```c
tr_status_t tr_bt_evt(void *buf, uint8_t *len)
```

#### `FUNCTION`: **tr_bt_acl_send** <sub>line 74</sub>
```c
tr_status_t tr_bt_acl_send(const void *data, uint16_t len)
```

#### `FUNCTION`: **tr_bt_acl_recv** <sub>line 94</sub>
```c
tr_status_t tr_bt_acl_recv(void *buf, uint16_t *len)
```

</details>

<details>
<summary><b>📄 kstd/src/user/fs.c</b> (4 items)</summary>

#### `FUNCTION`: **ds_poll** <sub>line 6</sub>
```c
extern void ds_poll(void);
```

#### `FUNCTION`: **ds_take** <sub>line 7</sub>
```c
extern int ds_take(uint64_t id, ds_msg_t *out);
```

#### `FUNCTION`: **tr_fs_read** <sub>line 11</sub>
```c
tr_status_t tr_fs_read(const char *path, void *buf, uint32_t cap, uint32_t *out)
```

#### `FUNCTION`: **tr_fs_exists** <sub>line 50</sub>
```c
tr_status_t tr_fs_exists(const char *path)
```

</details>

<details>
<summary><b>📄 kstd/src/user/input.c</b> (3 items)</summary>

#### `FUNCTION`: **ds_poll** <sub>line 6</sub>
```c
extern void ds_poll(void);
```

#### `FUNCTION`: **ds_take** <sub>line 7</sub>
```c
extern int ds_take(uint64_t id, ds_msg_t *out);
```

#### `FUNCTION`: **tr_input_key** <sub>line 9</sub>
```c
int32_t tr_input_key(void)
```

</details>

<details>
<summary><b>📄 kstd/src/user/log.c</b> (1 items)</summary>

#### `FUNCTION`: **tr_log** <sub>line 6</sub>
```c
void tr_log(const char *s)
```

</details>

<details>
<summary><b>📄 kstd/src/user/mem.c</b> (4 items)</summary>

#### `FUNCTION`: **ds_poll** <sub>line 6</sub>
```c
extern void ds_poll(void);
```

#### `FUNCTION`: **ds_take** <sub>line 7</sub>
```c
extern int ds_take(uint64_t id, ds_msg_t *out);
```

#### `FUNCTION`: **tr_free** <sub>line 27</sub>
```c
void tr_free(void *ptr, uint32_t bytes)
```

#### `FUNCTION`: **tr_map_mmio** <sub>line 33</sub>
```c
tr_status_t tr_map_mmio(uint64_t phys, uint32_t len, void **out_va)
```

</details>

## 📂 libs

<details>
<summary><b>📄 libs/dsasync.h</b> (2 items)</summary>

#### `FUNCTION`: **ds_req_poll** <sub>line 19</sub>
```c
static inline int ds_req_poll(ds_req_t *r)
```

#### `FUNCTION`: **ds_req_ok** <sub>line 34</sub>
```c
static inline int ds_req_ok(ds_req_t *r)
```

</details>

<details>
<summary><b>📄 libs/dsaudio.h</b> (6 items)</summary>

#### `FUNCTION`: **ds_audio_play** <sub>line 5</sub>
```c
static inline uint64_t ds_audio_play(uint64_t phys, uint32_t len)
```

#### `FUNCTION`: **ds_call** <sub>line 7</sub>
```c
return ds_call(SVC_AUDIO, AUD_PLAY, phys, len, 0);
```

#### `FUNCTION`: **ds_audio_stop** <sub>line 10</sub>
```c
static inline void ds_audio_stop(void)
```

#### `FUNCTION`: **ds_audio_jack_req** <sub>line 15</sub>
```c
static inline uint64_t ds_audio_jack_req(void)
```

#### `FUNCTION`: **ds_call** <sub>line 17</sub>
```c
return ds_call(SVC_AUDIO, AUD_JACK, 0, 0, 0);
```

#### `FUNCTION`: **ds_audio_amp** <sub>line 20</sub>
```c
static inline void ds_audio_amp(int on)
```

</details>

<details>
<summary><b>📄 libs/dsclient.c</b> (3 items)</summary>

#### `FUNCTION`: **ds_init** <sub>line 12</sub>
```c
void ds_init(uint64_t params_va)
```

#### `FUNCTION`: **ds_poll** <sub>line 49</sub>
```c
void ds_poll(void)
```

#### `FUNCTION`: **ds_take** <sub>line 67</sub>
```c
int ds_take(uint64_t id, ds_msg_t *out)
```

</details>

<details>
<summary><b>📄 libs/dsinput.h</b> (3 items)</summary>

#### `FUNCTION`: **ds_key_req** <sub>line 5</sub>
```c
static inline uint64_t ds_key_req(void)
```

#### `FUNCTION`: **ds_call** <sub>line 7</sub>
```c
return ds_call(SVC_INPUT, IN_KEY_POLL, 0, 0, 0);
```

#### `FUNCTION`: **ds_key_take** <sub>line 10</sub>
```c
static inline int ds_key_take(uint64_t id, uint8_t *out)
```

</details>

<details>
<summary><b>📄 libs/dslock.h</b> (6 items)</summary>

#### `FUNCTION`: **ds_blk_count** <sub>line 5</sub>
```c
static inline uint64_t ds_blk_count(uint64_t disk)
```

#### `FUNCTION`: **ds_call** <sub>line 7</sub>
```c
return ds_call(SVC_BLOCK, BLK_COUNT, disk, 0, 0);
```

#### `FUNCTION`: **ds_blk_read** <sub>line 10</sub>
```c
static inline uint64_t ds_blk_read(uint64_t disk, uint64_t block)
```

#### `FUNCTION`: **ds_call** <sub>line 12</sub>
```c
return ds_call(SVC_BLOCK, BLK_READ, disk, block, 0);
```

#### `FUNCTION`: **ds_blk_write** <sub>line 15</sub>
```c
static inline uint64_t ds_blk_write(uint64_t disk, uint64_t block)
```

#### `FUNCTION`: **ds_call** <sub>line 17</sub>
```c
return ds_call(SVC_BLOCK, BLK_WRITE, disk, block, 0);
```

</details>

<details>
<summary><b>📄 libs/dslog.h</b> (1 items)</summary>

#### `FUNCTION`: **ds_log** <sub>line 5</sub>
```c
static inline void ds_log(const char *s)
```

</details>

<details>
<summary><b>📄 libs/dsmem.h</b> (8 items)</summary>

#### `FUNCTION`: **ds_alloc_pages** <sub>line 5</sub>
```c
static inline uint64_t ds_alloc_pages(uint32_t pages)
```

#### `FUNCTION`: **ds_call** <sub>line 7</sub>
```c
return ds_call(SVC_SYS, OP_ALLOC, pages, 0, 0);
```

#### `FUNCTION`: **ds_free_pages** <sub>line 10</sub>
```c
static inline uint64_t ds_free_pages(uint64_t va)
```

#### `FUNCTION`: **ds_call** <sub>line 12</sub>
```c
return ds_call(SVC_SYS, OP_FREE, va, 0, 0);
```

#### `FUNCTION`: **ds_map_mmio** <sub>line 15</sub>
```c
static inline uint64_t ds_map_mmio(uint64_t phys, uint64_t len, uint64_t va)
```

#### `FUNCTION`: **ds_call** <sub>line 17</sub>
```c
return ds_call(SVC_SYS, OP_MAPMMIO, phys, len, va);
```

#### `FUNCTION`: **ds_page_phys** <sub>line 20</sub>
```c
static inline uint64_t ds_page_phys(uint64_t va)
```

#### `FUNCTION`: **ds_call** <sub>line 22</sub>
```c
return ds_call(SVC_SYS, OP_PAGEPHYS, va, 0, 0);
```

</details>

<details>
<summary><b>📄 libs/dsvgpu.h</b> (5 items)</summary>

#### `FUNCTION`: **ds_call** <sub>line 6</sub>
```c
extern uint64_t ds_call(uint32_t, uint32_t, uint64_t, uint64_t, uint64_t);
```

#### `FUNCTION`: **ds_poll** <sub>line 7</sub>
```c
extern void ds_poll(void);
```

#### `FUNCTION`: **ds_take** <sub>line 8</sub>
```c
extern int ds_take(uint64_t, ds_msg_t *);
```

#### `FUNCTION`: **dsvgpu_present** <sub>line 27</sub>
```c
static inline int dsvgpu_present(int32_t sid, uint32_t x, uint32_t y)
```

#### `FUNCTION`: **ds_take** <sub>line 34</sub>
```c
return ds_take(id, &r) && r.status == 0;
```

</details>

<details>
<summary><b>📄 libs/dsvideo.h</b> (6 items)</summary>

#### `FUNCTION`: **ds_fb_info_req** <sub>line 12</sub>
```c
static inline uint64_t ds_fb_info_req(void)
```

#### `FUNCTION`: **ds_call** <sub>line 14</sub>
```c
return ds_call(SVC_VIDEO, VID_FB_INFO, 0, 0, 0);
```

#### `FUNCTION`: **ds_fb_info_take** <sub>line 17</sub>
```c
static inline int ds_fb_info_take(uint64_t id, ds_fb_t *fb)
```

#### `FUNCTION`: **ds_fb_takeover** <sub>line 33</sub>
```c
static inline void ds_fb_takeover(void)
```

#### `FUNCTION`: **ds_fb_release** <sub>line 38</sub>
```c
static inline void ds_fb_release(void)
```

#### `FUNCTION`: **ds_rgb** <sub>line 49</sub>
```c
static inline uint32_t ds_rgb(uint8_t r, uint8_t g, uint8_t b)
```

</details>

## 📂 mp4_to_bmp/src

<details>
<summary><b>📄 mp4_to_bmp/src/main.rs</b> (6 items)</summary>

#### `FN`: **main** <sub>line 23</sub>
```rust
fn main() {
```

#### `FN`: **run** <sub>line 30</sub>
```rust
fn run() -> Result<(), String> {
```

#### `FN`: **check_tool** <sub>line 103</sub>
```rust
fn check_tool(name: &str) -> Result<(), String> {
```

#### `FN`: **probe_resolution** <sub>line 114</sub>
```rust
fn probe_resolution(input: &str) -> Result<(usize, usize), String> {
```
> Pobiera szerokosc i wysokosc pierwszego strumienia wideo przez ffprobe.

#### `FN`: **read_exact_or_eof** <sub>line 152</sub>
```rust
fn read_exact_or_eof<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<bool, String> {
```
> Czyta dokladnie `buf.len()` bajtow. Zwraca Ok(true) jesli sie udalo, 
> Ok(false) jesli strumien skonczyl sie dokladnie na granicy klatki 
> (koniec pliku), Err jesli urwal sie w polowie klatki (uszkodzony strumien).

#### `FN`: **write_bmp** <sub>line 173</sub>
```rust
fn write_bmp(path: &str, pixels: &[u8], width: usize, height: usize) -> std::io::Result<()> {
```
> BITMAPINFOHEADER + piksele). `pixels` to width*height*3 bajtow w kolejnosci 
> B,G,R, wiersz po wierszu, gora->dol (uzywamy ujemnej wysokosci w naglowku, 
> zeby BMP tez czytal je jako top-down — bez potrzeby odwracania wierszy).

</details>

## 📂 trangorgelibc/src

<details>
<summary><b>📄 trangorgelibc/src/abi/errno.rs</b> (2 items)</summary>

#### `ENUM`: **Errno** <sub>line 5</sub>
```rust
pub enum Errno {
```

#### `TYPE`: **TResult** <sub>line 19</sub>
```rust
pub type TResult<T> = Result<T, Errno>;
```

</details>

<details>
<summary><b>📄 trangorgelibc/src/abi/ktable.rs</b> (4 items)</summary>

#### `STRUCT`: **KernelTable** <sub>line 6</sub>
```rust
pub struct KernelTable {
```

#### `IMPL`: **KernelTable** <sub>line 24</sub>
```rust
impl KernelTable {
```

#### `FN`: **validate** <sub>line 25</sub>
```rust
pub fn validate(&self) -> bool {
```

#### `FN`: **print** <sub>line 30</sub>
```rust
pub fn print(&self, s: &str) {
```

</details>

<details>
<summary><b>📄 trangorgelibc/src/abi/syscall.rs</b> (1 items)</summary>

#### `ENUM`: **Syscall** <sub>line 4</sub>
```rust
pub enum Syscall {
```

</details>

<details>
<summary><b>📄 trangorgelibc/src/abi/types.rs</b> (5 items)</summary>

#### `STRUCT`: **SystemInfo** <sub>line 4</sub>
```rust
pub struct SystemInfo {
```

#### `STRUCT`: **Handle** <sub>line 14</sub>
```rust
pub struct Handle(pub u64);
```

#### `IMPL`: **Handle** <sub>line 16</sub>
```rust
impl Handle {
```

#### `FN`: **is_valid** <sub>line 19</sub>
```rust
pub fn is_valid(self) -> bool {
```

#### `STRUCT`: **Stat** <sub>line 26</sub>
```rust
pub struct Stat {
```

</details>

<details>
<summary><b>📄 trangorgelibc/src/lib.rs</b> (27 items)</summary>

#### `FN`: **sc0** <sub>line 31</sub>
```rust
fn sc0(n: u64) -> u64 {
```

#### `FN`: **sc1** <sub>line 38</sub>
```rust
fn sc1(n: u64, a0: u64) -> u64 {
```

#### `FN`: **sc3** <sub>line 45</sub>
```rust
fn sc3(n: u64, a0: u64, a1: u64, a2: u64) -> u64 {
```

#### `FN`: **cstr** <sub>line 58</sub>
```rust
fn cstr(s: &str) -> ([u8; 256], usize) {
```
>  
> Jadro czyta sciezki/teksty jako C-stringi, a `&str` z Rusta nie ma bajtu 
> NUL na koncu, dlatego kazda sciezka przechodzi przez ten pomocnik.

#### `FN`: **log** <sub>line 69</sub>
```rust
pub fn log(s: &str) {
```

#### `FN`: **print** <sub>line 75</sub>
```rust
pub fn print(s: &str) {
```
> Wypisuje tekst na konsole uzytkownika (na razie tym samym kanalem co `log`).

#### `FN`: **yield_cpu** <sub>line 79</sub>
```rust
pub fn yield_cpu() { sc0(SYS_YIELD); }
```

#### `FN`: **exit** <sub>line 81</sub>
```rust
pub fn exit(code: i32) -> ! {
```

#### `FN`: **getpid** <sub>line 86</sub>
```rust
pub fn getpid() -> u32 { sc0(SYS_GETPID) as u32 }
```

#### `FN`: **spawn** <sub>line 88</sub>
```rust
pub fn spawn(path: &str) -> i32 {
```

#### `FN`: **key** <sub>line 92</sub>
```rust
pub fn key() -> Option<u8> {
```

#### `FN`: **ipc_send** <sub>line 96</sub>
```rust
pub fn ipc_send(pid: u32, a0: u64, a1: u64) -> bool {
```

#### `STRUCT`: **Mail** <sub>line 100</sub>
```rust
pub struct Mail { pub from: u32, pub a0: u64, pub a1: u64 }
```

#### `FN`: **ipc_recv** <sub>line 102</sub>
```rust
pub fn ipc_recv() -> Option<Mail> {
```

#### `FN`: **open** <sub>line 119</sub>
```rust
pub fn open(path: &str) -> i32 {
```

#### `FN`: **read** <sub>line 124</sub>
```rust
pub fn read(fd: i32, buf: &mut [u8]) -> i32 {
```

#### `FN`: **write** <sub>line 128</sub>
```rust
pub fn write(fd: i32, buf: &[u8]) -> i32 {
```

#### `FN`: **close** <sub>line 132</sub>
```rust
pub fn close(fd: i32) -> i32 {
```

#### `FN`: **wait** <sub>line 137</sub>
```rust
pub fn wait() -> Option<(u32, i32)> {
```
> Czeka na zakonczenie dziecka. Zwraca `(pid, kod_wyjscia)` albo `None`.

#### `FN`: **readdir** <sub>line 148</sub>
```rust
pub fn readdir(idx: u64, name: &mut [u8]) -> Option<u8> {
```
> Wpis katalogu: `Some(typ)` (1 = plik, 2 = katalog), nazwa trafia do `name`.

#### `FN`: **runcl** <sub>line 154</sub>
```rust
pub fn runcl(path: &str) -> i32 {
```
> Uruchamia program w jezyku Trangorge (core-lang) z podanej sciezki.

#### `FN`: **ui_open** <sub>line 170</sub>
```rust
pub fn ui_open() -> Option<(u32, u32, u32)> {
```
> Mapuje framebuffer i czcionke do przestrzeni uzytkownika. 
> Zwraca `(szerokosc, wysokosc, stride_w_bajtach)`.

#### `FN`: **ui_pixel** <sub>line 187</sub>
```rust
pub fn ui_pixel(stride: u32, x: i32, y: i32, color: u32, w: u32, h: u32) {
```
> Ustawia jeden piksel (32-bit, format framebuffera).

#### `FN`: **ui_clear** <sub>line 199</sub>
```rust
pub fn ui_clear(stride: u32, w: u32, h: u32, color: u32) {
```
> Wypelnia caly ekran jednym kolorem.

#### `FN`: **ui_text** <sub>line 215</sub>
```rust
pub fn ui_text(stride: u32, x: i32, y: i32, s: &str, color: u32, w: u32, h: u32) {
```
> Rysuje tekst czcionka 8x8 (`font8x8` z jadra: glif `c` ma indeks `c - 32`).

#### `FN`: **malloc** <sub>line 242</sub>
```rust
pub fn malloc(n: usize) -> *mut u8 {
```

#### `FN`: **put_u32** <sub>line 253</sub>
```rust
pub fn put_u32(v: u32) {
```

</details>

## 📂 triang-lang/src

<details>
<summary><b>📄 triang-lang/src/ast.rs</b> (11 items)</summary>

#### `STRUCT`: **Program** <sub>line 2</sub>
```rust
pub struct Program {
```

#### `STRUCT`: **Function** <sub>line 7</sub>
```rust
pub struct Function {
```

#### `ENUM`: **Param** <sub>line 15</sub>
```rust
pub enum Param {
```

#### `ENUM`: **Type** <sub>line 21</sub>
```rust
pub enum Type {
```

#### `STRUCT`: **Layout** <sub>line 27</sub>
```rust
pub struct Layout {
```

#### `ENUM`: **Stmt** <sub>line 34</sub>
```rust
pub enum Stmt {
```

#### `STRUCT`: **OpCall** <sub>line 55</sub>
```rust
pub struct OpCall {
```

#### `ENUM`: **Target** <sub>line 62</sub>
```rust
pub enum Target {
```

#### `ENUM`: **Expr** <sub>line 69</sub>
```rust
pub enum Expr {
```

#### `STRUCT`: **Cond** <sub>line 76</sub>
```rust
pub struct Cond {
```

#### `ENUM`: **CmpOp** <sub>line 83</sub>
```rust
pub enum CmpOp {
```

</details>

<details>
<summary><b>📄 triang-lang/src/codegen/asm.rs</b> (7 items)</summary>

#### `STRUCT`: **Emitter** <sub>line 8</sub>
```rust
struct Emitter {
```

#### `IMPL`: **Emitter** <sub>line 13</sub>
```rust
impl Emitter {
```

#### `FN`: **label** <sub>line 14</sub>
```rust
fn label(&mut self, tag: &str) -> String {
```

#### `FN`: **line** <sub>line 19</sub>
```rust
fn line(&mut self, s: &str) {
```

#### `FN`: **op** <sub>line 26</sub>
```rust
fn op(t: &Target, v: &Val) -> String {
```

#### `FN`: **mem_len** <sub>line 33</sub>
```rust
fn mem_len(ir: &[Ir], name: &str) -> u64 {
```

#### `FN`: **emit** <sub>line 44</sub>
```rust
pub fn emit(ir: &[Ir]) -> String {
```

</details>

<details>
<summary><b>📄 triang-lang/src/codegen/c.rs</b> (7 items)</summary>

#### `STRUCT`: **FnChunk** <sub>line 4</sub>
```rust
struct FnChunk {
```

#### `FN`: **val** <sub>line 11</sub>
```rust
fn val(v: &Val) -> String {
```

#### `FN`: **binop** <sub>line 18</sub>
```rust
fn binop(op: BinOp) -> &'static str {
```

#### `FN`: **cmpop** <sub>line 30</sub>
```rust
fn cmpop(op: CmpOp) -> &'static str {
```

#### `FN`: **is_param** <sub>line 37</sub>
```rust
fn is_param(name: &str, args: usize) -> bool {
```

#### `FN`: **split** <sub>line 47</sub>
```rust
fn split(ir: &[Ir]) -> Vec<FnChunk> {
```

#### `FN`: **emit** <sub>line 67</sub>
```rust
pub fn emit(ir: &[Ir]) -> String {
```

</details>

<details>
<summary><b>📄 triang-lang/src/codegen/mod.rs</b> (1 items)</summary>

#### `ENUM`: **Emit** <sub>line 6</sub>
```rust
pub enum Emit {
```

</details>

<details>
<summary><b>📄 triang-lang/src/codegen/target.rs</b> (4 items)</summary>

#### `ENUM`: **Target** <sub>line 2</sub>
```rust
pub enum Target {
```

#### `IMPL`: **Target** <sub>line 8</sub>
```rust
impl Target {
```

#### `FN`: **reg** <sub>line 9</sub>
```rust
pub fn reg(&self, name: &str) -> &'static str {
```

#### `FN`: **ret_reg** <sub>line 19</sub>
```rust
pub fn ret_reg(&self) -> &'static str {
```

</details>

<details>
<summary><b>📄 triang-lang/src/ir.rs</b> (14 items)</summary>

#### `ENUM`: **Val** <sub>line 5</sub>
```rust
pub enum Val {
```

#### `ENUM`: **BinOp** <sub>line 11</sub>
```rust
pub enum BinOp {
```

#### `ENUM`: **Ir** <sub>line 22</sub>
```rust
pub enum Ir {
```

#### `STRUCT`: **Lower** <sub>line 43</sub>
```rust
pub struct Lower {
```

#### `IMPL`: **Lower** <sub>line 49</sub>
```rust
impl Lower {
```

#### `FN`: **new** <sub>line 50</sub>
```rust
pub fn new() -> Self {
```

#### `FN`: **label** <sub>line 58</sub>
```rust
fn label(&mut self, tag: &str) -> String {
```

#### `FN`: **lower_program** <sub>line 63</sub>
```rust
pub fn lower_program(mut self, program: &Program) -> Vec<Ir> {
```

#### `FN`: **lower_function** <sub>line 70</sub>
```rust
fn lower_function(&mut self, f: &Function) {
```

#### `FN`: **lower_stmt** <sub>line 83</sub>
```rust
fn lower_stmt(&mut self, s: &Stmt) {
```

#### `FN`: **lower_op** <sub>line 143</sub>
```rust
fn lower_op(&mut self, call: &OpCall) {
```

#### `FN`: **val** <sub>line 263</sub>
```rust
fn val(e: &Expr) -> Val {
```

#### `FN`: **invert** <sub>line 271</sub>
```rust
fn invert(op: CmpOp) -> CmpOp {
```

#### `FN`: **binop** <sub>line 278</sub>
```rust
fn binop(s: &str) -> BinOp {
```

</details>

<details>
<summary><b>📄 triang-lang/src/iso.rs</b> (8 items)</summary>

#### `STRUCT`: **IsoFile** <sub>line 1</sub>
```rust
pub struct IsoFile {
```

#### `FN`: **dstring** <sub>line 8</sub>
```rust
fn dstring(s: &str, len: usize) -> Vec<u8> {
```

#### `FN`: **push_both_u16** <sub>line 18</sub>
```rust
fn push_both_u16(out: &mut Vec<u8>, v: u16) {
```

#### `FN`: **push_both_u32** <sub>line 23</sub>
```rust
fn push_both_u32(out: &mut Vec<u8>, v: u32) {
```

#### `FN`: **put_both_u16** <sub>line 28</sub>
```rust
fn put_both_u16(buf: &mut [u8], off: usize, v: u16) {
```

#### `FN`: **put_both_u32** <sub>line 33</sub>
```rust
fn put_both_u32(buf: &mut [u8], off: usize, v: u32) {
```

#### `FN`: **dir_record** <sub>line 38</sub>
```rust
fn dir_record(name: &[u8], extent: u32, size: u32, flags: u8) -> Vec<u8> {
```

#### `FN`: **build** <sub>line 58</sub>
```rust
pub fn build(volume: &str, files: &[IsoFile]) -> Vec<u8> {
```

</details>

<details>
<summary><b>📄 triang-lang/src/lexer.rs</b> (13 items)</summary>

#### `STRUCT`: **LexError** <sub>line 5</sub>
```rust
pub struct LexError {
```

#### `STRUCT`: **Lexer** <sub>line 11</sub>
```rust
pub struct Lexer {
```

#### `IMPL`: **Lexer** <sub>line 18</sub>
```rust
impl Lexer {
```

#### `FN`: **new** <sub>line 19</sub>
```rust
pub fn new(src: &str) -> Self {
```

#### `FN`: **peek** <sub>line 28</sub>
```rust
fn peek(&self) -> Option<char> {
```

#### `FN`: **peek2** <sub>line 32</sub>
```rust
fn peek2(&self) -> Option<char> {
```

#### `FN`: **bump** <sub>line 36</sub>
```rust
fn bump(&mut self) -> Option<char> {
```

#### `FN`: **tokenize** <sub>line 50</sub>
```rust
pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
```

#### `FN`: **skip_ws_and_comments** <sub>line 98</sub>
```rust
fn skip_ws_and_comments(&mut self) {
```

#### `FN`: **read_number** <sub>line 114</sub>
```rust
fn read_number(&mut self) -> Result<TokenKind, LexError> {
```

#### `FN`: **read_ident** <sub>line 161</sub>
```rust
fn read_ident(&mut self) -> TokenKind {
```

#### `FN`: **is_ident_start** <sub>line 192</sub>
```rust
fn is_ident_start(c: char) -> bool {
```

#### `FN`: **is_ident_cont** <sub>line 196</sub>
```rust
fn is_ident_cont(c: char) -> bool {
```

</details>

<details>
<summary><b>📄 triang-lang/src/main.rs</b> (10 items)</summary>

#### `FN`: **main** <sub>line 18</sub>
```rust
fn main() {
```

#### `FN`: **read_libs** <sub>line 29</sub>
```rust
fn read_libs(dir: &str) -> Vec<String> {
```

#### `FN`: **pipeline** <sub>line 50</sub>
```rust
fn pipeline(srcs: &[String]) -> Vec<ir::Ir> {
```

#### `FN`: **cmd_new** <sub>line 83</sub>
```rust
fn cmd_new(args: &[String]) {
```

#### `FN`: **cmd_build** <sub>line 114</sub>
```rust
fn cmd_build(args: &[String]) {
```

#### `FN`: **cmd_file** <sub>line 178</sub>
```rust
fn cmd_file(args: &[String]) {
```

#### `FN`: **cmd_iso** <sub>line 256</sub>
```rust
fn cmd_iso(_args: &[String]) {
```

#### `FN`: **route_c** <sub>line 288</sub>
```rust
fn route_c(ir: &[ir::Ir], text_path: &str, elf_path: &str, bin_path: &str, want_elf: bool, want_bin: bool) {
```

#### `FN`: **route_asm** <sub>line 309</sub>
```rust
fn route_asm(
```

#### `FN`: **run_objcopy** <sub>line 357</sub>
```rust
fn run_objcopy(elf: &str, bin: &str) -> bool {
```

</details>

<details>
<summary><b>📄 triang-lang/src/parser.rs</b> (31 items)</summary>

#### `STRUCT`: **ParseError** <sub>line 5</sub>
```rust
pub struct ParseError {
```

#### `STRUCT`: **Parser** <sub>line 11</sub>
```rust
pub struct Parser {
```

#### `IMPL`: **Parser** <sub>line 16</sub>
```rust
impl Parser {
```

#### `FN`: **new** <sub>line 17</sub>
```rust
pub fn new(tokens: Vec<Token>) -> Self {
```

#### `FN`: **cur** <sub>line 21</sub>
```rust
fn cur(&self) -> &Token {
```

#### `FN`: **kind** <sub>line 25</sub>
```rust
fn kind(&self) -> TokenKind {
```

#### `FN`: **at** <sub>line 29</sub>
```rust
fn at(&self, kind: &TokenKind) -> bool {
```

#### `FN`: **bump** <sub>line 33</sub>
```rust
fn bump(&mut self) -> TokenKind {
```

#### `FN`: **err** <sub>line 41</sub>
```rust
fn err(&self, msg: String) -> ParseError {
```

#### `FN`: **expect** <sub>line 49</sub>
```rust
fn expect(&mut self, want: TokenKind) -> Result<TokenKind, ParseError> {
```

#### `FN`: **expect_ident** <sub>line 57</sub>
```rust
fn expect_ident(&mut self) -> Result<String, ParseError> {
```

#### `FN`: **expect_int** <sub>line 64</sub>
```rust
fn expect_int(&mut self) -> Result<u64, ParseError> {
```

#### `FN`: **parse_program** <sub>line 71</sub>
```rust
pub fn parse_program(&mut self) -> Result<Program, ParseError> {
```

#### `FN`: **parse_function** <sub>line 79</sub>
```rust
fn parse_function(&mut self) -> Result<Function, ParseError> {
```

#### `FN`: **parse_block** <sub>line 111</sub>
```rust
fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
```

#### `FN`: **parse_stmt** <sub>line 120</sub>
```rust
fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
```

#### `FN`: **parse_type** <sub>line 134</sub>
```rust
fn parse_type(&mut self) -> Result<Type, ParseError> {
```

#### `FN`: **parse_layout** <sub>line 148</sub>
```rust
fn parse_layout(&mut self) -> Result<Layout, ParseError> {
```

#### `FN`: **parse_const** <sub>line 160</sub>
```rust
fn parse_const(&mut self) -> Result<Stmt, ParseError> {
```

#### `FN`: **parse_static** <sub>line 169</sub>
```rust
fn parse_static(&mut self) -> Result<Stmt, ParseError> {
```

#### `FN`: **parse_reg** <sub>line 187</sub>
```rust
fn parse_reg(&mut self) -> Result<Stmt, ParseError> {
```

#### `FN`: **parse_mem** <sub>line 195</sub>
```rust
fn parse_mem(&mut self) -> Result<Stmt, ParseError> {
```

#### `FN`: **parse_target** <sub>line 207</sub>
```rust
fn parse_target(&mut self) -> Result<Target, ParseError> {
```

#### `FN`: **parse_op_call** <sub>line 219</sub>
```rust
fn parse_op_call(&mut self) -> Result<OpCall, ParseError> {
```

#### `FN`: **parse_op_suffix** <sub>line 224</sub>
```rust
fn parse_op_suffix(&mut self, target: Target) -> Result<OpCall, ParseError> {
```

#### `FN`: **parse_op_stmt** <sub>line 239</sub>
```rust
fn parse_op_stmt(&mut self) -> Result<Stmt, ParseError> {
```

#### `FN`: **parse_expr** <sub>line 245</sub>
```rust
fn parse_expr(&mut self) -> Result<Expr, ParseError> {
```

#### `FN`: **parse_cond** <sub>line 266</sub>
```rust
fn parse_cond(&mut self) -> Result<Cond, ParseError> {
```

#### `FN`: **parse_if** <sub>line 277</sub>
```rust
fn parse_if(&mut self) -> Result<Stmt, ParseError> {
```

#### `FN`: **parse_while** <sub>line 294</sub>
```rust
fn parse_while(&mut self) -> Result<Stmt, ParseError> {
```

#### `FN`: **parse_return** <sub>line 305</sub>
```rust
fn parse_return(&mut self) -> Result<Stmt, ParseError> {
```

</details>

<details>
<summary><b>📄 triang-lang/src/project.rs</b> (4 items)</summary>

#### `STRUCT`: **Project** <sub>line 4</sub>
```rust
pub struct Project {
```

#### `IMPL`: **Project** <sub>line 11</sub>
```rust
impl Project {
```

#### `FN`: **load** <sub>line 12</sub>
```rust
pub fn load(path: &str) -> Result<Project, String> {
```

#### `FN`: **parse** <sub>line 19</sub>
```rust
fn parse(text: &str) -> Project {
```

</details>

<details>
<summary><b>📄 triang-lang/src/sema.rs</b> (15 items)</summary>

#### `ENUM`: **Symbol** <sub>line 5</sub>
```rust
pub enum Symbol {
```

#### `STRUCT`: **SemaError** <sub>line 11</sub>
```rust
pub struct SemaError {
```

#### `STRUCT`: **Sema** <sub>line 15</sub>
```rust
pub struct Sema {
```

#### `IMPL`: **Sema** <sub>line 20</sub>
```rust
impl Sema {
```

#### `FN`: **new** <sub>line 21</sub>
```rust
pub fn new() -> Self {
```

#### `FN`: **err** <sub>line 28</sub>
```rust
fn err(&self, msg: String) -> SemaError {
```

#### `FN`: **check** <sub>line 32</sub>
```rust
pub fn check(&mut self, program: &Program) -> Result<(), SemaError> {
```

#### `FN`: **check_function** <sub>line 43</sub>
```rust
fn check_function(&mut self, f: &Function) -> Result<(), SemaError> {
```

#### `FN`: **declare** <sub>line 62</sub>
```rust
fn declare(&mut self, name: &str, sym: Symbol) -> Result<(), SemaError> {
```

#### `FN`: **check_stmt** <sub>line 70</sub>
```rust
fn check_stmt(&mut self, s: &Stmt) -> Result<(), SemaError> {
```

#### `FN`: **check_cond** <sub>line 104</sub>
```rust
fn check_cond(&self, cond: &Cond) -> Result<(), SemaError> {
```

#### `FN`: **check_operand** <sub>line 109</sub>
```rust
fn check_operand(&self, e: &Expr) -> Result<(), SemaError> {
```

#### `FN`: **check_mem_access** <sub>line 125</sub>
```rust
fn check_mem_access(&self, name: &str, idx: u64) -> Result<(), SemaError> {
```

#### `FN`: **expect_arity** <sub>line 141</sub>
```rust
fn expect_arity(&self, call: &OpCall, n: usize) -> Result<(), SemaError> {
```

#### `FN`: **check_op** <sub>line 149</sub>
```rust
fn check_op(&self, call: &OpCall) -> Result<(), SemaError> {
```

</details>

<details>
<summary><b>📄 triang-lang/src/token.rs</b> (2 items)</summary>

#### `ENUM`: **TokenKind** <sub>line 4</sub>
```rust
pub enum TokenKind {
```

#### `STRUCT`: **Token** <sub>line 41</sub>
```rust
pub struct Token {
```

</details>

## 📂 userspace-legasy/demo

<details>
<summary><b>📄 userspace-legasy/demo/src/main.rs</b> (1 items)</summary>

#### `FN`: **panic** <sub>line 24</sub>
```rust
fn panic(_i: &core::panic::PanicInfo) -> ! { loop {} }
```

</details>

## 📂 userspace-legasy/init

<details>
<summary><b>📄 userspace-legasy/init/src/main.rs</b> (2 items)</summary>

#### `FN`: **load_autostart** <sub>line 6</sub>
```rust
fn load_autostart() -> bool {
```

#### `FN`: **panic** <sub>line 73</sub>
```rust
fn panic(_i: &core::panic::PanicInfo) -> ! {
```

</details>

## 📂 userspace-legasy/shell

<details>
<summary><b>📄 userspace-legasy/shell/src/main.rs</b> (3 items)</summary>

#### `FN`: **cstr** <sub>line 13</sub>
```rust
fn cstr(buf: &[u8]) -> &str {
```
> Zamienia bufor zakonczony NUL na `&str` (nazwy z `readdir`).

#### `FN`: **run** <sub>line 48</sub>
```rust
fn run(cmd: &str) {
```

#### `FN`: **panic** <sub>line 141</sub>
```rust
fn panic(_i: &core::panic::PanicInfo) -> ! { loop {} }
```

</details>

## 📂 userspace-legasy/terminal

<details>
<summary><b>📄 userspace-legasy/terminal/src/main.rs</b> (2 items)</summary>

#### `FN`: **push_hist** <sub>line 16</sub>
```rust
fn push_hist(line: &[u8]) {
```

#### `FN`: **panic** <sub>line 107</sub>
```rust
fn panic(_i: &core::panic::PanicInfo) -> ! {
```

</details>

