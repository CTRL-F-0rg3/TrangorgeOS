//! IOMMU 设备类框架（`ds-fw-iommu`）。
//!
//! 与 `ds-fw-gpu` / `ds-fw-audio` 同级：这里只放**设备类的契约与线路词汇**，
//! 不放任何硬件寄存器逻辑。硬件部分留在 `drivers/iommu-driver`，管理部分留在
//! `crates/ds-manager`——两者都只依赖本 crate，从而满足工作区的
//! “零重复” 与 “`drivers/` 不得依赖 `crates/`” 规则。
//!
//! 分成三块：
//!
//! * [`traits::IommuDevice`] —— 驱动侧要实现的契约。
//! * [`service::IommuService`] —— 服务侧分发器：把 `DsMsg` 翻译成
//!   [`traits::IommuDevice`] 调用，并强制做能力检查。
//! * [`client::IommuClient`] —— 客户端：把调用编码成 `DsCmd` IPC。
//!
//! 线缆格式（`#[repr(C)]` 载荷 + opcode）全部来自 `kapi-abi`，本 crate
//! 不新增任何 ABI 表面。

#![no_std]

pub mod client;
pub mod codec;
pub mod service;
pub mod traits;
pub mod types;

pub use client::IommuClient;
pub use codec::MAX_PAYLOAD_LEN;
pub use service::{IommuService, Reply, Request};
pub use traits::IommuDevice;
pub use types::{ControllerId, DomainId, IoRange, RequesterId};
