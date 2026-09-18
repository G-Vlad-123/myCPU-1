Since this is supposed to be a simple 4 bit CPU I decided to make my own hyper simple architecture!
I have a tiny amount of expiriance hearing about x86 and RISC architectures but nothing deep.

# Architecture
All values are big edian

- `[register n]` = `n` 0 <= n < 16
- `[address n]` = `n`  0 <= n < 16
- `[value n]` = `n`    0 <= n <= 16
- 0 `0000`: `hlt`                                              = ends the program
- 1 `0001`: `mov [in register]  [out register]`                = in register is set to the value of out register
- 1 `0001`: `mov [in register]  1111           [value]`        = in register is set to the given value (using 0xF for the input register will always get the value)
- 2 `0010`: `cmp [1st register] [2nd register]`                = the first register is compater to the second, setting the cmp flags
- 3 `0011`: `smp [1st register] [2nd register]`                = same as cmp but the values are assumed to be signed
- 4 `0100`: `jmp [1st address]  [2nd address]`                 = jumps to the 8 bit address obtained by concatinating the two 4 bit addresses
- 5 `0101`: `skp [cmp check]    [valid check]`                 = jumps over the next instruction if the "check holds"
- 6 `0110`: `get [in register]  [1st address]  [2nd address]`  = in register is set by the value at the given address
- 7 `0111`: `set [in register]  [1st address]  [2nd address]`  = the value at the given address is set by the value in the in register
- 8 `1000`: `add [in register]  [out register]`                = in register is increased (overflowing) by the out register
- 9 `1001`: `sub [in register]  [out register]`                = in register is decreased (underflowing) by the value of out register
- A `1010`: `mul [in register]  [out register]`                = in register is scaled (overflowing) by the out register
- B`1011`: `div [div register] [rem register] [out register]` = div register is divibed by the value of the out register, the remained is put in the rem register
- C `1100`: `not [in register]  [out register]`                = in register is set to the bitwise-not of the value of out register
- D `1101`: `and [in register]  [out register]`                = in register is set to it's value bitwise and'ed by the out register
- E `1110`: `or  [in register]  [out register]`                = in register is set to it's value bitwise or'ed by the out register
- F `1111`: `xor [in register]  [out register]`                = in register is set to it's value bitwise xor'ed by the out register

All integers are unsigned unless specified otherwise.

When dividing or setting the modulus by 0 the `invalid` flag will be set to true*

skp checking will be fully defined later on*

*This is up to change
