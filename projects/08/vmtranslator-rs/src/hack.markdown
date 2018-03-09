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

better one

```
    //*(segment + i)
    @i
    D=A
    @SEGMENT
    A=M+D
    D=M
    //*sp = D
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
    M=M+D
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


# Function 

## pseucode code 

```
    function foo 3

    (filename.foo)
    repeat 3
    push 0
```

```
    @SP
    A=M
    M=0
    @SP
    M=M+1
```


# Return 

## 

```
frame = LCL // frame is a temp. variable
retAddr = *(frame-5) // retAddr is a temp. variable
*ARG = pop // repositions the return value

// for the caller
SP=ARG+1 // restores the caller’s SP
THAT = *(frame-1) // restores the caller’s THAT
THIS = *(frame-2) // restores the caller’s THIS
ARG = *(frame-3) // restores the caller’s ARG
LCL = *(frame-4) // restores the caller’s LCL
goto retAddr // goto returnAddress
/

```

### code 



```
//R13 = *(LCL-5)
@LCL
D=M
@5
A=D-A
D=M
@R13
M=D


//*ARG = pop
@SP
A=M-1
D=M
@ARG
A=M
M=D


//SP=ARG+1
@ARG
D=M+1
@SP
M=D

//THAT=*(LCL-1)
@LCL
A=M-1
D=M
@THAT
M=D

//THIS = *(LCL-2)
@LCL
D=M
@2
A=D-A
D=M
@THIS
M=D

//ARG = *(LCL-3)
@LCL
D=M
@3
A=D-A
D=M
@ARG
M=D

//LCL = *(LCL-4)
@LCL
D=M
@4
A=D-A
D=M
@LCL
M=D

//goto retAddr
@R13
A=M
0;JMP


```

# Call


```
push return address
push LCL
push ARG
push THIS
push THAT
ARG=SP-nARGS-5
LCL=SP
goto G

(return address)
```


return address = filename.foo.$ret.i


```
//push return address
@RETURN_ADDRESS
D=A
@SP
A=M
M=D
@SP
M=M+1
//push LCL
@LCL
D=M
@SP
A=M
M=D
@SP
M=M+1
//push ARG
@ARG
D=M
@SP
A=M
M=D
@SP
M=M+1
//push THIS
@THIS
D=M
@SP
A=M
M=D
@SP
M=M+1
//push THAT
@THAT
D=M
@SP
A=M
M=D
@SP
M=M+1
//NUM=nARGS+5
@NUM
D=A
@SP
D=M-D
@ARG
M=D
//LCL=SP
@SP
D=M
@LCL
M=D
//goto G
@G
0;JMP
(RETURN_ADDRESS)

```
