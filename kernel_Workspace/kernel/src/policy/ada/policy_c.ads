with Interfaces; use Interfaces;
with Policy;

package Policy_C is

   function policy_evaluate (ring : Unsigned_8;
                             cls  : Unsigned_8;
                             op   : Unsigned_8;
                             arg  : Unsigned_64)
                             return Unsigned_8
     with Convention => C,
          Export,
          External_Name => "policy_evaluate";

end Policy_C;