-- tg_comm_security.adb — bodies for the SPARK authorization-gate model.
--
-- The contracts live in the specification; this body gives the concrete
-- definitions. The two `Lemma_*` procedures are `Ghost`: their empty bodies
-- are discharged by GNATprove by unfolding the (transparent) definitions of
-- `Authorized`, `Has_Cap` and `Revokes`.

pragma SPARK_Mode (On);

package body Tg_Comm_Security with SPARK_Mode => On is

   function Route_Allowed (From, To : Layer) return Boolean is
   begin
      case From is
         when Kernel => return To /= Kernel;                 -- kernel -> any
         when Driverspace => return To = Kernel or To = Manager;
         when Userspace   => return To = Kernel or To = Manager;
         when Manager     => return To /= Manager;           -- manager -> any
         when User_Driver_Space => return To = Kernel or To = Manager;
      end case;
   end Route_Allowed;

   function Required_Right (O : Opcode) return Rights_Mask is
   begin
      case O.Class is
         when Sys => return Right_Call;
         when Mem =>
            if    O.Op = 1 then return Right_Manage;
            elsif O.Op = 3 then return Right_Map or Right_Write;
            elsif O.Op = 5 then return Right_Grant or Right_Map;
            else return Right_Call;
            end if;
         when Cap => return Right_Manage;
         when Ipc =>
            if    O.Op = 1 or O.Op = 3 then return Right_Send;
            elsif O.Op = 2 or O.Op = 4 then return Right_Recv;
            else return Right_Send or Right_Recv;
            end if;
         when Shmem =>
            if O.Op = 1 then return Right_Manage;
            else return Right_Map;
            end if;
         when Video | Audio | Input | Block | Net | Pci | Vgpu | Fs =>
            return Right_Call;
      end case;
   end Required_Right;

   function Required_Type (O : Opcode) return Type_Req is
   begin
      case O.Class is
         when Sys | Cap => return Any_Type;
         when Mem        => return Memory;
         when Ipc        => return Endpoint;
         when Shmem      => return Shmem_Region;
         when Video | Audio | Input | Block | Net | Pci | Vgpu | Fs =>
            return Device;
      end case;
   end Required_Type;

   function Rights_Of (T : Cap_Table; C : Natural) return Rights_Mask is
   begin
      return T (Cap_Id (C)).Rights;
   end Rights_Of;

   function Type_Of (T : Cap_Table; C : Natural) return Object_Type is
   begin
      return T (Cap_Id (C)).Obj_Type;
   end Type_Of;

   function Authorize (T : Cap_Table; M : Msg) return Verdict is
   begin
      -- Mirrors the Rust `authorize` in ../lib/src/filter.rs step by step.
      if not M.Well_Formed then
         return Denied;
      elsif not Route_Allowed (M.Src_Layer, M.Dst_Layer) then
         return Denied;
      elsif not Has_Cap (T, M.Cap) then
         return Denied;
      elsif not Contains (Rights_Of (T, M.Cap), Required_Right (M.Opcode_Value)) then
         return Denied;
      elsif not Matches (Required_Type (M.Opcode_Value), Type_Of (T, M.Cap)) then
         return Denied;
      else
         return Forwarded;
      end if;
   end Authorize;

   procedure Lemma_Empty_Table_Denies_All (M : Msg) is
   begin
      -- Unfold Authorized(Empty_Table, M): it requires Has_Cap(Empty_Table,
      -- M.Cap). But Empty_Table(I).Valid = False for every I, so Has_Cap is
      -- False, so Authorized is False. GNATprove discharges this directly.
      null;
   end Lemma_Empty_Table_Denies_All;

   procedure Lemma_Revocation_Monotonic
     (T, U : Cap_Table; C : Cap_Id; M : Msg) is
   begin
      -- Revokes(T,U,C) means U is T with capability C invalidated and all
      -- other entries unchanged. Hence, for every handle X:
      --   Has_Cap(U,X)  =>  Has_Cap(T,X)
      -- (removing a capability never creates one). Therefore
      --   Authorized(U,M)  =>  Authorized(T,M)
      -- because Authorized is monotone in the table. The contrapositive is
      -- exactly the post-condition:
      --   not Authorized(T,M)  =>  not Authorized(U,M).
      -- GNATprove discharges this by unfolding Has_Cap and Authorized and
      -- performing the case split on M.Cap = C versus M.Cap /= C.
      null;
   end Lemma_Revocation_Monotonic;

end Tg_Comm_Security;
