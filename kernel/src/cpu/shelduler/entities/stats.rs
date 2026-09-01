// stats.rs
// Moduł statystyk i monitorowania wydajności dla szkieletu jądra.
// Zapewnia:
// - globalne liczniki atomowe,
// - per-CPU struktury statystyk (do umieszczenia w RunQueue),
// - funkcje do śledzenia czasów wykonywania, przełączeń, migracji,
// - proste obliczanie load average,
// - snapshoty i formatowanie do celów debugowania.

use crate::cpu::scheduler::task::TaskStruct;
use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------
// Globalne liczniki atomowe – sumaryczne dla całego systemu
// ---------------------------------------------------------------------

static TOTAL_SWITCHES: AtomicU64 = AtomicU64::new(0);
static TOTAL_VOLUNTARY_SWITCHES: AtomicU64 = AtomicU64::new(0);
static TOTAL_INVOLUNTARY_SWITCHES: AtomicU64 = AtomicU64::new(0);
static TOTAL_MIGRATIONS: AtomicU64 = AtomicU64::new(0);
static TOTAL_USER_TIME_NS: AtomicU64 = AtomicU64::new(0);
static TOTAL_SYSTEM_TIME_NS: AtomicU64 = AtomicU64::new(0);
static TOTAL_IDLE_TIME_NS: AtomicU64 = AtomicU64::new(0);
static TOTAL_IOWAIT_TIME_NS: AtomicU64 = AtomicU64::new(0);
static TOTAL_RUNNING_TIME_NS: AtomicU64 = AtomicU64::new(0);

// Wygodne funkcje globalne (dla kompatybilności i szybkiego użycia)

pub fn account_switch() {
    TOTAL_SWITCHES.fetch_add(1, Ordering::Relaxed);
}

pub fn account_voluntary_switch(task: &mut TaskStruct) {
    TOTAL_VOLUNTARY_SWITCHES.fetch_add(1, Ordering::Relaxed);
    task.stats.nr_voluntary_switches = task.stats.nr_voluntary_switches.saturating_add(1);
}

pub fn account_involuntary_switch(task: &mut TaskStruct) {
    TOTAL_INVOLUNTARY_SWITCHES.fetch_add(1, Ordering::Relaxed);
    task.stats.nr_involuntary_switches = task.stats.nr_involuntary_switches.saturating_add(1);
}

pub fn account_migration(task: &mut TaskStruct) {
    TOTAL_MIGRATIONS.fetch_add(1, Ordering::Relaxed);
    task.stats.nr_migrations = task.stats.nr_migrations.saturating_add(1);
    task.se.nr_migrations = task.se.nr_migrations.saturating_add(1);
}

pub fn account_user_time(task: &mut TaskStruct, delta_ns: u64) {
    TOTAL_USER_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
    TOTAL_RUNNING_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
    task.stats.utime = task.stats.utime.saturating_add(delta_ns);
    task.se.sum_exec_runtime = task.se.sum_exec_runtime.saturating_add(delta_ns);
}

pub fn account_system_time(task: &mut TaskStruct, delta_ns: u64) {
    TOTAL_SYSTEM_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
    TOTAL_RUNNING_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
    task.stats.stime = task.stats.stime.saturating_add(delta_ns);
    task.se.sum_exec_runtime = task.se.sum_exec_runtime.saturating_add(delta_ns);
}

pub fn account_idle_time(delta_ns: u64) {
    TOTAL_IDLE_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
}

pub fn account_iowait_time(delta_ns: u64) {
    TOTAL_IOWAIT_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
}

