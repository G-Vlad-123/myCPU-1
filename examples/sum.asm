jmp Start

StoreLoop:
        set r0 _store_reg
        set r0 _store_pos
        const 7

    _store_reg:
        const 0 A

    _store_pos:
        const 0

        sub r0 1
        cmp r0 0
        skp equal
        jmp StoreLoop
        jmp EndStoreLoop

SumLoop:
        set r0 _sum_reg
        const 6 d A

    _sum_reg:
        const 0

        add re rd

        sub r0 1
        skp overflow
        jmp SumLoop
        jmp EndSumLoop

Start:
        set r0 A 0

        mov r0 E
        jmp StoreLoop

    EndStoreLoop:
        mov r0 E
        mov re 0
        jmp SumLoop

    EndSumLoop:
        set re F F
