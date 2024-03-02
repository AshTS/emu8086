# 8088 Emulator - Entry 1

The goal of this project is to make an Intel 8088 emulator which is accurate enough to boot MS-DOS as an IBM PC compatible. As references throughout this process, I will be using the [Intel 8086 Family User's Manual](https://edge.edx.org/c4x/BITSPilani/EEE231/asset/8086_family_Users_Manual_1_.pdf), the [IBM 5150 Technical Reference](https://minuszerodegrees.net/manuals/IBM_5150_Technical_Reference_6322507_APR84.pdf), and various other pieces of documentation which will be noted when used. The emulator will be written in Rust. Any libraries used will be noted in the writeups. Anything which is part of the emulation process (decoding instructions, manipulating memory, any operations specific to a chip set, etc.) will be custom written, whereas other libraries will be utilized to keep the scope of the project well contained. 

As of writing, decisions such as the library used for displaying the video output of the PC has not yet been made, and will be made as the project proceeds. For now, the focus is on getting a foundation set up so we know what we need to create for each step.

## Roadmap

As of writing, the order of operations is as follows:

1. Intel 8088 Emulator
    1. Memory Interface (BIU)
    2. Execution Unit
2. Memory Map
3. Loading the BIOS
4. TBD

## Getting Started

First, we will be setting up our development environment. Everything will be created as a workspace, and will contain many libraries and one executable. We will start with the `emu` executable crate, which will eventually give a CLI to the PC emulator. For now, we will leave this as just the default "Hello World" program. We will also want to create an `emu8088` library crate to contain the 8088 emulator code. Within this library, we create two modules, one for the BIU (Bus Interface Unit), and one for the EU (Execution Unit).