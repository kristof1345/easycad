


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