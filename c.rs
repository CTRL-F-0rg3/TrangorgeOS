// per other programers pls do not delete it beacuse it example for bunker forge it not be us code too implamate 
// thanks my friend {: 

// fn a(data: &[u8], start_bit: usize, bit_count: usize) -> u64 {
//     let mut value = 0u64;

//     for i in 0..bit_count {
//         let bit_position = start_bit + i;
//         let byte_index = bit_position / 8;
//         let bit_index = bit_position % 8;
//         let bit = (data[byte_index] >> bit_index) & 1;
//         value |= (bit as u64) << i;
//     }
//     value
// }

// fn main() {
//     let data = [
//         0b1011_0101,

//.......sssś 
//         0b1100_001,
//         0b0110_1001,
    
//     ];

//     let value = a(&data, 3, 5);

//     println!("format: {}", value);

//     let okey = Some(value);

//     println!(" OK: {:?}", okey);
// }

/*
 * Przykładowy Testowy Alokator Pamięci
 * 
 * Ten alokator używa statycznego bufora jako "puli pamięci"
 * i implementuje prostą strategię first-fit do zarządzania blokami.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

/* Konfiguracja alokatora */
#define MEMORY_POOL_SIZE (1024 * 1024)  /* 1 MB puli pamięci */
#define MIN_BLOCK_SIZE 16               /* Minimalny rozmiar bloku */
#define ALIGNMENT 8                     /* Wyrównanie do 8 bajtów */

/* Struktura nagłówka bloku pamięci */
typedef struct BlockHeader {
    size_t size;                        /* Rozmiar danych użytkownika */
    int is_free;                        /* Czy blok jest wolny */
    struct BlockHeader* next;           /* Wskaźnik do następnego bloku */
    struct BlockHeader* prev;           /* Wskaźnik do poprzedniego bloku */
} BlockHeader;

/* Globalna pula pamięci i lista bloków */
static uint8_t memory_pool[MEMORY_POOL_SIZE];
static BlockHeader* free_list = NULL;
static int allocator_initialized = 0;

/* Makro do wyrównywania rozmiaru */
#define ALIGN(size) (((size) + (ALIGNMENT - 1)) & ~(ALIGNMENT - 1))

/* Inicjalizacja alokatora */
void mem_init(void) {
    if (allocator_initialized) {
        return;
    }
    
    /* Utwórz pierwszy wolny blok obejmujący całą pulę */
    BlockHeader* initial_block = (BlockHeader*)memory_pool;
    initial_block->size = MEMORY_POOL_SIZE - sizeof(BlockHeader);
    initial_block->is_free = 1;
    initial_block->next = NULL;
    initial_block->prev = NULL;
    
    free_list = initial_block;
    allocator_initialized = 1;
    
    printf("[INIT] Alokator zainicjalizowany. Pula: %d bytes\n", MEMORY_POOL_SIZE);
}

/* Znajdź wolny blok pasujący do żądanego rozmiaru (first-fit) */
static BlockHeader* find_free_block(size_t size) {
    BlockHeader* current = free_list;
    
    while (current != NULL) {
        if (current->is_free && current->size >= size) {
            return current;
        }
        current = current->next;
    }
    
    return NULL;
}

/* Podziel blok jeśli jest za duży */
static void split_block(BlockHeader* block, size_t size) {
    size_t remaining = block->size - size;
    
    /* Sprawdź czy pozostała część jest wystarczająco duża */
    if (remaining >= sizeof(BlockHeader) + MIN_BLOCK_SIZE) {
        /* Utwórz nowy blok z pozostałej części */
        BlockHeader* new_block = (BlockHeader*)((uint8_t*)block + sizeof(BlockHeader) + size);
        new_block->size = remaining - sizeof(BlockHeader);
        new_block->is_free = 1;
        new_block->next = block->next;
        new_block->prev = block;
        
        /* Zaktualizuj wskaźniki */
        if (block->next != NULL) {
            block->next->prev = new_block;
        }
        block->next = new_block;
        block->size = size;
    }
}

/* Połącz sąsiadujące wolne bloki */
static void merge_blocks(BlockHeader* block) {
    /* Spróbuj połączyć z następnym blokiem */
    if (block->next != NULL && block->next->is_free) {
        block->size += sizeof(BlockHeader) + block->next->size;
        block->next = block->next->next;
        if (block->next != NULL) {
            block->next->prev = block;
        }
    }
    
    /* Spróbuj połączyć z poprzednim blokiem */
    if (block->prev != NULL && block->prev->is_free) {
        block->prev->size += sizeof(BlockHeader) + block->size;
        block->prev->next = block->next;
        if (block->next != NULL) {
            block->next->prev = block->prev;
        }
    }
}

/* Dynamiczna alokacja pamięci */
void* mem_alloc(size_t size) {
    if (!allocator_initialized) {
        mem_init();
    }
    
    if (size == 0) {
        return NULL;
    }
    
    /* Wyrównaj rozmiar */
    size_t aligned_size = ALIGN(size);
    if (aligned_size < MIN_BLOCK_SIZE) {
        aligned_size = MIN_BLOCK_SIZE;
    }
    
    /* Znajdź wolny blok */
    BlockHeader* block = find_free_block(aligned_size);
    if (block == NULL) {
        printf("[ERROR] Brak wystarczającej ilości pamięci!\n");
        return NULL;
    }
    
    /* Podziel blok jeśli potrzeba */
    split_block(block, aligned_size);
    
    /* Oznacz blok jako zajęty */
    block->is_free = 0;
    
    printf("[ALLOC] Przydzielono %zu bytes na adresie %p\n", size, (void*)((uint8_t*)block + sizeof(BlockHeader)));
    
    /* Zwróć wskaźnik do danych (za nagłówkiem) */
    return (void*)((uint8_t*)block + sizeof(BlockHeader));
}

