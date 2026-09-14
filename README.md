# My CPU 1
I tried making my own 4 bit CPU emulator!

Ofc since I am using a full programming language I could just make an asm interpreter
and therefor use high level constructs to avoid making the CPU it'self... but that is no fun!
So I'm setting some limits on myself right to hopefully make this more fun :)

# Rules
1. I mustn't use any language constructs other than `Bit` to store the data that will be read by
and written to by the CPU.
2. I am allowed to use a higher level structure to store the bits themselvs (eg: `[Bit; 1024]`)
but the CPU must see these bits as if they were in continous real memory at all times.
3. The only traits I am allowed to use are `Debug`, `Clone`, `Copy`, `PartialEq` and `Eq`. `Debug` in particular
is only allowed for debugging and not for i/o.
4. I am not allowed to modify any bits once the CPU started without using it directly. I am allowed to read a section
of those bits to emulate some sort of i/o tho. I am allowed to set the bits to anything I want before starting the CPU.
5. I am allowed to make my own functions to automate the manipulation of multiple bits but they must not include any function
from another library other than those provided by `Bit` and those that I already made that follow this rule.
6. I am allowed to ask for help! But it'd be best if I try to make things by myself with no help first. Help includes asking 
another person or AI about something. Not manually reaserching documentation on how CPUs work.

# Clarifications
Now there may have been some choices that I wanna preclarify before anyone asks (bdw if ur reading this thx sm!) ^v^

Why am I using Rust?
> I like it and it is a language I am comfortable with, there is no real reason to use one language over another for such a small project though

Why did I only allow myself to use specific traits?
> Some language features are alr implemented some stuff for me if I add those traits (for example I could skip making my own number compare algorithm
and use an array that impls `PartialOrd` if it's elemets also implement it instead) and I wanna avoid that for this specific project! As to why I
picked the traits I am allowed to use... `Debug` is useful for debugging, `Clone` is required by `Copy`, `Copy` is almost neceserry to have for something
like the `Bit` enum imo, `PartialEq` is required by `Eq`, `Eq` makes my life slightly simpler imo by removing a small complexity that is not really
adding anything by being there imo.

How am I gonna do i/o?
> Well I'll figure this out later (I'm writing this when I only have the `Bit` enum implemented) but most likely I will just have no input beside
the state of the data when the CPU starts, and some output eather while it's running or after it ran.

Why did I add so many rules?
> I like being verbose, plus if these rules end up being bad I can always try again under better limits!
