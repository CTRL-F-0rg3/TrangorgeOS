package body Policy
  with SPARK_Mode
is

   function Evaluate (Ring  : Ring_Id;
                      Class : Call_Class;
                      Op    : Unsigned_8;
                      Arg   : U64)
                      return Decision
   is
      pragma Unreferenced (Arg);
   begin
      if Ring = Ring_Kernel then
         return Allow;
      end if;

      if Ring = Ring_User
        and then Class = Cls_Block
        and then Op = BLK_WRITE
      then
         return Deny;
      end if;

      if Ring = Ring_User and then Class = Cls_Net then
         return Deny;
      end if;

      if Ring = Ring_Driver then
         return Allow;
      end if;

      case Class is
         when Cls_Sys | Cls_Video | Cls_Input | Cls_Audio | Cls_Block =>
            return Allow;
         when Cls_Net =>
            return Deny;
      end case;
   end Evaluate;

end Policy;