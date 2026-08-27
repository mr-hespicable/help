.globl _main
_main:
mov w0, #5
cmp w0, #0
cset w0, eq
ret