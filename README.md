Rust Sega Emulator
==================

Conversion of a C emulator, that I'd converted to C++, that I'd converted to
Python, that I've converted to Rust.

Original implementation was based on sega master system technical information from:

        SEGA MASTER SYSTEM TECHNICAL INFORMATION, by Richard Talbot-Watkins, 10th June 1998

        Z80 Instructions
        https://www.zilog.com/docs/z80/um0080.pdf

Building/Running
    Install Rust:
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  
    Install SDL:
        linux (debian based): 
                apt-get install libsdl2-dev
        rasbian (64-bit): 
                apt-get install libsdl2-dev
        rasberry pi (ubuntu mate 64-bit): 
                # Release 22.04 LTS (Jammy Jellyfish) 64-bit
                # Need to upgrade so 'sdl2' will install.
                apt-get update
                apt-get upgrade
                apt-get install git curl libsdl2-dev

                # 'pipewire' appears to be a good sound driver on the raspberry pi
                # SDL_AUDIODRIVER=pipewire 
        OSX: 
                brew install sdl2

        Webassembly
                From: https://puddleofcode.com/story/definitive-guide-to-rust-sdl2-and-emscriptem
                sudo apt-get install emscripten
                rustup target add asmjs-unknown-emscripten
                EM_CONFIG=$HOME/.emscripten emcc --generate-config
                (cd projects/emscripten/ && cargo build --release)

                # Start a web server and load in browser
                python3 -m http.server

                Note, rom file is statically included in the build (not as a command line argument).
                Place a file in "/tmp/test_file.rom" before building.

                # Note, the configuration file in 'projects/emscripten' are the same as running:
                export EMCC_CFLAGS="-s USE_SDL=2"
                cargo build --target asmjs-unknown-emscripten

    Build and run:
        cargo run --release <rom_file>

    Usage: rustsega <cartridge_name> [-d] [-n] [-s <stop-clock>] [-f] [-l]
    
    Rusty Sega Emulator.
    
    Positional Arguments:
      cartridge_name    name of cartridge to run
    
    Options:
      -d, --debug       print PC State Debug Info
      -n, --no-delay    run the emulator with no delay (rather than real-time)
      -s, --stop-clock  number of clock cycles to stop the emulator (for
                        benchmarking)
      -f, --fullscreen  run the emulator in full screen mode.
      -l, --list-drivers
                        list SDL drivers
      --help            display usage information

(Current) Inputs:
    Key mappings (Joystick 1):
    Up: Up, Down: Down, Left: Left, Right: Right
    Fire A: Z, Fire B: X
    Reset: R

    Quit: Escape

Note: Currently 'Quit' doesn't appear to work on Rasbian if audio output is set to HMI, when headphones are connected to the AV Jack (it just hangs).

Dependencies:
   Argument parsing dependency added via:
       cargo add clap --features derive

TODO:

 Non-functional:
    Improve structure (current structure is shortest path to get things running).
    Fix status flag calculations,  cross check with good known Z80 results.

    Clean up 'sega.rs' there's a bit too much 'glue' going on there, that should be shifted out to the submodules.

  Update vdp/cycle comparisons so they support clock rollover (currently just set cycles to 64-bit, but I doubt that's how the master system did it).

 Optimise:
 - Don't do full VDP processing per byte update (check for changes, isolate).
 - Look for CPU cycles
 - Profile

Add more tests
  - Capture Timing, put in fairly full set of op code checks, so op codes can be tidied up (there's currently a lot of repetition).

Sound
  - Set a better/dynamic audio queue length (based on speed/current buffer size, for better sound.)
  - Fix noise/periodic channel. When 'periodic' mode is enabled, it sounds
    worse.  Unsure what 'correct' sounds like (but superficially seems like it
    should have more noise, rather than high pitch pings). The 'noise' sounds
    reasonable, but not sure how accurate it is (currently have a frequency
    multiplier that probably isn't correct).

  
Constants
  - Make SMS Height/Width available to remove magic numbers

Rust General
  - cargo clippy
  - profiling
        cargo install flamegraph

        cargo flamegraph
        #
        # Raspberry pi (ubuntu mate):
        # sudo apt-get install linux-tools-raspi
        #

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

Compilation errors:

SDL2:

      = note: /usr/bin/ld: cannot find -lSDL2
              collect2: error: ld returned 1 exit status
              
    error: could not compile `rustsega` due to previous error

Fix: Install SDL2:

perf setup (for flamegraph, see https://docs.kernel.org/admin-guide/perf-security.html):
   which perf
   # as root/sudo:
   cd /usr/bin
   ls -l perf 
   chgrp perf_users perf
   chmod o-rwx perf
   setcap "cap_perfmon,cap_sys_ptrace,cap_syslog=ep" perf 
   setcap -v "cap_perfmon,cap_sys_ptrace,cap_syslog=ep" perf 
   getcap perf 
   usermod -a -G perf_users <username>


