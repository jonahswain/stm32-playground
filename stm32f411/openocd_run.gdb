target extended-remote :3333

# Print demangled symbols
set print asm-demangle on

# Detect unhandled exceptions, hard faults and panics
break DefaultHandler
break UserHardFault
break rust_begin_unwind

# Enable ARM semihosting
monitor arm semihosting enable

load
continue