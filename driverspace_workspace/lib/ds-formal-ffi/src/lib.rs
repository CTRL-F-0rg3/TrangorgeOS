#![no_std]

// Deklaracje funkcji wyeksportowanych z Ada SPARK
// (edycja 2024 wymaga `unsafe extern` dla bloków z deklaracjami FFI)
unsafe extern "C" {
    // Odpowiednik: function Safe_Add_32 (A, B : UInt32) return UInt32
    pub fn ds_spark_safe_add_32(a: u32, b: u32) -> u32;

    // Odpowiednik: function Safe_Array_Index ...
    pub fn ds_spark_safe_array_index(index: u32, max_size: u32) -> u32;
}

// Bezpieczne wrappery dla reszty kodu Rust w workspace
pub mod math {
    use super::*;

    /// Bezpieczne dodawanie. W trybie debug Rust i tak to sprawdzi, 
    /// ale w release mode (gdzie overflow to undefined behavior w C/Ada),
    /// ta funkcja ma matematyczny dowód z SPARK, że nigdy nie overflownie.
    #[inline(always)]
    pub fn safe_add(a: u32, b: u32) -> u32 {
        // W produkcji wołamy udowodniony kod z Ady
        unsafe { ds_spark_safe_add_32(a, b) }
    }
}

   unsafe extern "C" {
       pub fn ds_spark_buddy_get_buddy(addr: u64, size_exp: u64) -> u64;
       pub fn ds_spark_safe_dma_end(base: u64, len: u64) -> u64;
   }
   
   pub mod pmm {
       use super::*;
       #[inline(always)]
       pub fn get_buddy(addr: u64, exp: u64) -> u64 {
           // Wywołanie kodu, który ma MATEMATYCZNY DOWÓD poprawności
           unsafe { ds_spark_buddy_get_buddy(addr, exp) }
       }
   }