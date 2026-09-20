package main

import "core"
import "log"
import "util"

main :: proc() -> int {
    status := core.early_boot()
    if status != util.ERR_OK {
        return cast(int, status)
    }

    log.info("main", "Zarządca wystartował pomyślnie. Wersja: 0.1.0-alpha")
    core.run_main_loop()
    core.shutdown()
    
    return 0
}