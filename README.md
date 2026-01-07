# easycad
easycad is a simplistic and easy to use 2D CAD application written in Rust. It uses [WGPU](https://github.com/gfx-rs/wgpu) for graphics, [winit](https://github.com/rust-windowing/winit) for window management and [egui](https://github.com/emilk/egui) for GUI elements.

## Reason for creation
I created easycad mainly because I wanted to learn how CAD apps work. I make a lot of 2D drawings in CAD apps like AutoCAD, frankly, I just got fed up with the CAD software out there being slow, clunky and unnecesarrily overcomplicated/overbloated - or, costing an arm and a half - so I set out to create a CAD app for myself.

## Features
- [x] Line drawing
- [x] Draw line with length
- [x] Line thickness
- [ ] Change line thickness
- [x] Deleting lines
- [x] Selecting lines
- [x] Zooming
- [x] Panning
- [x] Drawing circles
- [x] Draw circle with radius
- [ ] Circle thickness
- [x] Save/Open
- [x] Create text
- [x] Edit text + make it annotative
- [x] Measure tool
- [ ] Export/Import

## Installation
```bash
git clone https://github.com/kristof1345/easycad.git

cd easycad

cargo run
```

## How to use
#### Drawing a line
Press `l`, your cursor will change into a crosshair. Click a position on screen, drag your cursor to the next position and click again. You have a line. You can keep drawing lines until you press `esc`.
#### Exiting a feature
Just press `esc` and whatever you were doing will finish/exit.
#### Zooming
**Mouse**: Just scroll 
**Touchpad**: Pinch
#### Panning
**Mouse**: Press the middle button on your mouse and move your mouse
**Touchpad**: Press `ctrl`, now you can move your model with your finger on the touchpad.

# Keybindings
- Draw circle - Key C
- Draw line - Key L -> toggle Ortho - Key O
- Selection - Click on object || Key S
- Deletion - Key Delete -> Click object to delete || Select object -> Key Delete
- Move - Key M -> Select objects -> Enter -> Click from where to move -> Click where to move
- Copy - Key K -> Select objects -> Enter -> Click from where to copy -> Click where to copy
- Measuring - key A
- Add text - key T
- Edit text - Right click in text
