with Policy; use Policy;

package body Policy_C is

   function To_Ring (V : Unsigned_8) return Ring_Id is
     (case V is
         when 0 => Ring_Kernel,
         when 1 => Ring_Driver,
         when others => Ring_User);

   function To_Class (V : Unsigned_8) return Call_Class is
     (case V is
         when 0 => Cls_Sys,
         when 1 => Cls_Video,
         when 2 => Cls_Audio,
         when 3 => Cls_Input,
         when 4 => Cls_Block,
         when others => Cls_Net);

   function policy_evaluate (ring : Unsigned_8;
                             cls  : Unsigned_8;
                             op   : Unsigned_8;
                             arg  : Unsigned_64)
                             return Unsigned_8
   is
      D : constant Decision :=
        Evaluate (To_Ring (ring), To_Class (cls), op, U64 (arg));
   begin
      return (case D is
                 when Allow => 0,
                 when Deny  => 1);
   end policy_evaluate;

end Policy_C;