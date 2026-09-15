Since this is supposed to be a simple 4 bit CPU I decided to make my own hyper simple architecture!
I have a tiny amount of expiriance hearing about x86 and RISC architectures but nothing deep.

# NOTE: Remove all instructions that take in a [value] other than `mov`

# Architecture
- `[register n]` = `n`
- `[value n]` = `n`
- `0000`: `mov [in register]  [value]`        = in register is set to the given value
- `0001`: `mov [in register]  [out register]` = in register is set to the value of out register
- `0010`: `cmp [1st register] [2nd register]` = the first register is compater to the second, setting the cmp flags
- `0011`: `smp [1st register] [2nd register]` = same as cmp but the values are assumed to be signed
- `0100`: `jmp [1st address]  [2nd address]`  = jumps to the given 8 bit address
- `0101`: `skp [cmp check]    [valid check]`  = jumps over the next instruction if the "check holds"
- `0110`: `---`                               = unused
- `0111`: `---`                               = unused
- `1000`: `add [in register]  [value]`        = in register is increased (overflowing) by the given value
- `1001`: `add [in register]  [out register]` = in register is increased (overflowing) by the value of out register
- `1010`: `mul [in register]  [value]`        = in register is scaled (overflowing) by the given value
- `1011`: `mul [in register]  [out register]` = in register is scaled (overflowing) by the value of out register
- `1100`: `div [in register]  [value]`        = in register is divided by the value of the given value
- `1101`: `div [in register]  [out register]` = in register is divibed by the value of the out register
- `1110`: `rem [in register]  [value]`        = in register is set to it's value mod the value of the given value
- `1111`: `rem [in register]  [out register]` = in register is set to it's value mod the value of the out register

All integers are unsigned unless specified otherwise.

When dividing or setting the modulus by 0 the `nan` flag will be set to true
