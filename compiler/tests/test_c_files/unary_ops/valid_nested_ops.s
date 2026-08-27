.globl _main
_main:
mov w0, #3
neg w0, w0
cmp w0, #0
cset w0, eq
ret