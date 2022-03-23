# Reggie — the Lua VM specification

Reggie is a module that specifies and implements a register-based (hence the name) VM. It defines memory model, instruction set, and behaviors associated in order to describe lua computations.

_Design have taken inspiration from BEAM VM._

## Reggie bytecode compiler

Although the Reggie runtime VM and bytecode compiler are mentioned as being separate systems, it is noted that they are the part of the same API. Bytecode generation is not guaranteed to be stable across releases or even to contain serializable representation. There is no verification process of the bytecode, meaning that compiler and runtime are closely coupled together. Bytecode compiler is guaranteed to produce a valid code, otherwise behavior is undefined (aka. expect crashed in the best cases, and unnoted data corruption in the worst).

## Abstract machine specification

Reggie has a number of different global registers, and a few local ones, constraint to function call frames. The machine does not have, nor allow arbitrary memory read/writes outside of the value mutation. The code running in VM is inherently single-threaded, meaning that there is no synchronization primitives defined, and code cannot concurrently access data structures.

Reggie's register can be of different [data types](#vm-data-values), notably:

-   Nil (a special distinct from any other type of value)
-   64-bit IEEE-754 Floating point number
-   UTF-8 string
-   [Table](#table) data structure
-   [Function reference](#function)
-   Arbitrary external data reference ([userdata](#userdata))
-   [Dynamic](#dynamic) can contain all of the types defined above.

#### Type notation

There is a shorthand notation for the Reggie's supported types:

-   N - nill
-   F - 64 bit IEEE-754 float
-   I - 32 bit integer <!-- -   L - 64 bit integer -->
-   S - string
-   T - table
-   C - function
-   U - userdata
-   D - dynamic

### Data registers

There three sets of data registers: argument, extended argument, local, and accumulator registers. All of these have variants for each of the types that Reggie supports, except for nil.

#### Accumulator registers

They are the primary way to operate upon a value. There is one register per each data type except for nil. They are global to the VM.

The notation used for addressing each of the registers is AX where X is the [shorthand notation of the type of a register](#type-notation)

#### Argument registers

Argument registers are global to the VM instance. There is set of N registers defined for each type of data values that Reggie supports, where N is defined by the concrete implementation of Reggie.

There is also a set of extended argument registers for each type. They are expected to be less performant than N of nominal ones. There is no limitation (other than instruction addressing constraints) of an amount of extended registers.

The notation used for addressing each of the registers is RX0-RXN for nominal argument, and ExtRX0-ExtRXN for extended ones, where X is the [shorthand notation of the type of a register](#type-notation)

#### Local registers

Upon execution of [global scope](#global-scope) or a [function call](#function-call-convention) the finite amount of local registers may be allocated for use. They are persisted in a function call frame. The bytecode compiler and Reggie VM compute the amount of local registers needed ahead of time, so for the code being executed, there is practically unlimited local registers, but in reality, the least amount needed is computed by the Reggie VM.

The notation used for addressing each of the local registers is LX0-LXN, where X is the [shorthand notation of the type of a register](#type-notation)

### Special use registers

The VM also defines couple of special use-case registers, such as:

-   Program counter (PC)
-   Return address (RA)
-   Value count (VC)
-   Test result (TR)
-   Type test result (TTR)

#### Program counter

Is the byte-offset of the currently executing instruction. It is global to the VM, and cannot be read, nor set directly.

#### Return address

Is the byte-offset of the instruction that currently executing function should jump to when execution reaches [return instruction](#return-instruction). It is located in every call stack, and is persisted between function calls. It cannot be read, nor set directly, rather `call` `typed_call` and `ret` instruction rely on it's value.

#### Value count

Program readable and writable global register. It contains the amount of argument the function was called with, or the amount of arguments function returns.

#### Test result

Is set to appropriate value of `EQ` `NE` `LT` `GT` when [comparing two values](#test). Is global to the runtime, cannot be read nor set directly.

#### Type test result

Is set to appropriate value of `N`, `F`, `S`, `T`, `C`, `U` when performing [type test](#type_test). Is global to the runtime, cannot be read nor set directly.

### Modules

`TODO`

### Global scope

There is two scopes of instruction in Reggie bytecode: global and local. Global scope can execute all of the instructions local scope can (including return) but also gains ability to provide function definitions. Values can be defined with name association in the global scope. The values of global scopes can be queried, modified and added from any place, including the host Reggie's runtime and FFI calls.

### Functions

`TODO`

### Function call convention

There is two ways to call a Reggie function. Either with `call` or `typed_call`. Let's start with `call`.

Call operand calls the value in the AC register. The arguments to a function are provided via the argument registers RD0 through RDN. If the number of arguments exceed that of N, then the rest of the arguments should be placed in ExtRD registers. Operand also takes the number of arguments passed via the [VC](#value-count) register.

Upon invoking `call` instruction, the new stack frame is created with the size determined from callee's function metadata. The callee receives all of it's args through RD and ExtRD registers.

Returned values are placed in the RD0-RDN and ExtRD registers. The [VC](#value-count) register should be set to the amount of values returned from the function.

Calling function with `typed_call` is only possible when the bytecode compiler or Reggie VM can guarantee that the callee expects to be called with typed arguments. There are no implicit checks to verify that the function is compatible with calling signature. Argument to the callee are passed through RX0-RXN and ExtRX registers, where X is the [shorthand notation of the type of a register](#type-notation). [VC](#value-count) register is not expected to be set neither on call, nor on return.

Bot `call` and `typed_call` set [RA](#return-address) register in the callee's scope to the offset where function should return after it's invocation is done.

It is to be noted, that calling a function is expected to clobber all of the argument, extended argument, accumulator, and special use registers, so relying on them staying unchanged after the call is complete is a futile effort. In order to persist values after function call, it is advised to put them into local registers, since those are restored after the function call is done.

### Error handling

`TODO`

## VM data values

`TODO`

### Nil

`TODO`

### Number

`TODO`

### String

`TODO`

### Function

`TODO`

### Table

`TODO`

### Userdata

`TODO`

### Dynamic

`TODO`

## Module structure

`TODO`

## Function definition structure

`TODO`

## Instruction set

### Instructions accessible in global scope

<!-- #### gl_ret

Global return. Values that are present in RD0-RDN and ExtRD registers are exported to the outside of the module (to whomever may have required it), and control flow of the code execution is terminated and execution jumps to the caller of the module defined in [RA](#return-address) register.

The amount of values returned depends on [VC](#value-count) register. -->

#### fn_decl

`TODO`

### Instructions accessible in all scopes

#### lda_XYZ

Load value from register XYZ into an appropriate accumulator register.

-   X is the type of register (`R`, `ExtR`, `L`)
-   Y is the [type of register's value](#type-notation) (`N`, `F`, `S`, `T`, `C`, `U`, `D`)
-   Z is the register's number

The type of register (Y) determines the type of an accumulator (AY). X cannot specify an accumulator register

#### str_XYZ

Stores the value from an accumulator register AY into an register XYZ.

-   X is the type of register (`R`, `ExtR`, `L`)
-   Y is the [type of register's value](#type-notation) (`N`, `F`, `S`, `T`, `C`, `U`, `D`)
-   Z is the register's number

The type of register (Y) determines the type of an accumulator (AY). X cannot specify an accumulator register

#### mov_ABC_XBZ

Moves the value of register ABC into XBZ, where

-   A is the source register type (`R`, `ExtR`, `L`)
-   B is the [type of register's value](#type-notation) (`N`, `F`, `S`, `T`, `C`, `U`, `D`)
-   C is the source register number
-   X is the destination register type (`R`, `ExtR`, `L`)
-   z is the destination register number

#### lda_gl

Loads global value described by AS register into a AD register. If the value is not present in global scope, AD is set to the dynamic value of `nil`.

#### F_add_XZ

Adds the value of the register XFZ to the current value of accumulator register AF.

-   X is the type of source register
-   Z is the number of source register

#### F_mul_XZ

Multiplies the value of the register XFZ to the current value of accumulator register AF.

-   X is the type of source register
-   Z is the number of source register

#### F_sub_XZ

Multiplies the value of the register XFZ to the current value of accumulator register AF.

-   X is the type of source register
-   Z is the number of source register

#### F_div_XZ

Divides the value of the register XFZ to the current value of accumulator register AF.

-   X is the type of source register
-   Z is the number of source register

#### I_add_XZ

Adds the value of the register XIZ to the current value of accumulator register AI.

-   X is the type of source register
-   Z is the number of source register

#### I_mul_XZ

Multiplies the value of the register XIZ to the current value of accumulator register AI.

-   X is the type of source register
-   Z is the number of source register

#### I_sub_XI

Multiplies the value of the register XIZ to the current value of accumulator register AF.

-   X is the type of source register
-   Z is the number of source register

#### I_div_XZ

Performs integer division of the value of the register XIZ to the current value of accumulator register AF.

-   X is the type of source register
-   Z is the number of source register

#### I_mod_XZ

Return the modulo of the value of the accumulator register AF divided by value in the register XIZ. The result is stored in AF.

-   X is the type of source register
-   Z is the number of source register

#### set_vc

Sets the [VC](#value-count) register from the current value in AI. AI is treated to have an unsigned value.

#### call

[Calls](#function-call-convention) the value located in the register AC. Creates new stack frame. Sets the register [RA](#return-address) in the callee's stack frame

#### typed_call

[Performs the typed call](#function-call-convention) of the value located in AC. Creates new stack frame. Sets the register [RA](#return-address) in the callee's stack frame

#### ret

Jumps to instruction pointed in [RA](#return-address), discards current call frame.

#### test_XYZ

Compares the values in AY to the value in register XYZ. Sets the [TR](#test-result) register the the corresponding value.

-   X is the type of register (e.g. `R`, `ExtR`, `L`)
-   Y is the type of register's value (can only be `F`, `I`, `S`, `D`)
-   Z is the number of register

in case Y is `F`, compare values as specified in IEEE-754
in case Y is `I`, do a signed integer comparison
in case Y is `S`, do lexicographical comparison
in case Y is `D`, do any of the above if types are the same, if they differ, convert to string and compare lexicographically. If any of the operands types are `N`, `T`, `U` or `C`, panic with the `order` error

#### type_test

Do a type test of a value in AD, and set [TTR](#type-test-result) to the appropriate value.

#### jmp &lt;offset&gt;

Unconditional jump to the instruction by relative offset from the current [PC](#program-counter) value.

#### (jmplt, jmpgt, jmpeq, jmpne, jmple, jmpge) &lt;offset&gt;

Conditional jump by the offset &lt;offset&gt; depending on the value of [TR](#test-result) register.

-   `jmplt` jump if [TR](#test-result) is set to `LT`
-   `jmpgt` jump if [TR](#test-result) is set to `GT`
-   `jmpeq` jump if [TR](#test-result) is set to `EQ`
-   `jmpne` jump if [TR](#test-result) is set to `LT`, `GT`, or `NE`
-   `jmple` jump if [TR](#test-result) is set to `LT` or `EQ`
-   `jmpge` jump if [TR](#test-result) is set to `GT` or `EQ`

#### (jmpN, jmpF, jmpI, jmpC, jmpT, jmpU) &lt;offset&gt;

Conditional jump by the offset &lt;offset&gt; depending on the value of [TTR](#type-test-result) register.

-   `jmpN` jump if [TTR](#type-test-result) is set to `N`
-   `jmpF` jump if [TTR](#type-test-result) is set to `F`
-   `jmpI` jump if [TTR](#type-test-result) is set to `I`
-   `jmpC` jump if [TTR](#type-test-result) is set to `C`
-   `jmpT` jump if [TTR](#type-test-result) is set to `T`
-   `jmpU` jump if [TTR](#type-test-result) is set to `U`

#### panic

`TODO`