/* Zwolnienie pamięci */
void mem_free(void* ptr) {
    if (ptr == NULL) {
        return;
    }
    
    /* Odzyskaj nagłówek bloku */
    BlockHeader* block = (BlockHeader*)((uint8_t*)ptr - sizeof(BlockHeader));
    
    if (block->is_free) {
        printf("[WARNING] Próba zwolnienia już wolnego bloku!\n");
        return;
    }
    
    printf("[FREE] Zwolniono %zu bytes z adresu %p\n", block->size, ptr);
    
    /* Oznacz blok jako wolny */
    block->is_free = 1;
    
    /* Połącz sąsiadujące wolne bloki */
    merge_blocks(block);
}

/* Realokacja pamięci */
void* mem_realloc(void* ptr, size_t new_size) {
    if (ptr == NULL) {
        return mem_alloc(new_size);
    }
    
    if (new_size == 0) {
        mem_free(ptr);
        return NULL;
    }
    
    BlockHeader* block = (BlockHeader*)((uint8_t*)ptr - sizeof(BlockHeader));
    
    /* Jeśli obecny blok jest wystarczająco duży */
    if (block->size >= ALIGN(new_size)) {
        return ptr;
    }
    
    /* Alokuj nowy blok i skopiuj dane */
    void* new_ptr = mem_alloc(new_size);
    if (new_ptr != NULL) {
        memcpy(new_ptr, ptr, block->size);
        mem_free(ptr);
    }
    
    return new_ptr;
}

/* Wyświetl statystyki alokatora */
void mem_stats(void) {
    if (!allocator_initialized) {
        printf("[STATS] Alokator nie zainicjalizowany\n");
        return;
    }
    
    size_t total_free = 0;
    size_t total_used = 0;
    int free_blocks = 0;
    int used_blocks = 0;
    
    BlockHeader* current = (BlockHeader*)memory_pool;
    while (current != NULL) {
        if (current->is_free) {
            total_free += current->size;
            free_blocks++;
        } else {
            total_used += current->size;
            used_blocks++;
        }
        current = current->next;
    }
    
    printf("\n=== STATYSTYKI ALOKATORA ===\n");
    printf("Całkowita pula: %d bytes\n", MEMORY_POOL_SIZE);
    printf("Wolne: %zu bytes (%d bloków)\n", total_free, free_blocks);
    printf("Użyte: %zu bytes (%d bloków)\n", total_used, used_blocks);
    printf("Wykorzystanie: %.2f%%\n", (total_used * 100.0) / MEMORY_POOL_SIZE);
    printf("============================\n\n");
}

/* Funkcja testowa */
void run_tests(void) {
    printf("\n=== TESTY ALOKATORA ===\n\n");
    
    mem_init();
    mem_stats();
    
    /* Test 1: Prosta alokacja */
    printf("Test 1: Prosta alokacja\n");
    void* ptr1 = mem_alloc(100);
    void* ptr2 = mem_alloc(200);
    void* ptr3 = mem_alloc(50);
    mem_stats();
    
    /* Test 2: Zwolnienie pamięci */
    printf("Test 2: Zwolnienie pamięci\n");
    mem_free(ptr2);
    mem_stats();
    
    /* Test 3: Ponowna alokacja w zwolnionym miejscu */
    printf("Test 3: Ponowna alokacja\n");
    void* ptr4 = mem_alloc(150);
    mem_stats();
    
    /* Test 4: Realokacja */
    printf("Test 4: Realokacja\n");
    ptr1 = mem_realloc(ptr1, 300);
    mem_stats();
    
    /* Test 5: Czyszczenie wszystkiego */
    printf("Test 5: Czyszczenie\n");
    mem_free(ptr1);
    mem_free(ptr3);
    mem_free(ptr4);
    mem_stats();
    
    printf("=== KONIEC TESTÓW ===\n");
}

int main(void) {
    printf("===========================================\n");
    printf("  Testowy Systemowy Alokator Pamięci\n");
    printf("===========================================\n\n");
    
    /* Uruchom testy */
    run_tests();
    
    /* Przykład użycia interaktywnego */
    printf("\nPrzykład użycia:\n");
    mem_init();
    
    char* str = (char*)mem_alloc(50);
    strcpy(str, "Hello from custom allocator!");
    printf("Tekst: %s\n", str);
    mem_free(str);
    
    int* numbers = (int*)mem_alloc(10 * sizeof(int));
    for (int i = 0; i < 10; i++) {
        numbers[i] = i * 10;
    }
    
    printf("Tablica liczb: ");
    for (int i = 0; i < 10; i++) {
        printf("%d ", numbers[i]);
    }
    printf("\n");
    
    mem_free(numbers);
    mem_stats();
    
    return 0;
}
