mov r0 0

Loop:
    set r0 _sum_reg

    const 7 0 C
    _sum_reg:
        const 0

    add r0 1
    skp overflow
    jmp Loop
    hlt
