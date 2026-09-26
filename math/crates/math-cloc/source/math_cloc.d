module math_cloc;

extern(C) {
    struct Time {
        ulong secs;
        uint nanos;
    }

    enum SUCCESS = 0;
    enum ERR_NULL_PTR = -1;
    enum ERR_OVERFLOW = -2;
    enum ERR_TIME_REWIND = -3;

    int math_cloc_add(Time a, Time b, Time* out);
    int math_cloc_validate_monotonic(Time current, Time previous);
}

/// Safe D wrapper that translates error codes to exceptions.
Time addTime(Time a, Time b) {
    Time result;
    int status = math_cloc_add(a, b, &result);
    
    if (status == ERR_NULL_PTR) throw new Exception("Null pointer in math-cloc");
    if (status == ERR_OVERFLOW) throw new Exception("Time overflow detected");
    
    return result;
}