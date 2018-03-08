# constant push

```
    push constant 10
```

## pseudo code:

```
    *sp = 10
    sp++
```

## code 
```
    @i
    D=A
    @SP
    A=M
    M=D
    @SP
    M=M+1
```

# push:
```
    push static 10
    push argument 10
```
## pseudo code:
```
    addr = segment + i
    *sp = *addr
    sp++
```

## code

```
    //addr = segment +i 
    @i
    D=A
    @SEGMENT
    D=M+D
    @addr
    M=D
    // *sp = *addr
    @addr
    A=M
    D=M
    @SP
    A=M
    M=D
    //sp++
    @SP
    M=M+1
```


# pop

## pseudo code:

```
    addr = segment +i 
    sp --;
    *addr = *sp
```


## code 

```
    //addr = segment +i 
    @i
    D=A
    @SEGMENT
    D=M+D
    @addr
    M=D
    //sp --;
    @SP
    M=M-1
    // *addr = *sp
    @SP
    A=M
    D=M
    @addr
    A=M
    M=D
```


# static push

## pseudo code:

```
    *sp=foo.i
    sp++
```
## code 

```
    @foo.i
    D=M
    @SP
    A=M
    M=D

    @SP
    M=M+1
```

# static pop

## pseudo code:

```
    sp --
    foo.i = * sp
```

## code

```
    @SP
    M=M-1

    @SP
    A=M
    D=M
    @foo.i
    M=D

```

# add/sub/and/or

## pseucode
```
sp --
R13 = *sp

sp --
R14 = *sp + R13
*sp = R14

sp++

```

## code
```
    @SP
    M=M-1

    // R13=*sp
    @SP
    A=M
    D=M
    @R13
    M=D


    @SP
    M=M-1
    // D =*sp + R13
    @SP
    A=M
    D=M
    @R13
    D=D+M

    // D is final result
    @SP
    A=M
    M=D


    @SP
    M=M+1
```

# not/neg

## pseucode code 

```
sp--
R13=*sp
*sp=!R13
sp++
```

## code

```
//sp--
@SP
M=M-1

//D=!*sp
@SP
A=M
D=!M

//*sp
@SP
A=M
M=D

@SP
M=M+1

```


## better code
```
@SP
A=A-1
M=!M
```

# eq/gt/lt

## pseucode code 
```
sp--
R13=*sp
sp--
R14=*sp
D=R14-R13;
*(sp) = 0 //false
*(sp) = -1 //true
sp ++
```

## code

```
//sp--
@SP
M=M-1

//R13=*sp
@SP
A=M
D=M
@R13
M=D

//sp--
@SP
M=M-1

//*sp - R13
@SP
A=M
D=M
@R13
D=D-M

@EQ.{{N}}
D;JEQ

@SP
A=M
M=0
@(EQ.END.{{N}})
0;JMP

(EQ.{{N}})
@SP
A=M
M=-1
(EQ.END.{{N}})
@SP
M=M+1

```


