

# Roadmap
Next features
- text
- line thickness




# Features
- Measuring - key A


# snap
...

## drawing the snap rectangle

```rust
// 1
// y pos.y + 10
// x1 pos.x + 10
// x2 pos.x + -10

// 2
// x pos.x - 10
// y1 pos.y + 10
// y2 pos.y + -10

// 3
// y pos.y - 10
// x1 pos.x + 10
// x2 pos.x + -10

// 4
// x pos.x + 10
// y1 pos.y + 10
// y2 pos.y + -10
```

## snap bugs
1. flickering
there's this thing when you zoom in and your cursor snaps to an object, and you say zoom out again. Well, the snap indicator(little square) will travel off to one of the sides and not follow the 
cursor untill you move the cursor and the position of the cursor adjusts to the zoom level.
This is part of a bigger bug:

### cursor position not adjesting to zoom when zooming, until the cursor is moved
potential solution: recalculate the cursor position on zoom

### What’s actually happening
##### commit 59
Your zoom is cursor-centric
Your pan uses last_position_for_pan
last_position_for_pan is only updated in CursorMoved
If the user:
Zooms
Immediately pans without moving the mouse

👉 then last_position_for_pan is now stale
👉 world space under the cursor has changed due to zoom
👉 first pan delta becomes huge
👉 everything “jumps”

# Solved!!!! - commit 59
Why this works
Zoom changes world→screen mapping
We:
Capture world position before zoom
Apply zoom
Pan camera so same world point stays under cursor
Update pan reference state immediately
Now:
Panning right after zoom = smooth
No jump
No need to move the mouse


# notes
1. 'snap' in 'states' is right now a type 'Vertex' which, in retrospect, might be too much. We only need an x and an y position for now from snap so I will change it into a '[f32; 2]'.
In case we need color for snap, I will change it back.

2.
In 'update_line'
        // this line is problematic once we get into more serious deletion and absolutely criminal is we get into editing a line... 
        let last_line = self.lines.last_mut().unwrap();

Instead...
Implementing u64 IDs. How? Just a counter that keeps on counting. It's important to keep on counting no matter what, never look back. 

when implementing line IDs don't forget to use:

``` rust
let last_line = &mut self.lines[i as usize];
```

becuase if you don't use ```&mut``` it will just copy the value out of the vector and not change it.

Footgun: Indexes might mengle up when you get into editing lines. Becuase of delete.

3. Later down the line it might be better to use a HashMap for lines and circles instead of a Vec 



# Important
12/17/2025:
Line IDs are commented out for now.


# Commit 65 - Implemented input handling for line length
Now it possible to get the input the length of a line or circle radius when it's drawing. The rendering logic isn't implemented yet, just the UI.

To make this work i had to destructure State in renderer.rs and add ui to State with a type of UiState that's in gui_elements.rs. I also made gui() which draws the gui elements to the screen a method
of the UiState struct. This way I have access to UiState without having to pass UiState onto gui().

Btw, this was one of the problems why I needed to do it this way. in state.egui.draw() I wasn't able to call gui() and pass in state to it... So I wasn't able to access state in gui_elements.

The way i solved input handling is that for the first digit winit is listening. And once the first digit is in, numeric_active - field on UiState - turns on and let's egui's TextEdit field take over
reditrecting all the input into UiState's numeric_buff. Then, when the user clicks Enter, the input is returned through UiAction's Input() enum, the buffer is cleared and the TextEdit field loses focus.







// in Mode::Move(FuncState::SelectPoint) | Mode::Copy(FuncState::SelectPoint) => {
let pos: [f32; 2];
if let Some(snap_pos) = state.snap {
        pos = snap_pos;
} else {
        pos = position;
}

// in Mode::Move(FuncState::Move(starting_position)) | Mode::Copy(FuncState::Copy(starting_position)) => {
let diff1: f32;
let diff2: f32;

if let Some(snap_pos) = state.snap {
        diff1 = starting_position[0] - snap_pos[0];
        diff2 = starting_position[1] - snap_pos[1];
} else {
        diff1 = starting_position[0] - position[0];
        diff2 = starting_position[1] - position[1];
}


# Different screens, different results
I'm working on implementing text atm, and the problem I'm facing is this: I'm converting world_pos to screen_pos and just for testing I made the text follow my cursor. Which it did on my monitor, but not on my laptop. On my laptop, the further I got away from 0.0 0.0 the further the text got away from my cursor.

The code:
pub fn world_to_screen(
    world_x: f32,
    world_y: f32,
    screen_rect: egui::Rect,
    camera: &Camera,
) -> [f32; 2] {
    let cen_x = screen_rect.width() / 2.0;
    let cen_y = screen_rect.height() / 2.0;

    let rel_x = world_x - camera.x_offset;
    println!("x offset: {:?}", camera.x_offset);
    println!("y offset: {:?}", camera.y_offset);
    println!("zoom: {:?}", camera.zoom);
    println!("cen_x: {:?}", cen_x);
    println!("cen_y: {:?}", cen_y);
    let rel_y = world_y - camera.y_offset;

    let screen_x = (rel_x * camera.zoom) + cen_x;
    let screen_y = cen_y - (rel_y * camera.zoom);

    [screen_x, screen_y]
}

Problem: screen resolution
I added this line into gui_elements where ui is an egui Context. On my laptop screen it returned 1.5, while on my monitor just 1.0.
let pixels_per_point = ui.pixels_per_point();

500 on a 1.0x screen is not the same as 500 on a 1.5x screen. This happens because of egui.

After this line, it still wasn't quite right. The text was offseted, but the offset was constant. Here i changed screen_rect which solved it:
let veiwport_rect = ui.available_rect();

This is why it fixed the issue on your laptop (1.5x scale).
Winit Size: Returns Physical Pixels (e.g., 1920x1080).
Egui Rect: Returns Logical Points (e.g., 1280x720).
Previously, you were grabbing the physical size (1920) and trying to map it to egui's logical coordinate system. This created a mismatch where egui thought the screen was huge (or tiny), causing the "drift" you saw when zooming.
ui.available_rect() gives you a rectangle that is already in Egui's native Logical Points. You no longer need to manually divide by the scale factor or worry about converting units for the screen bounds—egui has already done that math for you.
