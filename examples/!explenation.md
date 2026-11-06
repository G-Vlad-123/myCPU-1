# What is this?
An explenation of what each `[name].asm` program is ment to do,
they all test *both* the `assembler` and `myCPU-1`

### How do I test?
You just run two commands!
Assuming you are currently in directory containing the entire project.
These should work even if you run thigns from a diferent directory though,
it's just the assembler will create `[name].bin` files in ur current dir

I'll also assume you are using Windows and if not that
you know what the equivalent commands are on your OS

Firstly run ```./target/release/assembler.exe examples/[name].asm```
if nothing is printed into the terminal then it ran succsesfully!

Then run ```./target/release/myCPU-1.exe [name].bin [0-15 inputs]```
what the inputs mean and how many is desirable is a per-program thing,
when ran it will print an 8x8 grid of squares, what the output means is also
a per-program thing, in general their colour will represent a final value tho

### What value does each colour represent?

Each colour is given based on the bits of the value it represents
(in big edian) aka it's rgb colour is set based in the 3 lesser bits,
and weather it's normal (■) or shaded (▤) 

If you want a list of the exact values here it is:
- 0 = black
- 1 = blue
- 2 = green
- 3 = cyan
- 4 = red
- 5 = magenta
- 6 = yellow
- 7 = white
- 8 = shaded black
- 9 = shaded blue
- 10 = shaded green
- 11 = shaded cyan
- 12 = shaded red
- 13 = shaded magenta
- 14 = shaded yellow
- 15 = shaded white

Alternatively you can run `colours.asm` and it will have in
the first 2 rows all the colours in increasing order of the value
they represent, starting with 0 (black) and ending with 15 (shaded white)

# What each program does

### add.asm
2 arguments
 
Adds the 2 given values and prints the bottom right corner to
the colour coresponding to the final value. Overflow is ommited.

### calc.asm
2 arguments

Gets the 2 given numbers and performs the following operations
in the given order: `+`, `-`, `*`, `/`, `%`, `&`, `|`, `^`, `!`.
For each operation in order starting from the top left it's the
coresponding colour to the value of the result is printed.

Note: Not (`!`) is performed on only the first value

### colours.asm
no arguments

Prints in the first 2 rows every possible colour, in order of their
coresponding value.

### pattern.asm
no arguments

Displays a nice image.

### number.asm
1-2 arguments

Displays the decimal number representing the 1st given argument
The 2nd (optional) argument sets the colour of the number.
If the 2nd argument is black (0) or it's not given it automaticly
registers as white.

### sum.asm
0-15 arguments

Sums the first 15 given numbers and prints the result.
Overflow is ommited.
