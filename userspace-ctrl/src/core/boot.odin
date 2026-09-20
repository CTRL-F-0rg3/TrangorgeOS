package core

import "log"
import "util"
import "kernel_if"

pub early_boot :: proc() -> util.Errno {
    log.init()
    log.info("boot", "Rozpoczynanie wczesnej inicjalizacji Zarządcy...")
    
    err := kernel_if.debug_print("BOOT: Userspace manager starting\n")
    if err != util.ERR_OK {
        log.error("boot", "Brak komunikacji z jądrem (debug_print zwrócił: %v)", err)
        return err
    }
    
    log.debug("boot", "Połączenie z jądrem: OK")
    log.info("boot", "Wczesna inicjalizacja zakończona pomyślnie.")
    return util.ERR_OK
}

pub run_main_loop :: proc() {
    log.info("loop", "Zarządca wchodzi do głównej pętli zdarzeń.")
    for {
        break 
    }
    log.warn("loop", "Główna pętla została przerwana (tryb demonstracyjny).")
}

pub shutdown :: proc() {
    log.info("shutdown", "Zarządca zatrzymany. Przekazanie kontroli do jądra.")
}