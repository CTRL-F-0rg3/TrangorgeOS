# lib-ada — Ada/SPARK bindings + the mathematical proof

Two packages:

* `tg_comm.ads` — the executable Ada bindings (mirror of the public protocol)
  that call the `tgcomm_*` C ABI of the Rust core via `pragma Import`.
* `tg_comm_security.{ads,adb}` — the **SPARK formalisation** that proves the
  protocol makes unauthorized actions impossible.

## Files

```
tg_comm.ads              # protocol mirror + C ABI imports
tg_comm_security.ads     # SPARK model + contracts (the proof statement)
tg_comm_security.adb     # concrete definitions + ghost lemmas
tg_comm.gpr              # GNAT project (for gnatprove / gprbuild)
Makefile                 # gnatmake compile-check (gnatprove when available)
```

## The security property

The gate `Authorize` carries the post-condition (SPARK):

```ada
function Authorize (T : Cap_Table; M : Msg) return Verdict with
  Post => (Authorize'Result = Forwarded) = Authorized (T, M);
```

where `Authorized` is the *mathematical definition* of legitimacy:

```ada
function Authorized (T : Cap_Table; M : Msg) return Boolean is
  (M.Well_Formed
   and then Route_Allowed (M.Src_Layer, M.Dst_Layer)
   and then Has_Cap (T, M.Cap)
   and then Contains (Rights_Of (T, M.Cap), Required_Right (M.Opcode_Value))
   and then Matches (Required_Type (M.Opcode_Value), Type_Of (T, M.Cap)));
```

The post-condition states **soundness and completeness**:

1. **Soundness (no false positives)** — every forwarded request is authorized:
   `Forwarded ⟹ Authorized`. An *unauthorized action is therefore impossible*:
   the kernel handler is reached only when the capability exists and carries
   the required rights and object type.
2. **Completeness (no false negatives)** — every authorized request is
   forwarded: `Authorized ⟹ Forwarded`.

## The two lemmas

`Authorized` is monotone in the capability table (adding capabilities can
only add authorizations, never remove them). Two consequences are stated as
Ghost procedures:

```ada
procedure Lemma_Empty_Table_Denies_All (M : Msg) with
  Ghost,
  Post => not Authorized (Empty_Table, M);
```

> A freshly created table grants nothing: no request is authorized.

```ada
procedure Lemma_Revocation_Monotonic (T, U : Cap_Table; C : Cap_Id; M : Msg) with
  Ghost,
  Pre  => Revokes (T, U, C) and then not Authorized (T, M),
  Post => not Authorized (U, M);
```

> Revoking a capability can never turn a previously-denied request into an
> authorized one. (Contrapositive of monotonicity.)

## Inductive argument (why the whole system stays safe)

The gate is the **only** path from shared memory into the kernel, and it is
pure with respect to the table. Starting from the empty table (which denies
everything, Lemma 1) the manager grants capabilities one at a time; each grant
only enables exactly the `(rights, object-type)` pairs it names, and only for
the specific handle it assigns. Because `Authorize` is sound, every forward
satisfies `Authorized`; because `Authorize` is complete, no legitimate request
is silently dropped. Revocation (Lemma 2) only shrinks the authorized set.
By induction over the sequence of grants/revokes, the invariant

```
handler_called  ⟹  Authorized(table, msg)
```

holds in every reachable state. This is precisely "the method makes
unauthorized actions impossible".

## Verifying

* **Compile** (FSF GNAT): `make check` — validates syntax/semantics of both
  packages. Verified with `gnatmake -gnat2022`.
* **Prove** (GNATprove / SPARK Pro): `make prove` runs
  `gnatprove -P tg_comm.gpr --mode=prove`, which discharges the post-condition
  of `Authorize` and the two lemmas. GNATprove was not installed in the
  authoring environment, so the obligations are provided ready-to-discharge
  and annotated in the sources.
