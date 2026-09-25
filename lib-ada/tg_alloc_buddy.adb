-- tg_alloc_buddy.adb — bodies for the SPARK buddy-allocator model.
--
-- The contracts live in the specification; this body gives the concrete
-- (reference) definitions. `Allocate`/`Free` mirror the logic of
-- kernel_Workspace/kernel/src/mm/alloc/heap/buddy.c (split on allocate, merge
-- on free). The two `Lemma_*` procedures are `Ghost`; their empty bodies are
-- discharged by GNATprove by unfolding `Valid` and `Used_Disjoint`.

pragma SPARK_Mode (On);

package body Tg_Alloc_Buddy with SPARK_Mode => On is

   -- A modular word for buddy-index bit arithmetic (xor) --------------------
   type Page_Word is mod 2**16;

   function Valid (A : Buddy_Allocator) return Boolean is
   begin
      return
        (for all I in Page_Index =>
           (if A.State (I) = Used or else A.State (I) = Free then
              Aligned (I, A.Order (I)) and then In_Bounds (I, A.Order (I))))
        and then Used_Disjoint (A)
        and then Pages_Conserved (A);
   end Valid;

   function Used_Pages (A : Buddy_Allocator) return Natural is
      Total : Natural := 0;
   begin
      for I in Page_Index loop
         if A.State (I) = Used then
            Total := Total + Order_Size (A.Order (I));
         end if;
      end loop;
      return Total;
   end Used_Pages;

   function Free_Pages (A : Buddy_Allocator) return Natural is
      Total : Natural := 0;
   begin
      for I in Page_Index loop
         if A.State (I) = Free then
            Total := Total + Order_Size (A.Order (I));
         end if;
      end loop;
      return Total;
   end Free_Pages;

   procedure Allocate
     (A : in out Buddy_Allocator;
      Size : Natural;
      I : out Page_Index;
      O : out Order_Type;
      Ok : out Boolean)
   is
      Need : Order_Type := 0;
      Cur  : Order_Type;
      Idx  : Page_Index := 0;
   begin
      I := 0;
      O := 0;
      Ok := False;

      if Size = 0 then
         return;
      end if;

      -- Smallest order whose size covers the request.
      while Need < Max_Order and then Order_Size (Need) < Size loop
         Need := Need + 1;
      end loop;

      -- Smallest order >= Need that has a free block.
      Cur := Need;
      while Cur < Max_Order and then A.Heads (Cur) = No_Block loop
         Cur := Cur + 1;
      end loop;

      if A.Heads (Cur) = No_Block then
         return;  -- out of memory
      end if;

      Idx := Page_Index (A.Heads (Cur));
      A.Heads (Cur) := A.Next (Idx);

      -- Split the block down to the needed order, pushing the upper buddy
      -- halves back onto the free lists.
      while Cur > Need loop
         Cur := Cur - 1;
         declare
            Buddy : constant Page_Index := Idx + Order_Size (Cur);
         begin
            A.State (Buddy) := Free;
            A.Order (Buddy) := Cur;
            A.Next (Buddy) := A.Heads (Cur);
            A.Heads (Cur) := Integer (Buddy);
         end;
      end loop;

      A.State (Idx) := Used;
      A.Order (Idx) := Need;

      I := Idx;
      O := Need;
      Ok := True;
   end Allocate;

   procedure Free (A : in out Buddy_Allocator; I : Page_Index) is
      Cur : Order_Type := A.Order (I);
      Idx : Page_Index := I;
   begin
      A.State (Idx) := Free;

      -- Merge with the buddy while both halves are free at the same order.
      while Cur < Max_Order loop
         declare
            Buddy : constant Page_Index :=
              Page_Index (Page_Word (Idx) xor Page_Word (Order_Size (Cur)));
         begin
            if A.State (Buddy) /= Free or else A.Order (Buddy) /= Cur then
               exit;
            end if;

            A.State (Buddy) := Tail;
            if Buddy < Idx then
               Idx := Buddy;
            end if;

            Cur := Cur + 1;
            A.Order (Idx) := Cur;
         end;
      end loop;

      -- Push the (possibly merged) block back onto the free list.
      A.Next (Idx) := A.Heads (Cur);
      A.Heads (Cur) := Integer (Idx);
      A.Order (Idx) := Cur;
      A.State (Idx) := Free;
   end Free;

   procedure Lemma_Alloc_Disjoint
     (A : Buddy_Allocator; I : Page_Index; O : Order_Type) is
   begin
      -- Valid(A) implies Used_Disjoint(A); since State(I) = Used, block
      -- (I, O) is disjoint from every other used block. GNATprove discharges
      -- this by unfolding `Valid` and `Used_Disjoint`.
      null;
   end Lemma_Alloc_Disjoint;

   procedure Lemma_No_Double_Free
     (A : Buddy_Allocator; I : Page_Index) is
   begin
      -- The precondition State(I) = Used guarantees that a second Free on the
      -- same block is rejected (the contract of Free requires Used; after one
      -- Free the state is Free, so the precondition no longer holds).
      null;
   end Lemma_No_Double_Free;

end Tg_Alloc_Buddy;
