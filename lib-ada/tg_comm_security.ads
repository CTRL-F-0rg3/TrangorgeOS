-- tg_comm_security.ads — SPARK formalisation of the authorization gate.
--
-- This package proves, at the level of the model, the central security
-- property of the shared-memory protocol:
--
--   A request is forwarded to the kernel IF AND ONLY IF it is authorized,
--   i.e. the capability it names exists and carries the rights and the object
--   type that the requested operation requires.
--
-- Consequently an *unauthorized* action is impossible: `Authorize` can never
-- return `Forwarded` for a request that is not `Authorized`. This is expressed
-- as the post-condition of `Authorize` and reinforced by two lemmas:
--
--   * the empty table denies every well-formed request;
--   * revoking a capability never turns a denied request into an authorized
--     one (monotonicity under revocation).
--
-- The obligations above are discharged by GNATprove (SPARK 2014). The FSF
-- `gnatmake` compiles the model (the aspects are part of Ada 2012/2022).

pragma SPARK_Mode (On);

package Tg_Comm_Security with SPARK_Mode => On is

   -- Rights ----------------------------------------------------------------
   type Rights_Mask is mod 2**32;
   Right_Read     : constant Rights_Mask := 2**0;
   Right_Write    : constant Rights_Mask := 2**1;
   Right_Exec     : constant Rights_Mask := 2**2;
   Right_Map      : constant Rights_Mask := 2**3;
   Right_Grant    : constant Rights_Mask := 2**4;
   Right_Transfer : constant Rights_Mask := 2**5;
   Right_Send     : constant Rights_Mask := 2**6;
   Right_Recv     : constant Rights_Mask := 2**7;
   Right_Call     : constant Rights_Mask := 2**8;
   Right_Manage   : constant Rights_Mask := 2**9;

   function Contains (Set, Subset : Rights_Mask) return Boolean is
     ((Set and Subset) = Subset);

   -- Layers and routing ----------------------------------------------------
   type Layer is (Kernel, Driverspace, Userspace, Manager, User_Driver_Space);

   function Route_Allowed (From, To : Layer) return Boolean;

   -- Object types and the required-type relation ---------------------------
   type Object_Type is (Memory, Endpoint, Notification, Channel,
                        Shmem_Region, Device, Irq);

   type Type_Req is (Any_Type, Memory, Endpoint, Notification, Channel,
                     Shmem_Region, Device, Irq);

   function Matches (Req : Type_Req; Actual : Object_Type) return Boolean is
     (case Req is
         when Any_Type     => True,
         when Memory       => Actual = Memory,
         when Endpoint     => Actual = Endpoint,
         when Notification => Actual = Notification,
         when Channel      => Actual = Channel,
         when Shmem_Region => Actual = Shmem_Region,
         when Device       => Actual = Device,
         when Irq          => Actual = Irq);

   -- Opcodes ---------------------------------------------------------------
   type Op_Class is (Sys, Mem, Cap, Ipc, Shmem, Video, Audio, Input,
                     Block, Net, Pci, Vgpu, Fs);

   type Opcode is record
      Class : Op_Class;
      Op    : Natural;
   end record;

   function Required_Right (O : Opcode) return Rights_Mask;
   function Required_Type (O : Opcode) return Type_Req;

   -- Message (model of the 64-byte wire message) ---------------------------
   type Msg is record
      Src_Layer    : Layer;
      Dst_Layer    : Layer;
      Opcode_Value : Opcode;
      Cap          : Natural;
      Well_Formed  : Boolean;   -- magic + version + decodable layer bytes
   end record;

   -- Capability table (bounded, concrete) ----------------------------------
   Max_Cap : constant := 511;
   subtype Cap_Id is Natural range 0 .. Max_Cap;

   type Cap_Entry is record
      Valid    : Boolean;
      Obj_Type : Object_Type;
      Rights   : Rights_Mask;
   end record;

   type Cap_Table is array (Cap_Id) of Cap_Entry;

   function Empty_Table return Cap_Table is
     ([others => (Valid => False, Obj_Type => Memory, Rights => 0)]);

   function Has_Cap (T : Cap_Table; C : Natural) return Boolean is
     (if C in Cap_Id then T (Cap_Id (C)).Valid else False);

   function Rights_Of (T : Cap_Table; C : Natural) return Rights_Mask with
     Pre => Has_Cap (T, C);

   function Type_Of (T : Cap_Table; C : Natural) return Object_Type with
     Pre => Has_Cap (T, C);

   -- The security property (Ghost: exists only for proof) ------------------
   function Authorized (T : Cap_Table; M : Msg) return Boolean is
     (M.Well_Formed
      and then Route_Allowed (M.Src_Layer, M.Dst_Layer)
      and then Has_Cap (T, M.Cap)
      and then Contains (Rights_Of (T, M.Cap), Required_Right (M.Opcode_Value))
      and then Matches (Required_Type (M.Opcode_Value), Type_Of (T, M.Cap)))
     with Ghost;

   -- The gate --------------------------------------------------------------
   type Verdict is (Forwarded, Denied);

   function Authorize (T : Cap_Table; M : Msg) return Verdict with
     Post => (Authorize'Result = Forwarded) = Authorized (T, M);

   -- Revocation relation: U is T with capability C removed -----------------
   function Revokes (T, U : Cap_Table; C : Cap_Id) return Boolean is
     ((for all I in Cap_Id =>
         (if I = C then not U (I).Valid else U (I) = T (I))));

   -- Lemma 1: the empty table denies every request -------------------------
   procedure Lemma_Empty_Table_Denies_All (M : Msg) with
     Ghost,
     Post => not Authorized (Empty_Table, M);

   -- Lemma 2: revocation never authorizes a previously-denied request ------
   procedure Lemma_Revocation_Monotonic
     (T, U : Cap_Table; C : Cap_Id; M : Msg) with
     Ghost,
     Pre  => Revokes (T, U, C) and then not Authorized (T, M),
     Post => not Authorized (U, M);

end Tg_Comm_Security;
