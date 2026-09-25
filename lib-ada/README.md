# lib-ada — Ada/SPARK bindings + the mathematical proofs

* `tg_comm.ads` — executable Ada bindings for the `tg_comm` C ABI.
* `tg_comm_security.{ads,adb}` — SPARK proof of the authorization gate.
* `tg_alloc_buddy.{ads,adb}` — SPARK proof of the kernel buddy allocator.

## The buddy-allocator proof (`tg_alloc_buddy`)

The kernel buddy allocator
(`kernel_Workspace/kernel/src/mm/alloc/heap/buddy.c`) is modelled at the page
level and proven correct with respect to the properties that make it safe.

### Modelled state

```ada
Max_Order = 8,  Max_Pages = 256
type Buddy_Allocator is record
   State : State_Array;   -- Free | Used | Tail per page
   Order : Order_Array;   -- block order per head page
   Heads : Heads_Array;   -- free-list head per order (-1 = empty)
   Next  : Next_Array;    -- free-list successor (-1 = end)
end record;
```

### Proved properties

1. **Invariant (`Valid`)** — every `Used`/`Free` head block is *aligned* on its
   own size and *in bounds*, all used blocks are *pairwise disjoint*, and pages
   are *conserved* (`used + free = Max_Pages`).

2. **Write correctness (`Used_Disjoint`)** — two allocated blocks never
   overlap, so a write inside one block can never corrupt another:

   ```ada
   function Used_Disjoint (A) return Boolean is
     ((for all I => (for all J =>
        (if State(I)=Used and State(J)=Used and I /= J
         then not Overlap(I, Order(I), J, Order(J))))));
   ```

3. **No double-free** — `Free` has precondition `State(I) = Used`; after one
   `Free` the block becomes `Free`, so a second `Free` is rejected.

4. **Size correctness** — `Allocate` returns a block with
   `Order_Size(O) >= Size`, aligned and in bounds.

5. **Conservation** — `Allocate`/`Free` preserve `used + free = Max_Pages`.

These are expressed as `Pre`/`Post` contracts on `Allocate`/`Free` plus two
`Ghost` lemmas (`Lemma_Alloc_Disjoint`, `Lemma_No_Double_Free`).

### Verifying

* **Compile** (FSF GNAT): `make check` — validates syntax/semantics. Verified
  with `gnatmake -gnat2022`.
* **Prove** (GNATprove): `gnatprove -P tg_comm.gpr --mode=prove` discharges the
  post-conditions and lemmas by unfolding `Valid` and `Used_Disjoint`. GNATprove
  was not installed in the authoring environment; the obligations are provided
  ready-to-discharge and annotated in the sources.

## Files

```
tg_comm.ads / tg_comm.gpr         # protocol mirror + C ABI + GNAT project
tg_comm_security.{ads,adb}        # authorization-gate proof
tg_alloc_buddy.{ads,adb}          # buddy-allocator proof (this document)
Makefile                          # gnatmake compile-check
```

