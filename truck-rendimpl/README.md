# truck-rendimpl

[![Crates.io](https://img.shields.io/crates/v/truck-rendimpl.svg)](https://crates.io/crates/truck-rendimpl) [![Docs.rs](https://docs.rs/truck-rendimpl/badge.svg)](https://docs.rs/truck-rendimpl)

Visualization of shape and polygon mesh based on platform

## Sample Codes

### app

A GUI framework module providing MFC-like API.

### bsp-animation

Benchmark Animation

In each frame, the NURBS surface is devided into mesh.

### material-samples

Material Samples

- The more right the model, the higher the reflectance.
- The upper the model, the higher the roughness.

The most right and lowest model is black because it does not diffuse light
and no roughness in microfacet.

### rotate-objects

Rotate Objects

- Drag the mouse to rotate the camera.
- Drag and drop obj files into the window to switch models.
- Right-click to move the light to the camera's position.
- Enter "P" on the keyboard to switch between parallel projection and perspective projection of the camera.
- Enter "L" on the keyboard to switch the point light source/uniform light source of the light.

### simple-obj-viewer

Simple OBJ viewer

- Drag the mouse to rotate the model.
- Drag and drop obj files into the window to switch models.
- Right-click to move the light to the camera's position.
- Enter "P" on the keyboard to switch between parallel projection and perspective projection of the camera.
- Enter "L" on the keyboard to switch the point light source/uniform light source of the light.
- Enter "Space" on the keyboard to switch the rendering mode for the wireframe and surface.

### simple-shape-viewer

Simple shape viewer

- Drag the mouse to rotate the model.
- Drag and drop json files into the window to switch models.
- Right-click to move the light to the camera's position.
- Enter "P" on the keyboard to switch between parallel projection and perspective projection of the camera.
- Enter "L" on the keyboard to switch the point light source/uniform light source of the light.

A model json file can be generated by `serde_json`. See the examples of `truck-modeling`!

### textured-cube

An example of using texture.
