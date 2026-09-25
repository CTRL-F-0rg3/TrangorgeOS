-- tg_comm.ads — Ada bindings for the TrangorgeOS strict shared-memory
-- communication protocol.
--
-- This spec mirrors the PUBLIC protocol (message layout, layers, opcodes,
-- rights, object types) of the Rust core in ../lib. The capability table,
-- rings and the authorization gate remain opaque: Ada never re-implements
-- them, it only calls the stable `tgcomm_*` C ABI exported by libtg_comm.a.

with Interfaces;
with System;

package Tg_Comm is
   pragma Preelaborate;

   use type Interfaces.Unsigned_32;
   use type Interfaces.Unsigned_16;

   -- Protocol constants (mirror lib/src/consts.rs) -------------------------
   Magic   : constant Interfaces.Unsigned_32 := 16#5447_434D#; -- "TGCM"
   Version : constant Interfaces.Unsigned_8  := 1;
   Msg_Size      : constant := 64;
   Ring_Ctrl     : constant := 16;
   Cap_Table_Size_Entries : constant := 512;

   -- Message kind ----------------------------------------------------------
   type Msg_Kind is (Request, Reply, Event, Notification, Cap_Transfer);
   for Msg_Kind use (Request => 0, Reply => 1, Event => 2,
                     Notification => 3, Cap_Transfer => 4);
   for Msg_Kind'Size use 8;

   -- Layers ----------------------------------------------------------------
   type Layer is (Kernel, Driverspace, Userspace, Manager, User_Driver_Space);
   for Layer use (Kernel => 0, Driverspace => 1, Userspace => 2,
                  Manager => 3, User_Driver_Space => 4);
   for Layer'Size use 8;

   -- Object types ----------------------------------------------------------
   type Object_Type is (Memory, Endpoint, Notification, Channel,
                        Shmem_Region, Device, Irq);
   for Object_Type use (Memory => 0, Endpoint => 1, Notification => 2,
                        Channel => 3, Shmem_Region => 4, Device => 5, Irq => 6);
   for Object_Type'Size use 8;

   -- Rights bitmask --------------------------------------------------------
   type Rights is mod 2**32;
   Right_Read     : constant Rights := 2**0;
   Right_Write    : constant Rights := 2**1;
   Right_Exec     : constant Rights := 2**2;
   Right_Map      : constant Rights := 2**3;
   Right_Grant    : constant Rights := 2**4;
   Right_Transfer : constant Rights := 2**5;
   Right_Send     : constant Rights := 2**6;
   Right_Recv     : constant Rights := 2**7;
   Right_Call     : constant Rights := 2**8;
   Right_Manage   : constant Rights := 2**9;

   function Contains (Set, Subset : Rights) return Boolean is
     ((Set and Subset) = Subset);

   -- Opcode classes --------------------------------------------------------
   type Op_Class is (Sys, Mem, Cap, Ipc, Shmem, Video, Audio, Input,
                     Block, Net, Pci, Vgpu, Fs);
   for Op_Class use (Sys => 0, Mem => 1, Cap => 2, Ipc => 3, Shmem => 4,
                     Video => 5, Audio => 6, Input => 7, Block => 8,
                     Net => 9, Pci => 10, Vgpu => 11, Fs => 12);
   for Op_Class'Size use 8;

   function Opcode (Cls : Op_Class; Op : Interfaces.Unsigned_16)
                    return Interfaces.Unsigned_32 is
     (Interfaces.Unsigned_32 (Op_Class'Pos (Cls)) * 2**8
      or Interfaces.Unsigned_32 (Op and 16#FF#));

   -- Authorize result ------------------------------------------------------
   type Result is (Forwarded, Malformed, Unknown_Op, Route_Denied,
                   No_Capability, Rights_Insufficient, Type_Mismatch);
   for Result use (Forwarded => 0, Malformed => 1, Unknown_Op => 2,
                   Route_Denied => 3, No_Capability => 4,
                   Rights_Insufficient => 5, Type_Mismatch => 6);
   for Result'Size use 32;

   -- The fixed-size wire message (mirror lib/src/wire.rs, 64 bytes) --------
   type Comm_Msg is record
      Magic    : Interfaces.Unsigned_32;
      Version  : Interfaces.Unsigned_8;
      Kind     : Interfaces.Unsigned_8;
      Layer_Byte : Interfaces.Unsigned_8;
      Target   : Interfaces.Unsigned_8;
      Opcode   : Interfaces.Unsigned_32;
      Cap      : Interfaces.Unsigned_32;
      Seq      : Interfaces.Unsigned_32;
      Status   : Interfaces.Integer_32;
      A0       : Interfaces.Unsigned_64;
      A1       : Interfaces.Unsigned_64;
      A2       : Interfaces.Unsigned_64;
      A3       : Interfaces.Unsigned_64;
      Reserved : Interfaces.Unsigned_64;
   end record with Convention => C;

   for Comm_Msg use record
      Magic    at 0  range 0 .. 31;
      Version  at 4  range 0 .. 7;
      Kind     at 5  range 0 .. 7;
      Layer_Byte at 6  range 0 .. 7;
      Target   at 7  range 0 .. 7;
      Opcode   at 8  range 0 .. 31;
      Cap      at 12 range 0 .. 31;
      Seq      at 16 range 0 .. 31;
      Status   at 20 range 0 .. 31;
      A0       at 24 range 0 .. 63;
      A1       at 32 range 0 .. 63;
      A2       at 40 range 0 .. 63;
      A3       at 48 range 0 .. 63;
      Reserved at 56 range 0 .. 63;
   end record;
   for Comm_Msg'Size use 512;
   for Comm_Msg'Alignment use 8;

   -- Stable C ABI (provided by libtg_comm.a) -------------------------------
   function Tgcomm_Version return Interfaces.Unsigned_32;
   pragma Import (C, Tgcomm_Version, "tgcomm_version");

   function Tgcomm_Msg_Size return Interfaces.Unsigned_32;
   pragma Import (C, Tgcomm_Msg_Size, "tgcomm_msg_size");

   function Tgcomm_Ring_Size (Slots : Interfaces.Unsigned_32)
                              return Interfaces.Unsigned_64;
   pragma Import (C, Tgcomm_Ring_Size, "tgcomm_ring_size");

   procedure Tgcomm_Msg_Init (Msg : in out Comm_Msg);
   pragma Import (C, Tgcomm_Msg_Init, "tgcomm_msg_init");

   function Tgcomm_Authorize (Table : System.Address; Msg : System.Address)
                              return Interfaces.Integer_32;
   pragma Import (C, Tgcomm_Authorize, "tgcomm_authorize");

   procedure Tgcomm_Ring_Init (Ring : System.Address;
                               Slots : Interfaces.Unsigned_32);
   pragma Import (C, Tgcomm_Ring_Init, "tgcomm_ring_init");

   function Tgcomm_Ring_Push (Ring : System.Address; Msg : System.Address)
                              return Interfaces.Integer_32;
   pragma Import (C, Tgcomm_Ring_Push, "tgcomm_ring_push");

   function Tgcomm_Ring_Pop (Ring : System.Address; Out_Msg : System.Address)
                             return Interfaces.Integer_32;
   pragma Import (C, Tgcomm_Ring_Pop, "tgcomm_ring_pop");

   function Tgcomm_Ring_Available (Ring : System.Address)
                                   return Interfaces.Unsigned_32;
   pragma Import (C, Tgcomm_Ring_Available, "tgcomm_ring_available");

   procedure Tgcomm_Cap_Table_Init (Table : System.Address);
   pragma Import (C, Tgcomm_Cap_Table_Init, "tgcomm_cap_table_init");

   function Tgcomm_Cap_Insert (Table : System.Address;
                               Cap : Interfaces.Unsigned_32;
                               Obj_Id : Interfaces.Unsigned_64;
                               Obj_Type : Interfaces.Unsigned_8;
                               Rights_Mask : Interfaces.Unsigned_32)
                               return Interfaces.Integer_32;
   pragma Import (C, Tgcomm_Cap_Insert, "tgcomm_cap_insert");

   function Tgcomm_Cap_Check (Table : System.Address;
                              Cap : Interfaces.Unsigned_32;
                              Required_Rights : Interfaces.Unsigned_32)
                              return Interfaces.Integer_32;
   pragma Import (C, Tgcomm_Cap_Check, "tgcomm_cap_check");

   function Tgcomm_Cap_Remove (Table : System.Address;
                               Cap : Interfaces.Unsigned_32)
                               return Interfaces.Integer_32;
   pragma Import (C, Tgcomm_Cap_Remove, "tgcomm_cap_remove");

end Tg_Comm;
