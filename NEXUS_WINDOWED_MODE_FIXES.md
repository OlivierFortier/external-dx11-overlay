# Nexus Integration Windowed Mode Fixes

This document describes the fixes implemented to resolve issues with the Nexus integration in windowed-fullscreen and windowed modes.

## Issues Fixed

### 1. Windowed-Fullscreen Mode (Game Stretching)
**Problem**: The overlay was stretching outside the screen boundaries in windowed-fullscreen mode.
**Cause**: The code was using swapchain buffer dimensions which don't always match the actual window client area in windowed-fullscreen mode.
**Fix**: 
- Added proper client area detection using `GetClientRect`
- Compare client dimensions with swapchain dimensions and use the smaller values
- Properly scale the viewport to match the actual rendering area

### 2. Windowed Mode (Black/Frozen Screen with Flickering)
**Problem**: Black screen or frozen screen with flickering UI elements in windowed mode.
**Cause**: 
- Improper handling of texture size mismatches between Blish HUD and game window
- Missing render state backups causing conflicts with game rendering
- No proper scaling when overlay texture size differs from window size

**Fixes**:
- Added vertex buffer and constant buffer support for proper quad rendering
- Implemented texture coordinate scaling to handle size mismatches
- Enhanced render state backup/restore to prevent conflicts
- Always update texture resources when addresses change

## Technical Changes

### 1. Enhanced OverlayState Structure
- Added `vertex_buffer` for proper quad rendering
- Added `constant_buffer` for shader parameters
- Added helper structures `Vertex` and `ConstantBuffer`

### 2. Improved Resize Logic
- Now checks actual window client area dimensions
- Handles windowed-fullscreen mode by detecting when client area is smaller than swapchain
- Updates viewport based on actual rendering area, not just swapchain size

### 3. Better Texture Management
- Always updates textures when addresses change (not just size)
- Proper scaling via constant buffer when texture size doesn't match window size
- Enhanced error handling for texture creation failures

### 4. Render State Management
- More comprehensive backup of D3D11 pipeline state
- Proper restoration of all modified states
- Added constant buffer binding for vertex shader

### 5. Shader Improvements
- Created proper HLSL source files for shaders
- Added support for texture coordinate scaling
- Vertex buffer-based rendering for better control

## Building

To compile the shaders (requires Windows SDK with fxc.exe):
```bash
cd src/ui
compile_shaders.bat
```

## Testing

1. Test in fullscreen mode - should work as before
2. Test in windowed-fullscreen mode - overlay should not stretch beyond screen
3. Test in windowed mode - no black screen or flickering
4. Test window resizing - overlay should adapt properly
5. Test with different Blish HUD resolutions - proper scaling should be applied

## Compatibility

These changes maintain backward compatibility with the existing functionality while fixing the windowed mode issues. The overlay will automatically detect the window mode and apply appropriate rendering adjustments.