# TrangorgeOS lib-ada — compile-check the Ada bindings and the SPARK model.
#
# The full proof is discharged with GNATprove (see README.md). With only the
# FSF compiler available we can still syntax/semantics-check every unit here.

GNATMAKE ?= gnatmake
FLAGS    := -gnat2022 -gnatwa -c

.PHONY: all check clean

all: check

check:
	$(GNATMAKE) $(FLAGS) tg_comm.ads
	$(GNATMAKE) $(FLAGS) tg_comm_security.adb
	$(GNATMAKE) $(FLAGS) tg_alloc_buddy.adb
	@echo "[lib-ada] compile-check OK"

clean:
	rm -f *.o *.ali
	rm -rf build

# Only meaningful when GNATprove is installed.
prove:
	gnatprove -P tg_comm.gpr --mode=prove