// Gettery globalne
pub fn total_switches() -> u64 { TOTAL_SWITCHES.load(Ordering::Relaxed) }
pub fn total_voluntary_switches() -> u64 { TOTAL_VOLUNTARY_SWITCHES.load(Ordering::Relaxed) }
pub fn total_involuntary_switches() -> u64 { TOTAL_INVOLUNTARY_SWITCHES.load(Ordering::Relaxed) }
pub fn total_migrations() -> u64 { TOTAL_MIGRATIONS.load(Ordering::Relaxed) }
pub fn total_user_time_ns() -> u64 { TOTAL_USER_TIME_NS.load(Ordering::Relaxed) }
pub fn total_system_time_ns() -> u64 { TOTAL_SYSTEM_TIME_NS.load(Ordering::Relaxed) }
pub fn total_idle_time_ns() -> u64 { TOTAL_IDLE_TIME_NS.load(Ordering::Relaxed) }
pub fn total_iowait_time_ns() -> u64 { TOTAL_IOWAIT_TIME_NS.load(Ordering::Relaxed) }
pub fn total_running_time_ns() -> u64 { TOTAL_RUNNING_TIME_NS.load(Ordering::Relaxed) }

// ---------------------------------------------------------------------
// Per-CPU Statystyki (do umieszczenia w RunQueue)
// ---------------------------------------------------------------------

/// Struktura przechowująca statystyki per procesor.
/// Powinna być umieszczona w `RunQueue` i aktualizowana przez scheduler.
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct RqStats {
    /// Liczba przełączeń kontekstu na tym CPU.
    pub nr_switches: u64,
    /// Liczba dobrowolnych przełączeń.
    pub nr_voluntary_switches: u64,
    /// Liczba przymusowych przełączeń.
    pub nr_involuntary_switches: u64,
    /// Liczba migracji zadań z/do tego CPU.
    pub nr_migrations: u64,
    /// Czas spędzony w trybie użytkownika (ns).
    pub user_time_ns: u64,
    /// Czas spędzony w trybie jądra (ns).
    pub system_time_ns: u64,
    /// Czas bezczynności CPU (ns).
    pub idle_time_ns: u64,
    /// Czas oczekiwania na I/O (ns).
    pub iowait_time_ns: u64,
    /// Czas wykonywania zadań ogółem (user+system+...).
    pub running_time_ns: u64,
    /// Sumaryczny `vruntime` wszystkich zadań Fair (do load tracking).
    pub sum_vruntime: u64,
    /// Suma wag wszystkich zadań Fair.
    pub sum_weight: u64,
    /// Liczba zadań w kolejce (wszystkie klasy).
    pub nr_running: u32,
    /// Liczba zadań nieprzerywalnych.
    pub nr_uninterruptible: u32,
}

impl RqStats {
    pub const fn new() -> Self {
        Self {
            nr_switches: 0,
            nr_voluntary_switches: 0,
            nr_involuntary_switches: 0,
            nr_migrations: 0,
            user_time_ns: 0,
            system_time_ns: 0,
            idle_time_ns: 0,
            iowait_time_ns: 0,
            running_time_ns: 0,
            sum_vruntime: 0,
            sum_weight: 0,
            nr_running: 0,
            nr_uninterruptible: 0,
        }
    }

