# Nwin

This is an experimental Neovim UI that creates a new OS window for each Neovim window. This enables performing window management using i3/sway's regular window-management features.

## Why use this

If you don't care about controlling neovim windows from the comfort of Sway, there is absolutely no reason. [Neovim-qt](https://github.com/equalsraf/neovim-qt), [Goneovim](https://github.com/akiyosi/goneovim) and [Neovide](https://github.com/Kethku/neovide) are all much more polished.

## Dependencies

- The ext-win branch of my [neovim fork](https://github.com/glacambre/neovim/tree/ext-win) (make sure it's compiled and first in your path!).
- SDL and its ttf library (`sudo apt install libsdl2 libsdl2-ttf`)
- The font [NotoSansMono-Regular.ttf](https://noto-website-2.storage.googleapis.com/pkgs/NotoSansMono-hinted.zip ) in `$HOME/downloads/NotoSansMono` (yes, really).
- A very strong stomach if you're going to look at the code.

## Obligatory GIF

![video](https://user-images.githubusercontent.com/11534587/110248224-4f64c180-7f70-11eb-8ed7-31b930519cff.gif).
