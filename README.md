# m8db

Debugger and interpreter for the M8 pseudo-assembly language. Inspired by `gdb` or `lldb`

More infos: https://github.com/ah1m1/M8NI

Usage: `$ ./m8db <filename>`


# Instructions:  
* `INC r`
* `DEC r`
* `JUMP line`
* `STOP`
* `IS_ZERO r line`

Where `r` is a register number and `line` is a line number.  
`IS_ZERO` jumps to `line` if `r` is zero
