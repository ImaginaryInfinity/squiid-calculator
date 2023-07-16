# Inverted Store
`invstore` / `=`

The `invstore` command will store a value in a variable, with the variable name being the first argument. This is used for storing variables in algebraic mode, since that is the intuitive way `=` works. 

----

### Function Arguments
```plaintext
invstore(variable_name, value)
```

----

### Algebraic Example
```plaintext
a = 5
```

```plaintext
invstore(a, 5)
```

### RPN Example
```plaintext
a
5
invstore
```