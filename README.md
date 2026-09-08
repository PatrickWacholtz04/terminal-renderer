# terminal-renderer

Render 3D objects from .obj files directly in your terminal.


## Usage
#### Setup
``` 
    git clone https://github.com/PatrickWacholtz04/terminal-renderer.git
    cd renderer
```
To render .obj files, run
```
    cargo run -- "relative_path/to/file.obj"
```
So if the .obj is in the renderer directory, "file.obj".<br>
Or if it is in the src directory, "src/file.obj".<br>
If no filepath is given, it will default to "src/VideoShip.obj"

##### Terminal Size
The terminal size will need to be quite large in order to get any significant resolution.<br>
You may need to adjust your terminal font size in order to fit the entire contents of the render.

#### User Controls
W / S: Rotate around X axis<br>
A / D: Rotate around Y axis<br>
Q / E: Rotate around Z axis<br>
R / F: Translate along Z Axis<br>


## To be implemented:
Clipping<br>
Translation along X and Y axis<br>
Fit output resolution to terminal size and update when changed.<br>