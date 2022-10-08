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

vim setup
  Currently not really sure what the best way to setup vim is.  Generally, I like a 'minimal' setup, so I can easily get a consistent setup if plugins can't be used.

      git clone --depth 1 https://github.com/preservim/nerdtree.git  ~/.vim/pack/vendor/start/nerdtree
      git clone --depth 1 https://github.com/dense-analysis/ale.git ~/.vim/pack/git-plugins/start/ale
      git clone --depth 1 https://github.com/timonv/vim-cargo ~/.vim/pack/git-plugins/start/vim-cargo

     Noting, so far I haven't had much luck with 'RustFmt', not sure if there's much in that plugin (it seems as though syntax highlighting works out of the box with Vim 8.1).
     Also, '!cargo fmt' seems to be work better for me.

     ~/.vimrc:
       au FileType rust set makeprg=cargo
       au FileType rust set errorformat=%.%#-->\ %f:%l:%c


