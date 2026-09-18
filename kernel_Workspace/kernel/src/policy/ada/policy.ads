with Interfaces; use Interfaces;

package Policy
  with SPARK_Mode
is

   type Ring_Id is (Ring_Kernel, Ring_Driver, Ring_User)
     with Size => 8;

   type Call_Class is (Cls_Sys, Cls_Video, Cls_Audio,
                       Cls_Input, Cls_Block, Cls_Net)
     with Size => 8;

   type Decision is (Allow, Deny)
     with Size => 8;

   type U64 is mod 2**64;

   BLK_WRITE : constant := 3;

   function Evaluate (Ring  : Ring_Id;
                      Class : Call_Class;
                      Op    : Unsigned_8;
                      Arg   : U64)
                      return Decision
     with
       Global => null,
       Post   =>
         (if Ring = Ring_Kernel then Evaluate'Result = Allow)
         and then
         (if Ring = Ring_User
             and then Class = Cls_Block
             and then Op = BLK_WRITE
          then Evaluate'Result = Deny)
         and then
         (if Ring = Ring_User and then Class = Cls_Net
          then Evaluate'Result = Deny);

end Policy;