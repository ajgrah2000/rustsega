Rust Sega Emulator
==================

Conversion of a C emulator, that I'd converted to C++, that I'd converted to
Python, that I've converted to Rust.


TODO:

 Optimise:
 - Add (back) in some simple optimisations to skip calculation.
 - Don't do full VDP processing per byte update (check for changes, isolate).
 - Look for CPU cycles
 - Profile


 Fix scrolling
  - Currently not working      

Add more tests
  - Capture Timing, put in fairly full set of op code checks, so op codes can be tidied up (there's currently a lot of repetition).

Joysticks
  - Get inputs working (should be simple)

Sound
  - Figure out how to use SDL audio in rust
  - Make some noise
  - Figure out how to manage the SDL buffers so they don't starve (hopefully can query for buffer info/consumed/available/etc)
  
Constants
  - Make SMS Height/Width available to remove magic numbers

Rust General
  - cargo clippy
  - profiling
  - remove all warnigns