    #[inline]
    pub fn record_switch(&mut self, voluntary: bool) {
        self.nr_switches += 1;
        if voluntary {
            self.nr_voluntary_switches += 1;
        } else {
            self.nr_involuntary_switches += 1;
        }
        // Aktualizuj globalne liczniki
        TOTAL_SWITCHES.fetch_add(1, Ordering::Relaxed);
        if voluntary {
            TOTAL_VOLUNTARY_SWITCHES.fetch_add(1, Ordering::Relaxed);
        } else {
            TOTAL_INVOLUNTARY_SWITCHES.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn record_migration(&mut self) {
        self.nr_migrations += 1;
        TOTAL_MIGRATIONS.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn add_user_time(&mut self, delta_ns: u64) {
        self.user_time_ns += delta_ns;
        self.running_time_ns += delta_ns;
        TOTAL_USER_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
        TOTAL_RUNNING_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
    }

    #[inline]
    pub fn add_system_time(&mut self, delta_ns: u64) {
        self.system_time_ns += delta_ns;
        self.running_time_ns += delta_ns;
        TOTAL_SYSTEM_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
        TOTAL_RUNNING_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
    }

    #[inline]
    pub fn add_idle_time(&mut self, delta_ns: u64) {
        self.idle_time_ns += delta_ns;
        TOTAL_IDLE_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
    }

    #[inline]
    pub fn add_iowait_time(&mut self, delta_ns: u64) {
        self.iowait_time_ns += delta_ns;
        TOTAL_IOWAIT_TIME_NS.fetch_add(delta_ns, Ordering::Relaxed);
    }

    #[inline]
    pub fn set_nr_running(&mut self, nr: u32) {
        self.nr_running = nr;
    }

    #[inline]
    pub fn set_nr_uninterruptible(&mut self, nr: u32) {
        self.nr_uninterruptible = nr;
    }

    #[inline]
    pub fn update_load_stats(&mut self, sum_vruntime: u64, sum_weight: u64) {
        self.sum_vruntime = sum_vruntime;
        self.sum_weight = sum_weight;
    }

    /// Oblicza przybliżone obciążenie CPU (0-100).
    pub fn load_percent(&self) -> u32 {
        if self.running_time_ns == 0 {
            return 0;
        }
        let total = self.running_time_ns + self.idle_time_ns;
        if total == 0 {
            return 0;
        }
        ((self.running_time_ns * 100) / total) as u32
    }

    /// Zwraca 1-minutowy load average na podstawie ostatnich próbek (uproszczone).
    /// W rzeczywistości wymagałoby to próbkowania; tutaj podajemy natychmiastowe obciążenie.
    pub fn load_avg_1min(&self) -> f32 {
        self.load_percent() as f32 / 100.0
    }
}

// ---------------------------------------------------------------------
// Histogramy – do analizy rozkładów czasów (opcjonalnie)
// ---------------------------------------------------------------------

/// Prosty histogram czasów wykonania (w ns) w skali logarytmicznej.
#[derive(Debug, Clone, Copy)]
pub struct ExecTimeHistogram {
    // 16 przedziałów: 0-1us, 1-2us, 2-4us, 4-8us, ...  >8ms
    buckets: [u64; 16],
    total_samples: u64,
    total_time_ns: u64,
}

impl ExecTimeHistogram {
    pub const fn new() -> Self {
        Self {
            buckets: [0; 16],
            total_samples: 0,
            total_time_ns: 0,
        }
    }

    pub fn add_sample(&mut self, exec_ns: u64) {
        self.total_samples += 1;
        self.total_time_ns += exec_ns;

        // Wyznacz indeks na podstawie log2, z dolnym progiem 1024 ns
        let mut idx = 0;
        let mut threshold = 1024u64; // 1us
        let mut upper = 2048u64;
        for i in 0..15 {
            if exec_ns < upper {
                idx = i;
                break;
            }
            threshold = upper;
            upper *= 2;
        }
        if exec_ns >= upper {
            idx = 15;
        }
        self.buckets[idx] += 1;
    }

    pub fn average_ns(&self) -> f32 {
        if self.total_samples == 0 {
            0.0
        } else {
            self.total_time_ns as f32 / self.total_samples as f32
        }
    }

    pub fn total_samples(&self) -> u64 {
        self.total_samples
    }
}

impl Default for ExecTimeHistogram {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------
// Kolekcja statystyk globalnych i per-CPU
// ---------------------------------------------------------------------

/// Zbiorcza struktura do raportowania wszystkich statystyk systemu.
#[derive(Debug, Default, Clone)]
pub struct SystemStats {
    pub total_switches: u64,
    pub total_voluntary_switches: u64,
    pub total_involuntary_switches: u64,
    pub total_migrations: u64,
    pub total_user_time_ns: u64,
    pub total_system_time_ns: u64,
    pub total_idle_time_ns: u64,
    pub total_iowait_time_ns: u64,
    pub total_running_time_ns: u64,
    pub per_cpu: Vec<CpuStatsSnapshot>,
}

#[derive(Debug, Default, Clone)]
pub struct CpuStatsSnapshot {
    pub cpu: u32,
    pub nr_switches: u64,
    pub user_time_ns: u64,
    pub system_time_ns: u64,
    pub idle_time_ns: u64,
    pub iowait_time_ns: u64,
    pub running_time_ns: u64,
    pub load_percent: u32,
}

/// Funkcja zbierająca globalne statystyki (do wywołania przy raportowaniu).
/// Wymaga dostępu do tablicy per-CPU runqueues, więc tutaj uproszczona.
pub fn collect_system_stats() -> SystemStats {
    SystemStats {
        total_switches: TOTAL_SWITCHES.load(Ordering::Relaxed),
        total_voluntary_switches: TOTAL_VOLUNTARY_SWITCHES.load(Ordering::Relaxed),
        total_involuntary_switches: TOTAL_INVOLUNTARY_SWITCHES.load(Ordering::Relaxed),
        total_migrations: TOTAL_MIGRATIONS.load(Ordering::Relaxed),
        total_user_time_ns: TOTAL_USER_TIME_NS.load(Ordering::Relaxed),
        total_system_time_ns: TOTAL_SYSTEM_TIME_NS.load(Ordering::Relaxed),
        total_idle_time_ns: TOTAL_IDLE_TIME_NS.load(Ordering::Relaxed),
        total_iowait_time_ns: TOTAL_IOWAIT_TIME_NS.load(Ordering::Relaxed),
        total_running_time_ns: TOTAL_RUNNING_TIME_NS.load(Ordering::Relaxed),
        per_cpu: Vec::new(), // W praktyce wypełnić z tablicy CPU
    }
}

// ---------------------------------------------------------------------
// Implementacja Display dla łatwego debugowania
// ---------------------------------------------------------------------

impl fmt::Display for RqStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CPU stats:")?;
        writeln!(f, "  switches: {} (voluntary {}, involuntary {})",
            self.nr_switches, self.nr_voluntary_switches, self.nr_involuntary_switches)?;
        writeln!(f, "  migrations: {}", self.nr_migrations)?;
        writeln!(f, "  user time: {} ns", self.user_time_ns)?;
        writeln!(f, "  system time: {} ns", self.system_time_ns)?;
        writeln!(f, "  idle time: {} ns", self.idle_time_ns)?;
        writeln!(f, "  iowait time: {} ns", self.iowait_time_ns)?;
        writeln!(f, "  running time: {} ns", self.running_time_ns)?;
        writeln!(f, "  load: {}%", self.load_percent())?;
        writeln!(f, "  nr_running: {}, nr_uninterruptible: {}",
            self.nr_running, self.nr_uninterruptible)
    }
}

impl fmt::Display for SystemStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "System-wide scheduler stats:")?;
        writeln!(f, "  total switches: {}", self.total_switches)?;
        writeln!(f, "  voluntary: {}", self.total_voluntary_switches)?;
        writeln!(f, "  involuntary: {}", self.total_involuntary_switches)?;
        writeln!(f, "  migrations: {}", self.total_migrations)?;
        writeln!(f, "  user time: {} ns", self.total_user_time_ns)?;
        writeln!(f, "  system time: {} ns", self.total_system_time_ns)?;
        writeln!(f, "  idle time: {} ns", self.total_idle_time_ns)?;
        writeln!(f, "  iowait time: {} ns", self.total_iowait_time_ns)?;
        writeln!(f, "  running time: {} ns", self.total_running_time_ns)?;
        if !self.per_cpu.is_empty() {
            writeln!(f, "Per-CPU:")?;
            for cpu in &self.per_cpu {
                writeln!(f, "  CPU {}: switches={}, load={}%",
                    cpu.cpu, cpu.nr_switches, cpu.load_percent)?;
            }
        }
        Ok(())
    }
}