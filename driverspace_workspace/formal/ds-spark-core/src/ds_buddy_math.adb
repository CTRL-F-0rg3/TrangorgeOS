-- formal/ds-spark-core/src/ds_buddy_math.adb
package body DS_Buddy_Math with
   SPARK_Mode
is

   function Power_Of_Two (Exponent : Block_Size) return Phys_Addr is
   begin
      return Phys_Addr (Shift_Left (1, Integer (Exponent)));
   end Power_Of_Two;

   function Is_Aligned (Addr : Phys_Addr; Size_Exp : Block_Size) return Boolean is
      Mask : constant Phys_Addr := Power_Of_Two (Size_Exp) - 1;
   begin
      -- Udowodnienie, że (Addr mod 2^N) == 0 jest równoważne (Addr and (2^N - 1)) == 0
      return (Addr and Mask) = 0;
   end Is_Aligned;

   function Get_Buddy_Address (
      Addr     : Phys_Addr; 
      Size_Exp : Block_Size
   ) return Phys_Addr is
      -- Kluczowy bit maski dla alokatora Buddy
      Bit_Mask : constant Phys_Addr := Power_Of_Two (Size_Exp);
   begin
      -- XOR z maską odwraca dokładnie ten jeden bit, który definiuje pozycję buddy.
      -- SPARK udowodni, że ponieważ Addr jest wyrównane (Pre condition z .ads),
      -- to wynik również będzie wyrównany, a XOR nie spowoduje przepełnienia.
      return Addr xor Bit_Mask;
   end Get_Buddy_Address;

   function Get_Left_Child_Address (
      Parent_Addr : Phys_Addr; 
      Parent_Exp  : Block_Size
   ) return Phys_Addr is
   begin
      -- Lewe dziecko zawsze zaczyna się tam, gdzie rodzic
      return Parent_Addr;
   end Get_Left_Child_Address;

   function Get_Right_Child_Address (
      Parent_Addr : Phys_Addr; 
      Parent_Exp  : Block_Size
   ) return Phys_Addr is
      -- Prawe dziecko to rodzic + połowa rozmiaru rodzica
      Half_Size : constant Phys_Addr := Power_Of_Two (Parent_Exp - 1);
   begin
      return Parent_Addr + Half_Size;
   end Get_Right_Child_Address;

   function Safe_Calculate_DMA_End (
      Base_Addr : Phys_Addr; 
      Buffer_Len : Phys_Addr
   ) return Phys_Addr is
   begin
      -- Ponieważ Pre condition w .ads gwarantuje, że Base + Len <= Last,
      -- kompilator SPARK wie, że to dodawanie nigdy nie "owinie" (wrap-around) pamięci.
      return Base_Addr + Buffer_Len;
   end Safe_Calculate_DMA_End;

end DS_Buddy_Math;