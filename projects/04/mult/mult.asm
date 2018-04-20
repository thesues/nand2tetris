// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Mult.asm

// Multiplies R0 and R1 and stores the result in R2.
// (R0, R1, R2 refer to RAM[0], RAM[1], and RAM[2], respectively.)

// Put your code here.
//i=0
//n=R1

//i=0
@i
M=0

//n=R1
@R1
D=M
@n
M=D

//sum=0
@sum
M=0

//if i == n; jump to end
(LOOP)
	@i
	D=M
	@n
	D=D-M
	@END
	D;JEQ
	
	//sum += R0
	@R0
	D=M
	@sum
	M=D+M
	
	//i++
	@i
	M=M+1
@LOOP
0;JMP

(END)
//R2=sum
@sum
D=M
@R2
M=D

(FINAL)
@FINAL
0;JMP
