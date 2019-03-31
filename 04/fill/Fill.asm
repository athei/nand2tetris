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

    @offset
    M=0

(LOOP)
    @KBD
    D=M

    // branch -> pressed or not pressed
    @BLACKEN
    D;JNE

    // whiten if no button pressed
    @offset
    D=M
    @SCREEN
    A=D+A
    M=0
    @NEXT
    0;JMP

(BLACKEN)
    @offset
    D=M
    @SCREEN
    A=D+A
    M=-1

(NEXT)
    // increment pointer to next
    @offset
    MD=M+1
    @8192
    D=A-D
    @LOOP
    D;JGT

(RESET)
    @offset
    M=0
    @LOOP
    0;JMP


