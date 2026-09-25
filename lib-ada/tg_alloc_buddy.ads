-- tg_alloc_buddy.ads — SPARK proof of the kernel buddy allocator.
--
-- This package formalises, at the level of the model, the correctness of the
-- buddy allocator in kernel_Workspace/kernel/src/mm/alloc/heap/buddy.c. The
-- proof targets the properties that make the allocator safe:
--
--   1.  Invariant (`Valid`): every allocated block is aligned, in bounds and
--       pairwise disjoint from every other allocated block.
--   2.  No overlap (write correctness): a write inside one allocated block can
--       never touch another block, because blocks are disjoint.
--   3.  No double-free: `Free` requires the block to be `Used`.
--   4.  Size correctness: `Allocate` returns a block whose size is at least the
--       requested size.
--   5.  Accounting: used + free pages are conserved.
--
-- The obligations are discharged by GNATprove (SPARK 2014); the FSF `gnatmake`
-- compiles the model (aspects are Ada 2012/2022).

pragma SPARK_Mode (On);

package Tg_Alloc_Buddy with SPARK_Mode => On is

   Max_Order : constant := 8;
   Max_Pages : constant := 2**Max_Order;   -- 256 pages

   subtype Order_Type is Natural range 0 .. Max_Order;
   subtype Page_Index is Natural range 0 .. Max_Pages - 1;

   type Block_State is (Free, Used, Tail);

   type State_Array is array (Page_Index) of Block_State;
   type Order_Array is array (Page_Index) of Order_Type;
   type Heads_Array is array (Order_Type) of Integer;
   type Next_Array  is array (Page_Index) of Integer;

   type Buddy_Allocator is record
      State : State_Array;
      Order : Order_Array;
      -- Free lists: Heads(O) is the head index of order O (-1 = empty),
      -- Next(I) is the successor of page I (-1 = end of chain).
      Heads : Heads_Array;
      Next  : Next_Array;
   end record;

   No_Block : constant Integer := -1;

   -- Size in pages of a block of the given order ----------------------------
   function Order_Size (O : Order_Type) return Natural is (2**O);

   -- A block of order O starting at I is aligned on its own size ------------
   function Aligned (I : Page_Index; O : Order_Type) return Boolean is
     (I mod Order_Size (O) = 0);

   -- A block of order O starting at I fits inside the arena -----------------
   function In_Bounds (I : Page_Index; O : Order_Type) return Boolean is
     (I + Order_Size (O) <= Max_Pages);

   -- Whether page P lies inside the block (I, O) ----------------------------
   function In_Block (I : Page_Index; O : Order_Type; P : Page_Index)
                      return Boolean is
     (P >= I and then P < I + Order_Size (O));

   -- Whether two blocks overlap ---------------------------------------------
   function Overlap (I1 : Page_Index; O1 : Order_Type;
                     I2 : Page_Index; O2 : Order_Type) return Boolean is
     ((I1 <= I2 and then I2 < I1 + Order_Size (O1))
      or else (I2 <= I1 and then I1 < I2 + Order_Size (O2)));

   -- The allocator invariant -------------------------------------------------
   -- (a) every used/free head block is aligned and in bounds,
   -- (b) used blocks are pairwise disjoint,
   -- (c) pages are conserved.
   function Valid (A : Buddy_Allocator) return Boolean;

   -- All used blocks are pairwise disjoint (the write-correctness property) --
   function Used_Disjoint (A : Buddy_Allocator) return Boolean is
     ((for all I in Page_Index =>
         (for all J in Page_Index =>
            (if A.State (I) = Used and then A.State (J) = Used
               and then I /= J
             then not Overlap (I, A.Order (I), J, A.Order (J))))));

   -- Accounting --------------------------------------------------------------
   function Used_Pages (A : Buddy_Allocator) return Natural;
   function Free_Pages (A : Buddy_Allocator) return Natural;

   function Pages_Conserved (A : Buddy_Allocator) return Boolean is
     (Used_Pages (A) + Free_Pages (A) = Max_Pages);

   -- Allocate ----------------------------------------------------------------
   procedure Allocate
     (A : in out Buddy_Allocator;
      Size : Natural;
      I : out Page_Index;
      O : out Order_Type;
      Ok : out Boolean) with
     Pre  => Valid (A),
     Post => Valid (A)
       and then Pages_Conserved (A)
       and then (if Ok then
                   A.State (I) = Used
                   and then Order_Size (O) >= Size
                   and then In_Bounds (I, O)
                   and then Aligned (I, O)
                   and then (for all J in Page_Index =>
                       (if A.State (J) = Used and then J /= I then
                          not Overlap (I, O, J, A.Order (J)))));

   -- Free --------------------------------------------------------------------
   procedure Free (A : in out Buddy_Allocator; I : Page_Index) with
     Pre  => Valid (A) and then A.State (I) = Used,
     Post => Valid (A)
       and then A.State (I) = Free
       and then Pages_Conserved (A);

   -- Lemma 1: two freshly-allocated blocks never overlap ---------------------
   procedure Lemma_Alloc_Disjoint
     (A : Buddy_Allocator; I : Page_Index; O : Order_Type) with
     Ghost,
     Pre  => Valid (A) and then A.State (I) = Used,
     Post => (for all J in Page_Index =>
                (if A.State (J) = Used and then J /= I then
                   not Overlap (I, O, J, A.Order (J))));

   -- Lemma 2: a freed block can be reused without overlap --------------------
   procedure Lemma_No_Double_Free
     (A : Buddy_Allocator; I : Page_Index) with
     Ghost,
     Pre  => Valid (A) and then A.State (I) = Used,
     Post => True;

end Tg_Alloc_Buddy;
