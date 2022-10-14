Rust Sega Emulator
==================

Conversion of a C emulator, that I'd converted to C++, that I'd converted to
Python, that I've converted to Rust.


    Usage:
      target/debug/rustsega [OPTIONS] CARTRIDGE
    
    Rusty Sega Emulator
    
    Positional arguments:
      cartridge             Name of cartridge to run
    
    Optional arguments:
      -h,--help             Show this help message and exit
      -d,--debug            Print PC State Debug Info
      -n,--no_delay         Run the emulator with no delay (rather than real-time)


(Current) Inputs:
    Key mappings (Joystick 1):
    Up: Up, Down: Down, Left: Left, Right: Right
    Fire A: Z, Fire B: X
    Reset: R

    Quit: Escape



TODO:

 Non-functional:
    Improve structure (current structure is shortest path to get things running).
    Fix status flag calculations,  cross check with good known Z80 results.

    Clean up 'sega.rs' there's a bit too much 'glue' going on there, that should be shifted out to the submodules.

 Add references to documentation used for VDP (need to track it down from an old PC). 

 Optimise:
 - Don't do full VDP processing per byte update (check for changes, isolate).
 - Look for CPU cycles
 - Profile


Add more tests
  - Capture Timing, put in fairly full set of op code checks, so op codes can be tidied up (there's currently a lot of repetition).

Sound
  - Set a better/dynamic audio queue length (based on speed/current buffer size, for better sound.)
  
Constants
  - Make SMS Height/Width available to remove magic numbers

Rust General
  - cargo clippy
  - profiling
        - cargo flamegraph
  - remove all warnigns

vim setup
  Currently not really sure what the best way to setup vim is.  Generally, I like a 'minimal' setup, so I can easily get a consistent setup if plugins can't be used.

     It seems as though syntax highlighting works out of the box with Vim 8.1, and '!cargo fmt' seems to work well.

     my ~/.vimrc, (with rust additions):
        silent !stty -ixon

        " format for rust errors
        set efm=%.%#-->\ %f:%l:%c
        " format for git searches
        set efm+=%f:%l:%m
        
        au FileType rust set makeprg=cargo

        nnoremap <C-s> :cgetexpr system('git grep --recurse-submodules -n '. expand('<cword>'))<CR>

   Note to self, try these:
      git clone --depth 1 https://github.com/preservim/nerdtree.git  ~/.vim/pack/vendor/start/nerdtree
      git clone --depth 1 https://github.com/dense-analysis/ale.git ~/.vim/pack/git-plugins/start/ale
      git clone --depth 1 https://github.com/timonv/vim-cargo ~/.vim/pack/git-plugins/start/vim-cargo

