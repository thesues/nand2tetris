// This file is part of www.nand2tetris.org
// and the book "The Elements of Computing Systems"
// by Nisan and Schocken, MIT Press.
// File name: projects/04/Fill.asm

// Runs an infinite loop that listens to the keyboard input.
// When a key is pressed (any key), the program blackens the screen,
// i.e. writes "black" in every pixel;
// the screen should remain fully black as long as the key is pressed. 
// When no key is pressed, the program clears the screen, i.e. writes
// "white" in every pixel;
// the screen should remain fully clear as long as no key is pressed.

// Put your code here.



@KBD
D=A
@kbd_address
M=D
// kbd_address is 24567

//every row has 32 words, and we have 256 rows
@8192
D=A
@n
M=D

@SCREEN
D=A
@addr
M=D

(LOOP)

	@i
	M=0

	//if *kbd_address is 0, jump to clear
	//@kbd_address
	//A=M
	//D=M
	//@CLEAR
	//D;JEQ
	//OR
	@KBD
	D=M
	@CLEAR
	D;JEQ
	
	(FILL)
		//if i == n, jump to end
		@i
        	D=M
		@n
        	D=D-M
		@LOOP
        	D;JEQ

		//addr[i] = -1
		@addr
		D=M
		@i
		A=D+M
		M=-1

		//i++
		@i
        	M=M+1


		@FILL
		0;JMP
		
        (CLEAR)
		//if i == n, jump to end
		@i
        	D=M
		@n
        	D=D-M
		@LOOP
        	D;JEQ

		//addr[i] = 0
		@addr
		D=M
		@i
		A=D+M
		M=0 

		//i++
		@i
        	M=M+1

		@CLEAR
		0;JMP
	
@LOOP
0;JMP 


