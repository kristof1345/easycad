


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
