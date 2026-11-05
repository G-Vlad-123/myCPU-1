
Start:
    mov r2 0
    cmp r1 0
    skp equal
    jmp ShowNumber

    mov r1 7

ShowNumber:
    set r1 C 9

    cmp r0 4
    skp equal
    set r1 C A

    # cmp r0 1
    # skp equal
    # set r1 C B

    # cmp r0 A
    # skp equal greater
    # set r1 C B
