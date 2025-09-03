@echo off
REM Compile HLSL shaders to CSO format

echo Compiling vertex shader...
fxc /T vs_5_0 /E main /Fo vs.cso vs.hlsl

echo Compiling scaled vertex shader...
fxc /T vs_5_0 /E main /Fo vs_scaled.cso vs_scaled.hlsl

echo Compiling pixel shader...
fxc /T ps_5_0 /E main /Fo ps.cso ps.hlsl

echo Done!