-- formal/ds-spark-core/src/ds_buddy_math.ads
with Interfaces; use Interfaces;

package DS_Buddy_Math with
   SPARK_Mode,
   Pure
is
   -- Typy bazowe dla fizycznej pamięci (bare-metal)
   type Phys_Addr is new Unsigned_64;
   type Block_Size is new Unsigned_64;

   -- Minimalny i maksymalny rozmiar bloku (np. 4KB do 1GB)
   MIN_BLOCK_SIZE : constant Block_Size := 12; -- 2^12 = 4096 bytes (4KB)
   MAX_BLOCK_SIZE : constant Block_Size := 30; -- 2^30 = 1GB

   -- Funkcja pomocnicza: Obliczanie 2^N (Shift left)
   function Power_Of_Two (Exponent : Block_Size) return Phys_Addr with
      Pre  => Exponent <= 63,
      -- Use a bit-shift to produce a value of type Unsigned_64 / Phys_Addr
      Post => Power_Of_Two'Result = Shift_Left (Phys_Addr (1), Integer (Exponent));

   -- Sprawdzenie, czy adres jest poprawnie wyrównany do rozmiaru bloku
   function Is_Aligned (Addr : Phys_Addr; Size_Exp : Block_Size) return Boolean with
      Post => Is_Aligned'Result = ((Addr mod Power_Of_Two (Size_Exp)) = 0);

   -- =========================================================================
   -- ALGORYTM 1: OBLICZANIE ADRESU BUDDY (PAROWANIE)
   -- =========================================================================
   -- W alokatorze Buddy, "buddy" bloku to blok o tym samym rozmiarze, 
   -- który różni się tylko jednym bitem (na pozycji odpowiadającej rozmiarowi).
   
   function Get_Buddy_Address (
      Addr     : Phys_Addr; 
      Size_Exp : Block_Size
   ) return Phys_Addr with
      Pre  => Size_Exp >= MIN_BLOCK_SIZE and then Size_Exp <= MAX_BLOCK_SIZE,
      Post => Is_Aligned (Get_Buddy_Address'Result, Size_Exp),
      Post => Get_Buddy_Address'Result /= Addr, -- Buddy NIGDY nie jest tym samym adresem
               Post => Shift_Right (Get_Buddy_Address'Result, Integer (Size_Exp)) = 
                  (Shift_Right (Addr, Integer (Size_Exp)) xor Phys_Addr (1)); -- Udowodniona różnica dokładnie 1 bloku

   -- =========================================================================
   -- ALGORYTM 2: DZIELENIE BLOKU (SPLITTING)
   -- =========================================================================
   -- Dzielenie bloku o rozmiarze 2^N na dwa bloki o rozmiarze 2^(N-1).
   
   function Get_Left_Child_Address (
      Parent_Addr : Phys_Addr; 
      Parent_Exp  : Block_Size
   ) return Phys_Addr with
      Pre  => Parent_Exp > MIN_BLOCK_SIZE and then Is_Aligned (Parent_Addr, Parent_Exp),
      Post => Get_Left_Child_Address'Result = Parent_Addr,
      Post => Is_Aligned (Get_Left_Child_Address'Result, Parent_Exp - 1);

   function Get_Right_Child_Address (
      Parent_Addr : Phys_Addr; 
      Parent_Exp  : Block_Size
   ) return Phys_Addr with
      Pre  => Parent_Exp > MIN_BLOCK_SIZE and then Is_Aligned (Parent_Addr, Parent_Exp),
      Post => Get_Right_Child_Address'Result = Parent_Addr + Power_Of_Two (Parent_Exp - 1),
      Post => Is_Aligned (Get_Right_Child_Address'Result, Parent_Exp - 1),
      Post => Get_Right_Child_Address'Result > Parent_Addr; -- Prawe dziecko jest zawsze wyżej w pamięci

   -- =========================================================================
   -- ALGORYTM 3: BEZPIECZNE OBLICZANIE OFFSETU DMA
   -- =========================================================================
   -- Krytyczne dla sterowników: obliczanie adresu końcowego bufora bez przepełnienia (Overflow).
   
   function Safe_Calculate_DMA_End (
      Base_Addr : Phys_Addr; 
      Buffer_Len : Phys_Addr
   ) return Phys_Addr with
      Pre  => Base_Addr <= Phys_Addr'Last - Buffer_Len, -- Gwarancja braku przepełnienia UInt64
      Post => Safe_Calculate_DMA_End'Result = Base_Addr + Buffer_Len;

   -- Eksport do C/Rust (FFI)
   pragma Export (C, Get_Buddy_Address, "ds_spark_buddy_get_buddy");
   pragma Export (C, Get_Left_Child_Address, "ds_spark_buddy_left_child");
   pragma Export (C, Get_Right_Child_Address, "ds_spark_buddy_right_child");
   pragma Export (C, Safe_Calculate_DMA_End, "ds_spark_safe_dma_end");

end DS_Buddy_Math;