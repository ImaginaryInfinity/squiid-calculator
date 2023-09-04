Squiid has support for storing and recalling numbers in variables.

## Storing values in variables
To store a variable in algebraic mode, type the variable name followed by an equals symbol, and the value you wish to assign to the variable (eg `a=1`). See [invstore](commands/internal-funcs/invstore.md) documentation for more details.

To store a value in a variable in RPN mode, first place the number in the stack (eg `1`), then place the variable name in the stack (eg `a`), then type `store`. See [store](commands/rpn/store.md) documentation for more details.

## Recalling values from variables
Variables can be used in place of numbers when preceded with a dollar sign (eg `$a`). This is supported in algebraic mode (eg `$a+5`) as well as RPN mode.

## Deleting variables
Variables can be deleted with the purge command. (eg `purge(a)` in algebraic mode). See [purge](commands/functions/purge.md) documentation for more details.